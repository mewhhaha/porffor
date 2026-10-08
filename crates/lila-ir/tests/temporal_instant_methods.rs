use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, ObjectPropertyIr, PropertyKeyIr, SpecOperationIr, StandardBuiltinId,
    StatementIr, TypedExpr, ValueKind,
};

fn property_target<'a>(expression: &'a TypedExpr, name: &str) -> &'a TypedExpr {
    match &expression.expr {
        ExprIr::PropertyRead {
            target,
            key: PropertyKeyIr::StaticString(key),
        } => {
            assert_eq!(key, name);
            target
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => {
            assert_eq!(operands.len(), 2);
            assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == name));
            &operands[0]
        }
        _ => panic!("{name} must retain its live property read: {expression:?}"),
    }
}

fn acquired_instant_call(expression: &TypedExpr, builtin: StandardBuiltinId) -> &[TypedExpr] {
    let ExprIr::MaterializeBinding { name, value, body } = &expression.expr else {
        panic!("the Instant receiver is constructed once: {expression:?}");
    };
    let ExprIr::Construct {
        args: constructor_args,
        ..
    } = &value.expr
    else {
        panic!("the receiver keeps the original constructor call: {value:?}");
    };
    assert_eq!(constructor_args.len(), 1);
    assert!(matches!(constructor_args[0].expr, ExprIr::BigInt(_)));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("the method is acquired before its arguments: {body:?}");
    };
    let target = property_target(callee, builtin.native_function_name().unwrap());
    assert!(matches!(&target.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(callee
        .function_targets
        .known_targets()
        .contains(&builtin.function_id()));
    args
}

#[test]
fn instant_methods_retain_calls_arguments_and_result_property_reads() {
    for (name, builtin, arguments, property, kind) in [
        (
            "add",
            StandardBuiltinId::TemporalInstantPrototypeAdd,
            "{nanoseconds:1}",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "subtract",
            StandardBuiltinId::TemporalInstantPrototypeSubtract,
            "{nanoseconds:1}",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "round",
            StandardBuiltinId::TemporalInstantPrototypeRound,
            "'nanosecond'",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "until",
            StandardBuiltinId::TemporalInstantPrototypeUntil,
            "new Temporal.Instant(1n)",
            "nanoseconds",
            ValueKind::Number,
        ),
        (
            "since",
            StandardBuiltinId::TemporalInstantPrototypeSince,
            "new Temporal.Instant(1n)",
            "nanoseconds",
            ValueKind::Number,
        ),
    ] {
        assert!(builtin.may_run_user_code_synchronously());
        assert_eq!(builtin.native_function_name(), Some(name));
        let source = format!("new Temporal.Instant(0n).{name}({arguments}).{property};");
        let unit = parse(&source, ParseOptions::script()).expect("Instant fixture parses");
        let program = lower(&unit);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("script IR");
        let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
            panic!("the result property remains an expression");
        };
        assert!(result.possible_kinds.contains(kind), "{source}");
        let args = acquired_instant_call(property_target(result, property), builtin);
        assert_eq!(args.len(), 1);
        match name {
            "add" | "subtract" => {
                let ExprIr::ObjectLiteral(properties) = &args[0].expr else {
                    panic!("the duration bag remains observable: {:?}", args[0]);
                };
                let [ObjectPropertyIr::Data { key, value, .. }] = properties.as_slice() else {
                    panic!("the original one-field duration remains: {properties:?}");
                };
                assert_eq!(key, "nanoseconds");
                assert!(matches!(value.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 1.0));
            }
            "round" => {
                assert!(matches!(&args[0].expr, ExprIr::String(unit) if unit == "nanosecond"))
            }
            "until" | "since" => assert!(matches!(args[0].expr, ExprIr::Construct { .. })),
            _ => unreachable!("the fixture lists the five arithmetic methods"),
        }
    }
}

#[test]
fn instant_locale_method_retains_default_and_explicit_arguments_and_user_code_authority() {
    let builtin = StandardBuiltinId::TemporalInstantPrototypeToLocaleString;
    assert_eq!(builtin.native_function_name(), Some("toLocaleString"));
    assert!(!builtin.constructable());
    assert!(builtin.may_run_user_code_synchronously());
    assert!(builtin.requires_intl_host());
    assert_eq!(
        StandardBuiltinId::from_function_id(&builtin.function_id()),
        Some(builtin)
    );
    for (source, argument_count) in [
        ("new Temporal.Instant(0n).toLocaleString();", 0),
        (
            "new Temporal.Instant(0n)['toLocaleString']('en-US', {timeZone:'UTC'});",
            2,
        ),
    ] {
        let unit = parse(source, ParseOptions::script()).expect("locale fixture parses");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
            panic!("the locale call remains the result expression");
        };
        assert!(
            result.possible_kinds.contains(ValueKind::String),
            "{source}"
        );
        let args = acquired_instant_call(result, builtin);
        assert_eq!(args.len(), argument_count);
        if argument_count != 0 {
            assert!(matches!(&args[0].expr, ExprIr::String(locale) if locale == "en-US"));
            let ExprIr::ObjectLiteral(properties) = &args[1].expr else {
                panic!("locale options remain an actual argument: {:?}", args[1]);
            };
            let [ObjectPropertyIr::Data { key, value, .. }] = properties.as_slice() else {
                panic!("the original options field remains: {properties:?}");
            };
            assert_eq!(key, "timeZone");
            assert!(matches!(&value.expr, ExprIr::String(zone) if zone == "UTC"));
        }
    }
}
