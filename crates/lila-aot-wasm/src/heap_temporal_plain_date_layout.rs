#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET,
};

pub(crate) enum TemporalPlainDateHeapSlot {
    IsoYear,
    IsoMonth,
    IsoDay,
    CalendarPayload,
}

struct TemporalPlainDateHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalPlainDateHeapSlot {
    const fn metadata(&self) -> TemporalPlainDateHeapSlotMetadata {
        match self {
            Self::IsoYear => TemporalPlainDateHeapSlotMetadata {
                record: "temporal-plain-date-record",
                name: "iso_year",
                offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::IsoMonth => TemporalPlainDateHeapSlotMetadata {
                record: "temporal-plain-date-record",
                name: "iso_month",
                offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::IsoDay => TemporalPlainDateHeapSlotMetadata {
                record: "temporal-plain-date-record",
                name: "iso_day",
                offset: HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CalendarPayload => TemporalPlainDateHeapSlotMetadata {
                record: "temporal-plain-date-record",
                name: "calendar_payload",
                offset: HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_RECORD_LAYOUT: &[TemporalPlainDateHeapSlot] = &[
    TemporalPlainDateHeapSlot::IsoYear,
    TemporalPlainDateHeapSlot::IsoMonth,
    TemporalPlainDateHeapSlot::IsoDay,
    TemporalPlainDateHeapSlot::CalendarPayload,
];
