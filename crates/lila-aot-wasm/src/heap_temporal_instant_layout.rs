#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
    HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
};

pub(crate) enum TemporalInstantHeapSlot {
    EpochNanosecondsTag,
    EpochNanosecondsPayload,
}

struct TemporalInstantHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TemporalInstantHeapSlot {
    const fn metadata(&self) -> TemporalInstantHeapSlotMetadata {
        match self {
            Self::EpochNanosecondsTag => TemporalInstantHeapSlotMetadata {
                record: "temporal-instant-record",
                name: "epoch_nanoseconds_tag",
                offset: HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EpochNanosecondsPayload => TemporalInstantHeapSlotMetadata {
                record: "temporal-instant-record",
                name: "epoch_nanoseconds_payload",
                offset: HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_TEMPORAL_INSTANT_RECORD_LAYOUT: &[TemporalInstantHeapSlot] = &[
    TemporalInstantHeapSlot::EpochNanosecondsTag,
    TemporalInstantHeapSlot::EpochNanosecondsPayload,
];
