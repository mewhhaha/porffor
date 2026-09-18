//! Source assembly for the compiler-private synchronous module activation protocol.

use super::link::LinkedScriptSource;
use super::module_key::ANONYMOUS_MODULE_KEY;
use super::namespace::push_js_string_literal;
use super::record::{import_meta_binding, rewrite_import_meta, DefaultExportFormIr};
use super::source::DefaultExportRewrite;
use super::synchronous_definition::{SynchronousModuleDefinitions, SynchronousUnitDefinition};
use crate::*;

/// Validated eligibility for the private allocation/instantiation path. The
/// source builder requires this witness, so asynchronous, source-phase and
/// Script-entry graphs cannot accidentally enter synchronous evaluation.
pub(super) struct SynchronousInstantiationGraph<'a> {
    graph: &'a ModuleGraphIr,
}

impl<'a> SynchronousInstantiationGraph<'a> {
    pub(super) fn new(graph: &'a ModuleGraphIr, components: &[DynamicComponentIr]) -> Option<Self> {
        let eligible = !graph.entry_is_script
            && graph
                .units
                .iter()
                .enumerate()
                .all(|(index, _)| !graph.has_tla(index as u32))
            && graph
                .units
                .iter()
                .flat_map(|unit| &unit.record.requested_modules)
                .chain(components.iter().map(|component| component.request()))
                .all(|request| request.phase() != ImportPhaseIr::Source);
        eligible.then_some(Self { graph })
    }
}

pub(super) fn linked_synchronous_source(
    sources: &ModuleGraphSources,
    eligible: SynchronousInstantiationGraph<'_>,
) -> Result<LinkedScriptSource, Vec<IrDiagnostic>> {
    let graph = eligible.graph;
    let diagnostics = graph.check_dynamic_import_linkable();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut text = String::from("\"use strict\";\n");
    text.push_str(&graph.dynamic_import_prelude());
    text.push('\n');
    let graph_start_line = source_line_number(&text);
    let mut definitions = Vec::new();
    let components = super::link::evaluation_components(graph);
    let component_of_unit = graph.component_of_unit();
    text.push_str("[\n");
    for (module, _, unit) in graph.materialized_units() {
        let mut dependencies = Vec::new();
        for request in &unit.record.requested_modules {
            if request.phase() == ImportPhaseIr::Evaluation {
                let target = graph
                    .resolve_request(module, request)
                    .expect("linked dependency resolves");
                if !dependencies.contains(&target) {
                    dependencies.push(target);
                }
            }
        }
        let readiness = readiness_graph(graph, module);
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
        let body = rewrite_import_meta(&unit.source_text, &unit.record)
            .map_err(|error| error.reason)
            .and_then(|body| super::LinkedScriptDefinitions::rewrite_body(&body, rewrite))
            .and_then(|body| graph.rewrite_dynamic_import_calls(module, &body))
            .map_err(|reason| {
                vec![IrDiagnostic::unsupported(format!(
                    "module {}: {reason}",
                    unit.record.key.as_str()
                ))]
            })?;
        text.push_str("[async () => {\n\"use strict\";\n");
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
        text.push_str("\n;\n}, () => 0],\n");
        definitions.push(SynchronousUnitDefinition {
            module,
            imports,
            namespaces,
            has_import_meta: unit.record.import_meta_uses() > 0,
            default_export: unit.record.default_export_form(),
            evaluation: super::SynchronousModuleEvaluationIr::new(
                module,
                dependencies,
                components[component_of_unit[module as usize]].clone(),
            ),
            readiness,
        });
    }
    text.push_str("];\n");
    let graph_end_line = source_line_number(&text) - 1;
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
    // Entry dependencies must be traversed from the entry, in request order.
    // Pre-evaluating a cycle member would choose a different DFS root and can
    // execute the entry body before an earlier requested dependency.
    let mut entry_dependencies = BTreeSet::new();
    let mut pending = vec![graph.entry];
    while let Some(module) = pending.pop() {
        if entry_dependencies.insert(module) {
            pending.extend(
                graph
                    .evaluation_dependencies_of(module)
                    .into_iter()
                    .map(|dependency| dependency.target()),
            );
        }
    }
    // Preserve the existing eager dynamic-import roots outside the entry's
    // static dependency graph until their job scheduler is replaced.
    let initial_evaluation = graph
        .evaluation_order
        .iter()
        .copied()
        .filter(|module| {
            !entry_dependencies.contains(module)
                && graph.evaluation_mode(*module) == ModuleEvaluationModeIr::Eager
        })
        .chain(core::iter::once(graph.entry))
        .collect::<Vec<_>>();
    for _ in &initial_evaluation {
        text.push_str("0;\n");
    }
    let mut linked = super::LinkedScriptDefinitions::default();
    linked.synchronous = Some(SynchronousModuleDefinitions {
        graph_span: (
            boa_ast::Position::new(graph_start_line, 1),
            boa_ast::Position::new(graph_end_line, 2),
        ),
        units: definitions,
        record_count: graph.units.len() as u32,
        dispatcher_namespaces,
        initial_evaluation,
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

fn readiness_graph(graph: &ModuleGraphIr, root: u32) -> Vec<super::ModuleReadinessNodeIr> {
    let mut pending = vec![root];
    let mut seen = BTreeSet::new();
    let mut nodes = Vec::new();
    while let Some(module) = pending.pop() {
        if !seen.insert(module) {
            continue;
        }
        let mut dependencies = Vec::new();
        for request in &graph.unit(module).record.requested_modules {
            if request.phase() == ImportPhaseIr::Source {
                continue;
            }
            let target = graph
                .resolve_request(module, request)
                .expect("linked dependency resolves");
            if !dependencies.contains(&target) {
                dependencies.push(target);
                pending.push(target);
            }
        }
        nodes.push(super::ModuleReadinessNodeIr {
            module,
            dependencies,
        });
    }
    nodes
}
