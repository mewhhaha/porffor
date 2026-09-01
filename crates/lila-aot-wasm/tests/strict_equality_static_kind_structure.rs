const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/strict-equality-static-kind-domain.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn singleton_strict_equality_exhaustively_classifies_every_value_kind() {
    let body = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_strict_equality_i32(",
        "pub(crate) fn emit_assert_same_value(",
    );
    for kind in [
        "Undefined",
        "Null",
        "Boolean",
        "Number",
        "BigInt",
        "Symbol",
        "String",
        "Object",
        "Array",
        "Arguments",
        "Function",
        "Dynamic",
    ] {
        assert_eq!(body.matches(&format!("ValueKind::{kind}")).count(), 1);
    }
    assert!(!body.contains("_ =>"));
    assert!(!body.contains("unreachable!"));
}

#[test]
fn singleton_strict_equality_retains_the_four_exact_algorithms() {
    let source = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_strict_equality_i32(",
        "pub(crate) fn emit_assert_same_value(",
    );
    let body = normalized(source);
    assert!(body.contains(concat!(
        "ValueKind::Number=>{",
        "self.compile_expr_payload(lhs,function)?;",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "self.compile_expr_payload(rhs,function)?;",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "function.instruction(&Instruction::F64Eq);"
    )));
    let string_arm = normalized(bounded(
        source,
        "ValueKind::String => {",
        "ValueKind::Function | ValueKind::BigInt | ValueKind::Dynamic => {",
    ));
    assert!(string_arm.contains(concat!(
        "self.compile_expr_payload(lhs,function)?;",
        "self.compile_expr_payload(rhs,function)?;",
        "function.instruction(&Instruction::LocalSet(self.result_local));",
        "function.instruction(&Instruction::LocalSet(self.scratch_local));",
        "self.emit_string_payload_equality_i32("
    )));
    assert!(body.contains(concat!(
        "ValueKind::Function|ValueKind::BigInt|ValueKind::Dynamic=>{",
        "letlhs_payload=self.reserve_temp_local();"
    )));
    assert!(body.contains(concat!(
        "ValueKind::Undefined|ValueKind::Null|ValueKind::Boolean|",
        "ValueKind::Symbol|ValueKind::Object|ValueKind::Array|",
        "ValueKind::Arguments=>{",
        "self.compile_expr_payload(lhs,function)?;",
        "self.compile_expr_payload(rhs,function)?;",
        "function.instruction(&Instruction::I64Eq);"
    )));
}

#[test]
fn contract_and_task_record_total_strict_equality_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("ValueKind"));
        assert!(source.contains("Dynamic"));
        assert!(source.contains("tagged equality"));
        assert!(source.contains("raw-payload equality"));
    }
}
