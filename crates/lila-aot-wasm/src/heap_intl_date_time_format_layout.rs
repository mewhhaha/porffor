#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_INTL_DTF_BOUND_FORMAT_OFFSET, HEAP_INTL_DTF_CALENDAR_OFFSET,
    HEAP_INTL_DTF_DATE_STYLE_OFFSET, HEAP_INTL_DTF_DAY_OFFSET, HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
    HEAP_INTL_DTF_ERA_OFFSET, HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
    HEAP_INTL_DTF_HOUR12_OFFSET, HEAP_INTL_DTF_HOUR_CYCLE_OFFSET, HEAP_INTL_DTF_HOUR_OFFSET,
    HEAP_INTL_DTF_LOCALE_OFFSET, HEAP_INTL_DTF_MINUTE_OFFSET, HEAP_INTL_DTF_MONTH_OFFSET,
    HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET, HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
    HEAP_INTL_DTF_SECOND_OFFSET, HEAP_INTL_DTF_TIME_STYLE_OFFSET,
    HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET, HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
    HEAP_INTL_DTF_TIME_ZONE_OFFSET, HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
    HEAP_INTL_DTF_WEEKDAY_OFFSET, HEAP_INTL_DTF_YEAR_OFFSET,
};

pub(crate) enum IntlDateTimeFormatHeapSlot {
    LocalePayload,
    CalendarPayload,
    NumberingSystemPayload,
    TimeZonePayload,
    TimeZoneOffsetMinutes,
    TimeZoneGmtNamePayload,
    HourCycleCode,
    WeekdayCode,
    EraCode,
    YearCode,
    MonthCode,
    DayCode,
    DayPeriodCode,
    HourCode,
    MinuteCode,
    SecondCode,
    FractionalSecondDigits,
    TimeZoneNameCode,
    DateStyleCode,
    TimeStyleCode,
    Hour12Code,
    BoundFormatPayload,
    NeedDefaults,
}

struct IntlDateTimeFormatHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl IntlDateTimeFormatHeapSlot {
    const fn metadata(&self) -> IntlDateTimeFormatHeapSlotMetadata {
        match self {
            Self::LocalePayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "locale_payload",
                offset: HEAP_INTL_DTF_LOCALE_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::CalendarPayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "calendar_payload",
                offset: HEAP_INTL_DTF_CALENDAR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NumberingSystemPayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "numbering_system_payload",
                offset: HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::TimeZonePayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "time_zone_payload",
                offset: HEAP_INTL_DTF_TIME_ZONE_OFFSET,
                width: 8,
                pointer: true,
            },
            // The offset stays adjacent to the time-zone identifier because they are one value.
            Self::TimeZoneOffsetMinutes => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "time_zone_offset_minutes",
                offset: HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TimeZoneGmtNamePayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "time_zone_gmt_name_payload",
                offset: HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HourCycleCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "hour_cycle_code",
                offset: HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::WeekdayCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "weekday_code",
                offset: HEAP_INTL_DTF_WEEKDAY_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EraCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "era_code",
                offset: HEAP_INTL_DTF_ERA_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::YearCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "year_code",
                offset: HEAP_INTL_DTF_YEAR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::MonthCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "month_code",
                offset: HEAP_INTL_DTF_MONTH_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DayCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "day_code",
                offset: HEAP_INTL_DTF_DAY_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DayPeriodCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "day_period_code",
                offset: HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::HourCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "hour_code",
                offset: HEAP_INTL_DTF_HOUR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::MinuteCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "minute_code",
                offset: HEAP_INTL_DTF_MINUTE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::SecondCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "second_code",
                offset: HEAP_INTL_DTF_SECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::FractionalSecondDigits => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "fractional_second_digits",
                offset: HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TimeZoneNameCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "time_zone_name_code",
                offset: HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DateStyleCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "date_style_code",
                offset: HEAP_INTL_DTF_DATE_STYLE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TimeStyleCode => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "time_style_code",
                offset: HEAP_INTL_DTF_TIME_STYLE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Hour12Code => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "hour12_code",
                offset: HEAP_INTL_DTF_HOUR12_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::BoundFormatPayload => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "bound_format_payload",
                offset: HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NeedDefaults => IntlDateTimeFormatHeapSlotMetadata {
                record: "intl-date-time-format-record",
                name: "need_defaults",
                offset: HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
                width: 8,
                pointer: false,
            },
        }
    }

    pub(crate) const fn layout(&self) -> HeapLayoutSlot {
        let metadata = self.metadata();
        HeapLayoutSlot {
            record: metadata.record,
            name: metadata.name,
            offset: metadata.offset,
            width: metadata.width,
            pointer: metadata.pointer,
        }
    }
}

pub(crate) const HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT: &[IntlDateTimeFormatHeapSlot] = &[
    IntlDateTimeFormatHeapSlot::LocalePayload,
    IntlDateTimeFormatHeapSlot::CalendarPayload,
    IntlDateTimeFormatHeapSlot::NumberingSystemPayload,
    IntlDateTimeFormatHeapSlot::TimeZonePayload,
    IntlDateTimeFormatHeapSlot::TimeZoneOffsetMinutes,
    IntlDateTimeFormatHeapSlot::TimeZoneGmtNamePayload,
    IntlDateTimeFormatHeapSlot::HourCycleCode,
    IntlDateTimeFormatHeapSlot::WeekdayCode,
    IntlDateTimeFormatHeapSlot::EraCode,
    IntlDateTimeFormatHeapSlot::YearCode,
    IntlDateTimeFormatHeapSlot::MonthCode,
    IntlDateTimeFormatHeapSlot::DayCode,
    IntlDateTimeFormatHeapSlot::DayPeriodCode,
    IntlDateTimeFormatHeapSlot::HourCode,
    IntlDateTimeFormatHeapSlot::MinuteCode,
    IntlDateTimeFormatHeapSlot::SecondCode,
    IntlDateTimeFormatHeapSlot::FractionalSecondDigits,
    IntlDateTimeFormatHeapSlot::TimeZoneNameCode,
    IntlDateTimeFormatHeapSlot::DateStyleCode,
    IntlDateTimeFormatHeapSlot::TimeStyleCode,
    IntlDateTimeFormatHeapSlot::Hour12Code,
    IntlDateTimeFormatHeapSlot::BoundFormatPayload,
    IntlDateTimeFormatHeapSlot::NeedDefaults,
];
