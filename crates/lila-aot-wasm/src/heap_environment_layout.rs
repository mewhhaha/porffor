#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC environment cutover"
)]

use super::heap::{
    HeapLayoutSlot, ENV_PARENT_OFFSET, ENV_SLOT_PAYLOAD_OFFSET, ENV_SLOT_TAG_OFFSET,
};

pub(crate) enum EnvironmentHeapSlot {
    Parent,
    BindingTag,
    BindingPayload,
}

struct EnvironmentHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl EnvironmentHeapSlot {
    const fn metadata(&self) -> EnvironmentHeapSlotMetadata {
        match self {
            Self::Parent => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "parent",
                offset: ENV_PARENT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::BindingTag => EnvironmentHeapSlotMetadata {
                record: "environment-slot",
                name: "tag",
                offset: ENV_SLOT_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::BindingPayload => EnvironmentHeapSlotMetadata {
                record: "environment-slot",
                name: "payload",
                offset: ENV_SLOT_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_ENVIRONMENT_LAYOUT: &[EnvironmentHeapSlot] = &[
    EnvironmentHeapSlot::Parent,
    EnvironmentHeapSlot::BindingTag,
    EnvironmentHeapSlot::BindingPayload,
];
