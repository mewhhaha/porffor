const ANALYSIS_SOURCE: &str = include_str!("../src/analysis.rs");
const OWNER_SOURCE: &str = include_str!("../src/analysis/environment_kind.rs");
const FUNCTION_ENVIRONMENT_SOURCE: &str = include_str!("../src/analysis/function_environment.rs");
const EVAL_ENVIRONMENT_SOURCE: &str = include_str!("../src/analysis/eval_environment.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const FUNCTION_DEFINITION_SOURCE: &str = include_str!("../src/lowering/function_definition.rs");
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
fn environment_kind_has_one_private_file_owner_and_narrow_reexport() {
    assert_eq!(
        ANALYSIS_SOURCE.matches("\nmod environment_kind;\n").count(),
        1
    );
    assert_eq!(
        ANALYSIS_SOURCE
            .matches("pub(crate) use environment_kind::EnvironmentKind;")
            .count(),
        1
    );
    assert!(!ANALYSIS_SOURCE.contains("\npub mod environment_kind;\n"));
    assert!(!ANALYSIS_SOURCE.contains("\nmod environment_kind {\n"));
    assert!(!ANALYSIS_SOURCE.contains("enum EnvironmentKind"));
    assert!(!ANALYSIS_SOURCE.contains("impl EnvironmentKind"));
    assert!(OWNER_SOURCE.starts_with("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(crate) enum EnvironmentKind")
            .count(),
        1
    );
    assert!(ANALYSIS_SOURCE.contains("pub(crate) struct EnvironmentPlan"));
    assert!(ANALYSIS_SOURCE.contains("pub(crate) kind: EnvironmentKind"));
}

#[test]
fn environment_kind_preserves_the_closed_materialization_domains() {
    let variants = bounded(
        OWNER_SOURCE,
        "pub(crate) enum EnvironmentKind {",
        "impl EnvironmentKind",
    );
    assert_eq!(
        code_without_whitespace(variants),
        "Activation,FunctionParameters,FunctionBody,ParameterEvalVariable,Block,WithObject,ClassName,NamedFunctionExpression,SwitchCaseBlock,CatchParameter,SimpleCatchParameter,ForLexicalHead,ForInOfTdzHead,ForInOfIteration,}"
    );

    let stage_a = bounded(
        OWNER_SOURCE,
        "pub(crate) const fn is_materialized_in_stage_a(self) -> bool {",
        "pub(crate) const fn is_materialized(self) -> bool {",
    );
    assert_eq!(
        code_without_whitespace(stage_a),
        "matches!(self,Self::FunctionParameters|Self::FunctionBody|Self::Block|Self::SwitchCaseBlock|Self::CatchParameter|Self::SimpleCatchParameter)}"
    );

    let materialized = OWNER_SOURCE
        .split_once("pub(crate) const fn is_materialized(self) -> bool {")
        .expect("materialized environment domain")
        .1;
    assert_eq!(
        code_without_whitespace(materialized),
        "matches!(self,Self::FunctionParameters|Self::FunctionBody|Self::ParameterEvalVariable|Self::Block|Self::NamedFunctionExpression|Self::ClassName|Self::WithObject|Self::SwitchCaseBlock|Self::CatchParameter|Self::SimpleCatchParameter|Self::ForLexicalHead|Self::ForInOfTdzHead|Self::ForInOfIteration)}}"
    );
    assert!(!OWNER_SOURCE.contains("_ =>"));
}

#[test]
fn environment_kind_keeps_the_reviewed_projection_and_external_census() {
    let stage_a = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) fn materialized_stage_a_environment(",
        "pub(crate) fn environment_has_runtime_storage(",
    );
    assert!(stage_a.contains(".is_materialized_in_stage_a()"));
    let materialized = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) fn materialized_environment(",
        "fn environment_has_runtime_storage(",
    );
    assert!(materialized.contains("environment.kind.is_materialized().then_some(environment)"));
    let storage = bounded(
        ANALYSIS_SOURCE,
        "fn environment_has_runtime_storage(\n    environment:",
        "#[derive(Default)]",
    );
    assert!(storage.contains("environment.kind == EnvironmentKind::Activation"));
    assert!(storage.contains("environment.kind.is_materialized()"));
    assert!(storage.contains("match execution_kind"));
    assert!(storage.contains("FunctionExecutionKind::Ordinary => false"));
    for execution in ["Generator", "Async", "AsyncGenerator"] {
        assert!(storage.contains(&format!("FunctionExecutionKind::{execution}")));
    }
    assert!(!storage.contains("_ =>"));
    for kind in [
        "FunctionParameters",
        "FunctionBody",
        "ParameterEvalVariable",
    ] {
        assert!(FUNCTION_ENVIRONMENT_SOURCE.contains(&format!("EnvironmentKind::{kind}")));
    }
    assert!(EVAL_ENVIRONMENT_SOURCE.contains("EnvironmentKind::ForInOfIteration"));
    for source in [
        LOWERING_SOURCE,
        FUNCTION_DEFINITION_SOURCE,
        FUNCTION_ENVIRONMENT_SOURCE,
        EVAL_ENVIRONMENT_SOURCE,
        LIB_SOURCE,
    ] {
        assert!(!source.contains("enum EnvironmentKind"));
        assert!(!source.contains("impl EnvironmentKind"));
    }
    assert_eq!(LIB_SOURCE.matches("pub(crate) use analysis::*;").count(), 1);
}
