#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
    HEAP_PROMISE_REACTION_CAPABILITY_OFFSET, HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
    HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET, HEAP_PROMISE_REACTION_NEXT_OFFSET,
    HEAP_PROMISE_REACTION_REALM_OFFSET, HEAP_PROMISE_REACTION_TYPE_OFFSET,
};

pub(crate) enum PromiseReactionHeapSlot {
    Capability,
    HandlerTag,
    HandlerPayload,
    Realm,
    Next,
    Type,
    CallbackKind,
}

struct PromiseReactionHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PromiseReactionHeapSlot {
    const fn metadata(&self) -> PromiseReactionHeapSlotMetadata {
        match self {
            Self::Capability => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "capability",
                offset: HEAP_PROMISE_REACTION_CAPABILITY_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HandlerTag => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "handler_tag",
                offset: HEAP_PROMISE_REACTION_HANDLER_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::HandlerPayload => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "handler_payload",
                offset: HEAP_PROMISE_REACTION_HANDLER_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Realm => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "realm",
                offset: HEAP_PROMISE_REACTION_REALM_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Next => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "next",
                offset: HEAP_PROMISE_REACTION_NEXT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::Type => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "type",
                offset: HEAP_PROMISE_REACTION_TYPE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CallbackKind => PromiseReactionHeapSlotMetadata {
                record: "promise-reaction-record",
                name: "callback_kind",
                offset: HEAP_PROMISE_REACTION_CALLBACK_KIND_OFFSET,
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

pub(crate) const HEAP_PROMISE_REACTION_LAYOUT: &[PromiseReactionHeapSlot] = &[
    PromiseReactionHeapSlot::Capability,
    PromiseReactionHeapSlot::HandlerTag,
    PromiseReactionHeapSlot::HandlerPayload,
    PromiseReactionHeapSlot::Realm,
    PromiseReactionHeapSlot::Next,
    PromiseReactionHeapSlot::Type,
    PromiseReactionHeapSlot::CallbackKind,
];
