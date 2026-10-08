//! Explicit-base-region time zones from the pinned IANA country membership table.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::named_time_zones::NamedTimeZones;
use super::region_preference::base_region;
use crate::{CanonicalLocaleId, TimeZoneId};

mod kernel_identity;
mod profile_identity;
#[cfg(test)]
mod tests;

pub use kernel_identity::LOCALE_TIME_ZONES_KERNEL_SHA256;
pub use profile_identity::LOCALE_TIME_ZONES_PROFILE_SHA256;

#[cfg(test)]
const ZONE_TAB: &str = include_str!("../../data/locale-time-zones-iana2026a/zone.tab");
#[cfg(test)]
const REGIONS: &str = include_str!("../../data/locale-time-zones-iana2026a/regions.tsv");

/// The native query cannot be constructed without an actual explicit base region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleTimeZonesRequest {
    locale: CanonicalLocaleId,
    region: Box<str>,
}

impl LocaleTimeZonesRequest {
    pub fn new(locale: CanonicalLocaleId) -> Result<Self, LocaleTimeZonesError> {
        // The canonical owner includes reserved five-to-eight-letter languages
        // that ICU's narrower language parser cannot represent. Base-region
        // extraction requires no language matching or region inference.
        let region = base_region(locale.as_str())
            .ok_or(LocaleTimeZonesError::MissingExplicitRegion)?
            .into();
        Ok(Self { locale, region })
    }

    #[must_use]
    pub fn region(&self) -> &str {
        &self.region
    }

    #[must_use]
    pub fn into_locale(self) -> CanonicalLocaleId {
        self.locale
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleTimeZones {
    names: Box<[Box<str>]>,
}

impl LocaleTimeZones {
    #[must_use]
    pub fn names(&self) -> &[Box<str>] {
        &self.names
    }
}

#[derive(Debug, Clone)]
pub enum LocaleTimeZonesError {
    UnavailableService(crate::IntlService),
    MissingExplicitRegion,
    InvalidPinnedData(&'static str),
    Resource(&'static str),
}

impl fmt::Display for LocaleTimeZonesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableService(service) => crate::UnavailableIntlService(*service).fmt(f),
            Self::MissingExplicitRegion => {
                f.write_str("Locale time-zone requests require an explicit base region")
            }
            Self::InvalidPinnedData(reason) => write!(f, "invalid Locale time-zone data: {reason}"),
            Self::Resource(reason) => write!(f, "Locale time-zone resource limit: {reason}"),
        }
    }
}
impl std::error::Error for LocaleTimeZonesError {}

pub(crate) struct LocaleTimeZoneProfiles {
    regions: BTreeMap<Box<str>, Box<[Box<str>]>>,
    named: Arc<NamedTimeZones>,
}

fn source_memberships(
    source: &str,
    named: &NamedTimeZones,
) -> Result<BTreeMap<Box<str>, BTreeSet<Box<str>>>, LocaleTimeZonesError> {
    let mut regions: BTreeMap<Box<str>, BTreeSet<Box<str>>> = BTreeMap::new();
    let mut pairs = BTreeSet::new();
    for line in source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        if !(3..=4).contains(&fields.len())
            || fields[0].len() != 2
            || !fields[0].bytes().all(|byte| byte.is_ascii_uppercase())
            || !line.is_ascii()
        {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "malformed zone.tab row",
            ));
        }
        let identifier = TimeZoneId::parse(fields[2])
            .map_err(|_| LocaleTimeZonesError::InvalidPinnedData("invalid zone.tab identifier"))?;
        let identity = named.lookup(&identifier).map_err(|_| {
            LocaleTimeZonesError::InvalidPinnedData("zone.tab identifier absent from catalogue")
        })?;
        let primary = identity.primary_identifier();
        let primary_identifier = TimeZoneId::parse(primary)
            .map_err(|_| LocaleTimeZonesError::InvalidPinnedData("invalid catalogue primary"))?;
        let terminal = named
            .lookup(&primary_identifier)
            .map_err(|_| LocaleTimeZonesError::InvalidPinnedData("missing catalogue primary"))?;
        if terminal.identifier() != primary || terminal.primary_identifier() != primary {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "non-terminal catalogue primary",
            ));
        }
        if !pairs.insert((fields[0], fields[2])) {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "duplicate zone.tab country row",
            ));
        }
        regions
            .entry(fields[0].into())
            .or_default()
            .insert(primary.into());
    }
    if regions.is_empty() {
        return Err(LocaleTimeZonesError::InvalidPinnedData(
            "empty zone.tab table",
        ));
    }
    Ok(regions)
}

fn projection(
    source: &str,
) -> Result<BTreeMap<Box<str>, BTreeSet<Box<str>>>, LocaleTimeZonesError> {
    let mut regions: BTreeMap<Box<str>, BTreeSet<Box<str>>> = BTreeMap::new();
    let mut previous = None;
    for line in source.lines() {
        let mut fields = line.split('\t');
        let region = fields.next().unwrap_or("");
        let name = fields
            .next()
            .ok_or(LocaleTimeZonesError::InvalidPinnedData(
                "missing projected zone",
            ))?;
        if fields.next().is_some()
            || region.len() != 2
            || !region.bytes().all(|byte| byte.is_ascii_uppercase())
            || name.is_empty()
            || !name.is_ascii()
            || previous.is_some_and(|before| before >= (region, name))
        {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "malformed or unordered region projection",
            ));
        }
        TimeZoneId::parse(name).map_err(|_| {
            LocaleTimeZonesError::InvalidPinnedData("invalid projected zone identifier")
        })?;
        previous = Some((region, name));
        regions
            .entry(region.into())
            .or_default()
            .insert(name.into());
    }
    Ok(regions)
}

impl LocaleTimeZoneProfiles {
    pub(crate) fn from_image_data(
        zone_tab: &str,
        regions: &str,
        named: Arc<NamedTimeZones>,
    ) -> Result<Self, LocaleTimeZonesError> {
        let source_digest: [u8; 32] = Sha256::digest(zone_tab.as_bytes()).into();
        let projection_digest: [u8; 32] = Sha256::digest(regions.as_bytes()).into();
        if source_digest != profile_identity::ZONE_TAB_SHA256
            || projection_digest != LOCALE_TIME_ZONES_PROFILE_SHA256
        {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "pinned country membership digest mismatch",
            ));
        }
        let expected = source_memberships(zone_tab, &named)?;
        let observed = projection(regions)?;
        if expected != observed
            || observed.len() != 247
            || zone_tab
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .count()
                != 418
        {
            return Err(LocaleTimeZonesError::InvalidPinnedData(
                "country membership projection differs from IANA2026a",
            ));
        }
        Ok(Self {
            regions: observed
                .into_iter()
                .map(|(region, names)| (region, names.into_iter().collect()))
                .collect(),
            named,
        })
    }
    pub(crate) fn uses_named_zones(&self, named: &Arc<NamedTimeZones>) -> bool {
        Arc::ptr_eq(&self.named, named)
    }
}

pub(crate) fn resolve_locale_time_zones(
    request: LocaleTimeZonesRequest,
    profiles: &LocaleTimeZoneProfiles,
) -> Result<LocaleTimeZones, LocaleTimeZonesError> {
    let source = profiles
        .regions
        .get(request.region())
        .map_or(&[][..], |names| names.as_ref());
    let mut names = Vec::new();
    names
        .try_reserve_exact(source.len())
        .map_err(|_| LocaleTimeZonesError::Resource("zone list allocation"))?;
    for name in source {
        let mut owned = String::new();
        owned
            .try_reserve_exact(name.len())
            .map_err(|_| LocaleTimeZonesError::Resource("zone name allocation"))?;
        owned.push_str(name);
        names.push(owned.into_boxed_str());
    }
    Ok(LocaleTimeZones {
        names: names.into_boxed_slice(),
    })
}
