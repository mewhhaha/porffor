#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET, HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET, HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET, HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET, HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
};

pub(crate) enum TemporalPlainDateTimeHeapSlot {
    IsoYear,
    IsoMonth,
    IsoDay,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    CalendarPayload,
}

struct TemporalPlainDateTimeHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalPlainDateTimeHeapSlot {
    const fn metadata(&self) -> TemporalPlainDateTimeHeapSlotMetadata {
        match self {
            Self::IsoYear => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "iso_year",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::IsoMonth => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "iso_month",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::IsoDay => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "iso_day",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Hour => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "hour",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Minute => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "minute",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Second => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "second",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Millisecond => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "millisecond",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Microsecond => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "microsecond",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Nanosecond => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "nanosecond",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CalendarPayload => TemporalPlainDateTimeHeapSlotMetadata {
                record: "temporal-plain-date-time-record",
                name: "calendar_payload",
                offset: HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
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

pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT: &[TemporalPlainDateTimeHeapSlot] = &[
    TemporalPlainDateTimeHeapSlot::IsoYear,
    TemporalPlainDateTimeHeapSlot::IsoMonth,
    TemporalPlainDateTimeHeapSlot::IsoDay,
    TemporalPlainDateTimeHeapSlot::Hour,
    TemporalPlainDateTimeHeapSlot::Minute,
    TemporalPlainDateTimeHeapSlot::Second,
    TemporalPlainDateTimeHeapSlot::Millisecond,
    TemporalPlainDateTimeHeapSlot::Microsecond,
    TemporalPlainDateTimeHeapSlot::Nanosecond,
    TemporalPlainDateTimeHeapSlot::CalendarPayload,
];
