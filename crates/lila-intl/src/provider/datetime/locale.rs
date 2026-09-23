use std::collections::BTreeMap;

use crate::datetime::*;
use crate::CanonicalLocaleId;

use super::profile::{invalid, Locale, Profile};

fn unicode_extension(identifier: &str) -> (String, BTreeMap<&str, String>) {
    let subtags: Vec<_> = identifier.split('-').collect();
    let Some(start) = subtags.iter().position(|subtag| *subtag == "u") else {
        return (identifier.to_owned(), BTreeMap::new());
    };
    let end = subtags[start + 1..]
        .iter()
        .position(|subtag| subtag.len() == 1)
        .map_or(subtags.len(), |length| start + 1 + length);
    let mut base = subtags[..start].to_vec();
    base.extend_from_slice(&subtags[end..]);
    let mut keywords = BTreeMap::new();
    let mut cursor = start + 1;
    while cursor < end {
        if subtags[cursor].len() != 2 {
            cursor += 1;
            continue;
        }
        let key = subtags[cursor];
        cursor += 1;
        let first = cursor;
        while cursor < end && subtags[cursor].len() != 2 {
            cursor += 1;
        }
        keywords.insert(key, subtags[first..cursor].join("-"));
    }
    (base.join("-"), keywords)
}

fn best_available<'p>(profile: &'p Profile, candidate: &str) -> Option<&'p Locale> {
    let mut candidate = candidate;
    loop {
        if let Some(locale) = profile.locale(candidate) {
            return Some(locale);
        }
        let mut last = candidate.rfind('-')?;
        if last >= 2 && candidate.as_bytes()[last - 2] == b'-' {
            last -= 2;
        }
        candidate = &candidate[..last];
    }
}

pub(super) fn supported(
    profile: &Profile,
    request: DateTimeSupportedLocalesRequest,
) -> DateTimeSupportedLocalesResult {
    match request.matcher {
        DateTimeLocaleMatcher::Lookup | DateTimeLocaleMatcher::BestFit => {}
    }
    DateTimeSupportedLocalesResult {
        locales: request
            .requested
            .into_iter()
            .filter(|locale| {
                best_available(profile, &unicode_extension(locale.as_str()).0).is_some()
            })
            .collect(),
    }
}

pub(super) fn resolve(
    profile: &Profile,
    request: DateTimeLocaleRequest,
) -> Result<DateTimeLocaleResult, DateTimeFormatError> {
    match request.matcher {
        DateTimeLocaleMatcher::Lookup | DateTimeLocaleMatcher::BestFit => {}
    }
    let mut selected = None;
    let mut extension = BTreeMap::new();
    for requested in &request.requested {
        let (base, keywords) = unicode_extension(requested.as_str());
        if let Some(locale) = best_available(profile, &base) {
            selected = Some(locale);
            extension = keywords;
            break;
        }
    }
    let locale = selected
        .or_else(|| profile.locale(&profile.default_locale))
        .ok_or_else(|| invalid("default locale is absent"))?;
    let mut additions = BTreeMap::new();
    let mut calendar = locale.default_calendar;
    if let Some(value) = extension
        .get("ca")
        .and_then(|value| DateTimeCalendar::parse(value))
    {
        calendar = value;
        additions.insert("ca", value.as_str().to_owned());
    }
    if let Some(value) = request
        .calendar
        .as_ref()
        .and_then(|value| DateTimeCalendar::parse(value.as_str()))
    {
        if value != calendar {
            additions.remove("ca");
            calendar = value;
        }
    }
    let mut numbering = locale.default_numbering.clone();
    if let Some(value) = extension
        .get("nu")
        .filter(|value| profile.digits.contains_key(*value))
    {
        numbering = value.clone();
        additions.insert("nu", value.clone());
    }
    if let Some(value) = request
        .numbering_system
        .as_ref()
        .map(DateTimeKeyword::as_str)
        .filter(|value| profile.digits.contains_key(*value))
    {
        if value != numbering {
            additions.remove("nu");
            numbering = value.to_owned();
        }
    }
    let mut cycle = locale.hour_cycle;
    if let Some(value) = extension
        .get("hc")
        .and_then(|value| DateTimeHourCycle::parse(value))
    {
        cycle = value;
        additions.insert("hc", value.as_str().to_owned());
    }
    match request.hour_cycle {
        DateTimeHourCyclePreference::Default => {}
        DateTimeHourCyclePreference::Cycle(value) => {
            if cycle != value {
                additions.remove("hc");
                cycle = value;
            }
        }
        DateTimeHourCyclePreference::TwelveHour => {
            additions.remove("hc");
            cycle = locale.hour_cycle12;
        }
        DateTimeHourCyclePreference::TwentyFourHour => {
            additions.remove("hc");
            cycle = locale.hour_cycle24;
        }
    }
    let mut reported = locale.identifier.as_str().to_owned();
    if !additions.is_empty() {
        reported.push_str("-u");
        for (key, value) in additions {
            reported.push('-');
            reported.push_str(key);
            reported.push('-');
            reported.push_str(&value);
        }
    }
    Ok(DateTimeLocaleResult {
        locale: CanonicalLocaleId::from_data(reported)
            .map_err(|_| invalid("resolved locale spelling is invalid"))?,
        data_locale: locale.identifier.clone(),
        calendar,
        numbering_system: DateTimeKeyword::parse(numbering)?,
        hour_cycle: cycle,
    })
}

pub(super) fn validate<'p>(
    profile: &'p Profile,
    result: &DateTimeLocaleResult,
) -> Result<&'p Locale, DateTimeFormatError> {
    let locale = profile
        .locale(result.data_locale.as_str())
        .ok_or(DateTimeFormatError::InvalidPlan("unknown data locale"))?;
    let (base, keywords) = unicode_extension(result.locale.as_str());
    if base != result.data_locale.as_str()
        || !profile
            .digits
            .contains_key(result.numbering_system.as_str())
    {
        return Err(DateTimeFormatError::InvalidPlan(
            "inconsistent locale or numbering selection",
        ));
    }
    for (key, value) in keywords {
        let expected = match key {
            "ca" => result.calendar.as_str(),
            "hc" => result.hour_cycle.as_str(),
            "nu" => result.numbering_system.as_str(),
            _ => {
                return Err(DateTimeFormatError::InvalidPlan(
                    "unrelated extension in selected locale",
                ));
            }
        };
        if value != expected {
            return Err(DateTimeFormatError::InvalidPlan(
                "resolved keyword differs from locale",
            ));
        }
    }
    Ok(locale)
}
