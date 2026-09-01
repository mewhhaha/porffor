#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET, HEAP_SET_ITERATOR_KIND_OFFSET,
    HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET, HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET,
};

pub(crate) enum SetIteratorHeapSlot {
    SetPayload,
    NextIndex,
    Kind,
    CursorState,
}

struct SetIteratorHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl SetIteratorHeapSlot {
    const fn metadata(&self) -> SetIteratorHeapSlotMetadata {
        match self {
            Self::SetPayload => SetIteratorHeapSlotMetadata {
                record: "set-iterator-record",
                name: "set_payload",
                offset: HEAP_SET_ITERATOR_SET_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NextIndex => SetIteratorHeapSlotMetadata {
                record: "set-iterator-record",
                name: "next_index",
                offset: HEAP_SET_ITERATOR_NEXT_INDEX_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Kind => SetIteratorHeapSlotMetadata {
                record: "set-iterator-record",
                name: "kind",
                offset: HEAP_SET_ITERATOR_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CursorState => SetIteratorHeapSlotMetadata {
                record: "set-iterator-record",
                name: "cursor_state",
                offset: HEAP_SET_ITERATOR_CURSOR_STATE_OFFSET,
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

pub(crate) const HEAP_SET_ITERATOR_RECORD_LAYOUT: &[SetIteratorHeapSlot] = &[
    SetIteratorHeapSlot::SetPayload,
    SetIteratorHeapSlot::NextIndex,
    SetIteratorHeapSlot::Kind,
    SetIteratorHeapSlot::CursorState,
];
