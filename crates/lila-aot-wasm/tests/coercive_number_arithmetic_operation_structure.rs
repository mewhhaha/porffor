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
        "pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(",
    );
    let number_branch = bounded(
        body,
        "function.instruction(&Instruction::Else);\n        match op {",
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
fn every_number_operation_retains_its_exact_wasm_sequence() {
    let body = normalized(bounded(
        OPERATIONS_SOURCE,
        "function.instruction(&Instruction::Else);\n        match op {",
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
        "function.instruction(&Instruction::LocalGet(lhs_payload_local));",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "function.instruction(&Instruction::LocalGet(lhs_payload_local));",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "function.instruction(&Instruction::LocalGet(rhs_payload_local));",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "function.instruction(&Instruction::F64Div);",
        "function.instruction(&Instruction::F64Trunc);",
        "function.instruction(&Instruction::LocalGet(rhs_payload_local));",
        "function.instruction(&Instruction::F64ReinterpretI64);",
        "function.instruction(&Instruction::F64Mul);",
        "function.instruction(&Instruction::F64Sub);",
        "function.instruction(&Instruction::I64ReinterpretF64);",
        "function.instruction(&Instruction::LocalSet(payload_local));}"
    )));
    assert!(body.contains(concat!(
        "ArithmeticBinaryOp::Exp=>{",
        "self.emit_number_pow_payload(lhs_payload_local,rhs_payload_local,",
        "payload_local,function,)?;}"
    )));
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
