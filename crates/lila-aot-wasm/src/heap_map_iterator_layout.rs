#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET, HEAP_MAP_ITERATOR_KIND_OFFSET,
    HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET, HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET,
};

pub(crate) enum MapIteratorHeapSlot {
    MapPayload,
    NextIndex,
    Kind,
    CursorState,
}

struct MapIteratorHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl MapIteratorHeapSlot {
    const fn metadata(&self) -> MapIteratorHeapSlotMetadata {
        match self {
            Self::MapPayload => MapIteratorHeapSlotMetadata {
                record: "map-iterator-record",
                name: "map_payload",
                offset: HEAP_MAP_ITERATOR_MAP_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NextIndex => MapIteratorHeapSlotMetadata {
                record: "map-iterator-record",
                name: "next_index",
                offset: HEAP_MAP_ITERATOR_NEXT_INDEX_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Kind => MapIteratorHeapSlotMetadata {
                record: "map-iterator-record",
                name: "kind",
                offset: HEAP_MAP_ITERATOR_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CursorState => MapIteratorHeapSlotMetadata {
                record: "map-iterator-record",
                name: "cursor_state",
                offset: HEAP_MAP_ITERATOR_CURSOR_STATE_OFFSET,
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

pub(crate) const HEAP_MAP_ITERATOR_RECORD_LAYOUT: &[MapIteratorHeapSlot] = &[
    MapIteratorHeapSlot::MapPayload,
    MapIteratorHeapSlot::NextIndex,
    MapIteratorHeapSlot::Kind,
    MapIteratorHeapSlot::CursorState,
];
