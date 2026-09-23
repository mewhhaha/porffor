use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, SpecOperationIr, StatementIr, TypedExpr, ValueKind};

#[test]
fn computed_numeric_calls_keep_get_v_and_the_same_materialized_receiver() {
    for (source, kind) in [
        ("(1)['toLocaleString']('en-US');", ValueKind::Number),
        ("(1n)['toLocaleString']('en-US');", ValueKind::BigInt),
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("computed call parses");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::MaterializeBinding { name, value, body },
            ..
        }) = script.body.statements.last().expect("method call")
        else {
            panic!("computed primitive receiver must be materialized: {source}");
        };
        assert_eq!(value.kind, kind);
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(this_arg),
            ..
        } = &body.expr
        else {
            panic!("computed property remains an observable call: {body:?}");
        };
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } = &callee.expr
        else {
            panic!("computed callee must use GetV: {callee:?}");
        };
        assert_eq!(operands.len(), 2);
        assert!(matches!(&operands[0].expr, ExprIr::Identifier(base) if base == name));
        assert!(matches!(&this_arg.expr, ExprIr::Identifier(receiver) if receiver == name));
        assert_eq!(this_arg.kind, kind);
        assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == "toLocaleString"));
    }
}
