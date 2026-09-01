#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC object cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET, HEAP_PRIVATE_ENV_PARENT_OFFSET,
};

pub(crate) enum PrivateEnvironmentHeapSlot {
    Parent,
    ClassScope,
}

struct PrivateEnvironmentHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl PrivateEnvironmentHeapSlot {
    const fn metadata(&self) -> PrivateEnvironmentHeapSlotMetadata {
        match self {
            Self::Parent => PrivateEnvironmentHeapSlotMetadata {
                record: "private-environment",
                name: "parent",
                offset: HEAP_PRIVATE_ENV_PARENT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ClassScope => PrivateEnvironmentHeapSlotMetadata {
                record: "private-environment",
                name: "class_scope",
                offset: HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
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

pub(crate) const HEAP_PRIVATE_ENV_LAYOUT: &[PrivateEnvironmentHeapSlot] = &[
    PrivateEnvironmentHeapSlot::Parent,
    PrivateEnvironmentHeapSlot::ClassScope,
];
