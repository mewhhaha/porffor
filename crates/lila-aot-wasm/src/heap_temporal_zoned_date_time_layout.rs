#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
    HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
    HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
};

pub(crate) enum TemporalZonedDateTimeHeapSlot {
    EpochNanosecondsTag,
    EpochNanosecondsPayload,
    TimeZoneTag,
    TimeZonePayload,
    CalendarTag,
    CalendarPayload,
}

struct TemporalZonedDateTimeHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalZonedDateTimeHeapSlot {
    const fn metadata(&self) -> TemporalZonedDateTimeHeapSlotMetadata {
        match self {
            Self::EpochNanosecondsTag => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "epoch_nanoseconds_tag",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EpochNanosecondsPayload => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "epoch_nanoseconds_payload",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::TimeZoneTag => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "time_zone_tag",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TimeZonePayload => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "time_zone_payload",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::CalendarTag => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "calendar_tag",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CalendarPayload => TemporalZonedDateTimeHeapSlotMetadata {
                record: "temporal-zoned-date-time-record",
                name: "calendar_payload",
                offset: HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT: &[TemporalZonedDateTimeHeapSlot] = &[
    TemporalZonedDateTimeHeapSlot::EpochNanosecondsTag,
    TemporalZonedDateTimeHeapSlot::EpochNanosecondsPayload,
    TemporalZonedDateTimeHeapSlot::TimeZoneTag,
    TemporalZonedDateTimeHeapSlot::TimeZonePayload,
    TemporalZonedDateTimeHeapSlot::CalendarTag,
    TemporalZonedDateTimeHeapSlot::CalendarPayload,
];
