#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_MAP_ENTRIES_CAP_OFFSET, HEAP_MAP_ENTRIES_LEN_OFFSET,
    HEAP_MAP_ENTRIES_PTR_OFFSET, HEAP_MAP_LIVE_COUNT_OFFSET,
};

pub(crate) enum MapRecordHeapSlot {
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
    LiveCount,
}

struct MapRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl MapRecordHeapSlot {
    const fn metadata(&self) -> MapRecordHeapSlotMetadata {
        match self {
            Self::EntriesPointer => MapRecordHeapSlotMetadata {
                record: "map-record",
                name: "entries_ptr",
                offset: HEAP_MAP_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => MapRecordHeapSlotMetadata {
                record: "map-record",
                name: "entries_len",
                offset: HEAP_MAP_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => MapRecordHeapSlotMetadata {
                record: "map-record",
                name: "entries_cap",
                offset: HEAP_MAP_ENTRIES_CAP_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LiveCount => MapRecordHeapSlotMetadata {
                record: "map-record",
                name: "live_count",
                offset: HEAP_MAP_LIVE_COUNT_OFFSET,
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

pub(crate) const HEAP_MAP_RECORD_LAYOUT: &[MapRecordHeapSlot] = &[
    MapRecordHeapSlot::EntriesPointer,
    MapRecordHeapSlot::EntriesLength,
    MapRecordHeapSlot::EntriesCapacity,
    MapRecordHeapSlot::LiveCount,
];
