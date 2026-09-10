const OPTIONS_SOURCE: &str = include_str!("../src/builtins/temporal_options.rs");
const DURATION_SOURCE: &str = include_str!("../src/builtins/temporal_duration_methods.rs");
const PLAIN_DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_methods.rs");
const PLAIN_DATE_TIME_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const PLAIN_TIME_SOURCE: &str = include_str!("../src/builtins/temporal_plain_time_methods.rs");
const PLAIN_YEAR_MONTH_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_year_month_methods.rs");
const ZONED_DATE_TIME_FORMAT_SOURCE: &str =
    include_str!("../src/builtins/temporal_zoned_date_time_format.rs");

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
fn temporal_unit_option_property_projects_only_the_name_exhaustively() {
    let domain = normalized(bounded(
        OPTIONS_SOURCE,
        "pub(crate) enum TemporalUnitOptionProperty {",
        "/// What a `GetTemporalUnitValuedOption` read can produce.",
    ));
    for mapping in [
        "TemporalUnitOptionProperty::LargestUnit=>\"largestUnit\"",
        "TemporalUnitOptionProperty::SmallestUnit=>\"smallestUnit\"",
        "TemporalUnitOptionProperty::Unit=>\"unit\"",
    ] {
        assert_eq!(domain.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert_eq!(domain.matches("matchself{").count(), 1);
    assert_eq!(domain.matches("=>").count(), 3);
    assert!(!domain.contains("allows_auto"));
    assert!(!domain.contains("_=>"));
    assert!(!domain.contains("unreachable!"));
    assert!(!domain.contains("implDefault"));
}

#[test]
fn temporal_unit_option_reader_validates_spelling_before_returning_to_its_consumer() {
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
    assert!(!reader.contains("allows_auto"));
    assert!(!reader.contains("TemporalUnitOptionProperty::"));
    let auto = reader
        .find("self.emit_temporal_string_matches(value_payload_local,\"auto\",scratch_local,function);")
        .expect("auto is a recognized spelling for every property");
    let units = reader
        .find("forunitinTemporalUnit::ALL{")
        .expect("the parser recognizes every unit before the consumer restricts its range");
    let rejection = reader
        .find(concat!(
            "function.instruction(&Instruction::LocalGet(output_local));",
            "function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
        ))
        .expect("unknown spellings are rejected within the reader");
    let rejection_body = &reader[rejection..];
    let throw = rejection_body
        .find("self.emit_throw_current_function_realm_range_error(")
        .expect("unknown spellings throw RangeError in the active builtin realm");
    let abrupt_return = rejection_body
        .find("self.emit_return_current_completion(function);")
        .expect("a spelling error cannot reach the consumer's later option reads");
    assert!(auto < units && units < rejection);
    assert!(throw < abrupt_return);
    assert!(!reader.contains("emit_temporal_require_unit_range("));
}

#[test]
fn temporal_unit_option_callers_use_named_properties() {
    let callers = [
        (DURATION_SOURCE, 4),
        (PLAIN_DATE_SOURCE, 2),
        (PLAIN_DATE_TIME_SOURCE, 4),
        (PLAIN_TIME_SOURCE, 4),
        (PLAIN_YEAR_MONTH_SOURCE, 2),
        (ZONED_DATE_TIME_FORMAT_SOURCE, 1),
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

    assert_eq!(calls.len(), 17);
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
        11
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
