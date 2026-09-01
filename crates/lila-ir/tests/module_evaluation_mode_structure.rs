const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/evaluation_mode.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const ASYNC_EVALUATION_SOURCE: &str = include_str!("../src/modules/graph_async_evaluation.rs");
const CLASSIFICATION_SOURCE: &str =
    include_str!("../src/modules/graph_evaluation_classification.rs");
const MATERIALIZATION_SOURCE: &str = include_str!("../src/modules/graph_materialization.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");

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
fn module_evaluation_mode_has_one_private_owner_and_only_the_public_facade_reexport() {
    assert_eq!(
        MODULES_SOURCE.matches("\nmod evaluation_mode;\n").count(),
        1
    );
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use evaluation_mode::ModuleEvaluationModeIr;")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("pub use evaluation_mode::ModuleMaterializationModeIr"));
    assert!(!MODULES_SOURCE.contains("\npub mod evaluation_mode;\n"));
    assert_eq!(
        MODULES_SOURCE
            .matches("\nmod graph_materialization;\n")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod graph_materialization;\n"));
    assert!(!MODULES_SOURCE.contains("use graph_materialization::"));
    assert!(!GRAPH_SOURCE.contains("enum ModuleEvaluationModeIr"));
    assert!(!GRAPH_SOURCE.contains("enum ModuleMaterializationModeIr"));
    assert!(!GRAPH_SOURCE.contains("impl ModuleEvaluationModeIr"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub enum ModuleEvaluationModeIr")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) enum ModuleMaterializationModeIr")
            .count(),
        1
    );
    assert_eq!(LIB_SOURCE.matches("ModuleEvaluationModeIr").count(), 1);
    assert!(!LIB_SOURCE.contains("ModuleMaterializationModeIr"));
    assert_eq!(GRAPH_SOURCE.matches("pub fn evaluation_mode(").count(), 1);
    assert!(!MATERIALIZATION_SOURCE.contains("pub fn evaluation_mode("));
    assert!(!GRAPH_SOURCE.contains("fn materialization_mode("));
    assert!(!GRAPH_SOURCE.contains("fn materialized_units("));
    assert_eq!(
        MATERIALIZATION_SOURCE
            .matches("pub(super) fn materialization_mode(")
            .count(),
        1
    );
    assert_eq!(
        MATERIALIZATION_SOURCE
            .matches("pub(super) fn materialized_units(")
            .count(),
        1
    );
}

#[test]
fn module_evaluation_mode_preserves_the_closed_default_and_materialization_projection() {
    let evaluation_variants = bounded(
        OWNER_SOURCE,
        "pub enum ModuleEvaluationModeIr {",
        "/// How a linked unit participates in runtime source generation.",
    );
    assert_eq!(
        code_without_whitespace(evaluation_variants),
        "#[default]Eager,Deferred,NotEvaluated,}"
    );

    let materialization_variants = bounded(
        OWNER_SOURCE,
        "pub(super) enum ModuleMaterializationModeIr {",
        "impl ModuleEvaluationModeIr",
    );
    assert_eq!(
        code_without_whitespace(materialization_variants),
        "Eager,Deferred,}"
    );

    let diagnostic_names = bounded(
        OWNER_SOURCE,
        "pub const fn as_str(self) -> &'static str {",
        "/// Runtime source-generation participation for this evaluation mode.",
    );
    assert_eq!(
        code_without_whitespace(diagnostic_names),
        "matchself{Self::Eager=>\"eager\",Self::Deferred=>\"deferred\",\
         Self::NotEvaluated=>\"notevaluated\",}}"
    );

    let projection = OWNER_SOURCE
        .split_once(
            "pub(super) const fn materialization(self) -> Option<ModuleMaterializationModeIr> {",
        )
        .expect("materialization projection")
        .1;
    assert_eq!(
        code_without_whitespace(projection),
        "matchself{Self::Eager=>Some(ModuleMaterializationModeIr::Eager),\
         Self::Deferred=>Some(ModuleMaterializationModeIr::Deferred),\
         Self::NotEvaluated=>None,}}}"
    );
    assert!(!OWNER_SOURCE.contains("_ =>"));
}

#[test]
fn module_materialization_callers_import_the_private_type_from_its_real_owner() {
    assert_eq!(
        MATERIALIZATION_SOURCE
            .matches("use super::evaluation_mode::ModuleMaterializationModeIr;")
            .count(),
        1
    );
    assert_eq!(
        LINK_SOURCE
            .matches("use super::evaluation_mode::ModuleMaterializationModeIr;")
            .count(),
        1
    );
    assert_eq!(
        NAMESPACE_SOURCE
            .matches("use super::evaluation_mode::ModuleMaterializationModeIr;")
            .count(),
        1
    );
    for source in [
        GRAPH_SOURCE,
        MATERIALIZATION_SOURCE,
        LINK_SOURCE,
        NAMESPACE_SOURCE,
    ] {
        assert!(!source.contains("use super::graph::ModuleMaterializationModeIr;"));
    }
    assert_eq!(
        MATERIALIZATION_SOURCE
            .matches("ModuleMaterializationModeIr")
            .count(),
        3
    );
    assert_eq!(
        GRAPH_SOURCE.matches("ModuleMaterializationModeIr").count(),
        0
    );
    assert_eq!(
        LINK_SOURCE.matches("ModuleMaterializationModeIr").count(),
        7
    );
    assert_eq!(
        NAMESPACE_SOURCE
            .matches("ModuleMaterializationModeIr")
            .count(),
        12
    );
    assert!(!DYNAMIC_SOURCE.contains("ModuleMaterializationModeIr"));
    assert_eq!(GRAPH_SOURCE.matches("ModuleEvaluationModeIr").count(), 5);
    assert_eq!(
        GRAPH_TESTS_SOURCE.matches("ModuleEvaluationModeIr").count(),
        10
    );
    assert_eq!(
        ASYNC_EVALUATION_SOURCE
            .matches("ModuleEvaluationModeIr")
            .count(),
        4
    );
    assert_eq!(
        CLASSIFICATION_SOURCE
            .matches("ModuleEvaluationModeIr")
            .count(),
        6
    );
    assert_eq!(DYNAMIC_SOURCE.matches("ModuleEvaluationModeIr").count(), 2);
}
