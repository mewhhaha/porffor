#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
    HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
};

pub(crate) enum FinalizationRegistryCellHeapSlot {
    State,
    TargetTag,
    TargetPayload,
    HoldingsTag,
    HoldingsPayload,
    UnregisterTokenTag,
    UnregisterTokenPayload,
}

struct FinalizationRegistryCellHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl FinalizationRegistryCellHeapSlot {
    const fn metadata(&self) -> FinalizationRegistryCellHeapSlotMetadata {
        match self {
            Self::State => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "state",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TargetTag => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "target_tag",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::TargetPayload => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "target_payload",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::HoldingsTag => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "holdings_tag",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::HoldingsPayload => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "holdings_payload",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::UnregisterTokenTag => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "unregister_token_tag",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::UnregisterTokenPayload => FinalizationRegistryCellHeapSlotMetadata {
                record: "finalization-registry-cell",
                name: "unregister_token_payload",
                offset: HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
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

pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT: &[FinalizationRegistryCellHeapSlot] = &[
    FinalizationRegistryCellHeapSlot::State,
    FinalizationRegistryCellHeapSlot::TargetTag,
    FinalizationRegistryCellHeapSlot::TargetPayload,
    FinalizationRegistryCellHeapSlot::HoldingsTag,
    FinalizationRegistryCellHeapSlot::HoldingsPayload,
    FinalizationRegistryCellHeapSlot::UnregisterTokenTag,
    FinalizationRegistryCellHeapSlot::UnregisterTokenPayload,
];
