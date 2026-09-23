//! Provider-validated locale selection and NumberFormat configuration.

use super::options::{LocaleMatcher, NumberFormatOptions};
use super::partition_resource::{
    owned_text, NumberFormatKernelError, NumberPartitionResourceError, PartitionLimits,
};
use super::profiles::NumberProfiles;
use crate::CanonicalLocaleId;
use core::fmt;

/// Canonical BCP47 nu spelling, selected from one validated number profile.
/// Construction belongs to the generated locale/numbering-system resolver.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecimalNumberingSystem(Box<str>);
impl DecimalNumberingSystem {
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Public resolved locale and internal formatting locale are deliberately
/// separate: Unicode nu may survive in the former, never in the data key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedNumberLocale {
    resolved: CanonicalLocaleId,
    formatting: CanonicalLocaleId,
    numbering_system: DecimalNumberingSystem,
}
impl ResolvedNumberLocale {
    pub fn resolved(&self) -> &CanonicalLocaleId {
        &self.resolved
    }
    pub fn formatting(&self) -> &CanonicalLocaleId {
        &self.formatting
    }
    pub fn numbering_system(&self) -> &DecimalNumberingSystem {
        &self.numbering_system
    }
}

/// A well-formed Unicode type which may be unsupported by this provider.
/// Its syntax/casing/keyword-alias construction belongs to locale negotiation,
/// before style is observed; unsupported names resolve normally to defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberingSystemOption(Box<str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberLocaleRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
    pub numbering_system: Option<NumberingSystemOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberSupportedLocalesRequest {
    pub requested: Box<[CanonicalLocaleId]>,
    pub matcher: LocaleMatcher,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberFormatConfiguration {
    pub locale: ResolvedNumberLocale,
    pub options: NumberFormatOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidNumberingSystemOption {
    Syntax,
    Allocation,
}

impl fmt::Display for InvalidNumberingSystemOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => {
                formatter.write_str("numberingSystem must be a Unicode locale identifier type")
            }
            Self::Allocation => formatter.write_str("numberingSystem allocation failed"),
        }
    }
}

impl std::error::Error for InvalidNumberingSystemOption {}

impl NumberingSystemOption {
    pub fn parse(source: &str) -> Result<Self, InvalidNumberingSystemOption> {
        if !source.split('-').all(|part| {
            (3..=8).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        }) {
            return Err(InvalidNumberingSystemOption::Syntax);
        }
        let mut canonical = String::new();
        canonical
            .try_reserve_exact(source.len())
            .map_err(|_| InvalidNumberingSystemOption::Allocation)?;
        canonical.extend(
            source
                .bytes()
                .map(|byte| char::from(byte.to_ascii_lowercase())),
        );
        Ok(Self(canonical.into_boxed_str()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl ResolvedNumberLocale {
    pub fn from_resolved(
        resolved: CanonicalLocaleId,
        formatting: CanonicalLocaleId,
        numbering_system: &str,
        profiles: &NumberProfiles,
    ) -> Result<Self, NumberFormatKernelError> {
        if profiles.profile(formatting.as_str()).is_none()
            || profiles.system(numbering_system).is_none()
        {
            return Err(NumberFormatKernelError::InvalidResolvedLocale);
        }
        let resolved_text = resolved.as_str();
        let base = formatting.as_str();
        let relationship = resolved_text == base
            || resolved_text
                .strip_prefix(base)
                .and_then(|suffix| suffix.strip_prefix("-u-nu-"))
                == Some(numbering_system);
        if !relationship {
            return Err(NumberFormatKernelError::InvalidResolvedLocale);
        }
        Ok(Self {
            resolved,
            formatting,
            numbering_system: DecimalNumberingSystem(owned_text(
                numbering_system,
                &PartitionLimits::HOST_ABI,
            )?),
        })
    }
}

fn canonical_copy(source: &str) -> Result<CanonicalLocaleId, NumberFormatKernelError> {
    CanonicalLocaleId::from_data(owned_text(source, &PartitionLimits::HOST_ABI)?)
        .map_err(|_| NumberFormatKernelError::InvalidResolvedLocale)
}

fn matching_locale<'a>(
    requested: &str,
    matcher: LocaleMatcher,
    profiles: &'a NumberProfiles,
) -> Option<&'a str> {
    // Best fit uses the same permitted prefix policy as lookup. The inventory
    // remains complete; this only searches a request's locale fallback chain.
    match matcher {
        LocaleMatcher::Lookup | LocaleMatcher::BestFit => {}
    }
    let mut candidate = requested;
    while !candidate.is_empty() {
        if let Ok(index) = profiles.available_locales().binary_search(&candidate) {
            return Some(profiles.available_locales()[index]);
        }
        let position = candidate.rfind('-')?;
        candidate = &candidate[..position];
        if candidate
            .rsplit('-')
            .next()
            .is_some_and(|part| part.len() == 1)
        {
            let position = candidate.rfind('-')?;
            candidate = &candidate[..position];
        }
    }
    None
}

fn unicode_numbering_system(requested: &str) -> Option<&str> {
    let mut parts = requested.split('-');
    while let Some(part) = parts.next() {
        if part == "x" {
            return None;
        }
        if part != "u" {
            continue;
        }
        while let Some(part) = parts.next() {
            if part.len() == 1 {
                return None;
            }
            if part != "nu" {
                continue;
            }
            let value = parts.next()?;
            if value.len() < 3 {
                return None;
            }
            let start = value.as_ptr() as usize - requested.as_ptr() as usize;
            let mut end = start + value.len();
            for following in parts {
                if following.len() < 3 {
                    break;
                }
                end += 1 + following.len();
            }
            return Some(&requested[start..end]);
        }
        return None;
    }
    None
}

pub fn resolve_number_locale(
    request: &NumberLocaleRequest,
    profiles: &NumberProfiles,
) -> Result<ResolvedNumberLocale, NumberFormatKernelError> {
    let selected = request.requested.iter().find_map(|requested| {
        matching_locale(requested.as_str(), request.matcher, profiles)
            .map(|locale| (locale, unicode_numbering_system(requested.as_str())))
    });
    let (formatting, extension) = selected.unwrap_or(("en-US", None));
    let profile = profiles
        .profile(formatting)
        .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
    let mut numbering = profiles.numbering_systems()[usize::from(profile.default_numbering)];
    let mut retained = None;
    if let Some(extension) = extension.filter(|name| profiles.system(name).is_some()) {
        numbering = extension;
        retained = Some(extension);
    }
    if let Some(option) = request
        .numbering_system
        .as_ref()
        .filter(|option| profiles.system(option.name()).is_some())
    {
        if option.name() != numbering {
            retained = None;
        }
        numbering = option.name();
    }
    let mut resolved = String::new();
    let extent = formatting
        .len()
        .checked_add(retained.map_or(0, |name| name.len() + 6))
        .ok_or(NumberPartitionResourceError::Allocation)?;
    resolved
        .try_reserve_exact(extent)
        .map_err(|_| NumberPartitionResourceError::Allocation)?;
    resolved.push_str(formatting);
    if let Some(retained) = retained {
        resolved.push_str("-u-nu-");
        resolved.push_str(retained);
    }
    ResolvedNumberLocale::from_resolved(
        CanonicalLocaleId::from_data(resolved.into_boxed_str())
            .map_err(|_| NumberFormatKernelError::InvalidResolvedLocale)?,
        canonical_copy(formatting)?,
        numbering,
        profiles,
    )
}

pub fn filter_number_locales(
    request: &NumberSupportedLocalesRequest,
    profiles: &NumberProfiles,
) -> Result<Box<[CanonicalLocaleId]>, NumberFormatKernelError> {
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(request.requested.len())
        .map_err(|_| NumberPartitionResourceError::Allocation)?;
    for requested in &request.requested {
        if matching_locale(requested.as_str(), request.matcher, profiles).is_some() {
            selected.push(canonical_copy(requested.as_str())?);
        }
    }
    Ok(selected.into_boxed_slice())
}
