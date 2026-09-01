#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET, HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
};

pub(crate) enum AsyncDisposableStackRecordHeapSlot {
    State,
    EntriesPointer,
    EntriesLength,
    EntriesCapacity,
}

struct AsyncDisposableStackRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl AsyncDisposableStackRecordHeapSlot {
    const fn metadata(&self) -> AsyncDisposableStackRecordHeapSlotMetadata {
        match self {
            Self::State => AsyncDisposableStackRecordHeapSlotMetadata {
                record: "async-disposable-stack-record",
                name: "state",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesPointer => AsyncDisposableStackRecordHeapSlotMetadata {
                record: "async-disposable-stack-record",
                name: "entries_ptr",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::EntriesLength => AsyncDisposableStackRecordHeapSlotMetadata {
                record: "async-disposable-stack-record",
                name: "entries_len",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::EntriesCapacity => AsyncDisposableStackRecordHeapSlotMetadata {
                record: "async-disposable-stack-record",
                name: "entries_cap",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
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

pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_RECORD_LAYOUT:
    &[AsyncDisposableStackRecordHeapSlot] = &[
    AsyncDisposableStackRecordHeapSlot::State,
    AsyncDisposableStackRecordHeapSlot::EntriesPointer,
    AsyncDisposableStackRecordHeapSlot::EntriesLength,
    AsyncDisposableStackRecordHeapSlot::EntriesCapacity,
];
