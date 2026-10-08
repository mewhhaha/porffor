//! NumberFormat-owned default numbering system for a Locale with no nu slot.

use core::fmt;

use super::region_preference::unicode_keyword;
use crate::number_format::{
    locale_default_numbering_system, DecimalNumberingSystem, NumberFormatKernelError,
    NumberProfiles,
};
use crate::CanonicalLocaleId;

mod kernel_identity;
#[cfg(test)]
mod tests;

pub use kernel_identity::LOCALE_NUMBERING_SYSTEMS_KERNEL_SHA256;

/// Any present nu value, including empty or unknown, belongs to the compiled
/// immutable Locale slot. The native provider receives only absent-nu requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleNumberingSystemsRequest(CanonicalLocaleId);

impl LocaleNumberingSystemsRequest {
    pub fn new(locale: CanonicalLocaleId) -> Result<Self, LocaleNumberingSystemsError> {
        if unicode_keyword(locale.as_str(), "nu").is_some() {
            return Err(LocaleNumberingSystemsError::ExplicitNumberingSystem);
        }
        Ok(Self(locale))
    }

    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.0
    }
}

#[derive(Debug)]
pub enum LocaleNumberingSystemsError {
    UnavailableService(crate::IntlService),
    ExplicitNumberingSystem,
    NumberFormat(NumberFormatKernelError),
}

impl fmt::Display for LocaleNumberingSystemsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::ExplicitNumberingSystem => {
                f.write_str("explicit numbering-system values must use the compiled Locale slot")
            }
            Self::NumberFormat(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for LocaleNumberingSystemsError {}

impl From<NumberFormatKernelError> for LocaleNumberingSystemsError {
    fn from(error: NumberFormatKernelError) -> Self {
        Self::NumberFormat(error)
    }
}

pub(super) fn resolve_locale_numbering_systems(
    request: LocaleNumberingSystemsRequest,
    profiles: &NumberProfiles,
) -> Result<DecimalNumberingSystem, LocaleNumberingSystemsError> {
    Ok(locale_default_numbering_system(request, profiles)?)
}
