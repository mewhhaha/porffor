#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
    HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET, HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET, HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
};

pub(crate) enum TemporalPlainTimeHeapSlot {
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

struct TemporalPlainTimeHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalPlainTimeHeapSlot {
    const fn metadata(&self) -> TemporalPlainTimeHeapSlotMetadata {
        match self {
            Self::Hour => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "hour",
                offset: HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Minute => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "minute",
                offset: HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Second => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "second",
                offset: HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Millisecond => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "millisecond",
                offset: HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Microsecond => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "microsecond",
                offset: HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Nanosecond => TemporalPlainTimeHeapSlotMetadata {
                record: "temporal-plain-time-record",
                name: "nanosecond",
                offset: HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
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

pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT: &[TemporalPlainTimeHeapSlot] = &[
    TemporalPlainTimeHeapSlot::Hour,
    TemporalPlainTimeHeapSlot::Minute,
    TemporalPlainTimeHeapSlot::Second,
    TemporalPlainTimeHeapSlot::Millisecond,
    TemporalPlainTimeHeapSlot::Microsecond,
    TemporalPlainTimeHeapSlot::Nanosecond,
];
