#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET, HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
    HEAP_MAP_ENTRY_PRESENT_OFFSET, HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
    HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
};

pub(crate) enum MapEntryHeapSlot {
    Present,
    KeyTag,
    KeyPayload,
    ValueTag,
    ValuePayload,
}

struct MapEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl MapEntryHeapSlot {
    const fn metadata(&self) -> MapEntryHeapSlotMetadata {
        match self {
            Self::Present => MapEntryHeapSlotMetadata {
                record: "map-entry",
                name: "present",
                offset: HEAP_MAP_ENTRY_PRESENT_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::KeyTag => MapEntryHeapSlotMetadata {
                record: "map-entry",
                name: "key_tag",
                offset: HEAP_MAP_ENTRY_KEY_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::KeyPayload => MapEntryHeapSlotMetadata {
                record: "map-entry",
                name: "key_payload",
                offset: HEAP_MAP_ENTRY_KEY_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ValueTag => MapEntryHeapSlotMetadata {
                record: "map-entry",
                name: "value_tag",
                offset: HEAP_MAP_ENTRY_VALUE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ValuePayload => MapEntryHeapSlotMetadata {
                record: "map-entry",
                name: "value_payload",
                offset: HEAP_MAP_ENTRY_VALUE_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_MAP_ENTRY_LAYOUT: &[MapEntryHeapSlot] = &[
    MapEntryHeapSlot::Present,
    MapEntryHeapSlot::KeyTag,
    MapEntryHeapSlot::KeyPayload,
    MapEntryHeapSlot::ValueTag,
    MapEntryHeapSlot::ValuePayload,
];
