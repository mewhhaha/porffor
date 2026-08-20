const PLAIN_DATE_TIME_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const ZONED_DATE_TIME_SOURCE: &str =
    include_str!("../src/builtins/temporal_zoned_date_time_methods.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn date_time_difference_settings_plan_is_closed_and_receiver_specific() {
    let plain_direction = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "pub(crate) enum PlainDateTimeDifference {",
        "impl PlainDateTimeDifference {",
    );
    let plain_variants = plain_direction
        .split_once('}')
        .expect("plain direction end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(plain_variants, ["Until,", "Since,"]);

    let plan = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "enum TemporalDateTimeDifferenceSettingsPlan {",
        "impl TemporalDateTimeDifferenceSettingsPlan {",
    );
    let plan_variants = plan
        .split_once('}')
        .expect("settings plan end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        plan_variants,
        ["PlainUntil,", "PlainSince,", "ZonedDelegate,"]
    );

    let authority = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "impl TemporalDateTimeDifferenceSettingsPlan {",
        "struct ResolvedTemporalDateTimeDifferenceSettings {",
    );
    assert!(!authority.contains("_ =>"));
    assert_eq!(
        authority
            .matches("Self::PlainUntil | Self::PlainSince => TemporalUnit::Day,")
            .count(),
        1
    );
    assert_eq!(
        authority
            .matches("Self::ZonedDelegate => TemporalUnit::Hour,")
            .count(),
        1
    );
    assert_eq!(
        authority
            .matches("Self::PlainUntil | Self::ZonedDelegate => false,")
            .count(),
        1
    );
    assert_eq!(authority.matches("Self::PlainSince => true,").count(), 1);
}

#[test]
fn resolved_settings_are_a_linear_complete_witness() {
    let declaration = PLAIN_DATE_TIME_SOURCE
        .split_once("struct ResolvedTemporalDateTimeDifferenceSettings {")
        .expect("resolved settings witness")
        .0
        .rsplit_once("\n\n")
        .expect("witness attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!PLAIN_DATE_TIME_SOURCE
        .contains("impl Copy for ResolvedTemporalDateTimeDifferenceSettings"));

    let fields = PLAIN_DATE_TIME_SOURCE
        .split_once("struct ResolvedTemporalDateTimeDifferenceSettings {")
        .expect("resolved settings fields")
        .1
        .split_once('}')
        .expect("resolved settings fields end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "largest_unit_local: u32,",
            "smallest_unit_local: u32,",
            "increment_local: u32,",
            "mode_local: u32,",
        ]
    );
    assert_eq!(
        PLAIN_DATE_TIME_SOURCE
            .matches("let ResolvedTemporalDateTimeDifferenceSettings {")
            .count(),
        2,
        "the witness has exactly one PlainDateTime and one ZonedDateTime consumer"
    );
}

#[test]
fn shared_reader_gets_each_user_setting_once_in_spec_order() {
    let reader = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "    fn emit_temporal_date_time_difference_settings(",
        "    fn emit_temporal_difference_unit_string_payload(",
    );
    assert_eq!(
        reader
            .matches("emit_temporal_duration_options_object(")
            .count(),
        1
    );
    assert_eq!(
        reader
            .matches("emit_temporal_duration_unit_option(")
            .count(),
        2
    );
    assert_eq!(
        reader
            .matches("emit_temporal_duration_rounding_increment_option(")
            .count(),
        1
    );
    assert_eq!(
        reader
            .matches("emit_temporal_duration_rounding_mode_option(")
            .count(),
        1
    );
    assert_before(reader, "\"largestUnit\"", "rounding_increment_option(");
    assert_before(
        reader,
        "rounding_increment_option(",
        "rounding_mode_option(",
    );
    assert_before(reader, "rounding_mode_option(", "\"smallestUnit\"");
    assert!(reader.contains("let fallback_largest_unit = plan.fallback_largest_unit();"));
    assert!(reader.contains("if plan.negates_rounding_mode()"));
}

#[test]
fn plain_arithmetic_and_zoned_transport_are_the_only_consumers() {
    let plain = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_time_until_or_since(",
        "    pub(crate) fn emit_temporal_plain_date_time_to_locale_string(",
    );
    assert_eq!(
        plain
            .matches("emit_temporal_date_time_difference_settings(")
            .count(),
        1
    );
    assert!(plain.contains("difference.settings_plan()"));
    assert!(!plain.contains("emit_temporal_duration_unit_option("));
    assert!(!plain.contains("emit_temporal_duration_rounding_mode_option("));

    let transport = bounded(
        PLAIN_DATE_TIME_SOURCE,
        "    pub(crate) fn emit_temporal_zoned_date_time_difference_delegate_options(",
        "    pub(crate) fn emit_temporal_plain_date_time_until_or_since(",
    );
    assert_eq!(
        transport
            .matches("TemporalDateTimeDifferenceSettingsPlan::ZonedDelegate")
            .count(),
        1
    );
    assert_eq!(
        transport
            .matches("emit_alloc_plain_object_with_prototype(None, None, function)")
            .count(),
        1
    );
    assert_eq!(
        transport
            .matches("emit_object_define_enumerable_data(")
            .count(),
        4
    );
    assert_before(transport, "\"largestUnit\"", "\"roundingIncrement\"");
    assert_before(transport, "\"roundingIncrement\"", "\"roundingMode\"");
    assert_before(transport, "\"roundingMode\"", "\"smallestUnit\"");
}

#[test]
fn zoned_delegate_receives_only_the_normalized_options_bag() {
    let zoned = ZONED_DATE_TIME_SOURCE
        .split_once("    pub(crate) fn emit_temporal_zoned_date_time_until_or_since(")
        .expect("ZonedDateTime difference emitter")
        .1;
    assert_eq!(
        zoned
            .matches("emit_temporal_zoned_date_time_difference_delegate_options(")
            .count(),
        1
    );
    assert_before(
        zoned,
        "emit_temporal_zoned_date_time_difference_delegate_options(",
        "let difference_builtin = difference.plain_date_time_builtin();",
    );

    let delegate = bounded(
        zoned,
        "        let difference_builtin = difference.plain_date_time_builtin();",
        "        function.instruction(&Instruction::LocalGet(duration_payload_local));",
    );
    assert!(delegate.contains("delegate_options_payload_local"));
    assert!(delegate.contains("delegate_options_tag_local"));
    assert!(!delegate.contains("(options_payload_local, options_tag_local)"));

    assert!(
        STANDARD_SOURCE.contains("PlainDateTimeDifference::Until,\n                    function,")
    );
    assert!(
        STANDARD_SOURCE.contains("PlainDateTimeDifference::Since,\n                    function,")
    );
    assert!(
        !STANDARD_SOURCE.contains("emit_temporal_plain_date_time_until_or_since(false, function)")
    );
    assert!(
        !STANDARD_SOURCE.contains("emit_temporal_plain_date_time_until_or_since(true, function)")
    );
}
