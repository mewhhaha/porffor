#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_DURATION_DAYS_OFFSET, HEAP_TEMPORAL_DURATION_HOURS_OFFSET,
    HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET, HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,
    HEAP_TEMPORAL_DURATION_MINUTES_OFFSET, HEAP_TEMPORAL_DURATION_MONTHS_OFFSET,
    HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET, HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,
    HEAP_TEMPORAL_DURATION_WEEKS_OFFSET, HEAP_TEMPORAL_DURATION_YEARS_OFFSET,
};

pub(crate) enum TemporalDurationHeapSlot {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

struct TemporalDurationHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalDurationHeapSlot {
    const fn metadata(&self) -> TemporalDurationHeapSlotMetadata {
        match self {
            Self::Years => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "years",
                offset: HEAP_TEMPORAL_DURATION_YEARS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Months => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "months",
                offset: HEAP_TEMPORAL_DURATION_MONTHS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Weeks => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "weeks",
                offset: HEAP_TEMPORAL_DURATION_WEEKS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Days => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "days",
                offset: HEAP_TEMPORAL_DURATION_DAYS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Hours => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "hours",
                offset: HEAP_TEMPORAL_DURATION_HOURS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Minutes => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "minutes",
                offset: HEAP_TEMPORAL_DURATION_MINUTES_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Seconds => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "seconds",
                offset: HEAP_TEMPORAL_DURATION_SECONDS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Milliseconds => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "milliseconds",
                offset: HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Microseconds => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "microseconds",
                offset: HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Nanoseconds => TemporalDurationHeapSlotMetadata {
                record: "temporal-duration-record",
                name: "nanoseconds",
                offset: HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET,
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

pub(crate) const HEAP_TEMPORAL_DURATION_RECORD_LAYOUT: &[TemporalDurationHeapSlot] = &[
    TemporalDurationHeapSlot::Years,
    TemporalDurationHeapSlot::Months,
    TemporalDurationHeapSlot::Weeks,
    TemporalDurationHeapSlot::Days,
    TemporalDurationHeapSlot::Hours,
    TemporalDurationHeapSlot::Minutes,
    TemporalDurationHeapSlot::Seconds,
    TemporalDurationHeapSlot::Milliseconds,
    TemporalDurationHeapSlot::Microseconds,
    TemporalDurationHeapSlot::Nanoseconds,
];
