#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC BigInt cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_BIGINT_LIMBS_CAP_OFFSET, HEAP_BIGINT_LIMBS_LEN_OFFSET,
    HEAP_BIGINT_LIMBS_PTR_OFFSET, HEAP_BIGINT_SIGN_OFFSET,
};

pub(crate) enum BigIntHeapSlot {
    Sign,
    LimbsPointer,
    LimbsLength,
    LimbsCapacity,
}

struct BigIntHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl BigIntHeapSlot {
    const fn metadata(&self) -> BigIntHeapSlotMetadata {
        match self {
            Self::Sign => BigIntHeapSlotMetadata {
                record: "bigint-record",
                name: "sign",
                offset: HEAP_BIGINT_SIGN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LimbsPointer => BigIntHeapSlotMetadata {
                record: "bigint-record",
                name: "limbs_ptr",
                offset: HEAP_BIGINT_LIMBS_PTR_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::LimbsLength => BigIntHeapSlotMetadata {
                record: "bigint-record",
                name: "limbs_len",
                offset: HEAP_BIGINT_LIMBS_LEN_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::LimbsCapacity => BigIntHeapSlotMetadata {
                record: "bigint-record",
                name: "limbs_cap",
                offset: HEAP_BIGINT_LIMBS_CAP_OFFSET,
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

pub(crate) const HEAP_BIGINT_LAYOUT: &[BigIntHeapSlot] = &[
    BigIntHeapSlot::Sign,
    BigIntHeapSlot::LimbsPointer,
    BigIntHeapSlot::LimbsLength,
    BigIntHeapSlot::LimbsCapacity,
];
