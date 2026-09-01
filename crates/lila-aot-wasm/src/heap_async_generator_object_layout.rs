#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{HeapLayoutSlot, HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET};

pub(crate) enum AsyncGeneratorObjectHeapSlot {
    Activation,
}

struct AsyncGeneratorObjectHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl AsyncGeneratorObjectHeapSlot {
    const fn metadata(&self) -> AsyncGeneratorObjectHeapSlotMetadata {
        match self {
            Self::Activation => AsyncGeneratorObjectHeapSlotMetadata {
                record: "async-generator-object",
                name: "activation",
                offset: HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,
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

pub(crate) const HEAP_ASYNC_GENERATOR_OBJECT_LAYOUT: &[AsyncGeneratorObjectHeapSlot] =
    &[AsyncGeneratorObjectHeapSlot::Activation];
