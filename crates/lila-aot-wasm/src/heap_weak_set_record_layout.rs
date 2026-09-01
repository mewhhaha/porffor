#![allow(
    dead_code,
    reason = "T05 layout metadata precedes executable weak reachability"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_WEAK_SET_ENTRIES_CAP_OFFSET, HEAP_WEAK_SET_ENTRIES_LEN_OFFSET,
    HEAP_WEAK_SET_ENTRIES_PTR_OFFSET, HEAP_WEAK_SET_LIVE_COUNT_OFFSET,
};

pub(crate) enum WeakSetRecordHeapSlot {
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
    LiveCount,
}

struct WeakSetRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl WeakSetRecordHeapSlot {
    const fn metadata(&self) -> WeakSetRecordHeapSlotMetadata {
        match self {
            Self::EntriesPointer => WeakSetRecordHeapSlotMetadata {
                record: "weak-set-record",
                name: "entries_ptr",
                offset: HEAP_WEAK_SET_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => WeakSetRecordHeapSlotMetadata {
                record: "weak-set-record",
                name: "entries_len",
                offset: HEAP_WEAK_SET_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => WeakSetRecordHeapSlotMetadata {
                record: "weak-set-record",
                name: "entries_cap",
                offset: HEAP_WEAK_SET_ENTRIES_CAP_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LiveCount => WeakSetRecordHeapSlotMetadata {
                record: "weak-set-record",
                name: "live_count",
                offset: HEAP_WEAK_SET_LIVE_COUNT_OFFSET,
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

pub(crate) const HEAP_WEAK_SET_RECORD_LAYOUT: &[WeakSetRecordHeapSlot] = &[
    WeakSetRecordHeapSlot::EntriesPointer,
    WeakSetRecordHeapSlot::EntriesLength,
    WeakSetRecordHeapSlot::EntriesCapacity,
    WeakSetRecordHeapSlot::LiveCount,
];
