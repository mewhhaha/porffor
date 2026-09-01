const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_evaluation_order.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const ASYNC_EVALUATION_SOURCE: &str = include_str!("../src/modules/graph_async_evaluation.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn graph_evaluation_order_has_one_private_owner_without_a_compatibility_reexport() {
    assert_eq!(
        MODULES_SOURCE
            .matches("\nmod graph_evaluation_order;\n")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_evaluation_order;\n"));
    assert!(!MODULES_SOURCE.contains("use graph_evaluation_order::"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) fn compute_evaluation_order(")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("fn compute_evaluation_order("));
    assert_eq!(
        GRAPH_SOURCE
            .matches("use super::graph_evaluation_order::compute_evaluation_order;")
            .count(),
        1
    );
    assert_eq!(GRAPH_SOURCE.matches("compute_evaluation_order(").count(), 1);
}

#[test]
fn graph_evaluation_order_keeps_the_closed_tarjan_and_work_step_domains() {
    let tarjan_fields = bounded(OWNER_SOURCE, "struct Tarjan {", "let unit_count");
    assert_eq!(
        code_without_whitespace(tarjan_fields),
        "index:Vec<Option<usize>>,low:Vec<usize>,on_stack:Vec<bool>,\
         stack:Vec<ModuleUnitId>,next_index:usize,finish:Vec<usize>,\
         next_finish:usize,order:Vec<ModuleUnitId>,starts:Vec<usize>,}"
    );

    let work_steps = bounded(OWNER_SOURCE, "enum Step {", "let dependencies");
    assert_eq!(
        code_without_whitespace(work_steps),
        "Enter(ModuleUnitId),Resume(ModuleUnitId,usize),}"
    );
    assert!(!OWNER_SOURCE.contains("_ =>"));
}

#[test]
fn graph_evaluation_order_preserves_dependency_and_component_invariants() {
    assert_eq!(
        GRAPH_SOURCE
            .matches("pub(super) fn evaluation_dependencies_of(")
            .count(),
        1
    );
    assert_eq!(
        GRAPH_SOURCE.matches("evaluation_dependencies_of(").count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE.matches("evaluation_dependencies_of(").count(),
        1
    );
    assert_eq!(
        ASYNC_EVALUATION_SOURCE
            .matches("evaluation_dependencies_of(")
            .count(),
        2
    );
    assert!(OWNER_SOURCE.contains("let mut work = vec![Step::Enter(root_id)];"));
    assert!(OWNER_SOURCE.contains("while let Some(step) = work.pop()"));
    assert!(OWNER_SOURCE.contains("state.low[module] = state.low[module].min(state.low[target]);"));
    assert!(
        OWNER_SOURCE.contains("component.sort_by_key(|member| state.finish[*member as usize]);")
    );
    assert!(OWNER_SOURCE.contains("graph.evaluation_order = state.order;"));
    assert!(OWNER_SOURCE.contains("graph.scc_starts = state.starts;"));
}
