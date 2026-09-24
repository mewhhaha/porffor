//! Source assembly for the compiler-private module activation protocol.

use super::link::LinkedScriptSource;
use super::module_key::ANONYMOUS_MODULE_KEY;
use super::namespace::push_js_string_literal;
use super::record::{import_meta_binding, rewrite_import_meta, DefaultExportFormIr};
use super::source::DefaultExportRewrite;
use super::synchronous_definition::{
    ModuleExecutionDefinitions, ModuleExecutionEntry, ModuleUnitDefinition,
};
use crate::*;

/// Validated eligibility for the private allocation/instantiation path. The
/// source builder requires this witness, so graphs with a static source-phase
/// request (`import source x from ...`) retain their explicit admission
/// boundary.
///
/// A dynamic `import.source()` does not disqualify a graph: every module this
/// host loads is a Source Text Module, whose GetModuleSource throws a
/// SyntaxError, so the canonical dispatcher rejects such a call without any
/// module source object.
pub(super) struct ModuleInstantiationGraph<'a> {
    graph: &'a ModuleGraphIr,
}

impl<'a> ModuleInstantiationGraph<'a> {
    pub(super) fn new(graph: &'a ModuleGraphIr) -> Option<Self> {
        let eligible = graph
            .units
            .iter()
            .flat_map(|unit| &unit.record.requested_modules)
            .all(|request| request.phase() != ImportPhaseIr::Source);
        eligible.then_some(Self { graph })
    }
}

/// What the canonical graph's owner runs after allocation and instantiation.
///
/// Closed so the two entry shapes cannot be confused: a Module entry is one of
/// the graph's activations and is evaluated by the private entry operation; a
/// Script entry is not a module at all; its own statements follow the graph in
/// the Script's own strictness and global scope, and it reaches the graph only
/// through `import()`.
#[derive(Clone, Copy)]
enum CanonicalGraphEntry<'a> {
    Module(ModuleUnitId),
    Script {
        unit: ModuleUnitId,
        strict: bool,
        source: &'a str,
    },
}

impl<'a> CanonicalGraphEntry<'a> {
    fn of(
        graph: &'a ModuleGraphIr,
        sources: &ModuleGraphSources,
    ) -> Result<Self, Vec<IrDiagnostic>> {
        if !graph.entry_is_script {
            return Ok(Self::Module(graph.entry));
        }
        let strict = match sources
            .modules
            .get(sources.entry as usize)
            .map(|source| &source.parse)
        {
            Some(super::loaded_sources::ModuleParse::ScriptEntry(parsed)) => {
                parsed.with_compiler_session(|script, _| script.strict())
            }
            _ => unreachable!("the lowerer admits a Script entry graph only with Script syntax"),
        };
        let unit = graph.unit(graph.entry);
        // The Script's `import()` sites call a dispatcher declared in the
        // Script's own top-level scope, so a Script spelling a linker name
        // could shadow or redeclare it.
        if unit
            .source_text
            .contains(super::dynamic::LINKER_NAME_PREFIX)
        {
            return Err(vec![IrDiagnostic::unsupported(format!(
                "unsupported in lila wasm-aot: script {} spells the linker-reserved prefix `{}`",
                unit.record.key.as_str(),
                super::dynamic::LINKER_NAME_PREFIX
            ))]);
        }
        Ok(Self::Script {
            unit: graph.entry,
            strict,
            source: &unit.source_text,
        })
    }

    fn script_unit(&self) -> Option<ModuleUnitId> {
        match self {
            Self::Module(_) => None,
            Self::Script { unit, .. } => Some(*unit),
        }
    }
}

pub(super) fn linked_module_execution_source(
    sources: &ModuleGraphSources,
    eligible: ModuleInstantiationGraph<'_>,
) -> Result<LinkedScriptSource, Vec<IrDiagnostic>> {
    let graph = eligible.graph;
    let diagnostics = graph.check_dynamic_import_linkable();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let entry = CanonicalGraphEntry::of(graph, sources)?;
    // Module code is strict (16.2.1.6.1). A Script entry keeps its own
    // strictness: its Directive Prologue can no longer lead the merged text,
    // so a strict Script's is restated here, and every generated function
    // below carries its own "use strict" directive either way.
    let mut text = match entry {
        CanonicalGraphEntry::Script { strict: false, .. } => String::new(),
        CanonicalGraphEntry::Module(_) | CanonicalGraphEntry::Script { strict: true, .. } => {
            String::from("\"use strict\";\n")
        }
    };
    text.push_str(&graph.module_execution_dynamic_import_prelude());
    text.push('\n');
    let graph_start_line = source_line_number(&text);
    let mut definitions = Vec::new();
    text.push_str("[\n");
    for (module, _, unit) in graph.materialized_units() {
        if entry.script_unit() == Some(module) {
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
        let body = rewrite_import_meta(
            &super::record::embeddable_unit_source(&unit.source_text),
            &unit.record,
        )
        .map_err(|error| error.reason)
        .and_then(|body| super::LinkedScriptDefinitions::rewrite_body(&body, rewrite))
        .and_then(|body| graph.rewrite_dynamic_import_calls(module, &body))
        .map_err(|reason| {
            vec![IrDiagnostic::unsupported(format!(
                "module {}: {reason}",
                unit.record.key.as_str()
            ))]
        })?;
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
        // Trusted metadata turns this boundary into a private suspension.
        text.push_str("0;\n");
        text.push_str(&body);
        text.push_str("\n;\n},\n");
        definitions.push(ModuleUnitDefinition {
            module,
            imports,
            namespaces,
            has_import_meta: unit.record.import_meta_uses() > 0,
            default_export: unit.record.default_export_form(),
            evaluation: super::ModuleEvaluationIr::new(module),
            kind,
            requests,
        });
    }
    text.push_str("];\n");
    let graph_end_line = source_line_number(&text) - 1;
    // Only activations publish cells; a Script entry has none of its own.
    let activations = || {
        graph
            .materialized_units()
            .filter(|(module, _, _)| entry.script_unit() != Some(*module))
    };
    let dispatcher_namespaces = activations()
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
    let dispatcher_async_dependencies = activations()
        .map(|(module, _, _)| {
            (
                super::dynamic::module_async_dependencies_name(module),
                super::ModuleEvaluationIr::new(module),
            )
        })
        .collect();
    let dispatcher_deferred_imports = activations()
        .map(|(module, _, _)| {
            (
                super::dynamic::module_deferred_import_name(module),
                super::ModuleEvaluationIr::new(module),
            )
        })
        .collect();
    // Dynamic targets evaluate only from their import continuation. Starting
    // at a Module entry also preserves its static dependency DFS and SCC
    // owner. A Script entry is not evaluated as a module: its own statements
    // follow the graph, and only its `import()` calls evaluate modules.
    let execution_entry = match entry {
        CanonicalGraphEntry::Module(module) => {
            text.push_str("0;\n");
            ModuleExecutionEntry::Module(super::ModuleEvaluationIr::new(module))
        }
        CanonicalGraphEntry::Script { unit, source, .. } => {
            // A Hashbang Comment is legal only at the very start of the source,
            // which the Script no longer occupies; `//` keeps it a comment of
            // the same length on the same line.
            let source = match source.strip_prefix("#!") {
                Some(rest) => std::borrow::Cow::Owned(format!("//{rest}")),
                None => std::borrow::Cow::Borrowed(source),
            };
            let body = graph
                .rewrite_dynamic_import_calls(unit, &source)
                .map_err(|reason| {
                    vec![IrDiagnostic::unsupported(format!(
                        "unsupported in lila wasm-aot: script {}: {reason}",
                        graph.unit(unit).record.key.as_str()
                    ))]
                })?;
            text.push_str(&body);
            text.push('\n');
            ModuleExecutionEntry::Script
        }
    };
    let dispatcher_evaluations = definitions
        .iter()
        .map(|unit| {
            (
                super::dynamic::module_evaluator_name(unit.module),
                unit.evaluation.clone(),
            )
        })
        .collect();
    let mut linked = super::LinkedScriptDefinitions::default();
    linked.entry = match &execution_entry {
        ModuleExecutionEntry::Module(evaluation) => Some(super::LinkedModuleEntry::CanonicalGraph(
            evaluation.module(),
        )),
        ModuleExecutionEntry::Script => None,
    };
    linked.synchronous = Some(ModuleExecutionDefinitions {
        graph_span: (
            boa_ast::Position::new(graph_start_line, 1),
            boa_ast::Position::new(graph_end_line, 2),
        ),
        units: definitions,
        record_count: graph.units.len() as u32,
        dispatcher_namespaces,
        dispatcher_evaluations,
        dispatcher_async_dependencies,
        dispatcher_deferred_imports,
        entry: execution_entry,
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
