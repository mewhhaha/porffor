#![allow(
    dead_code,
    reason = "T05 layout metadata precedes executable weak reachability"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_WEAK_MAP_ENTRY_KEY_PAYLOAD_OFFSET, HEAP_WEAK_MAP_ENTRY_KEY_TAG_OFFSET,
    HEAP_WEAK_MAP_ENTRY_PRESENT_OFFSET, HEAP_WEAK_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
    HEAP_WEAK_MAP_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum WeakMapEntryHeapSlot {
    Present,
    KeyTag,
    KeyPayload,
    ValueTag,
    ValuePayload,
}

struct WeakMapEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl WeakMapEntryHeapSlot {
    const fn metadata(&self) -> WeakMapEntryHeapSlotMetadata {
        match self {
            Self::Present => WeakMapEntryHeapSlotMetadata {
                record: "weak-map-entry",
                name: "present",
                offset: HEAP_WEAK_MAP_ENTRY_PRESENT_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::KeyTag => WeakMapEntryHeapSlotMetadata {
                record: "weak-map-entry",
                name: "key_tag",
                offset: HEAP_WEAK_MAP_ENTRY_KEY_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::KeyPayload => WeakMapEntryHeapSlotMetadata {
                record: "weak-map-entry",
                name: "key_payload",
                offset: HEAP_WEAK_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValueTag => WeakMapEntryHeapSlotMetadata {
                record: "weak-map-entry",
                name: "value_tag",
                offset: HEAP_WEAK_MAP_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => WeakMapEntryHeapSlotMetadata {
                record: "weak-map-entry",
                name: "value_payload",
                offset: HEAP_WEAK_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_WEAK_MAP_ENTRY_LAYOUT: &[WeakMapEntryHeapSlot] = &[
    WeakMapEntryHeapSlot::Present,
    WeakMapEntryHeapSlot::KeyTag,
    WeakMapEntryHeapSlot::KeyPayload,
    WeakMapEntryHeapSlot::ValueTag,
    WeakMapEntryHeapSlot::ValuePayload,
];
