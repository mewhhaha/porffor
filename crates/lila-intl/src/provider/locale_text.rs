//! Pure pinned Locale text direction. JavaScript objects remain in Wasm.

mod profile;
pub(crate) use profile::LocaleTextProfile;
mod profile_identity;
mod record;
mod script;

pub use profile_identity::{LOCALE_TEXT_DATA_SHA256, LOCALE_TEXT_KERNEL_SHA256};
pub use record::{LocaleTextDirection, LocaleTextInfoRequest};

use core::fmt;

use super::language_domain::LocaleCanonicalizationData;
use crate::{InvalidLocaleId, LocaleTransformError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleTextProfileError {
    Encoding,
    Schema,
    Revision,
    Digest,
    Script,
    ScriptOrder,
    Coverage,
}

#[derive(Debug)]
pub enum LocaleTextError {
    UnavailableService(crate::IntlService),
    Profile(LocaleTextProfileError),
    InvalidLocale(InvalidLocaleId),
    LocaleTransform(LocaleTransformError),
}

impl fmt::Display for LocaleTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::Profile(error) => write!(f, "invalid pinned Locale text profile: {error:?}"),
            Self::InvalidLocale(error) => error.fmt(f),
            Self::LocaleTransform(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LocaleTextError {}

pub(crate) fn resolve_locale_text(
    request: LocaleTextInfoRequest,
    profile: &LocaleTextProfile,
    authority: &LocaleCanonicalizationData,
) -> Result<Option<LocaleTextDirection>, LocaleTextError> {
    let script = script::locale_script(request.into_locale(), authority)?;
    Ok(script
        .as_deref()
        .and_then(|script| profile.direction(script)))
}

#[cfg(test)]
mod tests;
