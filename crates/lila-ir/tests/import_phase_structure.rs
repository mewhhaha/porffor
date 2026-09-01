const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/import_phase.rs");
const RECORD_SOURCE: &str = include_str!("../src/modules/record.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_CLASSIFICATION_SOURCE: &str =
    include_str!("../src/modules/graph_evaluation_classification.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const LINK_ERROR_SOURCE: &str = include_str!("../src/modules/link_error.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const IR_SOURCE: &str = include_str!("../src/ir.rs");
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
fn import_phase_has_one_private_subsystem_owner_and_narrow_facade_reexport() {
    assert_eq!(MODULES_SOURCE.matches("\nmod import_phase;\n").count(), 1);
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use import_phase::ImportPhaseIr;")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod import_phase;\n"));
    assert!(!MODULES_SOURCE.contains("\nmod import_phase {\n"));
    assert!(!RECORD_SOURCE.contains("enum ImportPhaseIr"));
    assert!(!RECORD_SOURCE.contains("impl ImportPhaseIr"));
    assert!(!RECORD_SOURCE.contains("ImportKind, ImportPhase, ReExportKind"));
    assert!(OWNER_SOURCE.starts_with("use boa_ast::declaration::ImportPhase;\n\n"));
    assert_eq!(OWNER_SOURCE.matches("pub enum ImportPhaseIr").count(), 1);
    assert_eq!(LIB_SOURCE.matches("ImportPhaseIr").count(), 1);
}

#[test]
fn import_phase_preserves_the_closed_default_diagnostic_and_ast_domains() {
    let variants = bounded(
        OWNER_SOURCE,
        "pub enum ImportPhaseIr {",
        "impl ImportPhaseIr",
    );
    assert_eq!(
        code_without_whitespace(variants),
        "#[default]Evaluation,Defer,Source,}"
    );

    let diagnostic_names = bounded(
        OWNER_SOURCE,
        "pub const fn as_str(self) -> &'static str {",
        "pub(super) const fn from_ast(phase: ImportPhase) -> Self {",
    );
    assert_eq!(
        code_without_whitespace(diagnostic_names),
        "matchself{Self::Evaluation=>\"evaluation\",Self::Defer=>\"defer\",\
         Self::Source=>\"source\",}}"
    );

    let ast_projection = OWNER_SOURCE
        .split_once("pub(super) const fn from_ast(phase: ImportPhase) -> Self {")
        .expect("AST phase projection")
        .1;
    assert_eq!(
        code_without_whitespace(ast_projection),
        "matchphase{ImportPhase::Evaluation=>Self::Evaluation,\
         ImportPhase::Defer=>Self::Defer,ImportPhase::Source=>Self::Source,}}}"
    );
    assert!(!OWNER_SOURCE.contains("_ =>"));
}

#[test]
fn import_phase_keeps_the_reviewed_ast_projection_and_public_caller_census() {
    assert_eq!(RECORD_SOURCE.matches("ImportPhaseIr::from_ast(").count(), 2);
    assert_eq!(
        DYNAMIC_SOURCE.matches("ImportPhaseIr::from_ast(").count(),
        1
    );
    for source in [
        GRAPH_SOURCE,
        GRAPH_TESTS_SOURCE,
        GRAPH_CLASSIFICATION_SOURCE,
        LINK_SOURCE,
        LINK_ERROR_SOURCE,
        NAMESPACE_SOURCE,
        IR_SOURCE,
        LIB_SOURCE,
    ] {
        assert!(!source.contains("ImportPhaseIr::from_ast("));
        assert!(!source.contains("enum ImportPhaseIr"));
        assert!(!source.contains("impl ImportPhaseIr"));
    }
    assert_eq!(RECORD_SOURCE.matches("ImportPhaseIr").count(), 26);
    assert_eq!(DYNAMIC_SOURCE.matches("ImportPhaseIr").count(), 25);
    assert_eq!(GRAPH_SOURCE.matches("ImportPhaseIr").count(), 1);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ImportPhaseIr").count(), 4);
    assert_eq!(
        GRAPH_CLASSIFICATION_SOURCE.matches("ImportPhaseIr").count(),
        7
    );
    assert_eq!(LINK_SOURCE.matches("ImportPhaseIr").count(), 1);
    assert_eq!(LINK_ERROR_SOURCE.matches("ImportPhaseIr").count(), 2);
    assert_eq!(NAMESPACE_SOURCE.matches("ImportPhaseIr").count(), 3);
    assert_eq!(IR_SOURCE.matches("ImportPhaseIr").count(), 2);
}
