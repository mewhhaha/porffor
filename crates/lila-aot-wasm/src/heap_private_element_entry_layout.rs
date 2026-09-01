#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET, HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
    HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET, HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
    HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET, HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum PrivateElementEntryHeapSlot {
    Next,
    Receiver,
    Token,
    Kind,
    ValueTag,
    ValuePayload,
}

struct PrivateElementEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PrivateElementEntryHeapSlot {
    const fn metadata(&self) -> PrivateElementEntryHeapSlotMetadata {
        match self {
            Self::Next => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "next",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Receiver => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "receiver",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Token => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "token",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Kind => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "kind",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "value_tag",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => PrivateElementEntryHeapSlotMetadata {
                record: "private-element-entry",
                name: "value_payload",
                offset: HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_PRIVATE_ELEMENT_ENTRY_LAYOUT: &[PrivateElementEntryHeapSlot] = &[
    PrivateElementEntryHeapSlot::Next,
    PrivateElementEntryHeapSlot::Receiver,
    PrivateElementEntryHeapSlot::Token,
    PrivateElementEntryHeapSlot::Kind,
    PrivateElementEntryHeapSlot::ValueTag,
    PrivateElementEntryHeapSlot::ValuePayload,
];
