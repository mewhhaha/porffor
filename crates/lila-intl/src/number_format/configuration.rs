//! Provider-validated locale selection and NumberFormat configuration.

use super::options::{LocaleMatcher, NumberFormatOptions};
use super::partition_resource::{
    owned_text, NumberFormatKernelError, NumberPartitionResourceError, PartitionLimits,
};
use super::profiles::{NumberLocaleView, NumberProfiles};
use crate::CanonicalLocaleId;
use core::{
    fmt,
    hash::{Hash, Hasher},
};
use std::sync::Arc;

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
#[derive(Clone)]
pub struct ResolvedNumberLocale {
    resolved: CanonicalLocaleId,
    formatting: CanonicalLocaleId,
    numbering_system: DecimalNumberingSystem,
    profiles: Arc<NumberProfiles>,
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
        profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, NumberFormatKernelError> {
        Self::from_resolved_in(
            resolved,
            formatting,
            numbering_system,
            NumberLocaleView::public(profiles),
        )
    }

    pub(crate) fn from_resolved_in(
        resolved: CanonicalLocaleId,
        formatting: CanonicalLocaleId,
        numbering_system: &str,
        view: NumberLocaleView<'_>,
    ) -> Result<Self, NumberFormatKernelError> {
        let profiles = view.owner();
        if !view.contains(formatting.as_str())
            || profiles.profile(formatting.as_str()).is_none()
            || !profiles.supports_numbering_system(formatting.as_str(), numbering_system)
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
            profiles: Arc::clone(profiles),
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

pub(crate) fn matching_locale<'a>(
    requested: &str,
    matcher: LocaleMatcher,
    profiles: &'a NumberProfiles,
) -> Option<&'a str> {
    matching_locale_in(requested, matcher, profiles.available_locales(), profiles)
}

fn matching_locale_in<'a>(
    requested: &str,
    matcher: LocaleMatcher,
    locales: &'a [Box<str>],
    profiles: &NumberProfiles,
) -> Option<&'a str> {
    let mut lookup = |candidate: &str| {
        locales
            .binary_search_by(|name| name.as_ref().cmp(candidate))
            .ok()
            .map(|index| locales[index].as_ref())
    };
    match matcher {
        LocaleMatcher::Lookup => matching_locale_by(requested, lookup),
        LocaleMatcher::BestFit => {
            let mut locale: icu_locale::Locale = requested.parse().ok()?;
            let base = locale.id.to_string();
            if let Some(exact) = lookup(&base) {
                return Some(exact);
            }
            // Retain explicit exact associations. Otherwise let the selected
            // Locale authority infer missing script/region before falling back
            // to a language-only row (zh-TW must reach zh-Hant-TW, not zh).
            let fallback = matching_locale_by(&base, &mut lookup);
            profiles.maximize_for_matching(&mut locale.id);
            let maximized = matching_locale_by(&locale.id.to_string(), lookup);
            match (maximized, fallback) {
                (Some(maximized), Some(fallback)) => {
                    if maximized.split('-').count() > fallback.split('-').count() {
                        Some(maximized)
                    } else {
                        Some(fallback)
                    }
                }
                (maximized, fallback) => maximized.or(fallback),
            }
        }
    }
}

fn matching_locale_by<T>(requested: &str, mut lookup: impl FnMut(&str) -> Option<T>) -> Option<T> {
    // The inventory belongs to the admitted public/dependent view. Neither
    // lookup nor likely-subtag matching can expose a private physical row.
    let mut candidate = requested;
    while !candidate.is_empty() {
        if let Some(found) = lookup(candidate) {
            return Some(found);
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

/// The Locale method's absent-nu default uses NumberFormat's prefix inventory,
/// without service option resolution, likely subtags or a default-locale choice.
pub(crate) fn locale_default_numbering_system(
    request: crate::LocaleNumberingSystemsRequest,
    profiles: &NumberProfiles,
) -> Result<DecimalNumberingSystem, NumberFormatKernelError> {
    let locale = request.into_locale();
    let name = matching_locale_by(locale.as_str(), |candidate| {
        profiles.default_numbering_index(candidate)
    })
    .map_or("latn", |index| {
        profiles.numbering_systems()[usize::from(index)].as_ref()
    });
    Ok(DecimalNumberingSystem(owned_text(
        name,
        &PartitionLimits::HOST_ABI,
    )?))
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
    profiles: &Arc<NumberProfiles>,
) -> Result<ResolvedNumberLocale, NumberFormatKernelError> {
    resolve_number_locale_in(request, NumberLocaleView::public(profiles))
}

pub(crate) fn resolve_number_locale_in(
    request: &NumberLocaleRequest,
    view: NumberLocaleView<'_>,
) -> Result<ResolvedNumberLocale, NumberFormatKernelError> {
    let profiles = view.owner();
    let selected = request.requested.iter().find_map(|requested| {
        matching_locale_in(
            requested.as_str(),
            request.matcher,
            view.locales(),
            profiles,
        )
        .map(|locale| (locale, unicode_numbering_system(requested.as_str())))
    });
    let (formatting, extension) = selected.unwrap_or(("en-US", None));
    let profile = profiles
        .profile(formatting)
        .ok_or(NumberFormatKernelError::InvalidResolvedLocale)?;
    let mut numbering =
        profiles.numbering_systems()[usize::from(profile.default_numbering)].as_ref();
    let mut retained = None;
    if let Some(extension) =
        extension.filter(|name| profiles.supports_numbering_system(formatting, name))
    {
        numbering = extension;
        retained = Some(extension);
    }
    if let Some(option) = request
        .numbering_system
        .as_ref()
        .filter(|option| profiles.supports_numbering_system(formatting, option.name()))
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
    ResolvedNumberLocale::from_resolved_in(
        CanonicalLocaleId::from_data(resolved.into_boxed_str())
            .map_err(|_| NumberFormatKernelError::InvalidResolvedLocale)?,
        canonical_copy(formatting)?,
        numbering,
        view,
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

impl ResolvedNumberLocale {
    pub(crate) fn ensure_profiles(
        &self,
        profiles: &Arc<NumberProfiles>,
    ) -> Result<(), NumberFormatKernelError> {
        if !Arc::ptr_eq(&self.profiles, profiles) {
            return Err(NumberFormatKernelError::InvalidResolvedLocale);
        }
        Ok(())
    }
}
impl fmt::Debug for ResolvedNumberLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedNumberLocale")
            .field("resolved", &self.resolved)
            .field("formatting", &self.formatting)
            .field("numbering_system", &self.numbering_system)
            .finish_non_exhaustive()
    }
}
impl PartialEq for ResolvedNumberLocale {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.profiles, &other.profiles)
            && self.resolved == other.resolved
            && self.formatting == other.formatting
            && self.numbering_system == other.numbering_system
    }
}
impl Eq for ResolvedNumberLocale {}
impl Hash for ResolvedNumberLocale {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.profiles).hash(state);
        self.resolved.hash(state);
        self.formatting.hash(state);
        self.numbering_system.hash(state);
    }
}
