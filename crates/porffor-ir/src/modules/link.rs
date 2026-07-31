//! Merging a linked graph into the single `ScriptIr` the backend emits.
//!
//! # Artifact strategy
//!
//! One Wasm module per graph. Every unit of the graph contributes its body to
//! one merged `ScriptIr`, in the evaluation order the graph fixed
//! ([`evaluation_components`]), and the whole graph is compiled as one
//! artifact.
//!
//! The merge happens on *source text*, not on lowered IR, and that is a
//! deliberate choice rather than a shortcut around one:
//!
//! * `FunctionId`s are minted from source byte offsets. Lowering units
//!   separately and concatenating the results collides two units that happen to
//!   declare a function at the same offset; concatenating first makes every
//!   offset unique by construction.
//! * `owned_env_slots` are numbered per lowering. Two independent lowerings
//!   both start at slot 0, so merging them needs a slot remap that has to reach
//!   every slot reference in the IR; one lowering numbers the whole graph once.
//! * A cross-module binding read must be a read of the *exporting* unit's cell.
//!   With one merged top-level environment the importer's name and the
//!   exporter's name are the same binding, so the read is live and needs no
//!   runtime indirection: mutating an exported `let` through an exported
//!   function is observed by every importer, which a copied binding could not
//!   do. The import binding is also the exporter's `let`/`const` itself, so it
//!   carries whatever top-level TDZ the script pipeline enforces — the module
//!   path adds no TDZ of its own and loses none.
//!
//! Per-module identity survives in `ProgramIr::modules`, so splitting the graph
//! into several linked Wasm modules later is a backend change with no IR
//! change.
//!
//! # Module semantics
//!
//! * The merged source opens with a `"use strict"` prologue, so every unit is
//!   strict (16.2.1.6.1 parses module code as strict regardless of its text).
//! * Unit bodies are separated by an empty statement so that no unit's last
//!   token can join the next unit's first token through ASI.
//! * Module top-level `this` is `undefined`. The merged source is Script text,
//!   whose top-level `this` is `globalThis`, so a unit that observes top-level
//!   `this` is reported rather than silently given the wrong value — see
//!   [`lowering::lower_module_graph`].
//!
//! [`lowering::lower_module_graph`]: crate::lower_module_graph
//!
//! One deviation is knowingly left in place: a module top-level `var` belongs
//! to the module environment (9.1.1.5), but merged Script text makes it a
//! property of the global object. It is observable only by probing
//! `globalThis`, the cross-unit collision check already stops two units from
//! sharing such a name, and closing it needs the same per-unit renaming pass as
//! the aliases below.
//!
//! # What this stage does not link yet
//!
//! Each of these is reported as one honest `Unsupported` diagnostic rather than
//! mislinked, and each is a separate follow-up:
//!
//! * a binding whose importer-side name differs from the exporter-side name
//!   (`import { a as b }`, `export { a as b }`, and every default import, whose
//!   exporter-side name is the unspellable `*default*`) — the merged scope
//!   binds by name, so an alias needs a renaming pass over the unit body;
//! * `export default`, for the same reason;
//! * two units that declare the same top-level name, which the merged scope
//!   cannot hold side by side without that same renaming pass.
//!
//! Namespace objects, `import.meta` and dynamic `import()` *are* linked, all
//! three as generated Script text rather than as backend nodes:
//! `modules::namespace` owns the namespace objects, `modules::record` owns the
//! `import.meta` objects and the body rewrite that reaches them, and
//! `modules::dynamic` owns the `import()` dispatchers. Each reports its own
//! remaining gaps through [`check_linkable`]'s companions rather than through a
//! blanket rejection here.

use crate::*;

use super::dynamic::collect_components;
use super::namespace::{collect_observed_namespaces, namespace_prelude_source};
use super::record::{import_meta_binding, rewrite_import_meta};
use super::source::strip_module_syntax;

/// Result of merging a linked graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedProgram {
    /// The merged script, when every unit lowered.
    pub script: Option<ScriptIr>,
    /// Diagnostics collected while lowering and merging.
    pub diagnostics: Vec<IrDiagnostic>,
}

/// The order module bodies run in: one entry per strongly-connected component,
/// listing its members.
///
/// For an acyclic graph every component holds exactly one module, which
/// degenerates to the obvious "dependencies first" list.
#[must_use]
pub fn evaluation_components(graph: &ModuleGraphIr) -> Vec<Vec<ModuleUnitId>> {
    let mut components = Vec::with_capacity(graph.scc_starts.len());
    for (position, start) in graph.scc_starts.iter().copied().enumerate() {
        let end = graph
            .scc_starts
            .get(position + 1)
            .copied()
            .unwrap_or(graph.evaluation_order.len());
        if start < end {
            components.push(graph.evaluation_order[start..end].to_vec());
        }
    }
    components
}

/// Script-goal source text for the whole linked graph, or the reasons it could
/// not be linked.
///
/// The two collectors run first and unconditionally: they are pure functions of
/// the linked graph, they are what makes `ProgramIr::modules` describe the
/// namespaces and `import()` components the program actually observes, and the
/// engine keeps `modules` on a failing program.
pub(crate) fn linked_script_source(
    sources: &ModuleGraphSources,
    graph: &mut ModuleGraphIr,
) -> Result<SourceUnit, Vec<IrDiagnostic>> {
    collect_components(graph);
    collect_observed_namespaces(graph);

    let mut diagnostics = Vec::new();
    check_linkable(graph, &mut diagnostics);
    diagnostics.extend(graph.check_dynamic_import_linkable());
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    // 16.2.1.6.1: module code is always strict.
    let mut text = String::from("\"use strict\";\n");

    // Namespace objects come first. Their getters are deferred, so nothing they
    // name has to be initialized yet, and the `import * as ns` aliases they
    // declare are object references rather than shares of an exporter cell, so
    // copying one loses no liveness.
    match namespace_prelude_source(graph) {
        Ok(prelude) => text.push_str(&prelude),
        Err(mut errors) => {
            diagnostics.append(&mut errors);
            return Err(diagnostics);
        }
    }

    // Every `import.meta` object exists before any body runs: a unit's body can
    // call a function of a unit whose own body has not run yet, and 13.3.12
    // gives that function an object either way.
    for unit in &graph.units {
        if unit.record.import_meta_uses() > 0 {
            text.push_str(&import_meta_binding(unit.record.id, &unit.meta_url).declaration);
            text.push('\n');
        }
    }

    // Dispatchers last of the three: their bodies name the namespace bindings
    // above, and a `function` declaration is hoisted anyway. One line, or empty.
    let dispatchers = graph.dynamic_import_prelude();
    if !dispatchers.is_empty() {
        text.push_str(&dispatchers);
        text.push('\n');
    }

    for (position, unit_id) in emission_order(graph).into_iter().enumerate() {
        let unit = graph.unit(unit_id);
        // `rewrite_import_meta` first and `strip_module_syntax` second: both
        // preserve byte length because both are addressed by spans the record
        // captured against the original text. `rewrite_dynamic_import_calls`
        // runs last, because it is the only one that changes length.
        let rewritten = rewrite_import_meta(&unit.source_text, &unit.record)
            .map_err(|error| error.reason)
            .and_then(|rewritten| strip_module_syntax(&rewritten).map_err(|error| error.reason))
            .and_then(|stripped| graph.rewrite_dynamic_import_calls(unit_id, &stripped));
        match rewritten {
            Ok(body) => {
                // An empty statement *between* units, never after the last one:
                // without it a unit ending in an expression without a semicolon
                // would swallow the next unit's first token through ASI, and
                // with it after the last unit the merged script's completion
                // value would be the separator's rather than the entry's.
                // `emission_order` is what guarantees the entry is last.
                if position > 0 {
                    text.push_str("\n;\n");
                }
                text.push_str(&body);
            }
            Err(reason) => diagnostics.push(IrDiagnostic::unsupported(format!(
                "unsupported in porffor wasm-aot: module {}: {reason}",
                unit.record.key
            ))),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(SourceUnit {
        goal: ParseGoal::Script,
        filename: sources
            .modules
            .get(sources.entry as usize)
            .map(|module| module.key.clone())
            .filter(|key| key != ANONYMOUS_MODULE_KEY),
        source_text: text,
    })
}

/// Unit ids in the order their bodies are emitted, entry last.
///
/// `evaluation_order` is a Tarjan post-order whose root loop visits *every*
/// unit rather than only the entry, and the entry is unit 0, so it is the first
/// root. A module reachable only through `import()` has no edge in
/// `requested_modules` at all, becomes a later root, and therefore lands *after*
/// the entry. Two things break if that order is emitted as-is: the merged
/// script's completion value stops being the entry's, and an eagerly evaluated
/// dynamic dependency runs after its own dependent. Rotating the entry's
/// strongly-connected component to the end fixes both, and is a no-op for a
/// graph whose entry statically reaches everything.
fn emission_order(graph: &ModuleGraphIr) -> Vec<ModuleUnitId> {
    let components = evaluation_components(graph);
    let Some(entry_position) = components
        .iter()
        .position(|component| component.contains(&graph.entry))
    else {
        return graph.evaluation_order.clone();
    };
    let mut order = Vec::with_capacity(graph.evaluation_order.len());
    for (position, component) in components.iter().enumerate() {
        if position != entry_position {
            order.extend(component.iter().copied());
        }
    }
    order.extend(components[entry_position].iter().copied());
    order
}

/// Reports every reason the graph cannot be merged into one top-level scope.
fn check_linkable(graph: &ModuleGraphIr, diagnostics: &mut Vec<IrDiagnostic>) {
    for unit in &graph.units {
        let key = &unit.record.key;
        // `import.meta` and dynamic `import()` are linked as generated Script
        // text by `modules::record` and `modules::dynamic`; each reports its own
        // remaining gaps (`rewrite_import_meta`'s span check and
        // `check_dynamic_import_linkable`) rather than being rejected wholesale
        // here.
        if !unit.record.star_export_entries.is_empty() {
            diagnostics.push(unsupported(key, "`export * from`"));
        }
        if unit
            .record
            .local_export_entries
            .iter()
            .any(|entry| entry.export_name == MODULE_DEFAULT_EXPORT_NAME)
        {
            diagnostics.push(unsupported(key, "`export default`"));
        }
        for (index, entry) in unit.record.import_entries.iter().enumerate() {
            if entry.import_name == ImportNameIr::Namespace {
                // Bound by the namespace prelude, not by the merged scope
                // sharing the exporter's cell — so the alias check below does
                // not apply, and falling through to it would report a
                // "renamed import binding" that is not one: a namespace import
                // resolves to `ModuleBindingNameIr::Namespace`, never to
                // `Name(local_name)`.
                continue;
            }
            // The merged scope binds by name, so an import is a live binding
            // exactly when the importer spells it the way the exporter does.
            match unit.resolved_imports.get(index) {
                Some(ResolvedBindingIr::Resolved {
                    binding: ModuleBindingNameIr::Name(name),
                    ..
                }) if *name == entry.local_name => {}
                Some(_) | None => diagnostics.push(unsupported(
                    key,
                    &format!("renamed import binding `{}`", entry.local_name),
                )),
            }
        }
    }

    // Two units cannot declare the same top-level name: the merged environment
    // holds one cell per name. Import bindings are excluded because they are
    // deliberately the exporting unit's cell.
    let mut owners: BTreeMap<&str, &str> = BTreeMap::new();
    for unit_id in &graph.evaluation_order {
        let unit = graph.unit(*unit_id);
        for binding in &unit.record.environment {
            if binding.kind == ModuleBindingKindIr::Import {
                continue;
            }
            if let Some(previous) = owners.insert(binding.name.as_str(), unit.record.key.as_str()) {
                if previous != unit.record.key {
                    diagnostics.push(IrDiagnostic::unsupported(format!(
                        "unsupported in porffor wasm-aot: modules {previous} and {} both declare top-level `{}`",
                        unit.record.key, binding.name
                    )));
                }
            }
        }
    }
}

fn unsupported(key: &str, feature: &str) -> IrDiagnostic {
    IrDiagnostic::unsupported(format!(
        "unsupported in porffor wasm-aot: module {key}: {feature}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources_of(
        sources: &[(&str, &str)],
        entry: usize,
        resolutions: Vec<(ModuleUnitId, ModuleRequestIr, ModuleUnitId)>,
    ) -> ModuleGraphSources {
        let modules = sources
            .iter()
            .map(|(key, text)| ModuleSourceIr {
                key: (*key).to_string(),
                source_text: (*text).to_string(),
                meta_url: format!("file:///{key}"),
            })
            .collect::<Vec<_>>();
        ModuleGraphSources {
            entry: ModuleUnitId::try_from(entry).expect("entry index fits"),
            modules,
            resolutions,
        }
    }

    fn graph_of(sources: &ModuleGraphSources) -> ModuleGraphIr {
        let mut graph = crate::modules::build_graph(sources).expect("graph should build");
        crate::modules::link(&mut graph);
        graph
    }

    fn plain(specifier: &str) -> ModuleRequestIr {
        ModuleRequestIr {
            specifier: specifier.to_string(),
            phase: ImportPhaseIr::Evaluation,
            attributes: Vec::new(),
        }
    }

    #[test]
    fn evaluation_components_lists_one_member_per_acyclic_unit() {
        let sources = sources_of(&[("m", "export const value = 1;")], 0, Vec::new());
        let graph = graph_of(&sources);
        let components = evaluation_components(&graph);
        assert_eq!(components, vec![vec![0]]);
    }

    #[test]
    fn a_two_unit_graph_links_dependencies_first() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("b", "import { value } from \"a\";\nprint(value + 1);"),
            ],
            1,
            vec![(1, plain("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");

        assert_eq!(linked.goal, ParseGoal::Script);
        assert!(linked.source_text.starts_with("\"use strict\";"));
        let exporter = linked
            .source_text
            .find("const value = 41;")
            .expect("exporter body is present");
        let importer = linked
            .source_text
            .find("print(value + 1);")
            .expect("importer body is present");
        assert!(
            exporter < importer,
            "dependency must be emitted before its importer"
        );
        assert!(
            !linked.source_text.contains("import"),
            "import declarations are deleted, got: {}",
            linked.source_text
        );
    }

    #[test]
    fn renamed_imports_are_reported_rather_than_mislinked() {
        let sources = sources_of(
            &[
                ("a", "const value = 1;\nexport { value as outer };"),
                ("b", "import { outer } from \"a\";\nprint(outer);"),
            ],
            1,
            vec![(1, plain("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let diagnostics =
            linked_script_source(&sources, &mut graph).expect_err("alias must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("renamed import binding")),
            "got {diagnostics:?}"
        );
    }

    #[test]
    fn colliding_top_level_names_are_reported() {
        let sources = sources_of(
            &[
                ("a", "const shared = 1;\nexport { shared };"),
                ("b", "const shared = 2;\nprint(shared);"),
            ],
            1,
            Vec::new(),
        );
        let mut graph = graph_of(&sources);
        let diagnostics =
            linked_script_source(&sources, &mut graph).expect_err("collision must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("both declare top-level")),
            "got {diagnostics:?}"
        );
    }

    /// `import()` is linked, not rejected: the call site becomes a call to the
    /// referrer's dispatcher, and the dispatcher resolves with the *same*
    /// namespace binding `import * as ns` would alias.
    #[test]
    fn dynamic_import_is_desugared_into_a_dispatcher_call() {
        let sources = sources_of(&[("m", "import('m');")], 0, vec![(0, plain("m"), 0)]);
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("import() should link");

        assert_eq!(graph.components.len(), 1);
        assert!(graph.units[0].namespace.is_some());
        assert!(
            linked
                .source_text
                .contains("function $porffor$module$import$0("),
            "got {}",
            linked.source_text
        );
        assert!(
            linked
                .source_text
                .contains(&format!("resolve({})", module_namespace_cell_name(0))),
            "got {}",
            linked.source_text
        );
        // The call site itself no longer spells `import`, so no `ImportCall`
        // node survives to the backend.
        assert!(
            linked
                .source_text
                .contains("$porffor$module$import$0('m');"),
            "got {}",
            linked.source_text
        );
    }

    /// `import * as ns` is linked through the namespace prelude, and the alias
    /// is a `const` naming the object rather than a share of an exporter cell.
    #[test]
    fn a_namespace_import_binds_its_local_to_the_namespace_object() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("c", "import * as ns from \"a\";\nprint(ns.value);"),
            ],
            1,
            vec![(1, plain("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("namespace should link");

        let cell = module_namespace_cell_name(0);
        assert!(
            linked
                .source_text
                .contains(&format!("const {cell} = Object.create(null);")),
            "got {}",
            linked.source_text
        );
        assert!(
            linked.source_text.contains(&format!("const ns = {cell};")),
            "got {}",
            linked.source_text
        );
    }

    /// A unit reachable only through `import()` is a separate Tarjan root and so
    /// lands after the entry in `evaluation_order`. The entry's component must
    /// still be emitted last, or the merged script takes the wrong completion
    /// value and runs a dependency after its dependent.
    #[test]
    fn the_entry_component_is_emitted_last_even_with_an_import_only_target() {
        let sources = sources_of(
            &[
                ("d", "import(\"a\");\nprint(\"entry\");"),
                ("a", "print(\"target\");"),
            ],
            0,
            vec![(0, plain("a"), 1)],
        );
        let mut graph = graph_of(&sources);
        // The precondition this test exists for: the entry is *not* last here.
        assert_eq!(graph.evaluation_order.first().copied(), Some(0));
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");

        let target = linked
            .source_text
            .find("print(\"target\")")
            .expect("target body is present");
        let entry = linked
            .source_text
            .find("print(\"entry\")")
            .expect("entry body is present");
        assert!(
            target < entry,
            "the entry component must be emitted last: {}",
            linked.source_text
        );
    }
}
