//! Locale-specific admitted sort collations, independent of Collator formatting defaults.

use core::fmt;

use super::region_preference::unicode_keyword;
use crate::collator::{CollatorOperationError, CollatorProfiles};
use crate::CanonicalLocaleId;

mod kernel_identity;
#[cfg(test)]
mod tests;

pub use kernel_identity::LOCALE_COLLATIONS_KERNEL_SHA256;

/// Every present co value is already owned by the compiled immutable Locale slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCollationsRequest(CanonicalLocaleId);

impl LocaleCollationsRequest {
    pub fn new(locale: CanonicalLocaleId) -> Result<Self, LocaleCollationsError> {
        if unicode_keyword(locale.as_str(), "co").is_some() {
            return Err(LocaleCollationsError::ExplicitCollation);
        }
        Ok(Self(locale))
    }

    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.0
    }
}

/// Sorted unique names become public only through the admitted native profile query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCollations {
    names: Box<[Box<str>]>,
}

impl LocaleCollations {
    #[must_use]
    pub fn names(&self) -> &[Box<str>] {
        &self.names
    }
}

#[derive(Debug, Clone)]
pub enum LocaleCollationsError {
    UnavailableService(crate::IntlService),
    ExplicitCollation,
    Collator(CollatorOperationError),
    InvalidProfile(&'static str),
}

impl fmt::Display for LocaleCollationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::ExplicitCollation => {
                f.write_str("explicit collation values must use the compiled Locale slot")
            }
            Self::Collator(error) => error.fmt(f),
            Self::InvalidProfile(reason) => write!(f, "invalid Locale collation profile: {reason}"),
        }
    }
}
impl std::error::Error for LocaleCollationsError {}
impl From<CollatorOperationError> for LocaleCollationsError {
    fn from(error: CollatorOperationError) -> Self {
        Self::Collator(error)
    }
}

pub(super) fn resolve_locale_collations(
    request: LocaleCollationsRequest,
    profiles: &CollatorProfiles,
) -> Result<LocaleCollations, LocaleCollationsError> {
    let names = profiles.locale_sort_collations(&request.into_locale())?;
    let mut previous: Option<&str> = None;
    for name in &names {
        if matches!(name.as_ref(), "standard" | "search" | "searchjl")
            || name.split('-').any(|part| {
                !(3..=8).contains(&part.len())
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            })
            || previous.is_some_and(|before| before >= name.as_ref())
        {
            return Err(LocaleCollationsError::InvalidProfile(
                "unordered or invalid sort name",
            ));
        }
        previous = Some(name);
    }
    Ok(LocaleCollations { names })
}
