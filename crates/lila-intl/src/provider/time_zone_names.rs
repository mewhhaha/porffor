//! Pinned LDML47 names over exact UTC metazone periods and one IANA snapshot.

use crate::{CanonicalLocaleId, InvalidTimeZoneData, TimeZoneNameStyle, TimeZoneResolveError};

use super::time_zone_snapshot::{StandardTimeStability, TimeZoneNameInput, TimeZoneVariant};

mod generated;
#[cfg(test)]
mod tests;

pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = generated::PROVIDER_DATA_SHA256;

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

#[derive(Clone, Copy)]
struct NameVariants {
    generic: Option<&'static str>,
    standard: Option<&'static str>,
    daylight: Option<&'static str>,
}

impl NameVariants {
    const EMPTY: Self = Self {
        generic: None,
        standard: None,
        daylight: None,
    };

    fn select(
        self,
        kind: NameKind,
        has_daylight_name: bool,
        stability: StandardTimeStability,
    ) -> Option<&'static str> {
        let requested = match kind {
            NameKind::Generic => self.generic,
            NameKind::Standard => self.standard,
            NameKind::Daylight => self.daylight,
        };
        requested.or_else(|| {
            if !has_daylight_name {
                return self.generic.or(self.standard);
            }
            match (kind, stability) {
                (NameKind::Generic, StandardTimeStability::Stable) => self.standard,
                (NameKind::Generic, StandardTimeStability::NotProven)
                | (NameKind::Standard | NameKind::Daylight, _) => None,
            }
        })
    }

    fn valid(self) -> bool {
        [self.generic, self.standard, self.daylight]
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
    fn get(&self, width: NameWidth) -> NameVariants {
        match width {
            NameWidth::Short => self.short,
            NameWidth::Long => self.long,
        }
    }

    fn valid(&self) -> bool {
        self.short.valid() && self.long.valid()
    }

    fn has_daylight(&self) -> bool {
        self.short.daylight.or(self.long.daylight).is_some()
    }
}

struct Metazone {
    identifier: &'static str,
    names: WidthNames,
    preferred: &'static [(&'static str, &'static str)],
}

impl Metazone {
    fn preferred(&self, territory: &str) -> Option<&'static str> {
        self.preferred
            .binary_search_by_key(&territory, |&(territory, _)| territory)
            .ok()
            .map(|index| self.preferred[index].1)
    }

    fn qualify(&self, zone: &Zone, name: &str) -> String {
        // Both supported locales have likely territory US. LDML47 §4.3
        // qualifies nonpreferred generic and specific metazone names.
        let preferred = self
            .preferred("US")
            .or_else(|| self.preferred("001"))
            .expect("validated metazone has a golden zone");
        if preferred == zone.identifier {
            return name.to_owned();
        }
        let qualifier = if self.preferred(zone.territory) == Some(zone.identifier) {
            zone.country.unwrap_or(zone.city)
        } else {
            zone.city
        };
        generated::FALLBACK_FORMAT
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
    identifier: &'static str,
    territory: &'static str,
    city: &'static str,
    country: Option<&'static str>,
    location: Option<&'static str>,
    names: WidthNames,
    periods: &'static [MetazonePeriod],
}

impl Zone {
    fn metazone(&self, epoch: i64) -> Option<&'static Metazone> {
        let index = self.periods.partition_point(|period| period.start <= epoch);
        let period = self.periods.get(index.checked_sub(1)?)?;
        (epoch < period.end).then(|| &generated::METAZONES[period.metazone])
    }

    fn non_location_name(
        &self,
        epoch: i64,
        width: NameWidth,
        kind: NameKind,
        stability: StandardTimeStability,
    ) -> Option<String> {
        let direct = self.names.get(width);
        let metazone = self.metazone(epoch);
        let meta_names = metazone
            .map(|metazone| metazone.names.get(width))
            .unwrap_or(NameVariants::EMPTY);
        // A zone's daylight override survives the fallback to its metazone;
        // otherwise a standard-only metazone would erase seasonal evidence.
        let has_daylight_name = self.names.has_daylight()
            || metazone.is_some_and(|metazone| metazone.names.has_daylight());
        if let Some(name) = direct.select(kind, has_daylight_name, stability) {
            return Some(name.to_owned());
        }
        let name = meta_names.select(kind, has_daylight_name, stability)?;
        Some(metazone?.qualify(self, name))
    }

    fn generic_location_name(&self) -> Option<String> {
        self.location
            .map(|location| generated::REGION_FORMAT_GENERIC.replace("{0}", location))
    }
}

/// Construction validates all generated indices and interval/search invariants.
pub(super) struct TimeZoneNames {
    _validated: (),
}

impl TimeZoneNames {
    pub(super) fn from_pinned_data() -> Result<Self, InvalidTimeZoneData> {
        if generated::HOUR_FORMAT != "+HH:mm;-HH:mm"
            || !generated::ALIASES
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            || !generated::ZONES
                .windows(2)
                .all(|pair| pair[0].identifier < pair[1].identifier)
            || !generated::METAZONES
                .windows(2)
                .all(|pair| pair[0].identifier < pair[1].identifier)
        {
            return Err(InvalidTimeZoneData("invalid time-zone name search tables"));
        }
        for &(alias, index) in generated::ALIASES {
            if alias.is_empty() || index >= generated::ZONES.len() {
                return Err(InvalidTimeZoneData("invalid time-zone name alias"));
            }
        }
        for zone in generated::ZONES {
            if zone.identifier.is_empty()
                || zone.city.is_empty()
                || !zone.names.valid()
                || zone.location.is_some_and(str::is_empty)
                || zone.country.is_some_and(str::is_empty)
                || zone.periods.iter().any(|period| {
                    period.start >= period.end || period.metazone >= generated::METAZONES.len()
                })
                || !zone
                    .periods
                    .windows(2)
                    .all(|pair| pair[0].end <= pair[1].start)
            {
                return Err(InvalidTimeZoneData(
                    "invalid exact metazone periods or zone names",
                ));
            }
        }
        for metazone in generated::METAZONES {
            if metazone.identifier.is_empty()
                || !metazone.names.valid()
                || metazone.preferred("001").is_none()
                || !metazone
                    .preferred
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0)
                || metazone
                    .preferred
                    .iter()
                    .any(|&(territory, zone)| territory.is_empty() || Self::zone(zone).is_none())
            {
                return Err(InvalidTimeZoneData("invalid preferred metazone names"));
            }
        }
        Ok(Self { _validated: () })
    }

    fn zone(identifier: &str) -> Option<&'static Zone> {
        generated::ALIASES
            .binary_search_by_key(&identifier, |&(alias, _)| alias)
            .ok()
            .map(|index| &generated::ZONES[generated::ALIASES[index].1])
    }

    pub(super) fn format(
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
                return Ok(localized_offset(offset.seconds(), width));
            }
        };
        let kind = match style {
            TimeZoneNameStyle::Short | TimeZoneNameStyle::Long => match transition.variant() {
                TimeZoneVariant::Standard => NameKind::Standard,
                TimeZoneVariant::Daylight => NameKind::Daylight,
            },
            TimeZoneNameStyle::ShortGeneric | TimeZoneNameStyle::LongGeneric => NameKind::Generic,
            TimeZoneNameStyle::ShortOffset | TimeZoneNameStyle::LongOffset => {
                return Ok(localized_offset(transition.offset_seconds(), width));
            }
        };
        // ECMA country-preserving primary identities precede CLDR's older
        // alias grouping. Newly added IANA names may have no CLDR47 metadata.
        let Some(zone) = Self::zone(identity.primary_identifier()) else {
            return Ok(localized_offset(transition.offset_seconds(), width));
        };
        if let Some(name) = zone.non_location_name(
            epoch.get(),
            width,
            kind,
            transition.standard_time_stability(),
        ) {
            return Ok(name);
        }
        match kind {
            NameKind::Generic => {
                if let Some(name) = zone.generic_location_name() {
                    return Ok(name);
                }
            }
            NameKind::Standard | NameKind::Daylight => {}
        }
        Ok(localized_offset(transition.offset_seconds(), width))
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

fn localized_offset(seconds: i32, width: NameWidth) -> String {
    if seconds == 0 {
        return generated::GMT_ZERO_FORMAT.to_owned();
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
    generated::GMT_FORMAT.replace("{0}", &offset)
}
