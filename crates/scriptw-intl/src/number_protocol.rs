//! Lossless primitive NumberFormat messages shared by Wasm emission and host.
use std::sync::Arc;

use crate::number_format::numeric::ObservedNumericInput;
use crate::number_format::options::*;
use crate::number_format::*;
use crate::number_operation::*;
use crate::{CanonicalLocaleId, IntlHostOp};
use core::fmt;

mod configuration;
mod domains;
mod messages;
mod primitives;
mod rounding;
use crate::number_format::numeric::RoundingSettings;
pub use domains::{NumberConfigurationWord, NumberNumericKind, NumberPrecisionKind};
pub(crate) use primitives::{NumberWireReader, NumberWireWriter};
pub(crate) use rounding::{decode_rounding_words, rounding_wire_words};

pub const NUMBER_WIRE_VERSION: u64 = 1;
pub const NUMBER_WIRE_HEADER_BYTES: u64 = 16;
pub const NUMBER_CONFIGURATION_WORDS: usize = 17;
pub const NUMBER_APPROXIMATELY_SIGN_CODE: u64 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberWireError {
    UnavailableService(crate::IntlService),
    Malformed(&'static str),
    Configuration(InvalidNumberConfiguration),
    Kernel(NumberFormatKernelError),
    Resource(&'static str),
}
impl fmt::Display for NumberWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::Malformed(reason) => write!(f, "invalid number wire message: {reason}"),
            Self::Configuration(error) => write!(f, "invalid number wire configuration: {error:?}"),
            Self::Kernel(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "number wire resource limit: {reason}"),
        }
    }
}
impl std::error::Error for NumberWireError {}
impl From<InvalidNumberConfiguration> for NumberWireError {
    fn from(error: InvalidNumberConfiguration) -> Self {
        Self::Configuration(error)
    }
}
impl From<NumberFormatKernelError> for NumberWireError {
    fn from(error: NumberFormatKernelError) -> Self {
        Self::Kernel(error)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum NumberWireDirection {
    Request,
    Response,
}
impl NumberWireDirection {
    fn message_code(self, operation: IntlHostOp) -> u64 {
        u64::from(operation.code()) * 2
            + match self {
                Self::Request => 0,
                Self::Response => 1,
            }
    }
}

#[cfg(test)]
mod tests;
