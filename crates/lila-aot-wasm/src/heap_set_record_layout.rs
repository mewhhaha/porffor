#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_SET_ENTRIES_CAP_OFFSET, HEAP_SET_ENTRIES_LEN_OFFSET,
    HEAP_SET_ENTRIES_PTR_OFFSET, HEAP_SET_LIVE_COUNT_OFFSET,
};

pub(crate) enum SetRecordHeapSlot {
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
    LiveCount,
}

struct SetRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl SetRecordHeapSlot {
    const fn metadata(&self) -> SetRecordHeapSlotMetadata {
        match self {
            Self::EntriesPointer => SetRecordHeapSlotMetadata {
                record: "set-record",
                name: "entries_ptr",
                offset: HEAP_SET_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => SetRecordHeapSlotMetadata {
                record: "set-record",
                name: "entries_len",
                offset: HEAP_SET_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => SetRecordHeapSlotMetadata {
                record: "set-record",
                name: "entries_cap",
                offset: HEAP_SET_ENTRIES_CAP_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LiveCount => SetRecordHeapSlotMetadata {
                record: "set-record",
                name: "live_count",
                offset: HEAP_SET_LIVE_COUNT_OFFSET,
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

pub(crate) const HEAP_SET_RECORD_LAYOUT: &[SetRecordHeapSlot] = &[
    SetRecordHeapSlot::EntriesPointer,
    SetRecordHeapSlot::EntriesLength,
    SetRecordHeapSlot::EntriesCapacity,
    SetRecordHeapSlot::LiveCount,
];
