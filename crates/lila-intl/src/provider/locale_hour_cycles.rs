use core::fmt;

use super::language_domain::LocaleCanonicalizationData;
use super::region_preference::{region_preference, unicode_keyword, RegionPreferenceError};
use crate::{CanonicalLocaleId, DateTimeHourCycle, InvalidLocaleId, LocaleTransformError};

mod profile;
pub(crate) use profile::LocaleHourCyclesProfile;
mod profile_identity;
mod record;
#[cfg(test)]
mod tests;

pub use profile_identity::{LOCALE_HOUR_CYCLES_DATA_SHA256, LOCALE_HOUR_CYCLES_KERNEL_SHA256};
pub use record::LocaleHourCycles;

/// A canonical locale with no explicit hour-cycle keyword. Explicit values,
/// including unknown or empty ones, remain owned by the compiled Locale slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleHourCyclesRequest(CanonicalLocaleId);

impl LocaleHourCyclesRequest {
    pub fn new(locale: CanonicalLocaleId) -> Result<Self, LocaleHourCyclesError> {
        if unicode_keyword(locale.as_str(), "hc").is_some() {
            return Err(LocaleHourCyclesError::ExplicitHourCycle);
        }
        Ok(Self(locale))
    }
    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleHourCyclesProfileError {
    Encoding,
    Schema,
    Revision,
    Digest,
    Region,
    Selector,
    Order,
    Cycles,
    Coverage,
}

impl fmt::Display for LocaleHourCyclesProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Locale hour-cycle profile admission failed: {self:?}")
    }
}
impl std::error::Error for LocaleHourCyclesProfileError {}

#[derive(Debug)]
pub enum LocaleHourCyclesError {
    UnavailableService(crate::IntlService),
    ExplicitHourCycle,
    Profile(LocaleHourCyclesProfileError),
    InvalidLocale(InvalidLocaleId),
    LocaleTransform(LocaleTransformError),
}

impl fmt::Display for LocaleHourCyclesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::ExplicitHourCycle => {
                f.write_str("explicit hour-cycle values must use the compiled Locale slot")
            }
            Self::Profile(error) => error.fmt(f),
            Self::InvalidLocale(error) => error.fmt(f),
            Self::LocaleTransform(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for LocaleHourCyclesError {}

impl From<RegionPreferenceError> for LocaleHourCyclesError {
    fn from(error: RegionPreferenceError) -> Self {
        match error {
            RegionPreferenceError::InvalidLocale(error) => Self::InvalidLocale(error),
            RegionPreferenceError::LocaleTransform(error) => Self::LocaleTransform(error),
        }
    }
}

pub(crate) fn resolve_locale_hour_cycles(
    request: LocaleHourCyclesRequest,
    profile: &LocaleHourCyclesProfile,
    authority: &LocaleCanonicalizationData,
) -> Result<LocaleHourCycles, LocaleHourCyclesError> {
    let preference = region_preference(request.into_locale(), authority)?;
    for region in preference
        .region_override()
        .into_iter()
        .chain(Some(preference.region()))
    {
        if let Some(cycles) = profile.cycles_for_region(preference.language(), region) {
            return Ok(cycles.clone());
        }
    }
    Ok(LocaleHourCycles::single(DateTimeHourCycle::H23))
}
