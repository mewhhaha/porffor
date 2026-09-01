#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, HEAP_OBJECT_DATA_TAG_OFFSET,
    HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
    HEAP_OBJECT_GETTER_TAG_OFFSET, HEAP_OBJECT_KEY_OFFSET, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
    HEAP_OBJECT_SETTER_TAG_OFFSET,
};

pub(crate) enum ObjectEntryHeapSlot {
    Key,
    DescriptorKind,
    DataTag,
    DataPayload,
    GetterTag,
    GetterPayload,
    SetterTag,
    SetterPayload,
}

struct ObjectEntryHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl ObjectEntryHeapSlot {
    const fn metadata(&self) -> ObjectEntryHeapSlotMetadata {
        match self {
            Self::Key => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "key",
                offset: HEAP_OBJECT_KEY_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::DescriptorKind => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "descriptor_kind",
                offset: HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DataTag => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "data_tag",
                offset: HEAP_OBJECT_DATA_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DataPayload => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "data_payload",
                offset: HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::GetterTag => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "getter_tag",
                offset: HEAP_OBJECT_GETTER_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::GetterPayload => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "getter_payload",
                offset: HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::SetterTag => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "setter_tag",
                offset: HEAP_OBJECT_SETTER_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::SetterPayload => ObjectEntryHeapSlotMetadata {
                record: "object-entry",
                name: "setter_payload",
                offset: HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_OBJECT_ENTRY_LAYOUT: &[ObjectEntryHeapSlot] = &[
    ObjectEntryHeapSlot::Key,
    ObjectEntryHeapSlot::DescriptorKind,
    ObjectEntryHeapSlot::DataTag,
    ObjectEntryHeapSlot::DataPayload,
    ObjectEntryHeapSlot::GetterTag,
    ObjectEntryHeapSlot::GetterPayload,
    ObjectEntryHeapSlot::SetterTag,
    ObjectEntryHeapSlot::SetterPayload,
];
