const PLAIN: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const ZONED: &str = include_str!("../src/builtins/temporal_zoned_date_time_methods.rs");
const DIFFERENCE: &str = include_str!("../src/builtins/temporal_difference.rs");
const ZONED_DIFFERENCE: &str = include_str!("../src/builtins/temporal_zoned_difference.rs");

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
        [
            "PlainUntil,",
            "PlainSince,",
            "ZonedUntil,",
            "ZonedSince,",
            "InstantUntil,",
            "InstantSince,"
        ]
    );
    let authority = bounded(
        PLAIN,
        "impl TemporalDateTimeDifferenceSettingsPlan {",
        "/// The completed",
    );
    for arm in [
        "Self::PlainUntil | Self::PlainSince => TemporalUnit::Day,",
        "Self::ZonedUntil | Self::ZonedSince => TemporalUnit::Hour,",
        "Self::InstantUntil | Self::InstantSince => TemporalUnit::Second,",
        "Self::PlainUntil | Self::ZonedUntil | Self::InstantUntil => false,",
        "Self::PlainSince | Self::ZonedSince | Self::InstantSince => true,",
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
        assert_before(
            entry,
            "emit_temporal_require_same_calendar(",
            "emit_temporal_date_time_difference_settings(",
        );
        assert!(!entry.contains("emit_temporal_duration_unit_option("));
    }
    // The plain wall-clock difference never sees a zone, and the zoned one
    // never goes through it: every zoned date step is a time-zone query.
    assert_eq!(
        plain.matches("emit_temporal_difference_date_time(").count(),
        1
    );
    assert!(!zoned.contains("emit_temporal_difference_date_time("));
    assert_eq!(
        zoned
            .matches("emit_temporal_difference_zoned_date_time(")
            .count(),
        1
    );
    assert_eq!(
        zoned
            .matches("emit_temporal_round_relative_duration_zoned(")
            .count(),
        1
    );
    // A time largestUnit is `DifferenceInstant` and precedes `TimeZoneEquals`.
    assert_before(
        zoned,
        "emit_temporal_date_time_difference_settings(",
        "TemporalDifferenceGuard::ZonedDateTimeSameTimeZone",
    );
    let time_units = bounded(
        zoned,
        "let settings =",
        "TemporalDifferenceGuard::ZonedDateTimeSameTimeZone",
    );
    assert!(time_units.contains("LocalGet(settings.largest_unit_local)"));
    assert!(time_units.contains("I64Const(TemporalUnit::Day.code())"));
    assert!(time_units.contains("Instruction::I64GtS"));
    assert!(time_units.contains("emit_temporal_time_zone_equals("));
    assert_before(
        zoned,
        "emit_temporal_epoch_nanoseconds_pair(",
        "emit_temporal_iso_date_time_for(",
    );
}

#[test]
fn zoned_calendar_candidates_are_resolved_only_by_the_time_zone_kernel() {
    // The wall-clock machinery carries no zone: no context, no offset.
    assert!(!DIFFERENCE.contains("TemporalDifferenceContext"));
    assert!(!DIFFERENCE.contains("offset_seconds_local"));
    let nudge = bounded(
        DIFFERENCE,
        "fn emit_temporal_nudge_difference_calendar(",
        "/// Bubble only",
    );
    assert!(nudge.contains("emit_temporal_duration_round_up_i32("));
    assert!(DIFFERENCE.contains("fn emit_temporal_bubble_difference("));
    // Each zoned window bound is `GetEpochNanosecondsFor` of a wall-clock
    // candidate, whose range the kernel checks.
    let window = bounded(
        ZONED_DIFFERENCE,
        "fn emit_temporal_compute_nudge_window_zoned(",
        "fn emit_temporal_zoned_window_bound(",
    );
    assert_eq!(window.matches("emit_temporal_zoned_window_bound(").count(), 2);
    let bound = bounded(
        ZONED_DIFFERENCE,
        "fn emit_temporal_zoned_window_bound(",
        "/// `NudgeToCalendarUnit` with a time zone.",
    );
    assert!(bound.contains("emit_temporal_add_iso_date("));
    assert!(bound.contains("emit_temporal_zoned_epoch_for_date_at_wall_time("));
    for function in [
        "fn emit_temporal_nudge_to_calendar_unit_zoned(",
        "fn emit_temporal_nudge_to_zoned_time(",
        "fn emit_temporal_bubble_relative_duration_zoned(",
    ] {
        assert_eq!(ZONED_DIFFERENCE.matches(function).count(), 1, "{function}");
    }
}
