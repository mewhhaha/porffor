use super::super::profile::{invalid, Profile};
use super::{preferred, raw, template};
use crate::datetime::DateTimeFormatError;

pub(in crate::provider::datetime) fn validate(
    profile: &Profile,
) -> Result<(), DateTimeFormatError> {
    let geography = &profile.geography;
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
    for locale in &profile.locales {
        if locale.zones.patterns.len() != 7
            || template(locale, "hourFormat")? != "+HH:mm;-HH:mm"
            || locale.zones.zones.len() != geography.zones.len()
            || locale.zones.metazones.len() != geography.metazones.len()
        {
            return Err(invalid("incomplete localized zone profile"));
        }
        for key in [
            "gmtFormat",
            "regionFormat:generic",
            "regionFormat:standard",
            "regionFormat:daylight",
        ] {
            validate_template(template(locale, key)?, &["{0}"])?;
        }
        validate_template(template(locale, "fallbackFormat")?, &["{0}", "{1}"])?;
        validate_template(template(locale, "gmtZeroFormat")?, &[])?;
        for (zone, translated) in geography.zones.iter().zip(&locale.zones.zones) {
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
        for (meta, translated) in geography.metazones.iter().zip(&locale.zones.metazones) {
            if meta.identifier != translated.identifier || !valid_names(&translated.names) {
                return Err(invalid("invalid localized metazone names"));
            }
        }
    }
    Ok(())
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
