//! Evaluation-mode classification for canonical module activations.

use super::dynamic::DynamicComponentIr;
use super::evaluation_mode::ModuleEvaluationModeIr;
use super::graph::ModuleGraphIr;
use super::import_phase::ImportPhaseIr;
use super::record::ModuleUnitId;

/// Fixes [`ModuleEvaluationModeIr`] for every unit from the phases of the
/// requests that reach it.
///
/// The rule is reachability, not a per-request vote: a module evaluates when
/// something that *itself* evaluates asks for it in the evaluation phase. So a
/// module reached only through `import source` never runs, and neither does
/// anything only that module imports — which is the whole point of the source
/// phase, and what a per-request vote would get wrong.
///
/// # Roots
///
/// The entry always evaluates. So does every evaluation-phase `import()`
/// target, because `import()` resolves with an *evaluated* namespace. So does
/// every unit no request points at: a graph assembled by an embedder rather than
/// by `load_module_graph` may hold units with no importer at all, and dropping
/// their bodies would silently change what such a graph runs.
///
/// A dynamic request counts exactly as much as its static twin does, phase for
/// phase: `import.defer('m')` defers `m` the way `import defer * as ns from
/// 'm'` does, and `import.source('m')` neither evaluates nor instantiates it.
///
/// All executable graphs share private module activations. Evaluation
/// dependencies of a deferred target therefore remain deferred with it; source
/// targets are loaded and parsed without opening their outgoing dependencies.
pub(super) fn classify_evaluation_modes(
    graph: &mut ModuleGraphIr,
    components: &[DynamicComponentIr],
) {
    let count = graph.units.len();
    // `(referrer, phase, target)` once, so the fixed point below is a walk over
    // an edge list rather than a repeated resolve of every request.
    let mut edges: Vec<(usize, ImportPhaseIr, usize)> = Vec::new();
    let mut targeted = vec![false; count];
    for module in 0..count {
        let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
        for request in &graph.units[module].record.requested_modules {
            let Some(target) = graph
                .resolve_request(id, request)
                .map(|target| target as usize)
                .filter(|target| *target < count)
            else {
                continue;
            };
            targeted[target] = true;
            edges.push((module, request.phase(), target));
        }
    }
    // `import()` call sites, which are not in `[[RequestedModules]]` but reach
    // a module just as surely. Their referrer is a unit of this graph, so a
    // dynamic edge out of a module nothing evaluates opens nothing — an
    // `import()` written in a source-phase-only module never runs.
    for component in components {
        let (Ok(referrer), Ok(target)) = (
            usize::try_from(component.referrer()),
            usize::try_from(component.target()),
        ) else {
            continue;
        };
        if referrer >= count || target >= count {
            continue;
        }
        targeted[target] = true;
        edges.push((referrer, component.request().phase(), target));
    }

    let mut eager = vec![false; count];
    for resolution in graph.realm_requests.values() {
        if let super::RealmModuleResolutionIr::Loaded(module) = resolution {
            eager[*module as usize] = true;
        }
    }
    let mut deferred = vec![false; count];
    for module in 0..count {
        if !targeted[module] || ModuleUnitId::try_from(module) == Ok(graph.entry) {
            eager[module] = true;
        }
    }

    // Fixed point rather than one pass: a unit only becomes deferred through an
    // edge from a unit that itself runs, and an edge can promote an already
    // deferred unit to eager, which then opens its own outgoing edges.
    loop {
        let mut changed = false;
        for (module, phase, target) in &edges {
            if !eager[*module] && !deferred[*module] {
                continue;
            }
            match phase {
                ImportPhaseIr::Evaluation if !eager[*module] => {
                    if !eager[*target] && !deferred[*target] {
                        deferred[*target] = true;
                        changed = true;
                    }
                }
                ImportPhaseIr::Evaluation => {
                    if !eager[*target] {
                        eager[*target] = true;
                        deferred[*target] = false;
                        changed = true;
                    }
                }
                ImportPhaseIr::Defer => {
                    if !eager[*target] && !deferred[*target] {
                        deferred[*target] = true;
                        changed = true;
                    }
                }
                // A source-phase request neither evaluates nor instantiates its
                // target, so it opens no edge at all.
                ImportPhaseIr::Source => {}
            }
        }
        if !changed {
            break;
        }
    }

    graph.evaluation_modes = (0..count)
        .map(|module| {
            if eager[module] {
                ModuleEvaluationModeIr::Eager
            } else if deferred[module] {
                ModuleEvaluationModeIr::Deferred
            } else {
                ModuleEvaluationModeIr::NotEvaluated
            }
        })
        .collect();
}
