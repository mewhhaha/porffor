//! Checked DurationFormat frames with real native/global/host registration.
//! Complete AOT/product verification remains mandatory.
use crate::duration_format::{
    CheckedDurationConfiguration, DurationDisplay, DurationError, DurationFractionalDigits,
    DurationOptions, DurationPartition, DurationProfiles, DurationRecord, DurationStyle,
    DurationSupportedLocalesRequest, DurationUnit, DurationUnitOption, DurationUnitStyle,
    ResolvedDurationLocale,
};
use crate::duration_protocol::{DurationHostOp, DurationRequest, DurationResponse};
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{
    NumberLocaleRequest, NumberPart, NumberPartKind, NumberProfiles, NumberingSystemOption,
    PartitionLimits,
};
use crate::CanonicalLocaleId;
use core::fmt;
use std::sync::Arc;

mod configuration;
mod framing;
mod requests;
mod responses;
#[cfg(test)]
mod tests;
pub use configuration::{DurationConfigurationWord, DURATION_CONFIGURATION_WORDS};
use framing::{Reader, Writer};
pub use requests::{decode_duration_request, encode_duration_request};
pub use responses::{decode_duration_response, encode_duration_response};

pub const DURATION_WIRE_VERSION: u64 = 1;
pub const DURATION_HEADER_BYTES: u64 = 16;
pub const DURATION_RECORD_WORDS: usize = 10;
const _: () = assert!(DURATION_WIRE_VERSION == crate::number_protocol::NUMBER_WIRE_VERSION);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationWireError {
    Malformed(&'static str),
    Rejected(DurationError),
    Resource(&'static str),
}
impl From<DurationError> for DurationWireError {
    fn from(error: DurationError) -> Self {
        Self::Rejected(error)
    }
}
impl fmt::Display for DurationWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "invalid DurationFormat frame: {reason}"),
            Self::Rejected(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "DurationFormat frame resource limit: {reason}"),
        }
    }
}
impl std::error::Error for DurationWireError {}

/// Only a completed primitive field sequence may construct this owner. AOT
/// performs each Get/ToNumber/integrality check before reading the next field.
#[derive(Debug, Clone)]
pub struct DurationPartitionRequest {
    configuration: CheckedDurationConfiguration,
    record: DurationRecord,
    number_bits: [u64; DURATION_RECORD_WORDS],
}
impl DurationPartitionRequest {
    pub fn from_completed_number_fields(
        configuration: CheckedDurationConfiguration,
        fields: [f64; DURATION_RECORD_WORDS],
    ) -> Result<Self, DurationError> {
        let record = DurationRecord::from_number_fields(fields)?;
        Ok(Self {
            configuration,
            record,
            number_bits: fields.map(f64::to_bits),
        })
    }
    pub fn configuration(&self) -> &CheckedDurationConfiguration {
        &self.configuration
    }
    pub fn record(&self) -> &DurationRecord {
        &self.record
    }
    pub const fn number_bits(&self) -> &[u64; DURATION_RECORD_WORDS] {
        &self.number_bits
    }
}

#[derive(Debug, Clone)]
pub enum DurationWireRequest {
    Resolve(NumberLocaleRequest),
    SupportedLocales(DurationSupportedLocalesRequest),
    Parts(DurationPartitionRequest),
}
impl DurationWireRequest {
    pub const fn operation(&self) -> DurationHostOp {
        match self {
            Self::Resolve(_) => DurationHostOp::Resolve,
            Self::SupportedLocales(_) => DurationHostOp::SupportedLocales,
            Self::Parts(_) => DurationHostOp::Parts,
        }
    }
    /// Transfers the checked owners; it never reconstructs/revalidates a record.
    pub fn into_native(self) -> DurationRequest {
        match self {
            Self::Resolve(request) => DurationRequest::Resolve(request),
            Self::SupportedLocales(request) => DurationRequest::SupportedLocales(request),
            Self::Parts(request) => DurationRequest::Parts {
                configuration: request.configuration,
                record: request.record,
            },
        }
    }
}

/// Wire output owns localized text unchanged. Unitless parts are list/digital
/// literals; labeled parts keep their source NumberFormat kind and unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationWirePart {
    part: NumberPart,
    unit: Option<DurationUnit>,
}
impl DurationWirePart {
    pub const fn kind(&self) -> NumberPartKind {
        self.part.kind()
    }
    pub fn text(&self) -> &str {
        self.part.text()
    }
    pub const fn unit(&self) -> Option<DurationUnit> {
        self.unit
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationWirePartition {
    parts: Box<[DurationWirePart]>,
    text_bytes: usize,
}
impl DurationWirePartition {
    pub fn parts(&self) -> &[DurationWirePart] {
        &self.parts
    }
    pub fn to_text(&self) -> Result<String, DurationWireError> {
        let mut text = String::new();
        text.try_reserve_exact(self.text_bytes)
            .map_err(|_| DurationWireError::Resource("text allocation"))?;
        for part in &self.parts {
            text.push_str(part.text());
        }
        Ok(text)
    }
}
#[derive(Debug, Clone)]
pub enum DurationWireResponse {
    Resolved(ResolvedDurationLocale),
    SupportedLocales(Box<[CanonicalLocaleId]>),
    Parts(DurationWirePartition),
}
