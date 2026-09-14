use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, SpecOperationIr, StatementIr, ValueKind};

#[test]
fn number_calls_keep_bigint_policy_for_every_object_like_input() {
    for source in [
        "function value() {} Number(value);",
        "var value = {}; Number(value);",
        "var value = []; Number(value);",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
            panic!("expected final Number call: {source}");
        };
        assert_eq!(value.kind, ValueKind::Number);
        assert!(
            !matches!(
                value.expr,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::ToNumber,
                    ..
                }
            ),
            "Number must retain its BigInt policy after ToPrimitive: {source}"
        );
    }
}

#[test]
fn number_calls_on_arguments_do_not_fold_to_to_number() {
    let program = lower(
        &parse(
            "function convert() { return Number(arguments); } convert(1);",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let convert = script
        .functions
        .iter()
        .find(|function| function.name == "convert")
        .unwrap();
    let StatementIr::Return(value) = convert.body.statements.last().unwrap() else {
        panic!("expected Number call return");
    };
    assert!(!matches!(
        value.expr,
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToNumber,
            ..
        }
    ));
}
