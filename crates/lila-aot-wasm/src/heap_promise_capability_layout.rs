#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
    HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET, HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
    HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET, HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
    HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
};

pub(crate) enum PromiseCapabilityHeapSlot {
    PromiseTag,
    PromisePayload,
    ResolveTag,
    ResolvePayload,
    RejectTag,
    RejectPayload,
}

struct PromiseCapabilityHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PromiseCapabilityHeapSlot {
    const fn metadata(&self) -> PromiseCapabilityHeapSlotMetadata {
        match self {
            Self::PromiseTag => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "promise_tag",
                offset: HEAP_PROMISE_CAPABILITY_PROMISE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::PromisePayload => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "promise_payload",
                offset: HEAP_PROMISE_CAPABILITY_PROMISE_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ResolveTag => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "resolve_tag",
                offset: HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ResolvePayload => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "resolve_payload",
                offset: HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::RejectTag => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "reject_tag",
                offset: HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::RejectPayload => PromiseCapabilityHeapSlotMetadata {
                record: "promise-capability-record",
                name: "reject_payload",
                offset: HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_PROMISE_CAPABILITY_LAYOUT: &[PromiseCapabilityHeapSlot] = &[
    PromiseCapabilityHeapSlot::PromiseTag,
    PromiseCapabilityHeapSlot::PromisePayload,
    PromiseCapabilityHeapSlot::ResolveTag,
    PromiseCapabilityHeapSlot::ResolvePayload,
    PromiseCapabilityHeapSlot::RejectTag,
    PromiseCapabilityHeapSlot::RejectPayload,
];
