#![allow(
    dead_code,
    reason = "T05 layout metadata precedes executable weak reachability"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET, HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
    HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum WeakSetEntryHeapSlot {
    Present,
    ValueTag,
    ValuePayload,
}

struct WeakSetEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl WeakSetEntryHeapSlot {
    const fn metadata(&self) -> WeakSetEntryHeapSlotMetadata {
        match self {
            Self::Present => WeakSetEntryHeapSlotMetadata {
                record: "weak-set-entry",
                name: "present",
                offset: HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => WeakSetEntryHeapSlotMetadata {
                record: "weak-set-entry",
                name: "value_tag",
                offset: HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => WeakSetEntryHeapSlotMetadata {
                record: "weak-set-entry",
                name: "value_payload",
                offset: HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_WEAK_SET_ENTRY_LAYOUT: &[WeakSetEntryHeapSlot] = &[
    WeakSetEntryHeapSlot::Present,
    WeakSetEntryHeapSlot::ValueTag,
    WeakSetEntryHeapSlot::ValuePayload,
];
