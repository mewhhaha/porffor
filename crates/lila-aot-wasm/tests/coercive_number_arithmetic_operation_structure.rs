const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/coercive-number-arithmetic-operation.md");
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
fn number_arithmetic_emission_exhaustively_matches_every_ir_operation() {
    let body = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_coercive_binary_number_to_locals(",
        "pub(crate) fn emit_primitive_to_numeric_locals(",
    );
    let number_branch = bounded(
        body,
        "\n        match op {",
        "function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));",
    );
    for operation in ["Add", "Sub", "Mul", "Div", "Mod", "Exp"] {
        assert_eq!(
            number_branch
                .matches(&format!("ArithmeticBinaryOp::{operation} =>"))
                .count(),
            1
        );
    }
    assert!(!number_branch.contains("matches!(op"));
    assert!(!number_branch.contains("unreachable!"));
    assert!(!number_branch.contains("_ =>"));
}

#[test]
fn every_number_operation_uses_its_authoritative_wasm_emitter() {
    let owner = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_coercive_binary_number_to_locals(",
        "pub(crate) fn emit_primitive_to_numeric_locals(",
    );
    let body = normalized(bounded(
        owner,
        "\n        match op {",
        "function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));",
    ));
    for (operation, instruction) in [
        ("Add", "F64Add"),
        ("Sub", "F64Sub"),
        ("Mul", "F64Mul"),
        ("Div", "F64Div"),
    ] {
        let arm = concat!(
            "function.instruction(&Instruction::LocalGet(lhs_payload_local));",
            "function.instruction(&Instruction::F64ReinterpretI64);",
            "function.instruction(&Instruction::LocalGet(rhs_payload_local));",
            "function.instruction(&Instruction::F64ReinterpretI64);"
        );
        let suffix = concat!(
            "function.instruction(&Instruction::I64ReinterpretF64);",
            "function.instruction(&Instruction::LocalSet(payload_local));}"
        );
        assert!(body.contains(&format!(
            "ArithmeticBinaryOp::{operation}=>{{{arm}function.instruction(&Instruction::{instruction});{suffix}"
        )));
    }
    assert!(body.contains(concat!(
        "ArithmeticBinaryOp::Mod=>{",
        "self.emit_number_remainder_payload(lhs_payload_local,rhs_payload_local,",
        "payload_local,function,);}"
    )));
    assert!(body.contains(concat!(
        "ArithmeticBinaryOp::Exp=>{",
        "self.emit_number_pow_payload(lhs_payload_local,rhs_payload_local,",
        "payload_local,function,)?;}"
    )));
}

#[test]
fn remainder_has_one_integer_reduction_owner_and_all_four_consumers() {
    let remainder = include_str!("../src/operations/number_remainder.rs");
    let expressions = include_str!("../src/expressions.rs");
    assert_eq!(
        remainder
            .matches("fn emit_number_remainder_payload(")
            .count(),
        1
    );
    assert_eq!(remainder.matches("self.reserve_temp_local()").count(), 5);
    assert_eq!(remainder.matches("self.release_temp_local(").count(), 5);
    assert_eq!(
        OPERATIONS_SOURCE
            .matches("self.emit_number_remainder_payload(")
            .count(),
        1
    );
    assert_eq!(
        expressions
            .matches("self.emit_number_remainder_payload(")
            .count(),
        3
    );
    assert!(!remainder.contains("Instruction::F64Div"));
    assert!(!remainder.contains("Instruction::F64Trunc"));
    assert!(remainder.contains("Instruction::I64Clz"));
    assert!(remainder.contains("Instruction::I64Sub"));
    assert!(remainder.contains("Instruction::I64Shl"));
    assert!(remainder.contains("Instruction::I64ShrU"));
}

#[test]
fn contract_and_task_record_total_number_arithmetic_ownership() {
    for source in [CONTRACT, TASK] {
        assert!(source.contains("ArithmeticBinaryOp"));
        assert!(source.contains("Add"));
        assert!(source.contains("Mod"));
        assert!(source.contains("Exp"));
    }
}
