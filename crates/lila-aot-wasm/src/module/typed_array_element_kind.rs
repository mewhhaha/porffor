use lila_ir::StandardBuiltinId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TypedArrayContentType {
    Number,
    BigInt,
}

/// The element-kind slot is an internal ABI word, not a numeric ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub(crate) enum TypedArrayElementKind {
    Float64 = 1,
    Float32 = 2,
    Int8 = 3,
    Int16 = 4,
    Int32 = 5,
    Uint8Clamped = 6,
    Uint8 = 7,
    Uint16 = 8,
    Uint32 = 9,
    BigInt64 = 10,
    BigUint64 = 11,
    Float16 = 12,
}

impl TypedArrayElementKind {
    pub(crate) const ALL: [Self; 12] = [
        Self::Float64,
        Self::Float32,
        Self::Int8,
        Self::Int16,
        Self::Int32,
        Self::Uint8Clamped,
        Self::Uint8,
        Self::Uint16,
        Self::Uint32,
        Self::BigInt64,
        Self::BigUint64,
        Self::Float16,
    ];

    pub(crate) const fn abi_word(self) -> u64 {
        self as u64
    }

    pub(crate) const fn constructor(self) -> StandardBuiltinId {
        match self {
            Self::Float64 => StandardBuiltinId::Float64ArrayConstructor,
            Self::Float32 => StandardBuiltinId::Float32ArrayConstructor,
            Self::Int8 => StandardBuiltinId::Int8ArrayConstructor,
            Self::Int16 => StandardBuiltinId::Int16ArrayConstructor,
            Self::Int32 => StandardBuiltinId::Int32ArrayConstructor,
            Self::Uint8Clamped => StandardBuiltinId::Uint8ClampedArrayConstructor,
            Self::Uint8 => StandardBuiltinId::Uint8ArrayConstructor,
            Self::Uint16 => StandardBuiltinId::Uint16ArrayConstructor,
            Self::Uint32 => StandardBuiltinId::Uint32ArrayConstructor,
            Self::BigInt64 => StandardBuiltinId::BigInt64ArrayConstructor,
            Self::BigUint64 => StandardBuiltinId::BigUint64ArrayConstructor,
            Self::Float16 => StandardBuiltinId::Float16ArrayConstructor,
        }
    }

    pub(crate) fn from_constructor(constructor: StandardBuiltinId) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.constructor() == constructor)
    }

    pub(crate) const fn bytes_per_element(self) -> u64 {
        match self {
            Self::Float64 | Self::BigInt64 | Self::BigUint64 => 8,
            Self::Float32 | Self::Int32 | Self::Uint32 => 4,
            Self::Float16 | Self::Int16 | Self::Uint16 => 2,
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
        }
    }

    pub(crate) const fn content_type(self) -> TypedArrayContentType {
        match self {
            Self::BigInt64 | Self::BigUint64 => TypedArrayContentType::BigInt,
            Self::Float16
            | Self::Float32
            | Self::Float64
            | Self::Int8
            | Self::Int16
            | Self::Int32
            | Self::Uint8
            | Self::Uint16
            | Self::Uint32
            | Self::Uint8Clamped => TypedArrayContentType::Number,
        }
    }

    pub(crate) const fn is_atomics_integer(self) -> bool {
        match self {
            Self::Float16 | Self::Float32 | Self::Float64 | Self::Uint8Clamped => false,
            Self::Int8
            | Self::Int16
            | Self::Int32
            | Self::Uint8
            | Self::Uint16
            | Self::Uint32
            | Self::BigInt64
            | Self::BigUint64 => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_catalog_owns_unique_storage_words_and_content_types() {
        let kinds = TypedArrayElementKind::ALL;
        for (index, kind) in kinds.into_iter().enumerate() {
            assert_eq!(
                TypedArrayElementKind::from_constructor(kind.constructor()),
                Some(kind)
            );
            assert!(kinds[..index]
                .iter()
                .all(|prior| prior.abi_word() != kind.abi_word()));
            assert_eq!(
                kind.constructor()
                    .global_name()
                    .unwrap()
                    .trim_end_matches("Array"),
                format!("{kind:?}")
            );
        }
        assert_eq!(TypedArrayElementKind::Float16.bytes_per_element(), 2);
        assert_eq!(
            TypedArrayElementKind::Float16.content_type(),
            TypedArrayContentType::Number
        );
        assert!(!TypedArrayElementKind::Float16.is_atomics_integer());
    }
}
