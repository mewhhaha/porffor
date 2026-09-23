use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, StatementIr, ValueKind};

fn lower_addition(source: &str, name: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("compound addition parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == name)
        .expect("addition function")
}

#[test]
fn unknown_compound_addition_retains_numeric_and_string_results() {
    let function = lower_addition("function add(value) { value += 1; return value; }", "add");
    let StatementIr::Expression(assignment) = &function.body.statements[0] else {
        panic!("compound assignment expression");
    };
    assert_eq!(assignment.kind, ValueKind::Dynamic);
    assert!(assignment.possible_kinds.contains(ValueKind::Number));
    assert!(assignment.possible_kinds.contains(ValueKind::String));
    let ExprIr::AssignIdentifier { value, .. } = &assignment.expr else {
        panic!("write the shared addition result");
    };
    assert!(matches!(value.expr, ExprIr::CoerciveAdd { .. }));
    let StatementIr::Return(result) = &function.body.statements[1] else {
        panic!("read assigned result");
    };
    assert_eq!(result.possible_kinds, assignment.possible_kinds);
}

#[test]
fn unknown_right_operand_does_not_make_a_numeric_binding_a_proven_string() {
    let function = lower_addition(
        "function add(right) { let value = 1; value += right; return value; }",
        "add",
    );
    let StatementIr::Expression(assignment) = &function.body.statements[1] else {
        panic!("compound assignment expression");
    };
    assert_eq!(assignment.kind, ValueKind::Dynamic);
    assert!(assignment.possible_kinds.contains(ValueKind::Number));
    assert!(assignment.possible_kinds.contains(ValueKind::String));
}

#[test]
fn skipped_and_taken_statement_branches_merge_lexical_value_domains() {
    for source in [
        "function choose(flag) { let value = 1; const read = () => { if (flag) value = 'x'; return value + 1; }; return read(); }",
        "function choose(flag) { let value = 1; const read = () => { if (flag) value = 'x'; else value += 1; return value + 1; }; return read(); }",
    ] {
        let function = lower_addition(source, "read");
        let StatementIr::Return(result) = function.body.statements.last().expect("return") else {
            panic!("read merged lexical binding");
        };
        assert_eq!(result.kind, ValueKind::Dynamic);
        assert!(result.possible_kinds.contains(ValueKind::Number));
        assert!(result.possible_kinds.contains(ValueKind::String));
    }
}
