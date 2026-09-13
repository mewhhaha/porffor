use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ArithmeticBinaryOp, ExprIr, StatementIr, Strictness, TypedExpr, ValueKind};

fn last_expression(source: &str) -> TypedExpr {
    let unit = parse(source, ParseOptions::script()).expect("remainder fixture parses");
    let lowered = lower(&unit);
    assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
    let mut statements = lowered.script.expect("script IR").body.statements;
    let Some(StatementIr::Expression(expression)) = statements.pop() else {
        panic!("remainder must remain the final expression");
    };
    expression
}

#[test]
fn remainder_retains_raw_operands_until_the_ordered_numeric_operation() {
    let expression = last_expression(
        "const left = { valueOf() { return 7; } }; \
         const right = { valueOf() { return 3; } }; left % right;",
    );
    let ExprIr::CoerciveBinaryNumber {
        op: ArithmeticBinaryOp::Mod,
        lhs,
        rhs,
    } = expression.expr
    else {
        panic!("remainder must keep the canonical numeric operation");
    };
    assert_eq!(lhs.kind, ValueKind::Object);
    assert_eq!(rhs.kind, ValueKind::Object);
    assert!(matches!(lhs.expr, ExprIr::Identifier(_)));
    assert!(matches!(rhs.expr, ExprIr::Identifier(_)));
}

#[test]
fn compound_remainder_consumes_the_old_value_from_one_strict_property_reference() {
    let expression =
        last_expression("'use strict'; const target = {value: -1}; target.value %= -1;");
    let ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) = expression.expr else {
        panic!("compound remainder must retain its property Reference");
    };
    assert_eq!(assignment.strictness(), Strictness::Strict);
    let ExprIr::CoerciveBinaryNumber {
        op: ArithmeticBinaryOp::Mod,
        lhs,
        ..
    } = &assignment.result().expr
    else {
        panic!("the compound operation must use the shared numeric route");
    };
    let ExprIr::Identifier(name) = &lhs.expr else {
        panic!("the old value must be read from the Reference's retained slot");
    };
    assert_eq!(name, assignment.old_value_binding());
}
