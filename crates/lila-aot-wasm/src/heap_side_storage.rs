//! Closed layout vocabulary for retained linear-memory side storage.
//!
//! These spans contain raw scalar bytes owned by future Wasm-GC objects. They
//! are not JavaScript object identities and must never be scanned as GC
//! references.

#![allow(
    dead_code,
    reason = "T05 side-storage metadata precedes its Wasm-GC owner cutover"
)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinearSideStorageElement {
    Byte,
    Utf16CodeUnit,
    BigIntLimb,
}

impl LinearSideStorageElement {
    pub(crate) const fn byte_width(self) -> u64 {
        match self {
            Self::Byte => 1,
            Self::Utf16CodeUnit => 2,
            Self::BigIntLimb => 8,
        }
    }

    pub(crate) const fn is_reference_storage(self) -> bool {
        match self {
            Self::Byte => false,
            Self::Utf16CodeUnit => false,
            Self::BigIntLimb => false,
        }
    }
}

pub(crate) enum LinearSideStorage {
    ArrayBufferBackingStore,
    StringCodeUnits,
    BigIntLimbs,
}

struct LinearSideStorageMetadata {
    record: &'static str,
    length_source: &'static str,
    element: LinearSideStorageElement,
}

impl LinearSideStorage {
    const fn metadata(&self) -> LinearSideStorageMetadata {
        match self {
            Self::ArrayBufferBackingStore => LinearSideStorageMetadata {
                record: "array-buffer-backing-store",
                length_source: "array-buffer-object-header.max_byte_length",
                element: LinearSideStorageElement::Byte,
            },
            Self::StringCodeUnits => LinearSideStorageMetadata {
                record: "string-code-units",
                length_source: "string-record.code_unit_len",
                element: LinearSideStorageElement::Utf16CodeUnit,
            },
            Self::BigIntLimbs => LinearSideStorageMetadata {
                record: "bigint-limbs",
                length_source: "bigint-record.limbs_len",
                element: LinearSideStorageElement::BigIntLimb,
            },
        }
    }

    pub(crate) const fn record(&self) -> &'static str {
        self.metadata().record
    }

    pub(crate) const fn length_source(&self) -> &'static str {
        self.metadata().length_source
    }

    pub(crate) const fn element(&self) -> LinearSideStorageElement {
        self.metadata().element
    }
}

pub(crate) const LINEAR_SIDE_STORAGES: &[LinearSideStorage] = &[
    LinearSideStorage::ArrayBufferBackingStore,
    LinearSideStorage::StringCodeUnits,
    LinearSideStorage::BigIntLimbs,
];
