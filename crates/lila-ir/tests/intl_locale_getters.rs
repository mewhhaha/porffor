use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, PropertyKeyIr, SpecOperationIr, StandardBuiltinId, StatementIr, ValueKind,
};

#[test]
fn locale_numeric_accessor_retains_constructor_and_live_get() {
    let program = lower(&parse("new Intl.Locale('en').numeric;", ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
        panic!("numeric remains the result expression");
    };
    assert!(result.possible_kinds.contains(ValueKind::Boolean));
    let target = match &result.expr {
        ExprIr::PropertyRead {
            target,
            key: PropertyKeyIr::StaticString(key),
        } => {
            assert_eq!(key, "numeric");
            target.as_ref()
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => {
            assert_eq!(operands.len(), 2);
            assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == "numeric"));
            &operands[0]
        }
        _ => panic!("numeric must read the live prototype: {result:?}"),
    };
    let ExprIr::Construct { args, .. } = &target.expr else {
        panic!("the original Locale construction remains: {target:?}");
    };
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0].expr, ExprIr::String(locale) if locale == "en"));
}

#[test]
fn optional_locale_getters_remain_dynamic_and_are_not_constructors() {
    for (property, getter) in [
        (
            "calendar",
            StandardBuiltinId::IntlLocalePrototypeCalendarGetter,
        ),
        (
            "collation",
            StandardBuiltinId::IntlLocalePrototypeCollationGetter,
        ),
        (
            "firstDayOfWeek",
            StandardBuiltinId::IntlLocalePrototypeFirstDayOfWeekGetter,
        ),
        (
            "hourCycle",
            StandardBuiltinId::IntlLocalePrototypeHourCycleGetter,
        ),
        (
            "caseFirst",
            StandardBuiltinId::IntlLocalePrototypeCaseFirstGetter,
        ),
        (
            "numberingSystem",
            StandardBuiltinId::IntlLocalePrototypeNumberingSystemGetter,
        ),
        (
            "variants",
            StandardBuiltinId::IntlLocalePrototypeVariantsGetter,
        ),
    ] {
        let source = format!("new Intl.Locale('en').{property};");
        let program = lower(&parse(&source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert_eq!(
            program.script.unwrap().result_kind(),
            ValueKind::Dynamic,
            "{source}"
        );
        assert!(!getter.constructable());
    }
    assert!(!StandardBuiltinId::IntlLocalePrototypeNumericGetter.constructable());
}
