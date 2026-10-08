const PLAIN: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");

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
        "pub(super) struct ResolvedTemporalDateTimeDifferenceSettings",
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
fn shared_reader_gets_each_setting_once_in_spec_order() {
    let reader = bounded(
        PLAIN,
        "pub(super) fn emit_temporal_date_time_difference_settings(",
        "pub(super) fn emit_temporal_plain_date_time_until_or_since(",
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
