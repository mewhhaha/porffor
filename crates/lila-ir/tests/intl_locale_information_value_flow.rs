use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, PropertyKeyIr, ScriptIr, SpecOperationIr, StandardBuiltinId, StatementIr,
    ValueKind,
};
fn lower_script(source: &str) -> ScriptIr {
    let program =
        lower(&parse(source, ParseOptions::script()).expect("Locale value-flow source parses"));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}
#[test]
fn optional_time_zone_list_survives_a_binding_and_retained_expression() {
    let script =
        lower_script("let locale=new Intl.Locale('en');let value=locale.getTimeZones();value;");
    let StatementIr::Lexical { init, .. } = &script.body.statements[1] else {
        panic!("binding owner")
    };
    assert!(init.possible_kinds.contains(ValueKind::Undefined));
    assert!(init.possible_kinds.contains(ValueKind::Array));
    let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
        panic!("retained optional value")
    };
    assert!(value.possible_kinds.contains(ValueKind::Undefined));
    assert!(value.possible_kinds.contains(ValueKind::Array));
    assert_eq!(script.result_kind(), ValueKind::Dynamic);
}
#[test]
fn optional_time_zone_list_preserves_runtime_coercion_and_undefined_comparison() {
    for (tail, kind) in [
        ("value===undefined;", ValueKind::Boolean),
        ("value+'';", ValueKind::String),
    ] {
        let script = lower_script(&format!(
            "let locale=new Intl.Locale('en');let value=locale.getTimeZones();{tail}"
        ));
        assert_eq!(script.result_kind(), kind);
    }
}
#[test]
fn calendar_and_collation_default_calls_retain_live_callees_and_stored_results() {
    for (name, builtin) in [
        (
            "getCalendars",
            StandardBuiltinId::IntlLocalePrototypeGetCalendars,
        ),
        (
            "getCollations",
            StandardBuiltinId::IntlLocalePrototypeGetCollations,
        ),
    ] {
        let script = lower_script(&format!(
            "let locale=new Intl.Locale('en');let value=locale.{name}();value;"
        ));
        let StatementIr::Lexical {
            name: result_name,
            init,
            ..
        } = &script.body.statements[1]
        else {
            panic!("the call result remains in its source binding");
        };
        assert!(init.possible_kinds.contains(ValueKind::Array));
        let ExprIr::MaterializeBinding {
            name: receiver_name,
            value,
            body,
        } = &init.expr
        else {
            panic!("the Locale receiver is acquired once: {init:?}");
        };
        assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == "locale"));
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(receiver),
            args,
            ..
        } = &body.expr
        else {
            panic!("the acquired information method is called: {body:?}");
        };
        assert!(args.is_empty());
        assert!(matches!(&receiver.expr, ExprIr::Identifier(name) if name == receiver_name));
        assert!(callee
            .function_targets
            .known_targets()
            .contains(&builtin.function_id()));
        let target = match &callee.expr {
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(key),
            } => {
                assert_eq!(key, name);
                target.as_ref()
            }
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } => {
                assert_eq!(operands.len(), 2);
                assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == name));
                &operands[0]
            }
            _ => panic!("the information method must retain its live Get: {callee:?}"),
        };
        assert!(matches!(&target.expr, ExprIr::Identifier(name) if name == receiver_name));
        let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
            panic!("the stored result remains an expression");
        };
        assert!(result.possible_kinds.contains(ValueKind::Array));
        assert!(matches!(&result.expr, ExprIr::Identifier(name) if name == result_name));
    }
}
