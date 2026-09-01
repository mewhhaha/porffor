#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
    HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum AsyncDisposableStackEntryHeapSlot {
    Kind,
    ValueTag,
    ValuePayload,
    MethodTag,
    MethodPayload,
}

struct AsyncDisposableStackEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl AsyncDisposableStackEntryHeapSlot {
    const fn metadata(&self) -> AsyncDisposableStackEntryHeapSlotMetadata {
        match self {
            Self::Kind => AsyncDisposableStackEntryHeapSlotMetadata {
                record: "async-disposable-stack-entry",
                name: "kind",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => AsyncDisposableStackEntryHeapSlotMetadata {
                record: "async-disposable-stack-entry",
                name: "value_tag",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => AsyncDisposableStackEntryHeapSlotMetadata {
                record: "async-disposable-stack-entry",
                name: "value_payload",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::MethodTag => AsyncDisposableStackEntryHeapSlotMetadata {
                record: "async-disposable-stack-entry",
                name: "method_tag",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::MethodPayload => AsyncDisposableStackEntryHeapSlotMetadata {
                record: "async-disposable-stack-entry",
                name: "method_payload",
                offset: HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
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

/// Both the resource value and the dispose method are strongly reachable: an
/// `AsyncDisposableStack` keeps every registered resource alive until disposal,
/// which is the whole point of the type and the opposite of a
/// `FinalizationRegistry` cell.
pub(crate) const HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_LAYOUT: &[AsyncDisposableStackEntryHeapSlot] = &[
    AsyncDisposableStackEntryHeapSlot::Kind,
    AsyncDisposableStackEntryHeapSlot::ValueTag,
    AsyncDisposableStackEntryHeapSlot::ValuePayload,
    AsyncDisposableStackEntryHeapSlot::MethodTag,
    AsyncDisposableStackEntryHeapSlot::MethodPayload,
];
