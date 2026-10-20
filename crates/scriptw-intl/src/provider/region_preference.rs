//! Shared canonical RegionPreference; service-specific keywords stay with their owners.

use super::language_domain::{LikelySubtags, LocaleCanonicalizationData};
use crate::{CanonicalLocaleId, InvalidLocaleId, LocaleId, LocaleTransformError};

#[derive(Debug)]
pub(super) enum RegionPreferenceError {
    InvalidLocale(InvalidLocaleId),
    LocaleTransform(LocaleTransformError),
}

pub(super) struct RegionPreference {
    region: Box<str>,
    region_override: Option<Box<str>>,
    locale: CanonicalLocaleId,
}

impl RegionPreference {
    pub(super) fn region(&self) -> &str {
        &self.region
    }
    pub(super) fn region_override(&self) -> Option<&str> {
        self.region_override.as_deref()
    }
    pub(super) fn language(&self) -> &str {
        self.locale
            .as_str()
            .split('-')
            .next()
            .expect("canonical locale language")
    }
    pub(super) fn unicode_keyword(&self, key: &str) -> Option<String> {
        unicode_keyword(self.locale.as_str(), key)
    }
}

fn identifier(locale: &str) -> Result<LocaleId, RegionPreferenceError> {
    LocaleId::parse(locale).map_err(RegionPreferenceError::InvalidLocale)
}

pub(super) fn base_region(locale: &str) -> Option<&str> {
    let mut subtags = locale.split('-');
    subtags.next()?;
    let mut next = subtags.next()?;
    if next.len() == 4 && next.bytes().all(|b| b.is_ascii_alphabetic()) {
        next = subtags.next()?;
    }
    ((next.len() == 2 && next.bytes().all(|b| b.is_ascii_alphabetic()))
        || (next.len() == 3 && next.bytes().all(|b| b.is_ascii_digit())))
    .then_some(next)
}

/// The input has passed genuine locale canonicalization. Keep the entire
/// keyword value, so a multi-subtag value cannot masquerade as a subdivision.
pub(super) fn unicode_keyword(locale: &str, wanted: &str) -> Option<String> {
    let subtags: Vec<_> = locale.split('-').collect();
    let mut unicode_start = None;
    for (index, part) in subtags.iter().enumerate().skip(1) {
        if *part == "x" {
            break;
        }
        if *part == "u" {
            unicode_start = Some(index + 1);
            break;
        }
    }
    let start = unicode_start?;
    let end = subtags[start..]
        .iter()
        .position(|part| part.len() == 1)
        .map_or(subtags.len(), |i| start + i);
    let mut index = start;
    while index < end && subtags[index].len() >= 3 {
        index += 1;
    }
    while index < end {
        let key = subtags[index];
        index += 1;
        let value_start = index;
        while index < end && subtags[index].len() >= 3 {
            index += 1;
        }
        if key == wanted {
            return Some(subtags[value_start..index].join("-"));
        }
    }
    None
}

fn subdivision_region(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    let prefix = if bytes.len() >= 2 && bytes[..2].iter().all(u8::is_ascii_alphabetic) {
        2
    } else if bytes.len() >= 3 && bytes[..3].iter().all(u8::is_ascii_digit) {
        3
    } else {
        return None;
    };
    ((1..=4).contains(&(bytes.len() - prefix))
        && bytes[prefix..].iter().all(u8::is_ascii_alphanumeric))
    .then_some(&value[..prefix])
}

fn canonical_subdivision(
    locale: &str,
    key: &str,
    authority: &LocaleCanonicalizationData,
) -> Result<Option<Box<str>>, RegionPreferenceError> {
    let Some(value) = unicode_keyword(locale, key) else {
        return Ok(None);
    };
    let Some(region) = subdivision_region(&value) else {
        return Ok(None);
    };
    let canonical = authority
        .canonicalize(&identifier(&format!("und-{region}"))?)
        .map_err(RegionPreferenceError::LocaleTransform)?;
    Ok(base_region(canonical.as_str()).map(Into::into))
}

pub(super) fn region_preference(
    locale: CanonicalLocaleId,
    authority: &LocaleCanonicalizationData,
) -> Result<RegionPreference, RegionPreferenceError> {
    let canonical = authority
        .canonicalize(&identifier(locale.as_str())?)
        .map_err(RegionPreferenceError::LocaleTransform)?;
    let tag = canonical.as_str();
    let mut region = base_region(tag).map(Box::<str>::from);
    if region.is_none() {
        region = canonical_subdivision(tag, "sd", authority)?;
        if region.is_none() {
            let maximal =
                match authority.apply_likely_subtags(&identifier(tag)?, LikelySubtags::Maximize) {
                    Ok(maximal) => maximal,
                    // An unsupported likely-subtag lookup retains the original
                    // canonical locale. Corrupt provider output stays a fault.
                    Err(LocaleTransformError::Unsupported(_)) => canonical.clone(),
                    Err(error) => return Err(RegionPreferenceError::LocaleTransform(error)),
                };
            region = base_region(maximal.as_str()).map(Into::into);
        }
    }
    Ok(RegionPreference {
        region: region.unwrap_or_else(|| Box::from("001")),
        region_override: canonical_subdivision(tag, "rg", authority)?,
        locale: canonical,
    })
}
