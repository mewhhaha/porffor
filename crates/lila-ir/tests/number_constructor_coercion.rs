use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, PropertyKeyIr, SpecOperationIr, StandardBuiltinId, StatementIr, TypedExpr,
    ValueKind,
};

fn predicate_call<'a>(value: &'a TypedExpr, spelling: &str) -> &'a TypedExpr {
    let Some(property) = spelling.strip_prefix("globalThis.") else {
        return value;
    };
    let ExprIr::MaterializeBinding {
        name,
        value: receiver,
        body,
    } = &value.expr
    else {
        panic!("the globalThis property call must evaluate its receiver once: {value:?}");
    };
    assert!(
        matches!(&receiver.expr, ExprIr::ExecutionGlobalObject),
        "these root calls capture the proven initial global object: {receiver:?}"
    );
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        ..
    } = &body.expr
    else {
        panic!("the property call must retain its receiver: {body:?}");
    };
    assert!(matches!(&this_arg.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(matches!(&callee.expr, ExprIr::PropertyRead {
        target, key: PropertyKeyIr::StaticString(key),
    } if matches!(&target.expr, ExprIr::Identifier(storage) if storage == name)
        && key == property));
    body
}

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

#[test]
fn coercing_global_predicates_keep_literal_strings_in_real_calls() {
    for (source, builtin, input) in [
        ("isNaN('0x10');", StandardBuiltinId::GlobalIsNaN, "0x10"),
        (
            "isFinite('0O17');",
            StandardBuiltinId::GlobalIsFinite,
            "0O17",
        ),
        (
            "isNaN('\\u00A01\\u2029');",
            StandardBuiltinId::GlobalIsNaN,
            "\u{00A0}1\u{2029}",
        ),
        (
            "globalThis.isFinite('0b10');",
            StandardBuiltinId::GlobalIsFinite,
            "0b10",
        ),
        (
            "globalThis.isNaN('inf');",
            StandardBuiltinId::GlobalIsNaN,
            "inf",
        ),
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
            panic!("expected predicate expression: {source}");
        };
        let spelling = source.split_once('(').unwrap().0;
        let ExprIr::CallIndirect { callee, args, .. } = &predicate_call(value, spelling).expr
        else {
            panic!("ToNumber must remain an emitted call: {source}: {value:?}");
        };
        assert_eq!(value.kind, ValueKind::Boolean);
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id()),
            "the original intrinsic callee must survive: {source}"
        );
        assert_eq!(args.len(), 1, "{source}");
        assert!(matches!(&args[0].expr, ExprIr::String(value) if value == input));
    }
    for source in ["isNaN();", "isFinite();"] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
            panic!("expected no-argument predicate expression");
        };
        let ExprIr::CallIndirect { args, .. } = &value.expr else {
            panic!("missing argument must be supplied by the genuine builtin: {value:?}");
        };
        assert!(args.is_empty());
    }
}

#[test]
fn coercing_global_predicates_keep_arrays_after_prototype_hooks_change() {
    for predicate in ["isNaN", "isFinite"] {
        for literal in ["[]", "[1]", "[null]", "[undefined]", "[true]"] {
            let source = format!(
                "Array.prototype.toString = function () {{ throw 'conversion marker'; }}; \
                 {predicate}({literal});"
            );
            let program = lower(&parse(&source, ParseOptions::script()).unwrap());
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
                panic!("expected array-coercing predicate: {source}");
            };
            let ExprIr::CallIndirect { args, .. } = &value.expr else {
                panic!("array ToPrimitive must observe the live hook: {source}: {value:?}");
            };
            assert_eq!(args.len(), 1, "{source}");
            assert!(matches!(args[0].expr, ExprIr::ArrayLiteral(_)), "{source}");
        }
    }
}

#[test]
fn coercing_global_predicates_retain_ignored_argument_effects_after_the_first_value() {
    for predicate in [
        "isNaN",
        "isFinite",
        "globalThis.isNaN",
        "globalThis.isFinite",
    ] {
        let source = format!(
            "var observations = 0; \
             function observe() {{ observations++; return 7; }} \
             {predicate}('0x10', observe());"
        );
        let program = lower(&parse(&source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let observe = script
            .functions
            .iter()
            .find(|function| function.name == "observe")
            .unwrap();
        let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
            panic!("expected predicate with an observable ignored argument: {source}");
        };
        let ExprIr::CallIndirect { args, .. } = &predicate_call(value, predicate).expr else {
            panic!("predicate invocation must retain argument evaluation: {source}: {value:?}");
        };
        assert_eq!(args.len(), 2, "{source}");
        assert!(matches!(&args[0].expr, ExprIr::String(value) if value == "0x10"));
        let ExprIr::CallIndirect { callee, .. } = &args[1].expr else {
            panic!(
                "second argument must retain its observation: {source}: {:?}",
                args[1]
            );
        };
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&observe.id)
        );
    }
}
