const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const OWNER: &str = include_str!("../src/heap_intl_date_time_format_layout.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-heap-slot-authority.md"
);
const T05: &str = include_str!("../../../tasks/05-values-heap-gc.md");
const T23: &str = include_str!("../../../tasks/23-intl402.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn intl_date_time_format_heap_slot_is_the_exact_capability_free_domain() {
    let variants = bounded(
        OWNER,
        "pub(crate) enum IntlDateTimeFormatHeapSlot {",
        "\n}\n\nstruct IntlDateTimeFormatHeapSlotMetadata",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "LocalePayload,",
            "CalendarPayload,",
            "NumberingSystemPayload,",
            "TimeZonePayload,",
            "TimeZoneOffsetMinutes,",
            "TimeZoneGmtNamePayload,",
            "HourCycleCode,",
            "WeekdayCode,",
            "EraCode,",
            "YearCode,",
            "MonthCode,",
            "DayCode,",
            "DayPeriodCode,",
            "HourCode,",
            "MinuteCode,",
            "SecondCode,",
            "FractionalSecondDigits,",
            "TimeZoneNameCode,",
            "DateStyleCode,",
            "TimeStyleCode,",
            "Hour12Code,",
            "BoundFormatPayload,",
            "NeedDefaults,",
        ]
    );
    assert!(!OWNER.contains("#[derive("));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!OWNER.contains(&format!("impl {capability} for IntlDateTimeFormatHeapSlot")));
    }
}

#[test]
fn one_exhaustive_projection_owns_twenty_three_exact_rows() {
    let projection = bounded(
        OWNER,
        "    const fn metadata(&self) -> IntlDateTimeFormatHeapSlotMetadata {",
        "\n    pub(crate) const fn layout(&self)",
    );
    for (variant, name, offset, pointer) in [
        (
            "LocalePayload",
            "locale_payload",
            "HEAP_INTL_DTF_LOCALE_OFFSET",
            true,
        ),
        (
            "CalendarPayload",
            "calendar_payload",
            "HEAP_INTL_DTF_CALENDAR_OFFSET",
            true,
        ),
        (
            "NumberingSystemPayload",
            "numbering_system_payload",
            "HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET",
            true,
        ),
        (
            "TimeZonePayload",
            "time_zone_payload",
            "HEAP_INTL_DTF_TIME_ZONE_OFFSET",
            true,
        ),
        (
            "TimeZoneOffsetMinutes",
            "time_zone_offset_minutes",
            "HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET",
            false,
        ),
        (
            "TimeZoneGmtNamePayload",
            "time_zone_gmt_name_payload",
            "HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET",
            true,
        ),
        (
            "HourCycleCode",
            "hour_cycle_code",
            "HEAP_INTL_DTF_HOUR_CYCLE_OFFSET",
            false,
        ),
        (
            "WeekdayCode",
            "weekday_code",
            "HEAP_INTL_DTF_WEEKDAY_OFFSET",
            false,
        ),
        ("EraCode", "era_code", "HEAP_INTL_DTF_ERA_OFFSET", false),
        ("YearCode", "year_code", "HEAP_INTL_DTF_YEAR_OFFSET", false),
        (
            "MonthCode",
            "month_code",
            "HEAP_INTL_DTF_MONTH_OFFSET",
            false,
        ),
        ("DayCode", "day_code", "HEAP_INTL_DTF_DAY_OFFSET", false),
        (
            "DayPeriodCode",
            "day_period_code",
            "HEAP_INTL_DTF_DAY_PERIOD_OFFSET",
            false,
        ),
        ("HourCode", "hour_code", "HEAP_INTL_DTF_HOUR_OFFSET", false),
        (
            "MinuteCode",
            "minute_code",
            "HEAP_INTL_DTF_MINUTE_OFFSET",
            false,
        ),
        (
            "SecondCode",
            "second_code",
            "HEAP_INTL_DTF_SECOND_OFFSET",
            false,
        ),
        (
            "FractionalSecondDigits",
            "fractional_second_digits",
            "HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET",
            false,
        ),
        (
            "TimeZoneNameCode",
            "time_zone_name_code",
            "HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET",
            false,
        ),
        (
            "DateStyleCode",
            "date_style_code",
            "HEAP_INTL_DTF_DATE_STYLE_OFFSET",
            false,
        ),
        (
            "TimeStyleCode",
            "time_style_code",
            "HEAP_INTL_DTF_TIME_STYLE_OFFSET",
            false,
        ),
        (
            "Hour12Code",
            "hour12_code",
            "HEAP_INTL_DTF_HOUR12_OFFSET",
            false,
        ),
        (
            "BoundFormatPayload",
            "bound_format_payload",
            "HEAP_INTL_DTF_BOUND_FORMAT_OFFSET",
            true,
        ),
        (
            "NeedDefaults",
            "need_defaults",
            "HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET",
            false,
        ),
    ] {
        let arm = bounded(
            projection,
            &format!("            Self::{variant} => IntlDateTimeFormatHeapSlotMetadata {{"),
            "            },",
        );
        assert!(arm.contains("record: \"intl-date-time-format-record\""));
        assert!(arm.contains(&format!("name: \"{name}\"")));
        assert!(arm.contains(&format!("offset: {offset}")));
        assert!(arm.contains("width: 8"));
        assert!(arm.contains(&format!("pointer: {pointer}")));
    }
    assert_eq!(projection.matches("Self::").count(), 23);
    assert!(!projection.contains("_ =>"));
}

#[test]
fn typed_registry_preserves_date_time_format_slot_order() {
    let registry = bounded(
        OWNER,
        "pub(crate) const HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT:",
        "];",
    );
    for variant in [
        "LocalePayload",
        "CalendarPayload",
        "NumberingSystemPayload",
        "TimeZonePayload",
        "TimeZoneOffsetMinutes",
        "TimeZoneGmtNamePayload",
        "HourCycleCode",
        "WeekdayCode",
        "EraCode",
        "YearCode",
        "MonthCode",
        "DayCode",
        "DayPeriodCode",
        "HourCode",
        "MinuteCode",
        "SecondCode",
        "FractionalSecondDigits",
        "TimeZoneNameCode",
        "DateStyleCode",
        "TimeStyleCode",
        "Hour12Code",
        "BoundFormatPayload",
        "NeedDefaults",
    ] {
        assert_eq!(
            registry
                .matches(&format!("IntlDateTimeFormatHeapSlot::{variant}"))
                .count(),
            1
        );
    }
    assert_eq!(registry.matches("IntlDateTimeFormatHeapSlot::").count(), 23);
}

#[test]
fn intl_date_time_format_layout_has_one_private_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("mod heap_intl_date_time_format_layout;")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("pub mod heap_intl_date_time_format_layout;"));
    assert!(!HEAP_SOURCE.contains("record: \"intl-date-time-format-record\""));
    assert!(!HEAP_SOURCE.contains("HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT: &[HeapLayoutSlot]"));
    for evidence in [CONTRACT, T05, T23] {
        assert!(evidence.contains("IntlDateTimeFormatHeapSlot"));
        assert!(evidence.contains("passive metadata migration"));
        assert!(evidence.contains("no new Intl behavior"));
    }
}
