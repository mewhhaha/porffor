#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
    HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
    HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
    HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET, HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum DisposableStackEntryHeapSlot {
    Kind,
    ValueTag,
    ValuePayload,
    MethodTag,
    MethodPayload,
}

struct DisposableStackEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl DisposableStackEntryHeapSlot {
    const fn metadata(&self) -> DisposableStackEntryHeapSlotMetadata {
        match self {
            Self::Kind => DisposableStackEntryHeapSlotMetadata {
                record: "disposable-stack-entry",
                name: "kind",
                offset: HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => DisposableStackEntryHeapSlotMetadata {
                record: "disposable-stack-entry",
                name: "value_tag",
                offset: HEAP_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => DisposableStackEntryHeapSlotMetadata {
                record: "disposable-stack-entry",
                name: "value_payload",
                offset: HEAP_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::MethodTag => DisposableStackEntryHeapSlotMetadata {
                record: "disposable-stack-entry",
                name: "method_tag",
                offset: HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::MethodPayload => DisposableStackEntryHeapSlotMetadata {
                record: "disposable-stack-entry",
                name: "method_payload",
                offset: HEAP_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
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

/// A synchronous stack owns both the registered resource and its acquired
/// method until the entry has been consumed by the LIFO disposal walk.
pub(crate) const HEAP_DISPOSABLE_STACK_ENTRY_LAYOUT: &[DisposableStackEntryHeapSlot] = &[
    DisposableStackEntryHeapSlot::Kind,
    DisposableStackEntryHeapSlot::ValueTag,
    DisposableStackEntryHeapSlot::ValuePayload,
    DisposableStackEntryHeapSlot::MethodTag,
    DisposableStackEntryHeapSlot::MethodPayload,
];
