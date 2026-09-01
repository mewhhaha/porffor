#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
    HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET, HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
    HEAP_DISPOSABLE_STACK_STATE_OFFSET,
};

pub(crate) enum DisposableStackRecordHeapSlot {
    State,
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
}

struct DisposableStackRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl DisposableStackRecordHeapSlot {
    const fn metadata(&self) -> DisposableStackRecordHeapSlotMetadata {
        match self {
            Self::State => DisposableStackRecordHeapSlotMetadata {
                record: "disposable-stack-record",
                name: "state",
                offset: HEAP_DISPOSABLE_STACK_STATE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesPointer => DisposableStackRecordHeapSlotMetadata {
                record: "disposable-stack-record",
                name: "entries_ptr",
                offset: HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => DisposableStackRecordHeapSlotMetadata {
                record: "disposable-stack-record",
                name: "entries_len",
                offset: HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => DisposableStackRecordHeapSlotMetadata {
                record: "disposable-stack-record",
                name: "entries_cap",
                offset: HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
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

pub(crate) const HEAP_DISPOSABLE_STACK_RECORD_LAYOUT: &[DisposableStackRecordHeapSlot] = &[
    DisposableStackRecordHeapSlot::State,
    DisposableStackRecordHeapSlot::EntriesPointer,
    DisposableStackRecordHeapSlot::EntriesLength,
    DisposableStackRecordHeapSlot::EntriesCapacity,
];
