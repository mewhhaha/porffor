//! Pure image-backed Segmenter foundation consumed by the AOT host boundary.
//! JavaScript observations, Segments objects, iterator state and realms belong
//! to the emitted caller.

use crate::number_format::options::LocaleMatcher;
use crate::CanonicalLocaleId;
use core::fmt;
use std::sync::Arc;

mod boundaries;
mod configuration;
mod kernel_identity;
mod profiles;
#[cfg(test)]
mod tests;
pub use boundaries::{segment_utf16, SegmentBoundary, SegmenterResult};
pub use configuration::{CheckedSegmenterConfiguration, SegmentUtf16Request};
pub use kernel_identity::SEGMENTER_KERNEL_SHA256;
pub use profiles::{SegmenterProfiles, SEGMENTER_DATA_SHA256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmenterGranularity {
    Grapheme,
    Word,
    Sentence,
}
impl SegmenterGranularity {
    pub const ALL: [Self; 3] = [Self::Grapheme, Self::Word, Self::Sentence];
    pub const OPTIONS: [(&'static str, i64); 3] = [("grapheme", 1), ("word", 2), ("sentence", 3)];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Grapheme => "grapheme",
            Self::Word => "word",
            Self::Sentence => "sentence",
        }
    }
    pub const fn wire_code(self) -> u64 {
        match self {
            Self::Grapheme => 1,
            Self::Word => 2,
            Self::Sentence => 3,
        }
    }
    pub const fn from_wire_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::Grapheme),
            2 => Some(Self::Word),
            3 => Some(Self::Sentence),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmenterLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmenterSupportedLocalesResult {
    pub locales: Box<[CanonicalLocaleId]>,
}
/// Admission into the finite checked catalogue is the only minting path.
#[derive(Debug, Clone)]
pub struct ResolvedSegmenterLocale {
    profile: Arc<profiles::LocaleProfile>,
}
impl ResolvedSegmenterLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        &self.profile.locale
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmenterError {
    UnavailableService(crate::IntlService),
    InvalidResolvedLocale,
    InvalidData(Box<str>),
    InvalidBoundaries(&'static str),
    Resource(&'static str),
}
impl fmt::Display for SegmenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::InvalidResolvedLocale => f.write_str("unadmitted Segmenter locale"),
            Self::InvalidData(reason) => write!(f, "invalid Segmenter data: {reason}"),
            Self::InvalidBoundaries(reason) => write!(f, "invalid Segmenter boundaries: {reason}"),
            Self::Resource(reason) => write!(f, "Segmenter resource limit: {reason}"),
        }
    }
}
impl std::error::Error for SegmenterError {}
fn data_error(error: impl fmt::Display) -> SegmenterError {
    SegmenterError::InvalidData(error.to_string().into_boxed_str())
}

/// The complete model certificate and immutable locale associations are admitted
/// once and reused by the embedded provider and every host request.
pub fn embedded_segmenter_profiles() -> Result<&'static SegmenterProfiles, SegmenterError> {
    crate::segmenter_image::embedded_segmenter_data_image_ref()
        .map(|image| image.profiles_ref())
        .map_err(data_error)
}
