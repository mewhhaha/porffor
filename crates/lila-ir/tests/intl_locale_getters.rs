use lila_front::{parse, ParseOptions};
use lila_ir::{lower, StandardBuiltinId, ValueKind};

#[test]
fn locale_numeric_accessor_has_boolean_result_in_ir() {
    let program = lower(&parse("new Intl.Locale('en').numeric;", ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert_eq!(program.script.unwrap().result_kind(), ValueKind::Boolean);
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
