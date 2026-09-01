#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC collection cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
    HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
    HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET, HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
};

pub(crate) enum ClassFunctionContextHeapSlot {
    LexicalEnvironment,
    ActiveFunction,
    HomeObjectPayload,
    HomeObjectTag,
    FieldKeys,
    PrivateEnvironment,
}

struct ClassFunctionContextHeapSlotMetadata {
    record: &'static str,
    name: &'static str,
    offset: u64,
    width: u64,
    pointer: bool,
}

impl ClassFunctionContextHeapSlot {
    const fn metadata(&self) -> ClassFunctionContextHeapSlotMetadata {
        match self {
            Self::LexicalEnvironment => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "lexical_env",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::ActiveFunction => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "active_function",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HomeObjectPayload => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "home_object_payload",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::HomeObjectTag => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "home_object_tag",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                width: 8,
                pointer: false,
            },
            Self::FieldKeys => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "field_keys",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                width: 8,
                pointer: true,
            },
            Self::PrivateEnvironment => ClassFunctionContextHeapSlotMetadata {
                record: "class-function-context",
                name: "private_environment",
                offset: HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
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

pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT: &[ClassFunctionContextHeapSlot] = &[
    ClassFunctionContextHeapSlot::LexicalEnvironment,
    ClassFunctionContextHeapSlot::ActiveFunction,
    ClassFunctionContextHeapSlot::HomeObjectPayload,
    ClassFunctionContextHeapSlot::HomeObjectTag,
    ClassFunctionContextHeapSlot::FieldKeys,
    ClassFunctionContextHeapSlot::PrivateEnvironment,
];
