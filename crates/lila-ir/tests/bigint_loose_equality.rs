use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, SpecOperationIr, StatementIr};

#[test]
fn both_operators_keep_the_shared_runtime_equality_operation() {
    for operator in ["==", "!="] {
        let unit = parse(
            &format!("function compare(left, right) {{ return left {operator} right; }}"),
            ParseOptions::script(),
        )
        .expect("comparison parses");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "compare")
            .expect("compare");
        let value = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(value) => Some(value),
                _ => None,
            })
            .expect("return");
        let equality = if operator == "!=" {
            let ExprIr::LogicalNot { expr } = &value.expr else {
                panic!("inequality negates equality: {value:?}");
            };
            expr.as_ref()
        } else {
            value
        };
        assert!(
            matches!(&equality.expr, ExprIr::SpecOperation {
            operation: SpecOperationIr::IsLooselyEqual, operands,
        } if operands.len() == 2),
            "operands require runtime coercion: {equality:?}"
        );
    }
}
