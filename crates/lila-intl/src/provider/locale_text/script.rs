use super::super::language_domain::{LikelySubtags, LocaleCanonicalizationData};
use super::LocaleTextError;
use crate::{CanonicalLocaleId, LocaleId, LocaleTransformError};

fn identifier(locale: &str) -> Result<LocaleId, LocaleTextError> {
    LocaleId::parse(locale).map_err(LocaleTextError::InvalidLocale)
}

/// In a checked base identifier the script can only immediately follow language.
/// Extensions, transform-language scripts and private-use text cannot enter here.
fn base_script(locale: &str) -> Option<&str> {
    let candidate = locale.split('-').nth(1)?;
    (candidate.len() == 4 && candidate.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then_some(candidate)
}

pub(super) fn locale_script(
    locale: CanonicalLocaleId,
    authority: &LocaleCanonicalizationData,
) -> Result<Option<Box<str>>, LocaleTextError> {
    let canonical = authority
        .canonicalize(&identifier(locale.as_str())?)
        .map_err(LocaleTextError::LocaleTransform)?;
    // An explicit unknown script must remain unknown; maximization is permitted
    // only when the canonical base identifier has no script at all.
    if let Some(script) = base_script(canonical.as_str()) {
        return Ok(Some(script.into()));
    }
    let maximal = match authority
        .apply_likely_subtags(&identifier(canonical.as_str())?, LikelySubtags::Maximize)
    {
        Ok(maximal) => maximal,
        Err(LocaleTransformError::Unsupported(_)) => return Ok(None),
        Err(error) => return Err(LocaleTextError::LocaleTransform(error)),
    };
    Ok(base_script(maximal.as_str()).map(Into::into))
}
