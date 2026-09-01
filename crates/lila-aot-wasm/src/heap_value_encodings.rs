//! Closed value-encoding identities for the passive heap ABI inventory.

#![allow(
    dead_code,
    reason = "T05 value-encoding metadata precedes the atomic Wasm-GC value cutover"
)]

use lila_ir::ValueKind;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ValuePayloadEncoding {
    Immediate,
    BooleanBit,
    Ieee754Bits,
    HeapPointer,
    StaticOrHeapPointer,
    I64TemporaryOrHeapPointer,
    DynamicTaggedPair,
}

pub(crate) enum HeapValueEncoding {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    Object,
    Array,
    Function,
    Arguments,
    BigInt,
    Dynamic,
}

impl HeapValueEncoding {
    pub(crate) const fn kind(&self) -> ValueKind {
        match self {
            Self::Undefined => ValueKind::Undefined,
            Self::Null => ValueKind::Null,
            Self::Boolean => ValueKind::Boolean,
            Self::Number => ValueKind::Number,
            Self::String => ValueKind::String,
            Self::Symbol => ValueKind::Symbol,
            Self::Object => ValueKind::Object,
            Self::Array => ValueKind::Array,
            Self::Function => ValueKind::Function,
            Self::Arguments => ValueKind::Arguments,
            Self::BigInt => ValueKind::BigInt,
            Self::Dynamic => ValueKind::Dynamic,
        }
    }

    pub(crate) const fn payload(&self) -> ValuePayloadEncoding {
        match self {
            Self::Undefined => ValuePayloadEncoding::Immediate,
            Self::Null => ValuePayloadEncoding::Immediate,
            Self::Boolean => ValuePayloadEncoding::BooleanBit,
            Self::Number => ValuePayloadEncoding::Ieee754Bits,
            Self::String => ValuePayloadEncoding::StaticOrHeapPointer,
            Self::Symbol => ValuePayloadEncoding::StaticOrHeapPointer,
            Self::Object => ValuePayloadEncoding::HeapPointer,
            Self::Array => ValuePayloadEncoding::HeapPointer,
            Self::Function => ValuePayloadEncoding::HeapPointer,
            Self::Arguments => ValuePayloadEncoding::HeapPointer,
            Self::BigInt => ValuePayloadEncoding::I64TemporaryOrHeapPointer,
            Self::Dynamic => ValuePayloadEncoding::DynamicTaggedPair,
        }
    }

    pub(crate) const fn preserves_number_bits(&self) -> bool {
        match self {
            Self::Undefined => false,
            Self::Null => false,
            Self::Boolean => false,
            Self::Number => true,
            Self::String => false,
            Self::Symbol => false,
            Self::Object => false,
            Self::Array => false,
            Self::Function => false,
            Self::Arguments => false,
            Self::BigInt => false,
            Self::Dynamic => false,
        }
    }

    pub(crate) const fn arbitrary_precision_ready(&self) -> bool {
        match self {
            Self::Undefined => true,
            Self::Null => true,
            Self::Boolean => true,
            Self::Number => true,
            Self::String => true,
            Self::Symbol => true,
            Self::Object => true,
            Self::Array => true,
            Self::Function => true,
            Self::Arguments => true,
            Self::BigInt => false,
            Self::Dynamic => true,
        }
    }
}

pub(crate) const HEAP_VALUE_ENCODINGS: &[HeapValueEncoding] = &[
    HeapValueEncoding::Undefined,
    HeapValueEncoding::Null,
    HeapValueEncoding::Boolean,
    HeapValueEncoding::Number,
    HeapValueEncoding::String,
    HeapValueEncoding::Symbol,
    HeapValueEncoding::Object,
    HeapValueEncoding::Array,
    HeapValueEncoding::Function,
    HeapValueEncoding::Arguments,
    HeapValueEncoding::BigInt,
    HeapValueEncoding::Dynamic,
];
