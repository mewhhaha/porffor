#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET, HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,
};

pub(crate) enum FinalizationRegistryRecordHeapSlot {
    CleanupCallbackTag,
    CleanupCallbackPayload,
    CellsPointer,
    CellsLength,
    CellsCapacity,
}

struct FinalizationRegistryRecordHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl FinalizationRegistryRecordHeapSlot {
    const fn metadata(&self) -> FinalizationRegistryRecordHeapSlotMetadata {
        match self {
            Self::CleanupCallbackTag => FinalizationRegistryRecordHeapSlotMetadata {
                record: "finalization-registry-record",
                name: "cleanup_callback_tag",
                offset: HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CleanupCallbackPayload => FinalizationRegistryRecordHeapSlotMetadata {
                record: "finalization-registry-record",
                name: "cleanup_callback_payload",
                offset: HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::CellsPointer => FinalizationRegistryRecordHeapSlotMetadata {
                record: "finalization-registry-record",
                name: "cells_ptr",
                offset: HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::CellsLength => FinalizationRegistryRecordHeapSlotMetadata {
                record: "finalization-registry-record",
                name: "cells_len",
                offset: HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CellsCapacity => FinalizationRegistryRecordHeapSlotMetadata {
                record: "finalization-registry-record",
                name: "cells_cap",
                offset: HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
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

pub(crate) const HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT: &[FinalizationRegistryRecordHeapSlot] =
    &[
        FinalizationRegistryRecordHeapSlot::CleanupCallbackTag,
        FinalizationRegistryRecordHeapSlot::CleanupCallbackPayload,
        FinalizationRegistryRecordHeapSlot::CellsPointer,
        FinalizationRegistryRecordHeapSlot::CellsLength,
        FinalizationRegistryRecordHeapSlot::CellsCapacity,
    ];
