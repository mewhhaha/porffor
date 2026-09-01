const DATE_SOURCE: &str = include_str!("../src/builtins/date.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/date-component-setter-operation.md");
const T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const T22: &str = include_str!("../../../tasks/22-date-temporal.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn bounded_inclusive<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let from_start = &source[start_index..];
    let end_index = from_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"));
    &from_start[..end_index]
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn setter_body() -> &'static str {
    bounded(
        DATE_SOURCE,
        "    fn emit_date_component_setter(",
        "    pub(crate) fn emit_date_append_padded_decimal(",
    )
}

#[test]
fn component_setter_operation_is_an_exact_closed_domain() {
    let variants = bounded(
        DATE_SOURCE,
        "enum DateComponentSetterOperation {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a>",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();

    assert_eq!(
        variants,
        [
            "FullYear,",
            "Month,",
            "Date,",
            "Hours,",
            "Minutes,",
            "Seconds,",
            "Milliseconds,",
        ]
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!DATE_SOURCE.contains(&format!(
            "impl {capability} for DateComponentSetterOperation"
        )));
    }
    assert!(!DATE_SOURCE.contains("pub(super) enum DateComponentSetterOperation"));
    for evidence in [CONTRACT, T02, T22] {
        assert!(evidence.contains("private `DateComponentSetterOperation`"));
        assert!(evidence.contains("fixed Date setter entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new Date behavior"));
    }
}

#[test]
fn all_five_setter_operation_decisions_are_exhaustive() {
    let body = setter_body();

    assert_eq!(body.matches("match operation {").count(), 5);
    for variant in [
        "FullYear",
        "Month",
        "Date",
        "Hours",
        "Minutes",
        "Seconds",
        "Milliseconds",
    ] {
        assert_eq!(
            body.matches(&format!("DateComponentSetterOperation::{variant}"))
                .count(),
            5,
            "operation `{variant}` must be classified at every semantic decision"
        );
    }

    for forbidden in [
        "StandardBuiltinId::",
        "builtin: StandardBuiltinId",
        "match builtin",
        "is_full_year",
        "matches!(operation",
        "operation ==",
        "operation !=",
        "_ =>",
        "unreachable!",
    ] {
        assert!(
            !body.contains(forbidden),
            "Date component-setter operation must not escape through `{forbidden}`"
        );
    }
}

#[test]
fn exactly_fourteen_date_builtins_use_seven_fixed_setter_entries() {
    let producers = bounded_inclusive(
        STANDARD_SOURCE,
        "            StandardBuiltinId::DatePrototypeSetFullYear",
        "            StandardBuiltinId::DatePrototypeToIsoString => {",
    );
    let normalized = without_whitespace(producers).replace(",)", ")");

    assert!(!STANDARD_SOURCE.contains("DateComponentSetterOperation"));
    assert!(!STANDARD_SOURCE.contains("emit_date_component_setter("));
    assert_eq!(
        producers
            .matches("StandardBuiltinId::DatePrototypeSet")
            .count(),
        14
    );

    for (builtin_suffix, entry, operation) in [
        ("FullYear", "emit_date_set_full_year_builtin", "FullYear"),
        ("Month", "emit_date_set_month_builtin", "Month"),
        ("Date", "emit_date_set_date_builtin", "Date"),
        ("Hours", "emit_date_set_hours_builtin", "Hours"),
        ("Minutes", "emit_date_set_minutes_builtin", "Minutes"),
        ("Seconds", "emit_date_set_seconds_builtin", "Seconds"),
        (
            "Milliseconds",
            "emit_date_set_milliseconds_builtin",
            "Milliseconds",
        ),
    ] {
        let mapping = format!(
            "StandardBuiltinId::DatePrototypeSet{builtin_suffix}|StandardBuiltinId::DatePrototypeSetUtc{builtin_suffix}=>{{self.{entry}(function)?;}}"
        );
        assert_eq!(
            normalized.matches(&mapping).count(),
            1,
            "missing exact local/UTC setter mapping for `{builtin_suffix}`"
        );
        assert_eq!(
            DATE_SOURCE
                .matches(&format!(
                    "self.emit_date_component_setter(DateComponentSetterOperation::{operation}, function)"
                ))
                .count(),
            1,
            "fixed setter producer `{operation}`"
        );
    }
}

#[test]
fn emitted_argument_counts_match_the_read_only_builtin_length_matrix() {
    let setter = without_whitespace(setter_body());
    for projection in [
        "DateComponentSetterOperation::FullYear|DateComponentSetterOperation::Minutes=>3",
        "DateComponentSetterOperation::Month|DateComponentSetterOperation::Seconds=>2",
        "DateComponentSetterOperation::Hours=>4",
        "DateComponentSetterOperation::Date|DateComponentSetterOperation::Milliseconds=>1",
    ] {
        assert_eq!(setter.matches(projection).count(), 1, "{projection}");
    }

    let builtin_lengths = without_whitespace(bounded_inclusive(
        PLANNING_SOURCE,
        "        StandardBuiltinId::DatePrototypeSetFullYear",
        "        StandardBuiltinId::DataViewPrototypeBufferGetter",
    ));
    for projection in [
        "StandardBuiltinId::DatePrototypeSetFullYear|StandardBuiltinId::DatePrototypeSetUtcFullYear|StandardBuiltinId::DatePrototypeSetMinutes|StandardBuiltinId::DatePrototypeSetUtcMinutes=>3",
        "StandardBuiltinId::DatePrototypeSetMonth|StandardBuiltinId::DatePrototypeSetUtcMonth|StandardBuiltinId::DatePrototypeSetSeconds|StandardBuiltinId::DatePrototypeSetUtcSeconds=>2",
        "StandardBuiltinId::DatePrototypeSetDate|StandardBuiltinId::DatePrototypeSetUtcDate|StandardBuiltinId::DatePrototypeSetMilliseconds|StandardBuiltinId::DatePrototypeSetUtcMilliseconds=>1",
        "StandardBuiltinId::DatePrototypeSetHours|StandardBuiltinId::DatePrototypeSetUtcHours=>4",
    ] {
        assert_eq!(builtin_lengths.matches(projection).count(), 1, "{projection}");
    }
}
