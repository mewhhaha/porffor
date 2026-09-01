#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC Symbol cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET, HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
    HEAP_SYMBOL_ID_OFFSET, HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
};

pub(crate) enum SymbolHeapSlot {
    DescriptionTag,
    DescriptionPayload,
    RegistryKeyPayload,
    SymbolId,
}

struct SymbolHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl SymbolHeapSlot {
    const fn metadata(&self) -> SymbolHeapSlotMetadata {
        match self {
            Self::DescriptionTag => SymbolHeapSlotMetadata {
                record: "symbol-record",
                name: "description_tag",
                offset: HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::DescriptionPayload => SymbolHeapSlotMetadata {
                record: "symbol-record",
                name: "description_payload",
                offset: HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::RegistryKeyPayload => SymbolHeapSlotMetadata {
                record: "symbol-record",
                name: "registry_key_payload",
                offset: HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::SymbolId => SymbolHeapSlotMetadata {
                record: "symbol-record",
                name: "symbol_id",
                offset: HEAP_SYMBOL_ID_OFFSET,
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

pub(crate) const HEAP_SYMBOL_LAYOUT: &[SymbolHeapSlot] = &[
    SymbolHeapSlot::DescriptionTag,
    SymbolHeapSlot::DescriptionPayload,
    SymbolHeapSlot::RegistryKeyPayload,
    SymbolHeapSlot::SymbolId,
];
