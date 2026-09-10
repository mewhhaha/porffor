#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC environment cutover"
)]

use super::heap::{
    HeapLayoutSlot, ENV_FUNCTION_BODY_OFFSET, ENV_NAMED_COUNT_OFFSET, ENV_NAMED_ENTRIES_OFFSET,
    ENV_PARENT_OFFSET, ENV_RECORD_KIND_OFFSET, ENV_SLOT_PAYLOAD_OFFSET, ENV_SLOT_TAG_OFFSET,
    ENV_WITH_OBJECT_OFFSET,
};

pub(crate) enum EnvironmentHeapSlot {
    Parent,
    FunctionBody,
    NamedBindings,
    NamedBindingCount,
    WithObjectCell,
    RecordKind,
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
            Self::FunctionBody => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "function-body",
                offset: ENV_FUNCTION_BODY_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NamedBindings => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "named-bindings",
                offset: ENV_NAMED_ENTRIES_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::NamedBindingCount => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "named-binding-count",
                offset: ENV_NAMED_COUNT_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::WithObjectCell => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "with-object-cell",
                offset: ENV_WITH_OBJECT_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::RecordKind => EnvironmentHeapSlotMetadata {
                record: "environment",
                name: "record-kind",
                offset: ENV_RECORD_KIND_OFFSET,
                width: 8,
                pointer: false,
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
    EnvironmentHeapSlot::FunctionBody,
    EnvironmentHeapSlot::NamedBindings,
    EnvironmentHeapSlot::NamedBindingCount,
    EnvironmentHeapSlot::WithObjectCell,
    EnvironmentHeapSlot::RecordKind,
    EnvironmentHeapSlot::BindingTag,
    EnvironmentHeapSlot::BindingPayload,
];
