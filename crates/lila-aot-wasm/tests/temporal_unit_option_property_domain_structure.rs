const OPTIONS_SOURCE: &str = include_str!("../src/builtins/temporal_options.rs");
const DURATION_SOURCE: &str = include_str!("../src/builtins/temporal_duration_methods.rs");
const PLAIN_DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_methods.rs");
const PLAIN_DATE_TIME_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const PLAIN_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_time_methods.rs");
const PLAIN_YEAR_MONTH_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_year_month_methods.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn temporal_unit_option_property_projects_name_and_auto_policy_exhaustively() {
    let domain = normalized(bounded(
        OPTIONS_SOURCE,
        "pub(crate) enum TemporalUnitOptionProperty {",
        "/// What a `GetTemporalUnitValuedOption` read can produce.",
    ));
    for mapping in [
        "TemporalUnitOptionProperty::LargestUnit=>\"largestUnit\"",
        "TemporalUnitOptionProperty::SmallestUnit=>\"smallestUnit\"",
        "TemporalUnitOptionProperty::Unit=>\"unit\"",
        "TemporalUnitOptionProperty::LargestUnit=>true",
        "TemporalUnitOptionProperty::SmallestUnit|TemporalUnitOptionProperty::Unit=>false",
    ] {
        assert_eq!(domain.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert_eq!(domain.matches("matchself{").count(), 2);
    assert_eq!(domain.matches("=>").count(), 5);
    assert!(!domain.contains("_=>"));
    assert!(!domain.contains("unreachable!"));
    assert!(!domain.contains("implDefault"));
}

#[test]
fn temporal_unit_option_reader_accepts_only_the_closed_property_domain() {
    let signature = bounded(
        DURATION_SOURCE,
        "pub(crate) fn emit_temporal_duration_unit_option(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("property: TemporalUnitOptionProperty,"));
    assert!(!signature.contains("name: &str"));
    assert!(!signature.contains("allow_auto: bool"));

    let reader = normalized(bounded(
        DURATION_SOURCE,
        "pub(crate) fn emit_temporal_duration_unit_option(",
        "pub(crate) fn emit_temporal_duration_rounding_mode_option(",
    ));
    assert_eq!(reader.matches("property.name()").count(), 1);
    assert_eq!(reader.matches("property.allows_auto()").count(), 1);
}

#[test]
fn temporal_unit_option_callers_use_named_properties() {
    let callers = [
        (DURATION_SOURCE, 4),
        (PLAIN_DATE_SOURCE, 2),
        (PLAIN_DATE_TIME_SOURCE, 4),
        (PLAIN_TIME_SOURCE, 4),
        (PLAIN_YEAR_MONTH_SOURCE, 2),
    ];
    let mut calls = Vec::new();
    for (source, expected_count) in callers {
        let normalized_source = normalized(source);
        let source_calls = normalized_source
            .split("self.emit_temporal_duration_unit_option(")
            .skip(1)
            .map(|tail| {
                tail.split_once(")?;")
                    .expect("unterminated Temporal unit-option call")
                    .0
                    .to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(source_calls.len(), expected_count);
        calls.extend(source_calls);
    }

    assert_eq!(calls.len(), 16);
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.contains("TemporalUnitOptionProperty::LargestUnit"))
            .count(),
        5
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.contains("TemporalUnitOptionProperty::SmallestUnit"))
            .count(),
        10
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.contains("TemporalUnitOptionProperty::Unit"))
            .count(),
        1
    );
    for call in calls {
        assert!(!call.contains("\"largestUnit\""));
        assert!(!call.contains("\"smallestUnit\""));
        assert!(!call.contains("\"unit\""));
        assert!(!call.contains(",true,"));
        assert!(!call.contains(",false,"));
    }
}
