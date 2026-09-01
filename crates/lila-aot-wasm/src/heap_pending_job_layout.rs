#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET, HEAP_PENDING_JOB_ARG_TAG_OFFSET,
    HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET, HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
    HEAP_PENDING_JOB_KIND_OFFSET, HEAP_PENDING_JOB_NEXT_OFFSET, HEAP_PENDING_JOB_REALM_OFFSET,
};

pub(crate) enum PendingJobHeapSlot {
    CallbackTag,
    CallbackPayload,
    ArgumentTag,
    ArgumentPayload,
    Realm,
    Next,
    Kind,
}

struct PendingJobHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PendingJobHeapSlot {
    const fn metadata(&self) -> PendingJobHeapSlotMetadata {
        match self {
            Self::CallbackTag => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "callback_tag",
                offset: HEAP_PENDING_JOB_CALLBACK_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CallbackPayload => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "callback_payload",
                offset: HEAP_PENDING_JOB_CALLBACK_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ArgumentTag => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "arg_tag",
                offset: HEAP_PENDING_JOB_ARG_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ArgumentPayload => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "arg_payload",
                offset: HEAP_PENDING_JOB_ARG_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Realm => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "realm",
                offset: HEAP_PENDING_JOB_REALM_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Next => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "next",
                offset: HEAP_PENDING_JOB_NEXT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Kind => PendingJobHeapSlotMetadata {
                record: "pending-job-record",
                name: "kind",
                offset: HEAP_PENDING_JOB_KIND_OFFSET,
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

pub(crate) const HEAP_PENDING_JOB_LAYOUT: &[PendingJobHeapSlot] = &[
    PendingJobHeapSlot::CallbackTag,
    PendingJobHeapSlot::CallbackPayload,
    PendingJobHeapSlot::ArgumentTag,
    PendingJobHeapSlot::ArgumentPayload,
    PendingJobHeapSlot::Realm,
    PendingJobHeapSlot::Next,
    PendingJobHeapSlot::Kind,
];
