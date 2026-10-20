const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_evaluation_classification.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");

#[test]
fn graph_evaluation_classification_has_one_private_owner_without_a_reexport() {
    assert_eq!(
        MODULES_SOURCE
            .matches("\nmod graph_evaluation_classification;\n")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_evaluation_classification;\n"));
    assert!(!MODULES_SOURCE.contains("use graph_evaluation_classification::"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) fn classify_evaluation_modes(")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("fn classify_evaluation_modes("));
    assert!(!GRAPH_SOURCE.contains("fn report_unlinkable_phases("));
    assert_eq!(
        GRAPH_SOURCE
            .matches("use super::graph_evaluation_classification::classify_evaluation_modes;")
            .count(),
        1
    );
    assert_eq!(
        GRAPH_SOURCE.matches("classify_evaluation_modes(").count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("report_unlinkable_phases("));
}

#[test]
fn graph_evaluation_classification_keeps_the_phase_reachability_fixed_point() {
    let classification = OWNER_SOURCE
        .split_once("pub(super) fn classify_evaluation_modes(")
        .unwrap()
        .1;
    assert!(classification.contains("let mut edges: Vec<(usize, ImportPhaseIr, usize)>"));
    assert!(classification.contains("for component in components"));
    assert!(classification.contains("if !targeted[module]"));
    assert!(classification.contains("ModuleUnitId::try_from(module) == Ok(graph.entry)"));
    assert!(classification.contains("loop {"));
    assert!(classification.contains("deferred[*target] = false;"));
    assert!(classification.contains("ImportPhaseIr::Evaluation =>"));
    assert!(classification.contains("ImportPhaseIr::Defer =>"));
    assert!(classification.contains("ImportPhaseIr::Source => {}"));
    assert!(classification.contains("ModuleEvaluationModeIr::Eager"));
    assert!(classification.contains("ModuleEvaluationModeIr::Deferred"));
    assert!(classification.contains("ModuleEvaluationModeIr::NotEvaluated"));
    assert!(!classification.contains("_ =>"));
}
