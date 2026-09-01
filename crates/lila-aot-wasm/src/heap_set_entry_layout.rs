#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_SET_ENTRY_PRESENT_OFFSET, HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
    HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum SetEntryHeapSlot {
    Present,
    ValueTag,
    ValuePayload,
}

struct SetEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl SetEntryHeapSlot {
    const fn metadata(&self) -> SetEntryHeapSlotMetadata {
        match self {
            Self::Present => SetEntryHeapSlotMetadata {
                record: "set-entry",
                name: "present",
                offset: HEAP_SET_ENTRY_PRESENT_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => SetEntryHeapSlotMetadata {
                record: "set-entry",
                name: "value_tag",
                offset: HEAP_SET_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => SetEntryHeapSlotMetadata {
                record: "set-entry",
                name: "value_payload",
                offset: HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_SET_ENTRY_LAYOUT: &[SetEntryHeapSlot] = &[
    SetEntryHeapSlot::Present,
    SetEntryHeapSlot::ValueTag,
    SetEntryHeapSlot::ValuePayload,
];
