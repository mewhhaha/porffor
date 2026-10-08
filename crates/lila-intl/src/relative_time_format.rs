//! Data-backed RelativeTimeFormat primitives. JavaScript observation stays in Wasm.
use core::fmt;

use crate::number_format::numeric::NumericNormalizationError;
use crate::number_format::{NumberFormatKernelError, PartitionLimits};
use crate::plural_rules::PluralRulesOperationError;

mod configuration;
mod kernel_identity;
pub(crate) use kernel_identity::RELATIVE_TIME_KERNEL_SHA256;
mod domains;
mod partition;
mod profiles;
mod raw;

pub use configuration::{RelativeTimeConfiguration, ResolvedRelativeTimeLocale};
pub use domains::{FiniteRelativeNumber, RelativeNumeric, RelativeStyle, RelativeUnit};
pub use partition::{RelativePart, RelativePartition};
pub use profiles::{embedded_relative_profiles, RelativeProfiles};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeTimeError {
    UnavailableService(crate::IntlService),
    NonFinite,
    InvalidProfile(&'static str),
    InvalidLocale,
    Number(NumberFormatKernelError),
    Numeric(NumericNormalizationError),
    Plural(PluralRulesOperationError),
    Resource(&'static str),
}
impl fmt::Display for RelativeTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::NonFinite => f.write_str("RelativeTimeFormat value must be finite"),
            Self::InvalidProfile(reason) => write!(f, "invalid relative-time profile: {reason}"),
            Self::InvalidLocale => f.write_str("unadmitted relative-time locale"),
            Self::Number(error) => error.fmt(f),
            Self::Numeric(error) => error.fmt(f),
            Self::Plural(error) => error.fmt(f),
            Self::Resource(reason) => write!(f, "relative-time resource limit: {reason}"),
        }
    }
}
impl std::error::Error for RelativeTimeError {}
impl From<NumberFormatKernelError> for RelativeTimeError {
    fn from(error: NumberFormatKernelError) -> Self {
        Self::Number(error)
    }
}
impl From<NumericNormalizationError> for RelativeTimeError {
    fn from(error: NumericNormalizationError) -> Self {
        Self::Numeric(error)
    }
}
impl From<PluralRulesOperationError> for RelativeTimeError {
    fn from(error: PluralRulesOperationError) -> Self {
        Self::Plural(error)
    }
}

/// A primitive request can only hold finite input and the checked constructor state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatRelativeTimePartsRequest {
    configuration: RelativeTimeConfiguration,
    value: FiniteRelativeNumber,
    unit: RelativeUnit,
}
impl FormatRelativeTimePartsRequest {
    pub const fn new(
        configuration: RelativeTimeConfiguration,
        value: FiniteRelativeNumber,
        unit: RelativeUnit,
    ) -> Self {
        Self {
            configuration,
            value,
            unit,
        }
    }
    pub fn configuration(&self) -> &RelativeTimeConfiguration {
        &self.configuration
    }
    pub const fn value(&self) -> FiniteRelativeNumber {
        self.value
    }
    pub const fn unit(&self) -> RelativeUnit {
        self.unit
    }
}

/// The primitive host operation accepts only the checked constructor state.
pub fn format_relative_time_parts(
    configuration: &RelativeTimeConfiguration,
    value: FiniteRelativeNumber,
    unit: RelativeUnit,
    profiles: &RelativeProfiles,
    limits: &PartitionLimits,
) -> Result<RelativePartition, RelativeTimeError> {
    profiles.format_parts(configuration, value, unit, limits)
}

#[cfg(test)]
mod tests;
