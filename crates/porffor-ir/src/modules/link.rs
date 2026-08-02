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
//!   whose top-level `this` is `globalThis`, so a synchronous unit that
//!   observes top-level `this` is reported rather than silently given the wrong
//!   value — see [`lowering::lower_module_graph`].
//! * A graph any of whose modules has `[[HasTLA]]` has its whole body wrapped
//!   in an immediately-invoked strict async function — see [`wrap_async_body`],
//!   which is also where the two deviations below stop applying, because such a
//!   body has a function scope and a `this` of its own.
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
//!
//! # Phased requests
//!
//! A unit's [`ModuleEvaluationModeIr`] decides *whether and how* its body is
//! emitted here, and `modules::graph` decides the mode from the phases of the
//! requests that reach the unit:
//!
//! * [`Eager`] — the body is emitted inline, in evaluation order. Everything an
//!   unphased graph contains.
//! * [`Deferred`] (`import defer * as ns from "m"`) — the body is emitted as a
//!   thunk that `m`'s namespace object calls on the first read of any export.
//!   See [`deferred_body_source`] for the shape and its deviations.
//! * [`NotEvaluated`] (`import source src from "m"`) — no body is emitted at
//!   all; only a module source object is declared. See
//!   `modules::namespace::module_source_object_source` for what that object is
//!   and is not.
//!
//! [`Eager`]: ModuleEvaluationModeIr::Eager
//! [`Deferred`]: ModuleEvaluationModeIr::Deferred
//! [`NotEvaluated`]: ModuleEvaluationModeIr::NotEvaluated

use crate::*;

use super::dynamic::collect_components;
use super::namespace::{
    collect_observed_namespaces, deferred_body_source, namespace_prelude_source,
};
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

    // Everything below the `"use strict"` prologue, so that an asynchronous
    // graph can be wrapped whole. See `wrap_async_body`.
    let mut text = String::new();

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

    let mut position = 0usize;
    for unit_id in emission_order(graph) {
        // A module reached only through `import source` is resolved, loaded,
        // parsed and linked, but never instantiated: it contributes no body.
        if graph.evaluation_mode(unit_id) == ModuleEvaluationModeIr::NotEvaluated {
            continue;
        }
        let unit = graph.unit(unit_id);
        // `rewrite_import_meta` first and `strip_module_syntax` second: both
        // preserve byte length because both are addressed by spans the record
        // captured against the original text. `rewrite_dynamic_import_calls`
        // runs last, because it is the only one that changes length.
        let rewritten = rewrite_import_meta(&unit.source_text, &unit.record)
            .map_err(|error| error.reason)
            .and_then(|rewritten| strip_module_syntax(&rewritten).map_err(|error| error.reason))
            .and_then(|stripped| graph.rewrite_dynamic_import_calls(unit_id, &stripped))
            // `import defer`: the body becomes a thunk the namespace calls.
            .and_then(|body| {
                if graph.evaluation_mode(unit_id) == ModuleEvaluationModeIr::Deferred {
                    deferred_body_source(graph, unit_id, &body)
                } else {
                    Ok(body)
                }
            });
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
                position += 1;
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

    // 16.2.1.6.1: module code is always strict. The prologue stays outside any
    // wrapper so it is still the merged script's first Directive Prologue item.
    let mut source_text = String::from("\"use strict\";\n");
    if graph.async_evaluation().iter().any(|unit| *unit) {
        source_text.push_str(&wrap_async_body(&text));
    } else {
        source_text.push_str(&text);
    }

    Ok(SourceUnit {
        goal: ParseGoal::Script,
        filename: sources
            .modules
            .get(sources.entry as usize)
            .map(|module| module.key.clone())
            .filter(|key| key != ANONYMOUS_MODULE_KEY),
        source_text,
    })
}

/// Wraps the merged graph body in an immediately-invoked async function, which
/// is what makes a top-level `await` legal in the Script-goal text this stage
/// produces.
///
/// # Why one wrapper for the whole graph
///
/// The merge concatenates unit bodies in `evaluation_order`, so the emitted
/// text already *is* the sequence `InnerModuleEvaluation` walks. Suspending on
/// an `await` inside that sequence therefore suspends exactly the modules that
/// come after it in dependency order, which is what
/// `[[PendingAsyncDependencies]]` and `AsyncModuleExecutionFulfilled` exist to
/// arrange: an importer of an asynchronous module does not run until that
/// module's body has completed. No separate driver is needed to get that
/// relation right, because the order was already fixed by Tarjan.
///
/// # Where it deviates
///
/// The spec *starts* every dependency's body before awaiting any of them, so
/// two independent asynchronous modules interleave: `a` runs to its first
/// `await`, then `b` runs to its first `await`, and only then does either
/// resume. One wrapper serializes that instead — `a` runs to completion before
/// `b` begins. Every module still observes its own dependencies as fully
/// evaluated, and a graph with at most one asynchronous chain is unaffected;
/// what changes is the interleaving of side effects between asynchronous
/// *siblings*.
///
/// A regular `function` rather than an arrow is deliberate. Module top-level
/// `this` is `undefined` (16.2.1.6.2), and a strict function called with no
/// receiver is the one construct that gives the merged Script text that value —
/// so an asynchronous graph gets the module `this` right where
/// `lower_module_graph` has to report it for a synchronous one. `var`
/// declarations likewise become function-scoped, which is the module
/// environment's behaviour rather than the merged script's.
fn wrap_async_body(body: &str) -> String {
    // `void` because the call's value is the module's `[[TopLevelCapability]]`
    // promise, and an `ExpressionStatement` yielding it would make that promise
    // the merged script's completion value. A module evaluates to no value.
    //
    // The newline before `}` closes any unit body that ended in an expression
    // without a semicolon: ASI applies at the `}`, exactly as it already does
    // at the end of the unwrapped merged script.
    format!("void (async function () {{\n{body}\n}})();\n")
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
        // Nothing of a source-phase-only module reaches the merged script: no
        // body, no namespace, no binding. Its `export default` and its
        // `export * from` are therefore not this stage's problem, and rejecting
        // the graph over them would refuse `import source` of exactly the
        // ordinary modules it exists to take a handle on. Its *parse* still had
        // to succeed, which is where the source phase's real errors come from.
        if graph.evaluation_mode(unit.record.id) == ModuleEvaluationModeIr::NotEvaluated {
            continue;
        }
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
            if entry.request.phase == ImportPhaseIr::Source {
                // Bound by the module-source prelude. `[[ImportName]]` is
                // `default` only because the grammar reuses `ImportedBinding`;
                // nothing is resolved against the requested module's exports,
                // so the alias check below would report a rename that is not
                // one.
                continue;
            }
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
        // A deferred unit's bindings live in its thunk's scope and a
        // source-phase-only unit has no body at all, so neither can collide
        // with anything in the merged scope.
        if graph.evaluation_mode(*unit_id) != ModuleEvaluationModeIr::Eager {
            continue;
        }
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

    /// A graph with no `[[HasTLA]]` module keeps the flat merged body: the
    /// wrapper is not paid for by programs that do not need it.
    #[test]
    fn a_synchronous_graph_is_not_wrapped() {
        let sources = sources_of(&[("m", "print(1);")], 0, Vec::new());
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        assert!(
            !linked.source_text.contains("async function"),
            "got {}",
            linked.source_text
        );
    }

    /// Top-level `await` links instead of being reported, and the merged text
    /// is Script-legal because the whole body became an async function.
    #[test]
    fn a_top_level_await_module_is_wrapped_in_an_async_body() {
        let sources = sources_of(
            &[("m", "const value = await 1;\nprint(value);")],
            0,
            Vec::new(),
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("top-level await links");

        assert!(linked.source_text.starts_with("\"use strict\";"));
        assert!(
            linked
                .source_text
                .contains("void (async function () {\nconst value = await 1;"),
            "got {}",
            linked.source_text
        );
        assert!(
            linked.source_text.trim_end().ends_with("})();"),
            "got {}",
            linked.source_text
        );
    }

    /// `[[AsyncEvaluation]]` is transitive: an importer of an asynchronous
    /// module is asynchronous too, so a graph whose *dependency* holds the
    /// `await` is wrapped as a whole and the importer's body is emitted after
    /// the `await` that must precede it.
    #[test]
    fn an_importer_of_an_asynchronous_module_is_wrapped_and_ordered_after_it() {
        let sources = sources_of(
            &[
                ("a", "export const value = await 7;"),
                ("b", "import { value } from \"a\";\nprint(value + 1);"),
            ],
            1,
            vec![(1, plain("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(graph.async_evaluation(), vec![true, true]);
        assert_eq!(graph.pending_async_dependencies(1), 1);

        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        let exporter = linked
            .source_text
            .find("await 7")
            .expect("exporter body is present");
        let importer = linked
            .source_text
            .find("print(value + 1);")
            .expect("importer body is present");
        assert!(
            exporter < importer,
            "the awaited dependency must precede its importer: {}",
            linked.source_text
        );
        assert!(
            linked.source_text.contains("void (async function () {"),
            "got {}",
            linked.source_text
        );
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

    fn phased(specifier: &str, phase: ImportPhaseIr) -> ModuleRequestIr {
        ModuleRequestIr {
            specifier: specifier.to_string(),
            phase,
            attributes: Vec::new(),
        }
    }

    /// `import defer`: the dependency's body still reaches the merged script,
    /// but wrapped in the thunk its namespace getters call, so nothing of it
    /// runs until an export is read.
    #[test]
    fn a_deferred_dependency_body_is_emitted_as_a_thunk() {
        let sources = sources_of(
            &[
                ("a", "print(\"side effect\");\nexport const value = 41;"),
                (
                    "d",
                    "import defer * as ns from \"a\";\nprint(\"entry\");\nprint(ns.value);",
                ),
            ],
            1,
            vec![(1, phased("a", ImportPhaseIr::Defer), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(
            graph.evaluation_mode(0),
            ModuleEvaluationModeIr::Deferred,
            "{:?}",
            graph.evaluation_modes
        );
        let linked = linked_script_source(&sources, &mut graph).expect("defer should link");

        let evaluate = module_defer_evaluate_function_name(0);
        assert!(
            linked
                .source_text
                .contains(&format!("let {};", module_defer_cells_cell_name(0))),
            "got {}",
            linked.source_text
        );
        // The body is inside the thunk, not at top level.
        let thunk = linked
            .source_text
            .find(&format!("function {evaluate}()"))
            .expect("thunk is present");
        let side_effect = linked
            .source_text
            .find("print(\"side effect\")")
            .expect("dependency body is present");
        assert!(thunk < side_effect, "got {}", linked.source_text);
        // And the namespace getter is what calls it.
        assert!(
            linked
                .source_text
                .contains(&format!("get: () => {evaluate}()[\"value\"]()")),
            "got {}",
            linked.source_text
        );
    }

    /// `import source`: the module is resolved, loaded, parsed and linked, and
    /// then nothing of it is emitted at all — including the `export default`
    /// the merged scope could not have linked, which is not this stage's
    /// problem when no body, namespace or binding of the module is emitted.
    #[test]
    fn a_source_phase_dependency_contributes_no_body() {
        let sources = sources_of(
            &[
                (
                    "a",
                    "print(\"must not run\");\nexport default 1;\nexport const value = 41;",
                ),
                ("d", "import source src from \"a\";\nprint(typeof src);"),
            ],
            1,
            vec![(1, phased("a", ImportPhaseIr::Source), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(
            graph.evaluation_mode(0),
            ModuleEvaluationModeIr::NotEvaluated
        );
        let linked = linked_script_source(&sources, &mut graph).expect("source phase should link");

        assert!(
            !linked.source_text.contains("must not run"),
            "got {}",
            linked.source_text
        );
        let cell = module_source_cell_name(0);
        assert!(
            linked
                .source_text
                .contains(&format!("const {cell} = Object.create(null);")),
            "got {}",
            linked.source_text
        );
        assert!(
            linked.source_text.contains(&format!("const src = {cell};")),
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
