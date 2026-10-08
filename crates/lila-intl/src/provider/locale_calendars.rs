//! Locale calendar defaults selected from checked pinned regional preferences.
use super::language_domain::LocaleCanonicalizationData;
use super::region_preference::{region_preference, unicode_keyword, RegionPreferenceError};
use crate::{
    CanonicalLocaleId, DateTimeCalendar, DateTimeFormatError, InvalidLocaleId, LocaleTransformError,
};
use core::fmt;
mod profile;
pub(crate) use profile::LocaleCalendarsProfile;
mod profile_identity;
mod record;
#[cfg(test)]
mod tests;
pub use profile_identity::{LOCALE_CALENDARS_DATA_SHA256, LOCALE_CALENDARS_KERNEL_SHA256};
pub use record::LocaleCalendars;
/// Only defaults reach native code. Every explicit ca slot is handled losslessly in Wasm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleCalendarsRequest(CanonicalLocaleId);
impl LocaleCalendarsRequest {
    pub fn new(locale: CanonicalLocaleId) -> Result<Self, LocaleCalendarsError> {
        if unicode_keyword(locale.as_str(), "ca").is_some() {
            return Err(LocaleCalendarsError::ExplicitCalendar);
        }
        Ok(Self(locale))
    }
    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleCalendarsProfileError {
    Encoding,
    Schema,
    Revision,
    Digest,
    Region,
    Selector,
    Order,
    Calendars,
    Coverage,
}
impl fmt::Display for LocaleCalendarsProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Locale calendar profile admission failed: {self:?}")
    }
}
impl std::error::Error for LocaleCalendarsProfileError {}
#[derive(Debug)]
pub enum LocaleCalendarsError {
    UnavailableService(crate::IntlService),
    ExplicitCalendar,
    Profile(LocaleCalendarsProfileError),
    InvalidLocale(InvalidLocaleId),
    LocaleTransform(LocaleTransformError),
    DateTime(DateTimeFormatError),
}
impl fmt::Display for LocaleCalendarsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::ExplicitCalendar => {
                f.write_str("explicit calendar values must use the compiled Locale slot")
            }
            Self::Profile(e) => e.fmt(f),
            Self::InvalidLocale(e) => e.fmt(f),
            Self::LocaleTransform(e) => e.fmt(f),
            Self::DateTime(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for LocaleCalendarsError {}
impl From<RegionPreferenceError> for LocaleCalendarsError {
    fn from(e: RegionPreferenceError) -> Self {
        match e {
            RegionPreferenceError::InvalidLocale(e) => Self::InvalidLocale(e),
            RegionPreferenceError::LocaleTransform(e) => Self::LocaleTransform(e),
        }
    }
}
pub(crate) fn resolve_locale_calendars(
    request: LocaleCalendarsRequest,
    profile: &LocaleCalendarsProfile,
    authority: &LocaleCanonicalizationData,
    available: &[DateTimeCalendar],
) -> Result<LocaleCalendars, LocaleCalendarsError> {
    let pref = region_preference(request.into_locale(), authority)?;
    // CalendarsOfLocale chooses raw regional preferences before AvailableCalendars filtering.
    let preferences = pref
        .region_override()
        .into_iter()
        .chain(Some(pref.region()))
        .find_map(|region| {
            profile
                .calendars_for_region(pref.language(), region)
                .filter(|names| !names.is_empty())
        })
        .unwrap_or(&[]);
    let mut selected = Vec::new();
    for name in preferences {
        if let Some(calendar) = available
            .iter()
            .copied()
            .find(|calendar| calendar.as_str() == name.as_ref())
        {
            if !selected.contains(&calendar) {
                selected.push(calendar);
            }
        }
    }
    if !selected.is_empty() {
        return LocaleCalendars::checked(selected).map_err(LocaleCalendarsError::Profile);
    }
    if !available.contains(&DateTimeCalendar::Gregorian) {
        return Err(LocaleCalendarsError::DateTime(
            DateTimeFormatError::InvalidProfile("gregory fallback is unavailable".into()),
        ));
    }
    LocaleCalendars::checked(vec![DateTimeCalendar::Gregorian])
        .map_err(LocaleCalendarsError::Profile)
}
