//! Source assembly for the compiler-private module activation protocol.

use super::link::LinkedScriptSource;
use super::module_key::ANONYMOUS_MODULE_KEY;
use super::namespace::push_js_string_literal;
use super::record::{import_meta_binding, rewrite_import_meta, DefaultExportFormIr};
use super::source::DefaultExportRewrite;
use super::synchronous_definition::{ModuleExecutionDefinitions, ModuleUnitDefinition};
use super::synchronous_execution::ModuleExecutionEntry;
use crate::*;

/// Validated eligibility for the private allocation/instantiation path. The
/// source builder requires this witness: active static source bindings have no
/// Source Text Module representation, while dynamic source jobs reject without
/// allocating a source cell. Script roots cannot become Module entries.
pub(super) struct ModuleInstantiationGraph<'a> {
    graph: &'a ModuleGraphIr,
    entry: ModuleExecutionEntry,
}

impl<'a> ModuleInstantiationGraph<'a> {
    pub(super) fn new(graph: &'a ModuleGraphIr) -> Result<Self, Vec<IrDiagnostic>> {
        if !graph.link_errors.is_empty() {
            return Err(graph
                .link_errors
                .iter()
                .map(ModuleLinkErrorIr::to_diagnostic)
                .collect());
        }
        assert!(
            graph.materialized_units().all(|(_, _, unit)| unit
                .record
                .requested_modules
                .iter()
                .all(|request| request.phase() != ImportPhaseIr::Source)),
            "static source imports fail before execution ownership"
        );
        Ok(Self {
            graph,
            entry: if graph.entry_is_script {
                ModuleExecutionEntry::Script(graph.entry)
            } else {
                ModuleExecutionEntry::Module(graph.entry)
            },
        })
    }
}

pub(super) fn linked_module_execution_source(
    sources: &ModuleGraphSources,
    eligible: ModuleInstantiationGraph<'_>,
) -> Result<LinkedScriptSource, Vec<IrDiagnostic>> {
    let ModuleInstantiationGraph { graph, entry } = eligible;
    let diagnostics = graph.check_dynamic_import_linkable();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    // Generated scaffolding must not bury an entry Script's directive
    // prologue. Its own source remains at root, with its original strictness.
    let mut text = match &entry {
        ModuleExecutionEntry::Module(_) => String::from("\"use strict\";\n"),
        ModuleExecutionEntry::Script(script) => {
            if graph.unit(*script).record.script_entry_strict {
                String::from("\"use strict\";\n")
            } else {
                String::new()
            }
        }
    };
    text.push_str(&graph.module_initialization_dispatchers(true));
    text.push('\n');
    let mut linked = super::LinkedScriptDefinitions::default();
    // This source owner is materialized against the selected Realm's global
    // environment. Its private dispatcher cells never capture the entry Script.
    text.push_str("(() => {\n\"use strict\";\n");
    text.push_str(&graph.module_initialization_dispatchers(false));
    text.push(' ');
    text.push_str(&graph.realm_import_dispatcher_source());
    text.push('\n');
    let graph_start_line = source_line_number(&text);
    let mut definitions = Vec::new();
    text.push_str("[\n");
    for (module, _, unit) in graph.materialized_units() {
        if matches!(&entry, ModuleExecutionEntry::Script(script) if *script == module) {
            continue;
        }
        let requests = unit
            .record
            .requested_modules
            .iter()
            .map(|request| {
                let phase = match request.phase() {
                    ImportPhaseIr::Evaluation => super::ModuleRequestPhaseIr::Evaluation,
                    ImportPhaseIr::Defer => super::ModuleRequestPhaseIr::Defer,
                    ImportPhaseIr::Source => {
                        unreachable!("validated execution graph excludes source phase")
                    }
                };
                super::ModuleExecutionRequestIr::new(
                    phase,
                    graph
                        .resolve_request(module, request)
                        .expect("linked dependency resolves"),
                )
            })
            .collect();
        let kind = if unit.record.has_top_level_await {
            super::ModuleActivationKindIr::Async
        } else {
            super::ModuleActivationKindIr::Synchronous
        };
        let imports = unit.resolved_imports.clone();
        let namespaces = unit
            .namespaces
            .values()
            .map(|namespace| {
                (
                    namespace.mode(),
                    namespace
                        .exports
                        .iter()
                        .map(|export| export.target.clone())
                        .collect(),
                )
            })
            .collect();
        let default_name = MergedName::anonymous_default(module);
        let rewrite = match unit.record.default_export_form() {
            DefaultExportFormIr::Absent => DefaultExportRewrite::None,
            DefaultExportFormIr::Named => DefaultExportRewrite::DeleteKeywords,
            DefaultExportFormIr::Anonymous { hoisted } => DefaultExportRewrite::Bind {
                name: &default_name,
                hoisted,
            },
        };
        let json = unit
            .record
            .json_source()
            .map(|source| JsonModuleValueIr::new(module, source.clone()));
        let body = if json.is_some() {
            None
        } else {
            let original = super::callable_source::OriginalUnitSource::new(
                &super::record::embeddable_unit_source(&unit.source_text),
            );
            Some(
                rewrite_import_meta(original.text(), &unit.record)
                    .map_err(|error| error.reason)
                    .and_then(|body| {
                        super::default_export_definition::rewrite_source(
                            original.stable_rewrite(body),
                            rewrite,
                        )
                    })
                    .and_then(|body| graph.rewrite_dynamic_import_source(module, body))
                    .map_err(|reason| {
                        vec![IrDiagnostic::unsupported(format!(
                            "module {}: {reason}",
                            unit.record.key.as_str()
                        ))]
                    })?,
            )
        };
        text.push_str("async () => {\n\"use strict\";\n");
        for import in &unit.record.import_entries {
            text.push_str("const ");
            text.push_str(import.local_name.spec_name());
            text.push_str(" = 0;\n");
        }
        if unit.record.import_meta_uses() > 0 {
            text.push_str(&import_meta_binding(module, &unit.meta_url).declaration);
            text.push('\n');
        }
        for namespace in unit.namespaces.values() {
            text.push('[');
            match namespace.mode() {
                ModuleNamespaceModeIr::Eager => text.push_str("void 0"),
                ModuleNamespaceModeIr::Deferred => text.push_str("() => 0"),
            }
            for export in &namespace.exports {
                text.push_str(", ");
                push_js_string_literal(&mut text, export.export_name.as_str());
                text.push_str(", () => 0");
            }
            text.push_str("];\n");
        }
        if json.is_some() {
            // Synthetic default bindings exist at instantiation; JSON data is
            // evaluated later by the native operation, never by this scaffold.
            text.push_str("let ");
            text.push_str(default_name.as_str());
            text.push_str(" = void 0;\n");
        }
        // Trusted metadata turns this boundary into a private suspension.
        text.push_str("0;\n");
        if json.is_some() {
            text.push_str(default_name.as_str());
            text.push_str(" = 0;\n");
        } else {
            text.push_str(
                linked
                    .callable_sources
                    .record_body(body.expect("Source Text body"), &text),
            );
        }
        text.push_str("\n;\n},\n");
        definitions.push(ModuleUnitDefinition {
            module,
            imports,
            namespaces,
            has_import_meta: unit.record.import_meta_uses() > 0,
            default_export: if json.is_some() {
                DefaultExportFormIr::Absent
            } else {
                unit.record.default_export_form()
            },
            json,
            evaluation: super::ModuleEvaluationIr::new(module),
            kind,
            requests,
        });
    }
    text.push_str("];\n");
    let graph_end_line = source_line_number(&text) - 1;
    text.push_str("});\n");
    let dispatcher_namespaces = graph
        .materialized_units()
        .flat_map(|(_, _, unit)| unit.namespaces.values())
        .map(|namespace| {
            (
                namespace.cell.as_str().to_string(),
                super::ModuleCellIr::Namespace {
                    module: namespace.module,
                    mode: namespace.mode(),
                },
            )
        })
        .collect();
    // Instantiation above executes no module source. A Module entry then
    // starts its static traversal; a Script keeps its own root statements and
    // evaluates a module only when an import continuation requests it.
    match &entry {
        ModuleExecutionEntry::Module(module) => {
            text.push_str("0;\n");
            linked.entry = Some(super::LinkedModuleEntry::CanonicalGraph(*module));
        }
        ModuleExecutionEntry::Script(script) => {
            let body = graph
                .rewrite_dynamic_import_source(
                    *script,
                    super::callable_source::OriginalUnitSource::new(
                        &super::record::embeddable_unit_source(&graph.unit(*script).source_text),
                    ),
                )
                .map_err(|reason| vec![IrDiagnostic::unsupported(reason)])?;
            text.push_str(linked.callable_sources.record_body(body, &text));
        }
    }
    let dispatcher_evaluations = definitions
        .iter()
        .map(|unit| {
            (
                super::dynamic::module_evaluator_name(unit.module),
                unit.evaluation.clone(),
            )
        })
        .collect();
    linked.synchronous = Some(ModuleExecutionDefinitions {
        realm_requests: graph.realm_requests.clone(),
        graph_span: (
            boa_ast::Position::new(graph_start_line, 1),
            boa_ast::Position::new(graph_end_line, 2),
        ),
        units: definitions,
        record_count: graph.units.len() as u32,
        dispatcher_namespaces,
        dispatcher_evaluations,
        dispatcher_async_dependencies: graph
            .materialized_units()
            .map(|(module, _, _)| {
                (
                    super::dynamic::module_async_dependencies_name(module),
                    super::ModuleEvaluationIr::new(module),
                )
            })
            .collect(),
        dispatcher_deferred_imports: graph
            .materialized_units()
            .map(|(module, _, _)| {
                (
                    super::dynamic::module_deferred_import_name(module),
                    super::ModuleEvaluationIr::new(module),
                )
            })
            .collect(),
        entry,
    });
    Ok(LinkedScriptSource {
        definitions: linked,
        source: SourceUnit {
            goal: ParseGoal::Script,
            filename: sources
                .modules
                .get(sources.entry as usize)
                .map(|module| module.key().as_str().to_string())
                .filter(|key| key != ANONYMOUS_MODULE_KEY),
            source_text: text,
        },
    })
}

// Boa counts every ECMAScript LineTerminatorSequence, including source CR,
// CRLF, U+2028 and U+2029 inside module bodies, as one line.
fn source_line_number(source: &str) -> u32 {
    let terminators = source
        .chars()
        .filter(|character| matches!(character, '\r' | '\n' | '\u{2028}' | '\u{2029}'))
        .count();
    1 + (terminators - source.matches("\r\n").count()) as u32
}
