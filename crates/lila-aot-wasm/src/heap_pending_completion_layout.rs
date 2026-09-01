#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PENDING_COMPLETION_AUX_OFFSET, HEAP_PENDING_COMPLETION_KIND_OFFSET,
    HEAP_PENDING_COMPLETION_NEXT_OFFSET, HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET,
    HEAP_PENDING_COMPLETION_TAG_OFFSET,
};

pub(crate) enum PendingCompletionHeapSlot {
    Next,
    Payload,
    Tag,
    Kind,
    Aux,
}

struct PendingCompletionHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PendingCompletionHeapSlot {
    const fn metadata(&self) -> PendingCompletionHeapSlotMetadata {
        match self {
            Self::Next => PendingCompletionHeapSlotMetadata {
                record: "pending-completion-record",
                name: "next",
                offset: HEAP_PENDING_COMPLETION_NEXT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Payload => PendingCompletionHeapSlotMetadata {
                record: "pending-completion-record",
                name: "payload",
                offset: HEAP_PENDING_COMPLETION_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Tag => PendingCompletionHeapSlotMetadata {
                record: "pending-completion-record",
                name: "tag",
                offset: HEAP_PENDING_COMPLETION_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Kind => PendingCompletionHeapSlotMetadata {
                record: "pending-completion-record",
                name: "kind",
                offset: HEAP_PENDING_COMPLETION_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::Aux => PendingCompletionHeapSlotMetadata {
                record: "pending-completion-record",
                name: "aux",
                offset: HEAP_PENDING_COMPLETION_AUX_OFFSET,
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

pub(crate) const HEAP_PENDING_COMPLETION_LAYOUT: &[PendingCompletionHeapSlot] = &[
    PendingCompletionHeapSlot::Next,
    PendingCompletionHeapSlot::Payload,
    PendingCompletionHeapSlot::Tag,
    PendingCompletionHeapSlot::Kind,
    PendingCompletionHeapSlot::Aux,
];
