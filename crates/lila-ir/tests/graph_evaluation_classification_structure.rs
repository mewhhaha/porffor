const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/graph_evaluation_classification.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

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
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) fn report_unlinkable_phases(")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("fn classify_evaluation_modes("));
    assert!(!GRAPH_SOURCE.contains("fn report_unlinkable_phases("));
    assert_eq!(
        GRAPH_SOURCE
            .matches("use super::graph_evaluation_classification::{")
            .count(),
        1
    );
    assert_eq!(
        GRAPH_SOURCE.matches("classify_evaluation_modes(").count(),
        1
    );
    assert_eq!(GRAPH_SOURCE.matches("report_unlinkable_phases(").count(), 1);
}

#[test]
fn graph_evaluation_classification_keeps_the_phase_reachability_fixed_point() {
    let classification = bounded(
        OWNER_SOURCE,
        "pub(super) fn classify_evaluation_modes(",
        "/// Reports the phased requests the source-text linker still cannot express.",
    );
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

#[test]
fn graph_evaluation_classification_keeps_unsupported_defer_policy_together() {
    let policy = OWNER_SOURCE
        .split_once("pub(super) fn report_unlinkable_phases(")
        .expect("unsupported phase policy")
        .1;
    assert!(policy.contains("let components = graph.component_of_unit();"));
    assert!(policy.contains("mode != ModuleEvaluationModeIr::Deferred"));
    assert!(policy.contains("if graph.has_tla(id)"));
    assert!(policy.contains(".is_some_and(|size| *size > 1)"));
    assert_eq!(
        policy
            .matches("ModuleLinkErrorIr::UnsupportedPhase")
            .count(),
        2
    );
    assert_eq!(policy.matches("phase: ImportPhaseIr::Defer").count(), 2);
    assert!(policy.contains("graph.link_errors.append(&mut errors);"));
}
