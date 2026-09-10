const PLAIN: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const ZONED: &str = include_str!("../src/builtins/temporal_zoned_date_time_methods.rs");
const DIFFERENCE: &str = include_str!("../src/builtins/temporal_difference.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}

fn assert_before(source: &str, first: &str, second: &str) {
    assert!(
        source.find(first).expect(first) < source.find(second).expect(second),
        "{first} must precede {second}"
    );
}

#[test]
fn receiver_and_direction_jointly_own_difference_settings() {
    let variants = bounded(
        PLAIN,
        "enum TemporalDateTimeDifferenceSettingsPlan {",
        "\n}",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["PlainUntil,", "PlainSince,", "ZonedUntil,", "ZonedSince,"]
    );
    let authority = bounded(
        PLAIN,
        "impl TemporalDateTimeDifferenceSettingsPlan {",
        "/// The completed",
    );
    for arm in [
        "Self::PlainUntil | Self::PlainSince => TemporalUnit::Day,",
        "Self::ZonedUntil | Self::ZonedSince => TemporalUnit::Hour,",
        "Self::PlainUntil | Self::ZonedUntil => false,",
        "Self::PlainSince | Self::ZonedSince => true,",
    ] {
        assert_eq!(authority.matches(arm).count(), 1);
    }
    assert!(!authority.contains("_ =>"));
}

#[test]
fn resolved_settings_are_borrowed_by_arithmetic_without_an_observable_transport() {
    let fields = bounded(
        PLAIN,
        "struct ResolvedTemporalDateTimeDifferenceSettings {",
        "\n}",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "pub(super) largest_unit_local: u32,",
            "pub(super) smallest_unit_local: u32,",
            "pub(super) increment_local: u32,",
            "pub(super) mode_local: u32,",
        ]
    );
    let declaration = PLAIN
        .split_once("pub(super) struct ResolvedTemporalDateTimeDifferenceSettings")
        .unwrap()
        .0;
    assert!(declaration
        .rsplit_once("\n\n")
        .unwrap()
        .1
        .contains("#[must_use"));
    assert!(!declaration
        .rsplit_once("\n\n")
        .unwrap()
        .1
        .contains("derive"));
    assert_eq!(
        DIFFERENCE
            .matches("settings: &ResolvedTemporalDateTimeDifferenceSettings,")
            .count(),
        1
    );
    for source in [PLAIN, ZONED] {
        assert!(!source.contains("delegate_options"));
        assert!(!source.contains("difference_unit_string_payload"));
        assert!(!source.contains("difference_rounding_mode_string_payload"));
    }
}

#[test]
fn shared_reader_gets_each_setting_once_in_spec_order() {
    let reader = bounded(
        PLAIN,
        "pub(super) fn emit_temporal_date_time_difference_settings(",
        "/// Temporal proposal 5.3.x `until`",
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
    for (first, second) in [
        (
            "TemporalUnitOptionProperty::LargestUnit",
            "rounding_increment_option(",
        ),
        ("rounding_increment_option(", "rounding_mode_option("),
        (
            "rounding_mode_option(",
            "TemporalUnitOptionProperty::SmallestUnit",
        ),
    ] {
        assert_before(reader, first, second);
    }
    assert!(reader.contains("plan.fallback_largest_unit()"));
    assert!(reader.contains("if plan.negates_rounding_mode()"));
}

#[test]
fn entrypoints_read_options_once_and_share_the_typed_arithmetic_boundary() {
    let plain = bounded(
        PLAIN,
        "pub(super) fn emit_temporal_plain_date_time_until_or_since(",
        "pub(crate) fn emit_temporal_plain_date_time_to_locale_string(",
    );
    let zoned = ZONED
        .split_once("fn emit_temporal_zoned_date_time_until_or_since(")
        .unwrap()
        .1;
    for entry in [plain, zoned] {
        assert_eq!(
            entry
                .matches("emit_temporal_date_time_difference_settings(")
                .count(),
            1
        );
        assert_eq!(
            entry.matches("emit_temporal_difference_date_time(").count(),
            1
        );
        assert_before(
            entry,
            "emit_temporal_require_same_calendar(",
            "emit_temporal_date_time_difference_settings(",
        );
        assert!(!entry.contains("emit_temporal_duration_unit_option("));
    }
    assert!(plain.contains("TemporalDifferenceContext::Plain"));
    assert!(zoned.contains("TemporalDifferenceContext::Zoned"));
    assert_before(
        zoned,
        "emit_temporal_date_time_difference_settings(",
        "TemporalDifferenceGuard::ZonedDateTimeSameTimeZone",
    );
    let guard = bounded(
        zoned,
        "let settings =",
        "TemporalDifferenceGuard::ZonedDateTimeSameTimeZone",
    );
    assert!(guard.contains("LocalGet(settings.largest_unit_local)"));
    assert!(guard.contains("I64Const(TemporalUnit::Day.code())"));
    assert!(guard.contains("Instruction::I64LeS"));
    assert_before(
        zoned,
        "emit_temporal_zoned_date_time_epoch_pair(",
        "emit_temporal_zoned_date_time_to_plain_date_time(",
    );
}

#[test]
fn calendar_candidates_have_one_exhaustive_range_authority() {
    let context = bounded(DIFFERENCE, "enum TemporalDifferenceContext {", "\n}");
    assert_eq!(
        context
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Plain,", "Zoned { offset_seconds_local: u32 },"]
    );
    let validation = bounded(
        DIFFERENCE,
        "fn emit_temporal_difference_candidate_range(",
        "/// NudgeToCalendarUnit",
    );
    assert!(validation.contains("match context {"));
    assert!(!validation.contains("_ =>"));
    assert!(validation.contains("emit_temporal_instant_validate_range("));
    let nudge = bounded(
        DIFFERENCE,
        "fn emit_temporal_nudge_difference_calendar(",
        "/// Bubble only",
    );
    assert_eq!(
        nudge
            .matches("emit_temporal_difference_candidate_range(")
            .count(),
        2
    );
    assert_before(
        nudge,
        "emit_temporal_difference_candidate_range(",
        "emit_temporal_duration_round_up_i32(",
    );
    assert!(DIFFERENCE.contains("fn emit_temporal_zoned_time_nudge_range("));
    assert!(DIFFERENCE.contains("fn emit_temporal_bubble_difference("));
}
