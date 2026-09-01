#![allow(
    dead_code,
    reason = "T05 layout metadata precedes executable weak reachability"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET, HEAP_WEAK_REF_TARGET_TAG_OFFSET,
};

pub(crate) enum WeakRefHeapSlot {
    TargetTag,
    TargetPayload,
}

struct WeakRefHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl WeakRefHeapSlot {
    const fn metadata(&self) -> WeakRefHeapSlotMetadata {
        match self {
            Self::TargetTag => WeakRefHeapSlotMetadata {
                record: "weak-ref-record",
                name: "target_tag",
                offset: HEAP_WEAK_REF_TARGET_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TargetPayload => WeakRefHeapSlotMetadata {
                record: "weak-ref-record",
                name: "target_payload",
                offset: HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_WEAK_REF_RECORD_LAYOUT: &[WeakRefHeapSlot] =
    &[WeakRefHeapSlot::TargetTag, WeakRefHeapSlot::TargetPayload];
