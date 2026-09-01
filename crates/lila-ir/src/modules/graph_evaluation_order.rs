//! `InnerModuleEvaluation` ordering and strongly-connected components.

use super::graph::ModuleGraphIr;
use super::record::{ModuleEvaluationDependencyIr, ModuleUnitId};

/// `InnerModuleEvaluation` (16.2.1.5.3) order, via Tarjan's SCC algorithm.
///
/// Tarjan emits components in reverse topological order, which is exactly the
/// order dependencies must evaluate in, and it groups the members of a cycle
/// contiguously so the link stage can hoist all of them before executing any.
pub(super) fn compute_evaluation_order(graph: &mut ModuleGraphIr) {
    struct Tarjan {
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<ModuleUnitId>,
        next_index: usize,
        /// DFS *finish* order, which is the order `InnerModuleEvaluation`
        /// reaches step 13 in. Distinct from `index`, the entry order.
        finish: Vec<usize>,
        next_finish: usize,
        order: Vec<ModuleUnitId>,
        starts: Vec<usize>,
    }

    let unit_count = graph.units.len();
    let mut state = Tarjan {
        index: vec![None; unit_count],
        low: vec![0; unit_count],
        on_stack: vec![false; unit_count],
        stack: Vec::new(),
        next_index: 0,
        finish: vec![0; unit_count],
        next_finish: 0,
        order: Vec::new(),
        starts: Vec::new(),
    };

    // Explicit work stack: module graphs can nest deeper than the native stack
    // tolerates, and `unsafe_code = "forbid"` leaves no room for a guard page
    // trick.
    enum Step {
        Enter(ModuleUnitId),
        Resume(ModuleUnitId, usize),
    }

    let dependencies: Vec<Vec<ModuleEvaluationDependencyIr>> = (0..unit_count)
        .map(|module| {
            let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
            graph.evaluation_dependencies_of(id)
        })
        .collect();

    for root in 0..unit_count {
        if state.index[root].is_some() {
            continue;
        }
        let root_id = ModuleUnitId::try_from(root).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
        let mut work = vec![Step::Enter(root_id)];
        while let Some(step) = work.pop() {
            match step {
                Step::Enter(id) => {
                    let module = id as usize;
                    if state.index[module].is_some() {
                        continue;
                    }
                    state.index[module] = Some(state.next_index);
                    state.low[module] = state.next_index;
                    state.next_index += 1;
                    state.stack.push(id);
                    state.on_stack[module] = true;
                    work.push(Step::Resume(id, 0));
                }
                Step::Resume(id, cursor) => {
                    let module = id as usize;
                    if let Some(&dependency) = dependencies[module].get(cursor) {
                        work.push(Step::Resume(id, cursor + 1));
                        let next = dependency.target();
                        let target = next as usize;
                        match state.index[target] {
                            None => work.push(Step::Enter(next)),
                            Some(target_index) => {
                                if state.on_stack[target] {
                                    state.low[module] = state.low[module].min(target_index);
                                }
                            }
                        }
                        continue;
                    }
                    // All dependencies visited: fold their lowlinks in.
                    for &dependency in &dependencies[module] {
                        let next = dependency.target();
                        let target = next as usize;
                        if state.on_stack[target] {
                            state.low[module] = state.low[module].min(state.low[target]);
                        }
                    }
                    state.finish[module] = state.next_finish;
                    state.next_finish += 1;
                    if state.index[module] == Some(state.low[module]) {
                        state.starts.push(state.order.len());
                        let mut component = Vec::new();
                        while let Some(member) = state.stack.pop() {
                            state.on_stack[member as usize] = false;
                            component.push(member);
                            if member == id {
                                break;
                            }
                        }
                        // The stack pops a component in reverse *entry* order,
                        // which stops matching the spec as soon as a cycle
                        // branches. Finish order is what `InnerModuleEvaluation`
                        // executes bodies in, and the component root always
                        // finishes last, so it stays the component's last body.
                        component.sort_by_key(|member| state.finish[*member as usize]);
                        state.order.extend(component);
                    }
                }
            }
        }
    }

    graph.evaluation_order = state.order;
    graph.scc_starts = state.starts;
}
