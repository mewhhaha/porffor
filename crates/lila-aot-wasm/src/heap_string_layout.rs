#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC string cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_STRING_BYTE_LEN_OFFSET, HEAP_STRING_CODE_UNITS_PTR_OFFSET,
    HEAP_STRING_CODE_UNIT_LEN_OFFSET, HEAP_STRING_INTERN_ID_OFFSET,
};

pub(crate) enum StringHeapSlot {
    CodeUnitsPointer,
    ByteLength,
    CodeUnitLength,
    InternId,
}

struct StringHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl StringHeapSlot {
    const fn metadata(&self) -> StringHeapSlotMetadata {
        match self {
            Self::CodeUnitsPointer => StringHeapSlotMetadata {
                record: "string-record",
                name: "code_units_ptr",
                offset: HEAP_STRING_CODE_UNITS_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ByteLength => StringHeapSlotMetadata {
                record: "string-record",
                name: "byte_len",
                offset: HEAP_STRING_BYTE_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::CodeUnitLength => StringHeapSlotMetadata {
                record: "string-record",
                name: "code_unit_len",
                offset: HEAP_STRING_CODE_UNIT_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::InternId => StringHeapSlotMetadata {
                record: "string-record",
                name: "intern_id",
                offset: HEAP_STRING_INTERN_ID_OFFSET,
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

pub(crate) const HEAP_STRING_LAYOUT: &[StringHeapSlot] = &[
    StringHeapSlot::CodeUnitsPointer,
    StringHeapSlot::ByteLength,
    StringHeapSlot::CodeUnitLength,
    StringHeapSlot::InternId,
];
