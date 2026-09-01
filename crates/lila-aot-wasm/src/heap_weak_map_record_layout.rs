#![allow(
    dead_code,
    reason = "T05 layout metadata precedes executable weak reachability"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_WEAK_MAP_ENTRIES_CAP_OFFSET, HEAP_WEAK_MAP_ENTRIES_LEN_OFFSET,
    HEAP_WEAK_MAP_ENTRIES_PTR_OFFSET, HEAP_WEAK_MAP_LIVE_COUNT_OFFSET,
};

pub(crate) enum WeakMapRecordHeapSlot {
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
    LiveCount,
}

struct WeakMapRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl WeakMapRecordHeapSlot {
    const fn metadata(&self) -> WeakMapRecordHeapSlotMetadata {
        match self {
            Self::EntriesPointer => WeakMapRecordHeapSlotMetadata {
                record: "weak-map-record",
                name: "entries_ptr",
                offset: HEAP_WEAK_MAP_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => WeakMapRecordHeapSlotMetadata {
                record: "weak-map-record",
                name: "entries_len",
                offset: HEAP_WEAK_MAP_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => WeakMapRecordHeapSlotMetadata {
                record: "weak-map-record",
                name: "entries_cap",
                offset: HEAP_WEAK_MAP_ENTRIES_CAP_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LiveCount => WeakMapRecordHeapSlotMetadata {
                record: "weak-map-record",
                name: "live_count",
                offset: HEAP_WEAK_MAP_LIVE_COUNT_OFFSET,
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

pub(crate) const HEAP_WEAK_MAP_RECORD_LAYOUT: &[WeakMapRecordHeapSlot] = &[
    WeakMapRecordHeapSlot::EntriesPointer,
    WeakMapRecordHeapSlot::EntriesLength,
    WeakMapRecordHeapSlot::EntriesCapacity,
    WeakMapRecordHeapSlot::LiveCount,
];
