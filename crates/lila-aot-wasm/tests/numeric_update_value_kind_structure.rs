const OPERATIONS_SOURCE: &str = include_str!("../../lila-ir/src/operations.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const BACKEND_OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const SUPER_EXPRESSIONS_SOURCE: &str =
    include_str!("../src/expressions/super_property_mutation.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/numeric-update-value-kind.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let rest = &source[start..];
    let end = rest
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"));
    &rest[..end]
}

#[test]
fn numeric_update_kind_is_one_closed_domain_with_total_value_kind_projection() {
    let domain = bounded(
        OPERATIONS_SOURCE,
        "pub enum NumericUpdateValueKind {",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum UpdateReturnMode",
    );
    for variant in ["Number", "BigInt", "Dynamic"] {
        assert_eq!(domain.matches(&format!("    {variant},")).count(), 1);
        assert!(domain.contains(&format!("Self::{variant} => ValueKind::{variant}")));
    }
    assert_eq!(
        domain
            .lines()
            .filter(|line| matches!(line.trim(), "Number," | "BigInt," | "Dynamic,"))
            .count(),
        3
    );
    assert!(domain.contains("pub const fn value_kind(self) -> ValueKind"));
    assert!(!domain.contains("_ =>"));
}

#[test]
fn every_numeric_update_ir_carrier_stores_the_closed_kind() {
    assert_eq!(
        IR_SOURCE
            .matches("value_kind: NumericUpdateValueKind,")
            .count(),
        2
    );
    assert_eq!(
        REFERENCE_SOURCE
            .matches("value_kind: NumericUpdateValueKind,")
            .count(),
        3
    );
    assert!(!IR_SOURCE.contains("value_kind: ValueKind"));
    assert!(!REFERENCE_SOURCE.contains("value_kind: ValueKind"));
}

#[test]
fn backend_consumers_are_exhaustive_and_have_no_impossible_kind_branch() {
    let delta = bounded(
        BACKEND_OPERATIONS_SOURCE,
        "pub(crate) fn emit_update_delta_from_locals(",
        "pub(crate) fn compile_truthy_i32(",
    );
    for variant in ["Number", "BigInt", "Dynamic"] {
        assert_eq!(
            delta
                .matches(&format!("NumericUpdateValueKind::{variant} =>"))
                .count(),
            1
        );
    }
    assert!(!delta.contains("unreachable!"));
    assert!(!delta.contains("_ =>"));
    assert!(!BACKEND_OPERATIONS_SOURCE.contains("fn emit_update_delta("));

    for source in [
        EXPRESSIONS_SOURCE,
        SUPER_EXPRESSIONS_SOURCE,
        PLANNING_SOURCE,
    ] {
        assert!(!source.contains("numeric update requires Number, BigInt, or Dynamic"));
        assert!(!source.contains("ordinary property numeric update kind is closed"));
    }
}

#[test]
fn contract_and_task_record_the_closed_numeric_update_boundary() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("NumericUpdateValueKind"));
        assert!(source.contains("Number"));
        assert!(source.contains("BigInt"));
        assert!(source.contains("Dynamic"));
    }
}
