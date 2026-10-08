use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, ScriptIr, SpecOperationIr, StatementIr, TypedExpr};

fn lower_script(source: &str) -> ScriptIr {
    let program = lower(&parse(source, ParseOptions::script()).expect("valid source"));
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program.script.expect("script IR")
}

fn final_expression(script: &ScriptIr) -> &TypedExpr {
    let StatementIr::Expression(expression) = script.body.statements.last().expect("expression")
    else {
        panic!("expected final expression");
    };
    expression
}

fn call_body(mut expression: &TypedExpr) -> &TypedExpr {
    while let ExprIr::MaterializeBinding { body, .. } = &expression.expr {
        expression = body;
    }
    assert!(
        matches!(expression.expr, ExprIr::CallIndirect { .. }),
        "{expression:?}"
    );
    expression
}

#[test]
fn primitive_constructor_and_parse_float_calls_retain_ignored_arguments() {
    for invocation in [
        "Number('2', observe())",
        "Boolean(false, observe())",
        "String('text', observe())",
        "parseFloat('2tail', observe())",
        "Symbol(undefined, observe())",
    ] {
        let source = format!("function observe() {{ return 7; }} {invocation};");
        let script = lower_script(&source);
        let ExprIr::CallIndirect { args, .. } = &call_body(final_expression(&script)).expr else {
            unreachable!();
        };
        assert_eq!(args.len(), 2, "{invocation}");
        assert!(
            matches!(args[1].expr, ExprIr::CallIndirect { .. }),
            "{invocation}: {args:?}"
        );
    }
}

#[test]
fn undefined_symbol_description_retains_its_evaluation() {
    let script = lower_script("function observe() {} Symbol(observe());");
    let ExprIr::CallIndirect { args, .. } = &call_body(final_expression(&script)).expr else {
        unreachable!();
    };
    assert_eq!(args.len(), 1);
    assert!(
        matches!(args[0].expr, ExprIr::CallIndirect { .. }),
        "{args:?}"
    );
}

#[test]
fn primitive_method_calls_acquire_the_property_and_retain_receiver_evaluation() {
    for invocation in [
        "Number(5, observe()).toString()",
        "new Number(5, observe()).toFixed(1)",
        "Number(5, observe()).toExponential(1)",
        "Number(5, observe()).toPrecision(1)",
        "Boolean(false, observe()).toString()",
        "new Boolean(false, observe()).valueOf()",
    ] {
        let source = format!("function observe() {{ return 7; }} {invocation};");
        let script = lower_script(&source);
        let ExprIr::MaterializeBinding {
            value: receiver,
            body,
            ..
        } = &final_expression(&script).expr
        else {
            panic!("receiver evaluation must survive: {invocation}");
        };
        let args = match &receiver.expr {
            ExprIr::CallIndirect { args, .. } | ExprIr::Construct { args, .. } => args,
            _ => panic!("expected receiver call or construction: {receiver:?}"),
        };
        assert_eq!(args.len(), 2, "{invocation}");
        assert!(
            matches!(args[1].expr, ExprIr::CallIndirect { .. }),
            "{args:?}"
        );
        let ExprIr::CallIndirect { callee, .. } = &call_body(body).expr else {
            unreachable!();
        };
        assert!(
            matches!(
                callee.expr,
                ExprIr::PropertyRead { .. }
                    | ExprIr::SpecOperation {
                        operation: SpecOperationIr::GetV,
                        ..
                    }
            ),
            "property Get must survive: {invocation}: {callee:?}"
        );
    }
}

#[test]
fn constant_arguments_do_not_replace_mutable_property_calls() {
    for invocation in [
        "Object.is(1, 1)",
        "Object.keys({})",
        "Object.values({})",
        "Object.entries({})",
        "String.fromCharCode(65)",
        "String.fromCodePoint(65)",
        "String.raw({ raw: ['text'] })",
        "Math.pow(2, 3)",
        "Math.clz32(1)",
        "Math.round(1.5)",
        "globalThis.propertyIsEnumerable('NaN')",
        "Number.propertyIsEnumerable('MAX_VALUE')",
        "Boolean.propertyIsEnumerable('prototype')",
        "RangeError.propertyIsEnumerable('prototype')",
        "(1).toString()",
        "(1).toFixed(1)",
        "(1).toExponential(1)",
        "(1).toPrecision(1)",
        "true.toString()",
        "true.valueOf()",
        "'text'.toString()",
        "'text'.valueOf()",
    ] {
        let script = lower_script(&format!("{invocation};"));
        let ExprIr::CallIndirect { callee, .. } = &call_body(final_expression(&script)).expr else {
            unreachable!();
        };
        assert!(
            matches!(
                callee.expr,
                ExprIr::PropertyRead { .. }
                    | ExprIr::SpecOperation {
                        operation: SpecOperationIr::GetV,
                        ..
                    }
            ),
            "mutable method must be acquired: {invocation}: {callee:?}"
        );
    }
}

#[test]
fn shadowed_numeric_constant_properties_keep_getter_ir() {
    let script = lower_script(
        "function read(Number) { return +Number.NaN; } read({ get NaN() { return 7; } });",
    );
    let read = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .unwrap();
    let StatementIr::Return(value) = read.body.statements.last().unwrap() else {
        panic!("expected property coercion return");
    };
    assert!(!matches!(value.expr, ExprIr::Number(_)), "{value:?}");
}
