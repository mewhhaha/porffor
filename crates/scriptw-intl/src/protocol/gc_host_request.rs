//! A private byte-resource envelope for the existing pure Intl operations.

use super::IntlHostOp;
use core::fmt;

/// The emitted prefix uses one little-endian u64 closed operation code.
pub const INTL_GC_REQUEST_PREFIX_BYTES: usize = core::mem::size_of::<u64>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntlGcHostRequestError {
    ExceedsWasm32,
    IncompleteOperation,
    UnknownOperation,
}
impl fmt::Display for IntlGcHostRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ExceedsWasm32 => "Intl byte request exceeds the Wasm32 extent",
            Self::IncompleteOperation => "Intl byte request lacks its complete operation prefix",
            Self::UnknownOperation => "Intl byte request has an unknown operation",
        })
    }
}
impl std::error::Error for IntlGcHostRequestError {}

/// Only a checked prefix can expose the operation and borrowed payload.
/// Operation-specific native decoders retain their own version/domain checks.
#[derive(Debug)]
pub struct IntlGcHostRequest<'a> {
    operation: IntlHostOp,
    payload: &'a [u8],
}
impl<'a> IntlGcHostRequest<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, IntlGcHostRequestError> {
        u32::try_from(bytes.len()).map_err(|_| IntlGcHostRequestError::ExceedsWasm32)?;
        let prefix = bytes
            .get(..INTL_GC_REQUEST_PREFIX_BYTES)
            .ok_or(IntlGcHostRequestError::IncompleteOperation)?;
        let word = u64::from_le_bytes(prefix.try_into().expect("the prefix width is checked"));
        let code = i64::try_from(word).map_err(|_| IntlGcHostRequestError::UnknownOperation)?;
        let operation =
            IntlHostOp::from_wire(code).ok_or(IntlGcHostRequestError::UnknownOperation)?;
        Ok(Self {
            operation,
            payload: &bytes[INTL_GC_REQUEST_PREFIX_BYTES..],
        })
    }
    #[must_use]
    pub const fn operation(&self) -> IntlHostOp {
        self.operation
    }
    #[must_use]
    pub const fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_incomplete_prefix_cannot_mint_a_request() {
        for width in 0..INTL_GC_REQUEST_PREFIX_BYTES {
            assert_eq!(
                IntlGcHostRequest::decode(&[0; INTL_GC_REQUEST_PREFIX_BYTES][..width]).unwrap_err(),
                IntlGcHostRequestError::IncompleteOperation
            );
        }
    }

    #[test]
    fn unknown_and_negative_operation_words_are_rejected() {
        for word in [u64::MAX, u64::from(u16::MAX)] {
            assert_eq!(
                IntlGcHostRequest::decode(&word.to_le_bytes()).unwrap_err(),
                IntlGcHostRequestError::UnknownOperation
            );
        }
    }

    #[test]
    fn each_closed_operation_retains_the_original_payload() {
        for operation in IntlHostOp::ALL {
            let mut bytes = operation.wire().to_le_bytes().to_vec();
            bytes.extend_from_slice(b"payload");
            let request = IntlGcHostRequest::decode(&bytes).unwrap();
            assert_eq!(request.operation(), *operation);
            assert_eq!(request.payload(), b"payload");
            assert_eq!(
                request.payload().as_ptr(),
                bytes[INTL_GC_REQUEST_PREFIX_BYTES..].as_ptr()
            );
        }
        let bytes = IntlHostOp::CanonicalizeLocale.wire().to_le_bytes();
        assert!(IntlGcHostRequest::decode(&bytes)
            .unwrap()
            .payload()
            .is_empty());
    }
}
