use crate::datetime::DateTimeFormatError;
use crate::{NamedTimeZoneIdentity, TimeZoneEpochSeconds, TimeZoneNameStyle};

use super::super::time_zone_snapshot::{
    NamedTimeZoneTransition, StandardTimeStability, TimeZoneVariant,
};
use super::{
    profile::{invalid, Locale, Profile},
    raw,
};

mod validation;
pub(super) use validation::validate;

pub(super) enum Snapshot {
    Named {
        identity: NamedTimeZoneIdentity,
        epoch: TimeZoneEpochSeconds,
        transition: NamedTimeZoneTransition,
    },
    Fixed(i32),
    Plain,
}
#[derive(Clone, Copy)]
enum Width {
    Short,
    Long,
}
#[derive(Clone, Copy)]
enum Kind {
    Generic,
    Standard,
    Daylight,
}

fn template<'p>(locale: &'p Locale, key: &str) -> Result<&'p str, DateTimeFormatError> {
    locale
        .zones
        .patterns
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| invalid("missing localized zone template"))
}
fn width_names(names: &raw::WidthNames, width: Width) -> &raw::NameVariants {
    match width {
        Width::Short => &names.short,
        Width::Long => &names.long,
    }
}
fn has_daylight(names: &raw::WidthNames) -> bool {
    names.short.daylight.is_some() || names.long.daylight.is_some()
}
fn selected_name(
    names: &raw::NameVariants,
    kind: Kind,
    daylight: bool,
    stability: StandardTimeStability,
) -> Option<&str> {
    let selected = match kind {
        Kind::Generic => names.generic.as_deref(),
        Kind::Standard => names.standard.as_deref(),
        Kind::Daylight => names.daylight.as_deref(),
    };
    selected.or_else(|| {
        if !daylight {
            return names.generic.as_deref().or(names.standard.as_deref());
        }
        match (kind, stability) {
            (Kind::Generic, StandardTimeStability::Stable) => names.standard.as_deref(),
            _ => None,
        }
    })
}
fn preferred<'p>(meta: &'p raw::Metazone, territory: &str) -> Option<&'p str> {
    meta.preferred
        .binary_search_by(|(candidate, _)| candidate.as_str().cmp(territory))
        .ok()
        .map(|index| meta.preferred[index].1.as_str())
}

pub(super) fn format(
    profile: &Profile,
    locale: &Locale,
    numbering: &str,
    snapshot: &Snapshot,
    style: TimeZoneNameStyle,
) -> Result<String, DateTimeFormatError> {
    let width = match style {
        TimeZoneNameStyle::Short
        | TimeZoneNameStyle::ShortGeneric
        | TimeZoneNameStyle::ShortOffset => Width::Short,
        TimeZoneNameStyle::Long
        | TimeZoneNameStyle::LongGeneric
        | TimeZoneNameStyle::LongOffset => Width::Long,
    };
    let (identity, epoch, transition) = match snapshot {
        Snapshot::Named {
            identity,
            epoch,
            transition,
        } => (identity, *epoch, *transition),
        Snapshot::Fixed(offset) => return offset_name(profile, locale, numbering, *offset, width),
        Snapshot::Plain => {
            return Err(DateTimeFormatError::InvalidPlan(
                "Plain format retained a time-zone field",
            ));
        }
    };
    let kind = match style {
        TimeZoneNameStyle::Short | TimeZoneNameStyle::Long => match transition.variant() {
            TimeZoneVariant::Standard => Kind::Standard,
            TimeZoneVariant::Daylight => Kind::Daylight,
        },
        TimeZoneNameStyle::ShortGeneric | TimeZoneNameStyle::LongGeneric => Kind::Generic,
        TimeZoneNameStyle::ShortOffset | TimeZoneNameStyle::LongOffset => {
            return offset_name(
                profile,
                locale,
                numbering,
                transition.offset_seconds(),
                width,
            );
        }
    };
    let aliases = &profile.geography.aliases;
    let Some(canonical) = aliases
        .binary_search_by(|(alias, _)| alias.as_str().cmp(identity.primary_identifier()))
        .ok()
        .map(|index| aliases[index].1.as_str())
    else {
        return offset_name(
            profile,
            locale,
            numbering,
            transition.offset_seconds(),
            width,
        );
    };
    let index = profile
        .geography
        .zones
        .binary_search_by(|zone| zone.identifier.as_str().cmp(canonical))
        .map_err(|_| invalid("validated zone alias target missing"))?;
    let zone = &profile.geography.zones[index];
    let translated = &locale.zones.zones[index];
    let period = zone
        .periods
        .partition_point(|period| period.0 <= epoch.get())
        .checked_sub(1)
        .and_then(|index| zone.periods.get(index))
        .filter(|period| epoch.get() < period.1);
    let metazone = period
        .map(|period| {
            profile
                .geography
                .metazones
                .binary_search_by(|meta| meta.identifier.cmp(&period.2))
        })
        .transpose()
        .map_err(|_| invalid("validated metazone period target missing"))?;
    let has_daylight = has_daylight(&translated.names)
        || metazone.is_some_and(|index| has_daylight(&locale.zones.metazones[index].names));
    if let Some(name) = selected_name(
        width_names(&translated.names, width),
        kind,
        has_daylight,
        transition.standard_time_stability(),
    ) {
        return Ok(name.to_owned());
    }
    if let Some(index) = metazone {
        let meta = &profile.geography.metazones[index];
        if let Some(name) = selected_name(
            width_names(&locale.zones.metazones[index].names, width),
            kind,
            has_daylight,
            transition.standard_time_stability(),
        ) {
            let preferred_zone = preferred(meta, &locale.territory)
                .or_else(|| preferred(meta, "001"))
                .ok_or_else(|| invalid("metazone has no golden zone"))?;
            if preferred_zone == canonical {
                return Ok(name.to_owned());
            }
            let qualifier = if preferred(meta, &zone.territory) == Some(canonical) {
                translated.country.as_deref().unwrap_or(&translated.city)
            } else {
                &translated.city
            };
            return Ok(template(locale, "fallbackFormat")?
                .replace("{1}", name)
                .replace("{0}", qualifier));
        }
    }
    if matches!(kind, Kind::Generic) {
        if let Some(location) = &translated.location {
            return Ok(template(locale, "regionFormat:generic")?.replace("{0}", location));
        }
    }
    offset_name(
        profile,
        locale,
        numbering,
        transition.offset_seconds(),
        width,
    )
}

fn offset_name(
    profile: &Profile,
    locale: &Locale,
    numbering: &str,
    offset: i32,
    width: Width,
) -> Result<String, DateTimeFormatError> {
    if offset == 0 {
        return Ok(template(locale, "gmtZeroFormat")?.to_owned());
    }
    let absolute = i64::from(offset).abs();
    let hour = absolute / 3600;
    let minute = absolute / 60 % 60;
    let second = absolute % 60;
    let mut result = if offset < 0 { "-" } else { "+" }.to_owned();
    result.push_str(&super::render::positional(
        profile,
        locale,
        numbering,
        hour,
        if matches!(width, Width::Long) { 2 } else { 1 },
    )?);
    if matches!(width, Width::Long) || minute != 0 || second != 0 {
        result.push(':');
        result.push_str(&super::render::positional(
            profile, locale, numbering, minute, 2,
        )?);
    }
    if second != 0 {
        result.push(':');
        result.push_str(&super::render::positional(
            profile, locale, numbering, second, 2,
        )?);
    }
    Ok(template(locale, "gmtFormat")?.replace("{0}", &result))
}
