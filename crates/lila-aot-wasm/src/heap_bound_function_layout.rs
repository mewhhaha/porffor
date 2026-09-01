#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
    HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET, HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
    HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET, HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
    HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
};

pub(crate) enum BoundFunctionHeapSlot {
    TargetPayload,
    TargetTag,
    ThisPayload,
    ThisTag,
    ArgumentsPayload,
    SelfPayload,
}

struct BoundFunctionHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl BoundFunctionHeapSlot {
    const fn metadata(&self) -> BoundFunctionHeapSlotMetadata {
        match self {
            Self::TargetPayload => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "target_payload",
                offset: HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::TargetTag => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "target_tag",
                offset: HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ThisPayload => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "this_payload",
                offset: HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ThisTag => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "this_tag",
                offset: HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::ArgumentsPayload => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "args_payload",
                offset: HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::SelfPayload => BoundFunctionHeapSlotMetadata {
                record: "bound-function",
                name: "self_payload",
                offset: HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_BOUND_FUNCTION_LAYOUT: &[BoundFunctionHeapSlot] = &[
    BoundFunctionHeapSlot::TargetPayload,
    BoundFunctionHeapSlot::TargetTag,
    BoundFunctionHeapSlot::ThisPayload,
    BoundFunctionHeapSlot::ThisTag,
    BoundFunctionHeapSlot::ArgumentsPayload,
    BoundFunctionHeapSlot::SelfPayload,
];
