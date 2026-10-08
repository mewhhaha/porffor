//! Pinned LDML47 names over exact UTC metazone periods and one IANA snapshot.

use super::NamedTimeZones;
use crate::{
    CanonicalLocaleId, InvalidTimeZoneData, ResolveTimeZoneRequest, ResolvedTimeZoneSnapshot,
    TimeZoneNameStyle, TimeZoneResolveError, TimeZoneSelection,
};
use std::sync::Arc;

use super::time_zone_snapshot::{StandardTimeStability, TimeZoneNameInput, TimeZoneVariant};

mod raw;
#[cfg(test)]
mod tests;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = [
    0x52, 0x50, 0x8d, 0xfd, 0x4f, 0x0b, 0xca, 0x47, 0xb3, 0x94, 0x43, 0x58, 0x9c, 0x97, 0xe9, 0x59,
    0x36, 0x1f, 0x1f, 0x57, 0xd7, 0x32, 0x48, 0x76, 0xf0, 0xb6, 0x81, 0xb0, 0x50, 0x90, 0x12, 0xf4,
];

#[derive(Clone, Copy)]
enum NameWidth {
    Short,
    Long,
}

#[derive(Clone, Copy)]
enum NameKind {
    Generic,
    Standard,
    Daylight,
}

struct NameVariants {
    generic: Option<Box<str>>,
    standard: Option<Box<str>>,
    daylight: Option<Box<str>>,
}

impl NameVariants {
    fn select(
        &self,
        kind: NameKind,
        has_daylight_name: bool,
        stability: StandardTimeStability,
    ) -> Option<&str> {
        let requested = match kind {
            NameKind::Generic => self.generic.as_deref(),
            NameKind::Standard => self.standard.as_deref(),
            NameKind::Daylight => self.daylight.as_deref(),
        };
        requested.or_else(|| {
            if !has_daylight_name {
                return self.generic.as_deref().or(self.standard.as_deref());
            }
            match (kind, stability) {
                (NameKind::Generic, StandardTimeStability::Stable) => self.standard.as_deref(),
                (NameKind::Generic, StandardTimeStability::NotProven)
                | (NameKind::Standard | NameKind::Daylight, _) => None,
            }
        })
    }

    fn valid(&self) -> bool {
        [
            self.generic.as_deref(),
            self.standard.as_deref(),
            self.daylight.as_deref(),
        ]
        .into_iter()
        .flatten()
        .all(|name| !name.is_empty())
    }
}

struct WidthNames {
    short: NameVariants,
    long: NameVariants,
}

impl WidthNames {
    fn get(&self, width: NameWidth) -> &NameVariants {
        match width {
            NameWidth::Short => &self.short,
            NameWidth::Long => &self.long,
        }
    }

    fn valid(&self) -> bool {
        self.short.valid() && self.long.valid()
    }

    fn has_daylight(&self) -> bool {
        self.short.daylight.is_some() || self.long.daylight.is_some()
    }
}

struct Metazone {
    identifier: Box<str>,
    names: WidthNames,
    preferred: Vec<(Box<str>, Box<str>)>,
}

impl Metazone {
    fn preferred(&self, territory: &str) -> Option<&str> {
        self.preferred
            .binary_search_by(|(key, _)| key.as_ref().cmp(territory))
            .ok()
            .map(|index| self.preferred[index].1.as_ref())
    }

    fn qualify(&self, zone: &Zone, name: &str, fallback_format: &str) -> String {
        // Both supported locales have likely territory US. LDML47 §4.3
        // qualifies nonpreferred generic and specific metazone names.
        let preferred = self
            .preferred("US")
            .or_else(|| self.preferred("001"))
            .expect("validated metazone has a golden zone");
        if preferred == zone.identifier.as_ref() {
            return name.to_owned();
        }
        let qualifier = if self.preferred(&zone.territory) == Some(zone.identifier.as_ref()) {
            zone.country.as_deref().unwrap_or(&zone.city)
        } else {
            &zone.city
        };
        fallback_format
            .replace("{1}", name)
            .replace("{0}", qualifier)
    }
}

struct MetazonePeriod {
    start: i64,
    end: i64,
    metazone: usize,
}

struct Zone {
    identifier: Box<str>,
    territory: Box<str>,
    city: Box<str>,
    country: Option<Box<str>>,
    location: Option<Box<str>>,
    names: WidthNames,
    periods: Vec<MetazonePeriod>,
}

impl Zone {
    fn metazone<'a>(&self, epoch: i64, metazones: &'a [Metazone]) -> Option<&'a Metazone> {
        let index = self.periods.partition_point(|period| period.start <= epoch);
        let period = self.periods.get(index.checked_sub(1)?)?;
        (epoch < period.end).then(|| &metazones[period.metazone])
    }

    fn non_location_name(
        &self,
        epoch: i64,
        metazones: &[Metazone],
        fallback_format: &str,
        width: NameWidth,
        kind: NameKind,
        stability: StandardTimeStability,
    ) -> Option<String> {
        let direct = self.names.get(width);
        let metazone = self.metazone(epoch, metazones);
        let meta_names = metazone.map(|metazone| metazone.names.get(width));
        // A zone's daylight override survives the fallback to its metazone;
        // otherwise a standard-only metazone would erase seasonal evidence.
        let has_daylight_name = self.names.has_daylight()
            || metazone.is_some_and(|metazone| metazone.names.has_daylight());
        if let Some(name) = direct.select(kind, has_daylight_name, stability) {
            return Some(name.to_owned());
        }
        let name = meta_names?.select(kind, has_daylight_name, stability)?;
        Some(metazone?.qualify(self, name, fallback_format))
    }

    fn generic_location_name(&self, region_format: &str) -> Option<String> {
        self.location
            .as_deref()
            .map(|location| region_format.replace("{0}", location))
    }
}

/// A completely validated native catalogue and its actual IANA foundation.
pub(crate) struct TimeZoneNames {
    named: Arc<NamedTimeZones>,
    zones: Vec<Zone>,
    metazones: Vec<Metazone>,
    aliases: Vec<(Box<str>, usize)>,
    fallback_format: Box<str>,
    region_format: Box<str>,
    gmt_format: Box<str>,
    gmt_zero_format: Box<str>,
}

fn width_names(raw: [[Option<String>; 3]; 2]) -> WidthNames {
    let variants = |[generic, standard, daylight]: [Option<String>; 3]| NameVariants {
        generic: generic.map(String::into_boxed_str),
        standard: standard.map(String::into_boxed_str),
        daylight: daylight.map(String::into_boxed_str),
    };
    let [short, long] = raw;
    WidthNames {
        short: variants(short),
        long: variants(long),
    }
}

fn valid_pattern(source: &str, slots: &[&str]) -> bool {
    if source.is_empty() {
        return false;
    }
    let mut remaining = source.to_owned();
    for slot in slots {
        if remaining.matches(*slot).count() != 1 {
            return false;
        }
        remaining = remaining.replace(*slot, "");
    }
    !remaining.contains(['{', '}'])
}

impl TimeZoneNames {
    pub(crate) fn from_json(
        bytes: &[u8],
        named: Arc<NamedTimeZones>,
    ) -> Result<Self, InvalidTimeZoneData> {
        Self::from_bytes(bytes, named, None)
    }
    pub(crate) fn from_projection(
        catalogue: &crate::time_zone_names_image::projection::TimeZoneNameCatalogue<'_>,
        named: Arc<NamedTimeZones>,
    ) -> Result<Self, InvalidTimeZoneData> {
        Self::from_bytes(catalogue.bytes(), named, Some(catalogue))
    }
    fn from_bytes(
        bytes: &[u8],
        named: Arc<NamedTimeZones>,
        catalogue: Option<&crate::time_zone_names_image::projection::TimeZoneNameCatalogue<'_>>,
    ) -> Result<Self, InvalidTimeZoneData> {
        let raw: raw::NativeProfile = serde_json::from_slice(bytes)
            .map_err(|_| InvalidTimeZoneData("invalid native time-zone names JSON"))?;
        if raw.schema != 1
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.country_icu_commit != "457157a92aa053e632cc7fcfd0e12f8a943b2d11"
            || raw.source_manifest_sha256
                != "51a23e38b7a70a0fd831c4c9779d72d9fc75bf85f4ac86ad23d4f82da7abafbd"
            || raw.supported_locales != ["en", "en-US"]
            || raw.fallback_chain != ["root", "en", "en_US"]
            || (catalogue.is_none()
                && (raw.rows.zones.len() != 446
                    || raw.rows.aliases.len() != 600
                    || raw.rows.metazones.len() != 190
                    || raw
                        .rows
                        .zones
                        .iter()
                        .map(|zone| zone.periods.len())
                        .sum::<usize>()
                        != 669))
        {
            return Err(InvalidTimeZoneData(
                "native time-zone names metadata/domain",
            ));
        }
        let mut patterns = std::collections::BTreeMap::new();
        for (tag, kind, value) in raw.rows.patterns {
            if patterns.insert((tag, kind), value).is_some() {
                return Err(InvalidTimeZoneData("duplicate native time-zone pattern"));
            }
        }
        let required = [
            ("fallbackFormat", "generic", &["{0}", "{1}"][..]),
            ("gmtFormat", "generic", &["{0}"][..]),
            ("gmtZeroFormat", "generic", &[][..]),
            ("hourFormat", "generic", &[][..]),
            ("regionFormat", "daylight", &["{0}"][..]),
            ("regionFormat", "generic", &["{0}"][..]),
            ("regionFormat", "standard", &["{0}"][..]),
        ];
        if patterns.len() != required.len()
            || required.iter().any(|(tag, kind, slots)| {
                !patterns
                    .get(&(tag.to_string(), kind.to_string()))
                    .is_some_and(|value| valid_pattern(value, slots))
            })
            || patterns
                .get(&("hourFormat".into(), "generic".into()))
                .map(String::as_str)
                != Some("+HH:mm;-HH:mm")
        {
            return Err(InvalidTimeZoneData(
                "native time-zone pattern domain/placeholders",
            ));
        }
        let metazones: Vec<_> = raw
            .rows
            .metazones
            .into_iter()
            .map(|row| Metazone {
                identifier: row.identifier.into_boxed_str(),
                names: width_names(row.names),
                preferred: row
                    .preferred
                    .into_iter()
                    .map(|(territory, zone)| (territory.into_boxed_str(), zone.into_boxed_str()))
                    .collect(),
            })
            .collect();
        if !metazones
            .windows(2)
            .all(|pair| pair[0].identifier < pair[1].identifier)
        {
            return Err(InvalidTimeZoneData("unordered native metazones"));
        }
        let mut zones = Vec::new();
        for row in raw.rows.zones {
            let mut periods = Vec::new();
            for (start, end, metazone) in row.periods {
                let metazone = metazones
                    .binary_search_by(|row| row.identifier.as_ref().cmp(&metazone))
                    .map_err(|_| InvalidTimeZoneData("missing native metazone target"))?;
                periods.push(MetazonePeriod {
                    start,
                    end,
                    metazone,
                });
            }
            zones.push(Zone {
                identifier: row.identifier.into_boxed_str(),
                territory: row.territory.into_boxed_str(),
                city: row.city.into_boxed_str(),
                country: row.country.map(String::into_boxed_str),
                location: row.location.map(String::into_boxed_str),
                names: width_names(row.names),
                periods,
            });
        }
        if !zones
            .windows(2)
            .all(|pair| pair[0].identifier < pair[1].identifier)
        {
            return Err(InvalidTimeZoneData("unordered native time-zone names"));
        }
        let mut aliases = Vec::new();
        for (alias, canonical) in raw.rows.aliases {
            let index = zones
                .binary_search_by(|zone| zone.identifier.as_ref().cmp(&canonical))
                .map_err(|_| InvalidTimeZoneData("missing native alias target"))?;
            aliases.push((alias.into_boxed_str(), index));
        }
        if !aliases.windows(2).all(|pair| pair[0].0 < pair[1].0)
            || aliases.iter().any(|(alias, _)| alias.is_empty())
        {
            return Err(InvalidTimeZoneData("invalid native time-zone alias search"));
        }
        let mut pattern = |tag: &str| {
            patterns
                .remove(&(tag.into(), "generic".into()))
                .expect("complete pattern domain was validated")
                .into_boxed_str()
        };
        let owner = Self {
            named,
            zones,
            metazones,
            aliases,
            fallback_format: pattern("fallbackFormat"),
            region_format: pattern("regionFormat"),
            gmt_format: pattern("gmtFormat"),
            gmt_zero_format: pattern("gmtZeroFormat"),
        };
        for zone in &owner.zones {
            if zone.identifier.is_empty()
                || zone.territory.is_empty()
                || zone.city.is_empty()
                || !zone.names.valid()
                || zone.location.as_deref().is_some_and(str::is_empty)
                || zone.country.as_deref().is_some_and(str::is_empty)
                || zone.periods.iter().any(|period| {
                    period.start >= period.end || period.metazone >= owner.metazones.len()
                })
                || !zone
                    .periods
                    .windows(2)
                    .all(|pair| pair[0].end <= pair[1].start)
                || owner
                    .zone(&zone.identifier)
                    .is_none_or(|target| target.identifier != zone.identifier)
            {
                return Err(InvalidTimeZoneData(
                    "invalid exact metazone periods or zone names",
                ));
            }
        }
        for (index, metazone) in owner.metazones.iter().enumerate() {
            if metazone.identifier.is_empty()
                || !metazone.names.valid()
                || metazone.preferred("001").is_none()
                || !metazone
                    .preferred
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0)
                || metazone.preferred.iter().any(|(territory, identifier)| {
                    territory.is_empty()
                        || (catalogue.is_none()
                            && owner.zone(identifier).is_none_or(|zone| {
                                !zone.periods.iter().any(|period| period.metazone == index)
                            }))
                })
            {
                return Err(InvalidTimeZoneData("invalid preferred metazone names"));
            }
        }
        Ok(owner)
    }

    pub(crate) fn uses_named_zones(&self, zones: &Arc<NamedTimeZones>) -> bool {
        Arc::ptr_eq(&self.named, zones)
    }
    pub(crate) fn named_time_zone_selection(&self) -> Option<&[crate::TimeZoneId]> {
        self.named.named_time_zone_selection()
    }

    fn zone(&self, identifier: &str) -> Option<&Zone> {
        self.aliases
            .binary_search_by(|(alias, _)| alias.as_ref().cmp(identifier))
            .ok()
            .map(|index| &self.zones[self.aliases[index].1])
    }

    pub(crate) fn resolve(
        &self,
        request: ResolveTimeZoneRequest,
    ) -> Result<ResolvedTimeZoneSnapshot, TimeZoneResolveError> {
        let identity;
        let (offset_seconds, input) = match request.selection() {
            TimeZoneSelection::Named(identifier) => {
                identity = self.named.lookup(identifier).map_err(|_| {
                    TimeZoneResolveError::InvalidNamedIdentifier(identifier.clone())
                })?;
                let transition = self.named.transition(&identity, request.epoch())?;
                (
                    transition.offset_seconds(),
                    TimeZoneNameInput::Named {
                        identity: &identity,
                        epoch: request.epoch(),
                        transition,
                    },
                )
            }
            TimeZoneSelection::FixedOffset(offset) => {
                (offset.seconds(), TimeZoneNameInput::FixedOffset(*offset))
            }
        };
        let display_name = request
            .name_style()
            .map(|style| self.format(input, style, request.locale()))
            .transpose()?;
        ResolvedTimeZoneSnapshot::from_data(offset_seconds, display_name).map_err(Into::into)
    }

    fn format(
        &self,
        input: TimeZoneNameInput<'_>,
        style: TimeZoneNameStyle,
        locale: &CanonicalLocaleId,
    ) -> Result<String, TimeZoneResolveError> {
        if !supported_name_locale(locale.as_str()) {
            return Err(TimeZoneResolveError::UnsupportedNameLocale(locale.clone()));
        }
        let width = match style {
            TimeZoneNameStyle::Short
            | TimeZoneNameStyle::ShortOffset
            | TimeZoneNameStyle::ShortGeneric => NameWidth::Short,
            TimeZoneNameStyle::Long
            | TimeZoneNameStyle::LongOffset
            | TimeZoneNameStyle::LongGeneric => NameWidth::Long,
        };
        let (identity, epoch, transition) = match input {
            TimeZoneNameInput::Named {
                identity,
                epoch,
                transition,
            } => (identity, epoch, transition),
            TimeZoneNameInput::FixedOffset(offset) => {
                return Ok(self.localized_offset(offset.seconds(), width));
            }
        };
        let kind = match style {
            TimeZoneNameStyle::Short | TimeZoneNameStyle::Long => match transition.variant() {
                TimeZoneVariant::Standard => NameKind::Standard,
                TimeZoneVariant::Daylight => NameKind::Daylight,
            },
            TimeZoneNameStyle::ShortGeneric | TimeZoneNameStyle::LongGeneric => NameKind::Generic,
            TimeZoneNameStyle::ShortOffset | TimeZoneNameStyle::LongOffset => {
                return Ok(self.localized_offset(transition.offset_seconds(), width));
            }
        };
        // ECMA country-preserving primary identities precede CLDR's older
        // alias grouping. Newly added IANA names may have no CLDR47 metadata.
        let Some(zone) = self.zone(identity.primary_identifier()) else {
            return Ok(self.localized_offset(transition.offset_seconds(), width));
        };
        if let Some(name) = zone.non_location_name(
            epoch.get(),
            &self.metazones,
            &self.fallback_format,
            width,
            kind,
            transition.standard_time_stability(),
        ) {
            return Ok(name);
        }
        match kind {
            NameKind::Generic => {
                if let Some(name) = zone.generic_location_name(&self.region_format) {
                    return Ok(name);
                }
            }
            NameKind::Standard | NameKind::Daylight => {}
        }
        Ok(self.localized_offset(transition.offset_seconds(), width))
    }
}

fn supported_name_locale(locale: &str) -> bool {
    let Some((base, extension)) = locale.split_once("-u-") else {
        return matches!(locale, "en" | "en-US");
    };
    if !matches!(base, "en" | "en-US") {
        return false;
    }
    // ResolveLocale only retains the DateTimeFormat keys ca, hc and nu.
    // CanonicalLocaleId has already validated extension syntax and casing.
    let mut key_seen = false;
    for subtag in extension.split('-') {
        match subtag.len() {
            2 if matches!(subtag, "ca" | "hc" | "nu") => key_seen = true,
            3..=8 if key_seen => {}
            _ => return false,
        }
    }
    key_seen
}

impl TimeZoneNames {
    fn localized_offset(&self, seconds: i32, width: NameWidth) -> String {
        if seconds == 0 {
            return self.gmt_zero_format.to_string();
        }
        let absolute = i64::from(seconds).unsigned_abs();
        let hours = absolute / 3600;
        let minutes = (absolute / 60) % 60;
        let seconds_part = absolute % 60;
        let sign = if seconds < 0 { '-' } else { '+' };
        let mut offset = match width {
            NameWidth::Short => format!("{sign}{hours}"),
            NameWidth::Long => format!("{sign}{hours:02}"),
        };
        if matches!(width, NameWidth::Long) || minutes != 0 || seconds_part != 0 {
            offset.push_str(&format!(":{minutes:02}"));
        }
        if seconds_part != 0 {
            offset.push_str(&format!(":{seconds_part:02}"));
        }
        self.gmt_format.replace("{0}", &offset)
    }
}
