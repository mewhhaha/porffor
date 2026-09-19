//! Merging a linked graph into the single `ScriptIr` the backend emits.
//!
//! # Synchronous instantiation
//!
//! A Module-entry graph uses private module activations when every unit is
//! synchronous and no request uses the source phase. That path allocates every
//! module environment before linking import cells and constructing namespaces,
//! then evaluates the selected module and its dependencies in request order.
//! Static evaluation components retain Evaluating state until the first active
//! evaluator completes the component or caches the same throw on entered members.
//! `synchronous_source` records trusted source spans; `synchronous_definition`
//! turns those exact AST owners into typed private operations. The existing
//! dynamic import dispatchers retain their Promise and coercion behavior.
//!
//! The source-merge constraints and retained drivers described below apply to
//! graphs outside that bounded instantiation path.
//!
//! # Retained artifact strategy
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
//! * Module-entry graphs use a private arrow owner, so their declarations stay
//!   outside the global Script environment. The Module goal's root-`this`
//!   binding supplies `undefined` through every lexical arrow, and no implicit
//!   `arguments` binding is introduced.
//! * A graph with `[[HasTLA]]` uses an async arrow owner instead; see
//!   [`wrap_async_body`] for the retained asynchronous scheduling limits.
//! * Script-entry module closures and deferred units retain their existing
//!   strict ordinary-function wrappers.
//!
//! A retained merged owner still shares one lexical environment between module
//! units. A dependency's free name can therefore resolve to an unrelated
//! importer's declaration. The private owner separates the global Script from
//! the graph; it does not supply per-module scope isolation.
//!
//! # Renamed bindings and `export default`
//!
//! The merged scope binds by name, so an import whose importer-side name
//! differs from the exporter-side name (`import { a as b }`, `export { a as b }`
//! re-exported onward, and every default import) has no cell of its own to
//! share. JavaScript has exactly one construct that makes a bare
//! `IdentifierReference` read through a function: an accessor property of the
//! global object. [`binding_alias_prelude`] emits one per alias, so a renamed
//! import stays a *live* read of the exporter's cell — a copied `const` would
//! not be — and stays read-only, which is what an import binding is.
//!
//! Two things follow from the alias being a global property rather than a
//! declaration, and [`collect_binding_aliases`] reports both rather than
//! mislinking them: an alias is hidden by any top-level declaration of the same
//! name anywhere in the graph, and two aliases of the same name must name the
//! same export. Installing an alias also mutates the global object instead of
//! creating an importing-module cell, so it can conflict with an earlier Script
//! binding. The private merged owner does not repair that retained-driver gap.
//!
//! `export default` binds `*default*` (8.2.2), which no source text can spell,
//! so `modules::source` rewrites the two keywords into a declaration of
//! `MergedName::anonymous_default` in place — the named forms
//! (`export default function f() {}`) already bind their own name and only lose
//! the keywords. `export * from` needs nothing here at all: `GetExportedNames`
//! and `ResolveExport` already walk star paths, so the importer resolves
//! straight through to the originating unit's cell.
//!
//! # Retained driver constraints
//!
//! * two units that declare the same top-level name, which the merged scope
//!   cannot hold side by side without a renaming pass over the unit bodies.
//!
//! Namespace objects, `import.meta` and dynamic `import()` are linked through
//! generated Script text and trusted compiler metadata. Namespace initializer
//! spans lower to explicit constructor IR with private live-binding readers;
//! ordinary source arrays never receive namespace semantics.
//! `modules::namespace` owns that prelude, `modules::record` owns the
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

use super::evaluation_mode::ModuleMaterializationModeIr;
use super::module_key::ANONYMOUS_MODULE_KEY;
use super::namespace::{
    collect_observed_namespaces, deferred_body_source, namespace_prelude_source,
    namespace_target_reference, shadows_prelude_global,
};
use super::record::{import_meta_binding, rewrite_import_meta, DefaultExportFormIr};
use super::source::DefaultExportRewrite;

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

#[derive(Debug)]
pub(crate) struct LinkedScriptSource {
    pub(crate) source: SourceUnit,
    pub(crate) definitions: super::LinkedScriptDefinitions,
}

/// Script-goal source text for the whole linked graph, or the reasons it could
/// not be linked.
///
/// Namespace collection runs first and unconditionally. Dynamic components
/// were already discovered for evaluation-mode classification and filtered to
/// materialized referrers by `modules::graph::link`; together those steps make
/// `ProgramIr::modules` describe exactly the runtime objects and call sites the
/// artifact can observe, including on a failing program.
pub(crate) fn linked_script_source(
    sources: &ModuleGraphSources,
    graph: &mut ModuleGraphIr,
) -> Result<LinkedScriptSource, Vec<IrDiagnostic>> {
    collect_observed_namespaces(graph);

    // Eligibility uses the same complete discovered edge set as classification;
    // artifact components have already dropped unreachable referrers here.
    if let Some(eligible) = super::synchronous_source::SynchronousInstantiationGraph::new(
        graph,
        &super::dynamic::discover_components(graph),
    ) {
        return super::synchronous_source::linked_synchronous_source(sources, eligible);
    }

    let mut diagnostics = Vec::new();
    check_linkable(graph, &mut diagnostics);
    let aliases = collect_binding_aliases(graph, &mut diagnostics);
    diagnostics.extend(graph.check_dynamic_import_linkable());
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    // Everything below the `"use strict"` prologue, so that an asynchronous
    // graph can be wrapped whole. See `wrap_async_body`.
    let mut text = String::new();
    let mut definitions = super::LinkedScriptDefinitions::default();

    // Namespace objects come first. Their getters are deferred, so nothing they
    // name has to be initialized yet, and the `import * as ns` aliases they
    // declare are object references rather than shares of an exporter cell, so
    // copying one loses no liveness.
    match namespace_prelude_source(graph) {
        Ok(prelude) => {
            definitions.record_namespaces(&prelude, graph);
            text.push_str(&prelude);
        }
        Err(mut errors) => {
            diagnostics.append(&mut errors);
            return Err(diagnostics);
        }
    }

    // Renamed-binding aliases next. Their getters are deferred too, so nothing
    // they name has to be initialized yet — and being deferred is the whole
    // point: an alias has to read the exporter's cell at every read, not once.
    text.push_str(&binding_alias_prelude(&aliases));

    // Every materialized unit's `import.meta` object exists before any body
    // runs: a unit's body can call a function of a unit whose own body has not
    // run yet, and 13.3.12 gives that function an object either way. A
    // source-only unit has no body or environment and therefore no object.
    for (_, _, unit) in graph.materialized_units() {
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
    // The Script entry of a script graph, kept aside: it is emitted after the
    // wrapper that holds every module, not inside it.
    let mut script_entry_body = String::new();
    for unit_id in emission_order(graph) {
        // A module reached only through `import source` is resolved, loaded,
        // parsed and linked, but never instantiated: it contributes no body.
        let Some(mode) = graph.materialization_mode(unit_id) else {
            continue;
        };
        let unit = graph.unit(unit_id);
        if graph.entry_is_script && unit_id == graph.entry {
            // Script text, emitted as itself: there is no module syntax to
            // strip, no `import.meta` to rewrite (it is a SyntaxError in a
            // Script), and no `export default` to bind. Only the `import()`
            // call sites move, onto the dispatchers the wrapper exports.
            match graph.rewrite_script_entry_import_calls(&unit.source_text) {
                Ok(body) => script_entry_body = body,
                Err(reason) => diagnostics.push(IrDiagnostic::unsupported(format!(
                    "unsupported in lila wasm-aot: script {}: {reason}",
                    unit.record.key.as_str()
                ))),
            }
            continue;
        }
        // The only other caller of `MergedName::anonymous_default`; the one in
        // `LocalName::merged_in` is what every reader of this binding goes
        // through, and the two agree by construction.
        let default_name = MergedName::anonymous_default(unit_id);
        let default_export = match unit.record.default_export_form() {
            DefaultExportFormIr::Absent => DefaultExportRewrite::None,
            DefaultExportFormIr::Named => DefaultExportRewrite::DeleteKeywords,
            DefaultExportFormIr::Anonymous { hoisted } => DefaultExportRewrite::Bind {
                name: &default_name,
                hoisted,
            },
        };
        // import.meta consumes original source spans first. The definition
        // rewrite then preserves callable source while terminating anonymous
        // declarations. Dynamic import rescans the resulting text, cross-checking
        // its call counts and phases against the original record.
        let rewritten = rewrite_import_meta(&unit.source_text, &unit.record)
            .map_err(|error| error.reason)
            .and_then(|rewritten| {
                super::LinkedScriptDefinitions::rewrite_body(&rewritten, default_export)
            })
            .and_then(|stripped| graph.rewrite_dynamic_import_calls(unit_id, &stripped))
            // `import defer`: the body becomes a thunk the namespace calls.
            .and_then(|body| match mode {
                ModuleMaterializationModeIr::Eager => Ok(body),
                ModuleMaterializationModeIr::Deferred => {
                    deferred_body_source(graph, unit_id, &body)
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
                if let Err(reason) = definitions.record_body(
                    &body,
                    unit_id,
                    mode,
                    unit.record.default_export_form(),
                    &text,
                ) {
                    diagnostics.push(IrDiagnostic::lowering(reason));
                }
                text.push_str(&body);
                position += 1;
            }
            Err(reason) => diagnostics.push(IrDiagnostic::unsupported(format!(
                "unsupported in lila wasm-aot: module {}: {reason}",
                unit.record.key.as_str()
            ))),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let asynchronous = graph.async_evaluation().iter().any(|unit| *unit);
    let source_text = if graph.entry_is_script {
        if asynchronous {
            // The wrapper would have to be an async function, and the Script
            // after it would then run before the modules had finished — so an
            // `import()` served from it would resolve over uninitialized
            // bindings. Refusing beats answering with a TDZ.
            return Err(vec![IrDiagnostic::unsupported(
                "unsupported in lila wasm-aot: a script's `import()` target has a top-level \
                 `await`",
            )]);
        }
        wrap_script_graph_modules(graph, &text, &mut definitions) + &script_entry_body
    } else {
        // 16.2.1.6.1: module code is always strict. The prologue stays outside
        // any wrapper so it is still the merged script's first Directive
        // Prologue item.
        let mut source_text = String::from("\"use strict\";\n");
        definitions.entry = Some(super::LinkedModuleEntry::RetainedDriver(if asynchronous {
            ModuleEntryEvaluationKindIr::Promise
        } else {
            ModuleEntryEvaluationKindIr::Synchronous
        }));
        if asynchronous {
            source_text.push_str(&wrap_async_body(&text, &mut definitions));
        } else {
            source_text.push_str(&wrap_sync_body(&text, &mut definitions));
        }
        definitions.prepend("\"use strict\";\n");
        source_text
    };

    Ok(LinkedScriptSource {
        definitions,
        source: SourceUnit {
            goal: ParseGoal::Script,
            filename: sources
                .modules
                .get(sources.entry as usize)
                .map(|module| module.key().as_str().to_string())
                .filter(|key| key != ANONYMOUS_MODULE_KEY),
            source_text,
        },
    })
}

/// Wraps every module of a *script* graph in one immediately-invoked strict
/// function, and hands the entry Script the dispatchers it has to call.
///
/// A script graph is a Script that writes `import()` plus the closure of the
/// modules those calls name. The Script itself is not module code and must not
/// be made strict, must keep `globalThis` as its top-level `this`, and must keep
/// its own top-level scope; the modules are module code and must be strict and
/// must not leak their bindings into the Script's scope. One wrapper gives both:
///
/// * `"use strict"` inside it makes every module strict (16.2.1.6.1) without
///   touching the Script that follows;
/// * a plain call gives the wrapper a `this` of `undefined`, which is module
///   top-level `this` (16.2.1.6.2); Module-entry wrappers instead inherit
///   `undefined` through the Module goal's lexical root binding;
/// * `var` and function declarations in a module body become function-scoped,
///   which is the module environment's behaviour rather than the global
///   object's.
///
/// The `var` declarations in front are how the Script reaches a dispatcher: a
/// `function` declaration inside the wrapper is not in scope outside it, so the
/// wrapper assigns each one out. They are declared even before the wrapper runs,
/// so a Script that calls `import()` at its very first statement still finds a
/// function there.
///
/// Emitted with no interior line terminator apart from the module material's
/// own, so the Script's line numbers are displaced by a fixed amount.
fn wrap_script_graph_modules(
    graph: &ModuleGraphIr,
    modules: &str,
    names: &mut super::LinkedScriptDefinitions,
) -> String {
    let exports = graph.script_entry_dispatcher_exports();
    if exports.is_empty() && modules.trim().is_empty() {
        return String::new();
    }
    let mut text = String::new();
    if !exports.is_empty() {
        text.push_str("var ");
        for (position, (exported, _)) in exports.iter().enumerate() {
            if position > 0 {
                text.push_str(", ");
            }
            text.push_str(exported);
        }
        text.push_str(";\n");
    }
    text.push_str("(function () { \"use strict\";\n");
    names.prepend(&text);
    text.push_str(modules);
    text.push('\n');
    for (exported, dispatcher) in &exports {
        text.push_str(exported);
        text.push_str(" = ");
        text.push_str(dispatcher);
        text.push_str("; ");
    }
    text.push_str("\n})();\n");
    text
}

/// A private arrow keeps the retained driver's shared module declarations out
/// of the global Script environment. It inherits the Module goal's undefined
/// `this` and does not introduce an `arguments` or `new.target` binding.
fn wrap_sync_body(body: &str, names: &mut super::LinkedScriptDefinitions) -> String {
    let prefix = "void (() => {\n";
    names.prepend(prefix);
    format!("{prefix}{body}\n}})();\n")
}

/// Wraps the merged graph body in an immediately-invoked async arrow, which
/// is what makes a top-level `await` legal in the Script-goal text this stage
/// produces while retaining Module lexical `this` and `arguments` semantics.
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
/// The arrow inherits `undefined` from the Module goal's lexical root binding.
/// It introduces no implicit `arguments` binding, and its `var`, function and
/// lexical declarations remain separate from an earlier global Script. The
/// retained driver still shares this one environment between its module units.
fn wrap_async_body(body: &str, names: &mut super::LinkedScriptDefinitions) -> String {
    // The trusted entry boundary replaces this `void` with a private operation
    // that adopts the driver promise and yields undefined. The source spelling
    // itself grants no authority to capture a Script's ordinary async calls.
    //
    // The newline before `}` closes any unit body that ended in an expression
    // without a semicolon: ASI applies at the `}`, exactly as it already does
    // at the end of the unwrapped merged script.
    let prefix = "void (async () => {\n";
    names.prepend(prefix);
    format!("{prefix}{body}\n}})();\n")
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
///
/// `import.meta`, dynamic `import()`, `export default` and `export * from` are
/// all linked, each by the stage that owns it: `modules::record` rewrites
/// `import.meta`, `modules::dynamic` desugars `import()`, `modules::source`
/// rewrites the `export default` keywords, and `GetExportedNames` /
/// `ResolveExport` walk star paths before this stage sees them. Each reports its
/// own remaining gaps rather than being rejected wholesale here.
fn check_linkable(graph: &ModuleGraphIr, diagnostics: &mut Vec<IrDiagnostic>) {
    // Two units cannot declare the same top-level name: the merged environment
    // holds one cell per name. Import bindings are excluded because they are
    // deliberately the exporting unit's cell.
    let mut owners: BTreeMap<MergedName, &str> = BTreeMap::new();
    for (unit_id, mode, unit) in graph.materialized_units() {
        match mode {
            ModuleMaterializationModeIr::Eager => {}
            // A deferred unit's bindings live in its thunk scope and cannot
            // collide with declarations in the merged top level.
            ModuleMaterializationModeIr::Deferred => continue,
        }
        // The Script entry of a script graph declares into the Script's own
        // top-level scope, outside the wrapper the modules share, so its names
        // collide with nothing here. (A *global* alias it shadows is a real
        // problem, and `collect_binding_aliases` still sees it.)
        if graph.entry_is_script && unit_id == graph.entry {
            continue;
        }
        for binding in &unit.record.environment {
            if binding.kind == ModuleBindingKindIr::Import {
                continue;
            }
            // The merged spelling, not the spec one: two units with an
            // anonymous `export default` both name it `*default*` and would
            // collide over a name neither of them declares. The type is what
            // says which of the two this is.
            let name = unit.record.merged(&binding.name);
            if let Some(previous) = owners.insert(name.clone(), unit.record.key.as_str()) {
                if previous != unit.record.key.as_str() {
                    diagnostics.push(IrDiagnostic::unsupported(format!(
                        "unsupported in lila wasm-aot: modules {previous} and {} both declare top-level `{}`",
                        unit.record.key.as_str(),
                        name.as_str()
                    )));
                }
            }
        }
    }
}

/// `(local name, expression it reads)` for every import binding whose local
/// name is not the exporter's own name.
///
/// A namespace or module-source import is *not* one of these: the namespace
/// prelude binds it as a `const` naming an object, and copying an object
/// reference loses nothing. What is left is exactly the value imports, which
/// have to stay live.
///
/// Every rejection below is a case where the alias would be *shadowed* rather
/// than merely absent, which is the one outcome worse than reporting it: the
/// importer would silently read some other module's binding.
fn collect_binding_aliases(
    graph: &ModuleGraphIr,
    diagnostics: &mut Vec<IrDiagnostic>,
) -> Vec<(MergedName, MergedName)> {
    let declared = merged_lexical_names(graph);
    let mut aliases: Vec<(MergedName, MergedName)> = Vec::new();
    let mut owners: BTreeMap<MergedName, (MergedName, String)> = BTreeMap::new();

    for (_, _, unit) in graph.materialized_units() {
        let key = unit.record.key.as_str();
        for (index, entry) in unit.record.import_entries.iter().enumerate() {
            if entry.request.phase() == ImportPhaseIr::Source {
                // Bound by the module-source prelude. `[[ImportName]]` is
                // `default` only because the grammar reuses `ImportedBinding`;
                // nothing is resolved against the requested module's exports,
                // so this is never a rename.
                continue;
            }
            let Some(resolved) = unit.resolved_imports.get(index) else {
                continue;
            };
            // A namespace or module-source resolution is the namespace
            // prelude's business, and an unresolved one is already a link error
            // (`ResolveExport` reported it) rather than this stage's.
            if !matches!(
                resolved,
                ResolvedBindingIr::Resolved {
                    binding: ModuleBindingNameIr::Name(_),
                    ..
                }
            ) {
                continue;
            }
            let Some(reference) = namespace_target_reference(resolved) else {
                diagnostics.push(unsupported(
                    key,
                    &format!(
                        "import binding `{}` resolves to a name the merged script cannot spell",
                        entry.local_name.spec_name()
                    ),
                ));
                continue;
            };
            // The importer spells it the way the exporter does, so the merged
            // scope already shares the exporter's cell: no alias needed.
            //
            // This comparison is D3 against D3, written out. It used to be a
            // `String` `reference` compared against a `String` `local_name`,
            // i.e. a merged name against a spec name, and was right only
            // because the two coincide on a source-spelled binding. An import
            // binding is always source-spelled, so `merged_in` is the identity
            // here — but that is now a fact the reader can see rather than one
            // the code depended on silently.
            let local = unit.record.merged(&entry.local_name);
            if reference == local {
                continue;
            }
            if shadows_prelude_global(local.as_str()) {
                diagnostics.push(unsupported(
                    key,
                    &format!(
                        "renamed import binding `{}` is a global the merged script's own prelude spells",
                        entry.local_name.spec_name()
                    ),
                ));
                continue;
            }
            if let Some(owner) = declared.get(&local) {
                diagnostics.push(unsupported(
                    key,
                    &format!(
                        "renamed import binding `{}` is shadowed by a top-level declaration in module {owner}",
                        entry.local_name.spec_name()
                    ),
                ));
                continue;
            }
            match owners.get(&local) {
                Some((previous, owner)) if *previous != reference => {
                    diagnostics.push(unsupported(
                        key,
                        &format!(
                            "renamed import binding `{}` already names a different export in module {owner}",
                            entry.local_name.spec_name()
                        ),
                    ));
                }
                // Two importers of the same export under the same local name
                // want the same alias, so one definition serves both.
                Some(_) => {}
                None => {
                    owners.insert(local.clone(), (reference.clone(), key.to_string()));
                    aliases.push((local, reference));
                }
            }
        }
    }
    aliases
}

/// Every name the merged top-level scope declares outright, and the module that
/// declares it.
///
/// Only an eagerly evaluated unit contributes: a deferred unit's declarations
/// live inside its thunk and a source-phase-only unit has no body. A deferred
/// unit's own aliases are safe from its own thunk-local names, because a module
/// that both imports and declares one name is an early error.
fn merged_lexical_names(graph: &ModuleGraphIr) -> BTreeMap<MergedName, String> {
    let mut declared = BTreeMap::new();
    for (_, mode, unit) in graph.materialized_units() {
        match mode {
            ModuleMaterializationModeIr::Eager => {}
            ModuleMaterializationModeIr::Deferred => continue,
        }
        for binding in &unit.record.environment {
            if binding.kind != ModuleBindingKindIr::Import {
                declared.insert(
                    unit.record.merged(&binding.name),
                    unit.record.key.as_str().to_string(),
                );
            }
        }
        // A namespace or module-source alias is a real `const` in the merged
        // scope too — see `modules::namespace`.
        for (index, entry) in unit.record.import_entries.iter().enumerate() {
            if matches!(
                unit.resolved_imports.get(index),
                Some(ResolvedBindingIr::Resolved {
                    binding: ModuleBindingNameIr::Namespace(_) | ModuleBindingNameIr::ModuleSource,
                    ..
                })
            ) {
                declared.insert(
                    unit.record.merged(&entry.local_name),
                    unit.record.key.as_str().to_string(),
                );
            }
        }
    }
    declared
}

/// Merged-script prelude defining one global accessor per renamed binding.
///
/// An accessor rather than a `const` copy because an import is a *live* view of
/// the exporter's cell, and no setter because an import binding is immutable.
/// Non-enumerable and non-configurable so the property is as close to invisible
/// as a global property gets.
fn binding_alias_prelude(aliases: &[(MergedName, MergedName)]) -> String {
    let mut text = String::new();
    for (local, reference) in aliases {
        text.push_str(OBJECT_NAME);
        text.push_str(".defineProperty(");
        text.push_str(GLOBAL_THIS_NAME);
        text.push_str(", \"");
        // A local name is a `BindingIdentifier`, so it holds nothing a string
        // literal would have to escape.
        text.push_str(local.as_str());
        text.push_str("\", { get: () => ");
        text.push_str(reference.as_str());
        text.push_str(", enumerable: false, configurable: false });\n");
    }
    text
}

fn unsupported(key: &str, feature: &str) -> IrDiagnostic {
    IrDiagnostic::unsupported(format!(
        "unsupported in lila wasm-aot: module {key}: {feature}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources_of(
        sources: &[(&str, &str)],
        entry: usize,
        resolutions: Vec<(ModuleUnitId, ModuleRequestKeyIr, ModuleUnitId)>,
    ) -> ModuleGraphSources {
        let modules = sources
            .iter()
            .map(|(key, text)| {
                ModuleSourceIr::new(
                    ModuleKey::from_host(*key),
                    (*text).to_string(),
                    format!("file:///{key}"),
                )
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

    fn request_key(specifier: &str) -> ModuleRequestKeyIr {
        ModuleRequestKeyIr::plain(specifier)
    }

    #[test]
    fn evaluation_components_lists_one_member_per_acyclic_unit() {
        let sources = sources_of(&[("m", "export const value = 1;")], 0, Vec::new());
        let graph = graph_of(&sources);
        let components = evaluation_components(&graph);
        assert_eq!(components, vec![vec![0]]);
    }

    #[test]
    fn a_synchronous_graph_keeps_its_declarations_in_a_private_activation() {
        let sources = sources_of(&[("m", "print(1);")], 0, Vec::new());
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        assert_eq!(definitions.units.len(), 1);
        assert_eq!(definitions.initial_evaluation, [0]);
        assert_eq!(definitions.units[0].evaluation.module(), 0);
        assert_eq!(definitions.units[0].evaluation.component_members(), [0]);
        assert!(linked.source.source_text.contains("print(1);"));
    }

    /// Top-level `await` links instead of being reported, and the merged text
    /// is Script-legal because the whole body became an async arrow.
    #[test]
    fn a_top_level_await_module_is_wrapped_in_an_async_body() {
        let sources = sources_of(
            &[("m", "const value = await 1;\nprint(value);")],
            0,
            Vec::new(),
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("top-level await links");

        assert!(linked.source.source_text.starts_with("\"use strict\";"));
        assert!(
            linked
                .source
                .source_text
                .contains("void (async () => {\nconst value = await 1;"),
            "got {}",
            linked.source.source_text
        );
        assert!(
            linked.source.source_text.trim_end().ends_with("})();"),
            "got {}",
            linked.source.source_text
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
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(graph.async_evaluation(), vec![true, true]);
        assert_eq!(graph.pending_async_dependencies(1), 1);

        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        let exporter = linked
            .source
            .source_text
            .find("await 7")
            .expect("exporter body is present");
        let importer = linked
            .source
            .source_text
            .find("print(value + 1);")
            .expect("importer body is present");
        assert!(
            exporter < importer,
            "the awaited dependency must precede its importer: {}",
            linked.source.source_text
        );
        assert!(
            linked.source.source_text.contains("void (async () => {"),
            "got {}",
            linked.source.source_text
        );
    }

    #[test]
    fn a_two_unit_graph_visits_its_dependencies_through_the_entry_evaluator() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("b", "import { value } from \"a\";\nprint(value + 1);"),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        assert_eq!(linked.source.goal, ParseGoal::Script);
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        assert_eq!(definitions.initial_evaluation, [1]);
        let entry = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        assert_eq!(entry.evaluation.dependencies(), [0]);
        assert!(!linked.source.source_text.contains("import"));
    }

    /// A renamed import resolves to the exporter's canonical environment cell.
    #[test]
    fn renamed_imports_resolve_to_private_exporter_cells() {
        let sources = sources_of(
            &[
                ("a", "let value = 1;\nexport { value as outer };"),
                ("b", "import { outer } from \"a\";\nprint(outer);"),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("alias should link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let entry = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        assert_eq!(entry.imports.len(), 1);
        assert_eq!(
            namespace_target_reference(&entry.imports[0])
                .unwrap()
                .as_str(),
            "value"
        );
        assert!(!linked
            .source
            .source_text
            .contains("Object.defineProperty(globalThis"));
        assert!(linked.source.source_text.contains("print(outer);"));
    }

    /// Unrelated module declarations cannot shadow a private import cell.
    #[test]
    fn an_alias_is_separate_from_another_modules_top_level_declaration() {
        let sources = sources_of(
            &[
                ("a", "export const value = 1;"),
                ("c", "const outer = 99;\nprint(outer);"),
                (
                    "b",
                    "import { value as outer } from \"a\";\nimport \"c\";\nprint(outer);",
                ),
            ],
            2,
            vec![(2, request_key("a"), 0), (2, request_key("c"), 1)],
        );
        let mut graph = graph_of(&sources);
        let linked =
            linked_script_source(&sources, &mut graph).expect("separate module bindings link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        assert_eq!(definitions.units.len(), 3);
        let entry = definitions
            .units
            .iter()
            .find(|unit| unit.module == 2)
            .unwrap();
        assert_eq!(
            namespace_target_reference(&entry.imports[0])
                .unwrap()
                .as_str(),
            "value"
        );
    }

    /// Import-local spellings do not need to be unique across a graph.
    #[test]
    fn two_modules_can_alias_the_same_name_to_different_exports() {
        let sources = sources_of(
            &[
                ("a", "export const first = 1;\nexport const second = 2;"),
                ("c", "import { first as z } from \"a\";\nprint(z);"),
                (
                    "b",
                    "import { second as z } from \"a\";\nimport \"c\";\nprint(z);",
                ),
            ],
            2,
            vec![
                (1, request_key("a"), 0),
                (2, request_key("a"), 0),
                (2, request_key("c"), 1),
            ],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("private aliases link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let imports = |module| {
            &definitions
                .units
                .iter()
                .find(|unit| unit.module == module)
                .unwrap()
                .imports
        };
        assert_eq!(
            namespace_target_reference(&imports(1)[0]).unwrap().as_str(),
            "first"
        );
        assert_eq!(
            namespace_target_reference(&imports(2)[0]).unwrap().as_str(),
            "second"
        );
        assert!(!linked
            .source
            .source_text
            .contains("Object.defineProperty(globalThis"));
    }

    /// Distinct importer cells can share one ultimate export target.
    #[test]
    fn distinct_import_cells_can_resolve_to_the_same_export() {
        let sources = sources_of(
            &[
                ("a", "export const first = 1;"),
                ("c", "import { first as z } from \"a\";\nprint(z);"),
                (
                    "b",
                    "import { first as z } from \"a\";\nimport \"c\";\nprint(z);",
                ),
            ],
            2,
            vec![
                (1, request_key("a"), 0),
                (2, request_key("a"), 0),
                (2, request_key("c"), 1),
            ],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("agreeing aliases link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let first = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        let second = definitions
            .units
            .iter()
            .find(|unit| unit.module == 2)
            .unwrap();
        assert_eq!(first.imports, second.imports);
        assert_eq!(first.imports.len(), 1);
        assert_eq!(linked.source.source_text.matches("const z = 0;").count(), 2);
        assert!(!linked
            .source
            .source_text
            .contains("Object.defineProperty(globalThis"));
    }

    /// `export default` links: the keywords become a declaration of the minted
    /// name, in place and without moving the initializer.
    #[test]
    fn an_anonymous_export_default_becomes_a_declaration_of_its_minted_name() {
        let sources = sources_of(
            &[
                ("a", "export default 42;"),
                ("b", "import d from \"a\";\nprint(d);"),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("default export links");

        let binding = MergedName::anonymous_default(0);
        let binding = binding.as_str();
        assert!(
            linked
                .source
                .source_text
                .contains(&format!("let {binding}     = 42;")),
            "got {}",
            linked.source.source_text
        );
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let importer = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        assert_eq!(
            namespace_target_reference(&importer.imports[0])
                .unwrap()
                .as_str(),
            binding
        );
    }

    /// There is no line-terminator restriction between `export` and `default`.
    /// The dependency's split pair still declares the minted cell that the
    /// importer's live alias reads, and the merged result remains Script text.
    #[test]
    fn a_split_anonymous_default_links_to_its_importer() {
        let sources = sources_of(
            &[
                ("a", "export\ndefault 42;"),
                ("b", "import answer from \"a\";\nanswer;"),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("split default links");

        let binding = MergedName::anonymous_default(0);
        assert!(
            linked
                .source
                .source_text
                .contains(&format!("let {}    =\n 42;", binding.as_str())),
            "got {}",
            linked.source.source_text
        );
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let importer = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        assert_eq!(
            namespace_target_reference(&importer.imports[0])
                .unwrap()
                .as_str(),
            binding.as_str()
        );
        lila_front::parse(
            linked.source.source_text,
            lila_front::ParseOptions::script(),
        )
        .expect("span-stable linked output should remain valid Script text");
    }

    /// Two units may both have an anonymous `export default`: the spec calls
    /// both bindings `*default*`, but the merged scope names them apart.
    #[test]
    fn two_anonymous_default_exports_do_not_collide() {
        let sources = sources_of(
            &[
                ("a", "export default 1;"),
                ("c", "export default 2;"),
                (
                    "b",
                    "import x from \"a\";\nimport y from \"c\";\nprint(x + y);",
                ),
            ],
            2,
            vec![(2, request_key("a"), 0), (2, request_key("c"), 1)],
        );
        let mut graph = graph_of(&sources);
        let linked =
            linked_script_source(&sources, &mut graph).expect("two anonymous defaults link");
        assert!(
            linked.source.source_text.contains(&format!(
                "let {}     = 1;",
                MergedName::anonymous_default(0).as_str()
            )),
            "got {}",
            linked.source.source_text
        );
        assert!(
            linked.source.source_text.contains(&format!(
                "let {}     = 2;",
                MergedName::anonymous_default(1).as_str()
            )),
            "got {}",
            linked.source.source_text
        );
    }

    /// `export * from` needs nothing of this stage: `ResolveExport` already
    /// walked the star path, so the importer shares the *originating* unit's
    /// cell and no alias is minted at all.
    #[test]
    fn a_star_re_export_resolves_through_to_the_originating_cell() {
        let sources = sources_of(
            &[
                ("a", "export const value = 10;"),
                ("s", "export * from \"a\";"),
                ("b", "import { value } from \"s\";\nprint(value);"),
            ],
            2,
            vec![(1, request_key("a"), 0), (2, request_key("s"), 1)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("star re-export links");

        assert!(
            linked.source.source_text.contains("const value = 10;"),
            "got {}",
            linked.source.source_text
        );
        assert!(
            !linked
                .source
                .source_text
                .contains("defineProperty(globalThis"),
            "a star re-export needs no alias: {}",
            linked.source.source_text
        );
        assert!(
            !linked.source.source_text.contains("export"),
            "got {}",
            linked.source.source_text
        );
    }

    #[test]
    fn same_spelled_top_level_names_belong_to_separate_modules() {
        let sources = sources_of(
            &[
                ("a", "const shared = 1;\nexport { shared };"),
                ("b", "const shared = 2;\nprint(shared);"),
            ],
            1,
            Vec::new(),
        );
        let mut graph = graph_of(&sources);
        let linked =
            linked_script_source(&sources, &mut graph).expect("separate module declarations link");
        assert_eq!(linked.definitions.synchronous.unwrap().units.len(), 2);
        assert!(linked.source.source_text.contains("const shared = 1;"));
        assert!(linked.source.source_text.contains("const shared = 2;"));
    }

    /// `import()` is linked, not rejected: the call site becomes a call to the
    /// referrer's dispatcher, and the dispatcher resolves with the *same*
    /// namespace binding `import * as ns` would alias.
    #[test]
    fn dynamic_import_is_desugared_into_a_dispatcher_call() {
        let sources = sources_of(&[("m", "import('m');")], 0, vec![(0, request_key("m"), 0)]);
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("import() should link");

        assert_eq!(graph.dynamic_components().len(), 1);
        assert!(graph.units[0]
            .namespaces
            .contains_key(&ModuleNamespaceModeIr::Eager));
        assert!(
            linked
                .source
                .source_text
                .contains("function $lila$module$import$0("),
            "got {}",
            linked.source.source_text
        );
        assert!(
            linked.source.source_text.contains(&format!(
                "return {};",
                MergedName::minted(0, UnitCellRole::Namespace).as_str()
            )),
            "got {}",
            linked.source.source_text
        );
        // The call site itself no longer spells `import`, so no `ImportCall`
        // node survives to the backend.
        assert!(
            linked
                .source
                .source_text
                .contains("$lila$module$import$0('m');"),
            "got {}",
            linked.source.source_text
        );
    }

    /// Namespace imports resolve to the exporting record's private cell.
    #[test]
    fn a_namespace_import_binds_its_local_to_the_private_namespace_cell() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("c", "import * as ns from \"a\";\nprint(ns.value);"),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        let linked = linked_script_source(&sources, &mut graph).expect("namespace should link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        let exporter = definitions
            .units
            .iter()
            .find(|unit| unit.module == 0)
            .unwrap();
        assert_eq!(exporter.namespaces.len(), 1);
        assert_eq!(exporter.namespaces[0].0, ModuleNamespaceModeIr::Eager);
        let importer = definitions
            .units
            .iter()
            .find(|unit| unit.module == 1)
            .unwrap();
        assert_eq!(
            importer.imports,
            [ResolvedBindingIr::Resolved {
                module: 0,
                binding: ModuleBindingNameIr::Namespace(ModuleNamespaceModeIr::Eager),
            }]
        );
    }

    /// Deferred bodies are instantiated without starting their evaluation.
    #[test]
    fn a_deferred_dependency_has_an_activation_without_initial_evaluation() {
        let sources = sources_of(
            &[
                ("a", "print(\"side effect\");\nexport const value = 41;"),
                (
                    "d",
                    "import defer * as ns from \"a\";\nprint(\"entry\");\nprint(ns.value);",
                ),
            ],
            1,
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(graph.evaluation_mode(0), ModuleEvaluationModeIr::Deferred);
        let linked = linked_script_source(&sources, &mut graph).expect("defer should link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        assert_eq!(definitions.initial_evaluation, [1]);
        let deferred = definitions
            .units
            .iter()
            .find(|unit| unit.module == 0)
            .unwrap();
        assert_eq!(deferred.namespaces[0].0, ModuleNamespaceModeIr::Deferred);
        assert_eq!(deferred.evaluation.module(), 0);
        assert!(linked
            .source
            .source_text
            .contains("print(\"side effect\");"));
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
            vec![(1, request_key("a"), 0)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(
            graph.evaluation_mode(0),
            ModuleEvaluationModeIr::NotEvaluated
        );
        let linked = linked_script_source(&sources, &mut graph).expect("source phase should link");

        assert!(
            !linked.source.source_text.contains("must not run"),
            "got {}",
            linked.source.source_text
        );
        let cell = MergedName::minted(0, UnitCellRole::ModuleSource);
        let cell = cell.as_str();
        assert!(
            linked
                .source
                .source_text
                .contains(&format!("const {cell} = Object.create(null);")),
            "got {}",
            linked.source.source_text
        );
        assert!(
            linked
                .source
                .source_text
                .contains(&format!("const src = {cell};")),
            "got {}",
            linked.source.source_text
        );
    }

    /// A source-only unit is still parsed and linked, but none of the runtime
    /// scaffolding named by its own body belongs to the artifact. This is one
    /// contract rather than four feature tests: every collector must consume
    /// the same materialization witness or an inactive alias leaks into the
    /// entry's merged scope.
    #[test]
    fn module_source_only_units_contribute_no_runtime_scaffolding() {
        let sources = sources_of(
            &[
                (
                    "inactive",
                    "import * as ghost from \"namespace\";\n\
                     import source nested from \"nested\";\n\
                     import.meta;\n\
                     import(\"dynamic\");\n\
                     export const value = 1;",
                ),
                ("namespace", "export const visible = 1;"),
                ("nested", "export const nested = 1;"),
                ("dynamic", "export const dynamic = 1;"),
                (
                    "entry",
                    "import source artifact from \"inactive\";\n\
                     print(typeof ghost);\n\
                     artifact;",
                ),
            ],
            4,
            vec![
                (0, request_key("namespace"), 1),
                (0, request_key("nested"), 2),
                (0, request_key("dynamic"), 3),
                (4, request_key("inactive"), 0),
            ],
        );
        let mut graph = graph_of(&sources);

        for module in 0..4 {
            assert_eq!(
                graph.evaluation_mode(module),
                ModuleEvaluationModeIr::NotEvaluated
            );
        }
        assert!(
            graph.dynamic_components().is_empty(),
            "an import() in a source-only referrer is not an artifact component"
        );
        assert_eq!(
            crate::modules::namespace::ensure_namespace(
                &mut graph,
                0,
                ModuleNamespaceModeIr::Eager
            ),
            None,
            "a source-only unit has no environment for a namespace"
        );

        let linked = linked_script_source(&sources, &mut graph)
            .expect("source-only runtime scaffolding must not reject the active graph");
        let text = &linked.source.source_text;
        let inactive_source = MergedName::minted(0, UnitCellRole::ModuleSource);
        assert!(
            text.contains(&format!(
                "const {} = Object.create(null);",
                inactive_source.as_str()
            )),
            "the active referrer's source object is required: {text}"
        );
        assert!(
            text.contains(&format!("const artifact = {};", inactive_source.as_str())),
            "the active source binding is required: {text}"
        );

        for absent in [
            MergedName::minted(1, UnitCellRole::Namespace),
            MergedName::minted(2, UnitCellRole::ModuleSource),
            MergedName::minted(3, UnitCellRole::Namespace),
            MergedName::minted(0, UnitCellRole::ImportMeta),
        ] {
            assert!(
                !text.contains(absent.as_str()),
                "inactive runtime cell {} leaked into: {text}",
                absent.as_str()
            );
        }
        assert!(
            !text.contains("const ghost ="),
            "inactive alias leaked: {text}"
        );
        assert!(
            !text.contains("function $lila$module$import$0("),
            "inactive dispatcher leaked: {text}"
        );
        assert!(
            text.contains("print(typeof ghost)"),
            "the active observation must remain: {text}"
        );
    }

    /// Dynamic-only targets instantiate eagerly but evaluate from import jobs.
    #[test]
    fn initial_evaluation_starts_only_the_entry_static_traversal() {
        let sources = sources_of(
            &[
                ("d", "import(\"a\");\nprint(\"entry\");"),
                ("a", "print(\"target\");"),
            ],
            0,
            vec![(0, request_key("a"), 1)],
        );
        let mut graph = graph_of(&sources);
        assert_eq!(graph.evaluation_order.first().copied(), Some(0));
        let linked = linked_script_source(&sources, &mut graph).expect("graph should link");
        let definitions = linked
            .definitions
            .synchronous
            .expect("canonical module definitions");
        assert_eq!(definitions.initial_evaluation, [0]);
        assert!(definitions
            .dispatcher_evaluations
            .contains_key("$lila$module$evaluate$1"));
    }
}
