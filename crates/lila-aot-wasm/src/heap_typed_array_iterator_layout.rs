#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET, HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET,
    HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
    HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET,
};

pub(crate) enum TypedArrayIteratorHeapSlot {
    TypedArrayPayload,
    NextIndex,
    Kind,
    Done,
}

struct TypedArrayIteratorHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl TypedArrayIteratorHeapSlot {
    const fn metadata(&self) -> TypedArrayIteratorHeapSlotMetadata {
        match self {
            Self::TypedArrayPayload => TypedArrayIteratorHeapSlotMetadata {
                record: "typed-array-iterator-record",
                name: "typed_array_payload",
                offset: HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NextIndex => TypedArrayIteratorHeapSlotMetadata {
                record: "typed-array-iterator-record",
                name: "next_index",
                offset: HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Kind => TypedArrayIteratorHeapSlotMetadata {
                record: "typed-array-iterator-record",
                name: "kind",
                offset: HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Done => TypedArrayIteratorHeapSlotMetadata {
                record: "typed-array-iterator-record",
                name: "done",
                offset: HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET,
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

pub(crate) const HEAP_TYPED_ARRAY_ITERATOR_RECORD_LAYOUT: &[TypedArrayIteratorHeapSlot] = &[
    TypedArrayIteratorHeapSlot::TypedArrayPayload,
    TypedArrayIteratorHeapSlot::NextIndex,
    TypedArrayIteratorHeapSlot::Kind,
    TypedArrayIteratorHeapSlot::Done,
];
