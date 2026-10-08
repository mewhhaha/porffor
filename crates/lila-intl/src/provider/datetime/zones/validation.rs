use super::super::profile::invalid;
use super::{preferred, raw, records::OffsetPattern};
use crate::datetime::DateTimeFormatError;

pub(super) fn geography(geography: &raw::Geography) -> Result<(), DateTimeFormatError> {
    if geography.zones.is_empty()
        || geography.metazones.is_empty()
        || !geography
            .aliases
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0)
        || !geography
            .zones
            .windows(2)
            .all(|pair| pair[0].identifier < pair[1].identifier)
        || !geography
            .metazones
            .windows(2)
            .all(|pair| pair[0].identifier < pair[1].identifier)
    {
        return Err(invalid("invalid zone geography search tables"));
    }
    for (alias, canonical) in &geography.aliases {
        if alias.is_empty()
            || !geography
                .zones
                .iter()
                .any(|zone| zone.identifier == *canonical)
        {
            return Err(invalid("invalid localized zone alias"));
        }
    }
    for zone in &geography.zones {
        if zone.identifier.is_empty()
            || zone.territory.is_empty()
            || (zone.territory == "001" && zone.location_is_country)
            || zone.periods.iter().any(|(start, end, meta)| {
                start >= end
                    || !geography
                        .metazones
                        .iter()
                        .any(|candidate| candidate.identifier == *meta)
            })
            || !zone.periods.windows(2).all(|pair| pair[0].1 <= pair[1].0)
        {
            return Err(invalid("invalid zone period/country record"));
        }
    }
    for meta in &geography.metazones {
        if !meta.preferred.windows(2).all(|pair| pair[0].0 < pair[1].0)
            || preferred(meta, "001").is_none()
            || meta.preferred.iter().any(|(territory, zone)| {
                territory.is_empty()
                    || !geography
                        .zones
                        .iter()
                        .any(|candidate| candidate.identifier == *zone)
            })
        {
            return Err(invalid("invalid preferred metazone record"));
        }
    }
    Ok(())
}

pub(super) fn names(
    names: &raw::ZoneNames,
    geography: &raw::Geography,
    selected: Option<&[Box<str>]>,
) -> Result<OffsetPattern, DateTimeFormatError> {
    let offset = OffsetPattern::from_pattern(template(names, "hourFormat")?)?;
    let zones = geography
        .zones
        .iter()
        .filter(|zone| {
            selected.is_none_or(|names| {
                names
                    .binary_search_by(|name| name.as_ref().cmp(zone.identifier.as_str()))
                    .is_ok()
            })
        })
        .collect::<Vec<_>>();
    let used_meta = zones
        .iter()
        .flat_map(|zone| zone.periods.iter().map(|period| period.2.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    let metas = geography
        .metazones
        .iter()
        .filter(|meta| selected.is_none() || used_meta.contains(meta.identifier.as_str()))
        .collect::<Vec<_>>();
    if names.patterns.len() != 7
        || names.zones.len() != zones.len()
        || names.metazones.len() != metas.len()
    {
        return Err(invalid("incomplete localized zone profile"));
    }
    for key in [
        "gmtFormat",
        "regionFormat:generic",
        "regionFormat:standard",
        "regionFormat:daylight",
    ] {
        validate_template(template(names, key)?, &["{0}"])?;
    }
    validate_template(template(names, "fallbackFormat")?, &["{0}", "{1}"])?;
    validate_template(template(names, "gmtZeroFormat")?, &[])?;
    for (zone, translated) in zones.iter().zip(&names.zones) {
        if zone.identifier != translated.identifier
            || translated.city.is_empty()
            || translated.country.as_ref().is_some_and(String::is_empty)
            || translated.location.as_ref().is_some_and(String::is_empty)
            || (zone.territory == "001") != translated.country.is_none()
            || !valid_names(&translated.names)
        {
            return Err(invalid("invalid localized zone names"));
        }
    }
    for (meta, translated) in metas.iter().zip(&names.metazones) {
        if meta.identifier != translated.identifier || !valid_names(&translated.names) {
            return Err(invalid("invalid localized metazone names"));
        }
    }
    Ok(offset)
}

fn template<'a>(names: &'a raw::ZoneNames, key: &str) -> Result<&'a str, DateTimeFormatError> {
    names
        .patterns
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| invalid("missing localized zone template"))
}
fn valid_names(names: &raw::WidthNames) -> bool {
    [&names.short, &names.long].into_iter().all(|names| {
        [&names.generic, &names.standard, &names.daylight]
            .into_iter()
            .flatten()
            .all(|name| !name.is_empty())
    })
}
fn validate_template(source: &str, slots: &[&str]) -> Result<(), DateTimeFormatError> {
    let mut remaining = source.to_owned();
    for slot in slots {
        if remaining.matches(slot).count() != 1 {
            return Err(invalid("invalid localized zone placeholder"));
        }
        remaining = remaining.replace(slot, "");
    }
    if source.is_empty() || remaining.contains(['{', '}']) {
        return Err(invalid("unexpected localized zone placeholder"));
    }
    Ok(())
}
