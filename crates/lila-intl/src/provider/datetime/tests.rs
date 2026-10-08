use std::sync::OnceLock;

use crate::datetime::*;
use crate::{CanonicalLocaleId, TimeZoneId, TimeZoneSelection};

use super::{DateTimeProvider, NamedTimeZones};

mod buddhist;
mod locale_selection;
mod parts;
mod range_endpoint_patterns;
mod ranges;
mod tols;
mod validation;

fn calendar_record<'a>(profile: &'a serde_json::Value, canonical: &str) -> &'a serde_json::Value {
    let index = profile["locales"][0]["calendar_refs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row[0] == canonical)
        .unwrap()[1]
        .as_u64()
        .unwrap() as usize;
    &profile["calendar_pool"][index]
}

fn calendar_record_mut<'a>(
    profile: &'a mut serde_json::Value,
    canonical: &str,
) -> &'a mut serde_json::Value {
    let index = profile["locales"][0]["calendar_refs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row[0] == canonical)
        .unwrap()[1]
        .as_u64()
        .unwrap() as usize;
    &mut profile["calendar_pool"][index]
}

fn provider() -> &'static DateTimeProvider {
    static IMAGE: OnceLock<crate::DateTimeDataImage> = OnceLock::new();
    IMAGE
        .get_or_init(|| crate::embedded_date_time_data_image().expect("admitted DateTime image"))
        .provider_ref()
}
fn zones() -> &'static NamedTimeZones {
    &provider().named_time_zones
}
fn locale_request(locales: &[&str]) -> DateTimeLocaleRequest {
    DateTimeLocaleRequest {
        requested: locales
            .iter()
            .map(|locale| CanonicalLocaleId::from_data(*locale).unwrap())
            .collect(),
        matcher: DateTimeLocaleMatcher::Lookup,
        calendar: None,
        numbering_system: None,
        hour_cycle: DateTimeHourCyclePreference::Default,
    }
}
fn request(locale: &str, selection: DateTimeStyleSelection) -> DateTimePlanRequest {
    DateTimePlanRequest {
        locale: provider()
            .resolve_locale(locale_request(&[locale]))
            .unwrap(),
        time_zone: TimeZoneSelection::Named(TimeZoneId::parse("UTC").unwrap()),
        selection,
        matcher: DateTimeFormatMatcher::Basic,
        required: DateTimeRequired::Any,
        defaults: DateTimeDefaults::Date,
    }
}
fn date_components() -> DateTimeComponents {
    DateTimeComponents {
        year: Some(DateTimeNumericWidth::Numeric),
        month: Some(DateTimeMonthWidth::Numeric),
        day: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    }
}
fn time_components() -> DateTimeComponents {
    DateTimeComponents {
        hour: Some(DateTimeNumericWidth::Numeric),
        minute: Some(DateTimeNumericWidth::Numeric),
        second: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    }
}
fn plain(kind: DateTimeValueKind, year: i32, month: u8, day: u8) -> DateTimeInput {
    DateTimeInput::Plain(
        DateTimePlainInput::new(
            kind,
            DateTimeIsoFields {
                year,
                month,
                day,
                hour: 12,
                minute: 0,
                second: 0,
                nanosecond: 0,
            },
        )
        .unwrap(),
    )
}
fn date(year: i32, month: u8, day: u8) -> DateTimeInput {
    plain(DateTimeValueKind::PlainDate, year, month, day)
}
fn instant(seconds: i64, nanosecond: u32) -> DateTimeInput {
    DateTimeInput::Exact(
        DateTimeExactInput::new(DateTimeValueKind::Instant, seconds, nanosecond).unwrap(),
    )
}
fn format(request: DateTimePlanRequest, input: DateTimeInput) -> DateTimeParts {
    let plan = provider().select_plan(request).unwrap().plan;
    provider()
        .format_parts(DateTimeFormatRequest { plan, input })
        .unwrap()
}
fn range(
    request: DateTimePlanRequest,
    start: DateTimeInput,
    end: DateTimeInput,
) -> DateTimeRangeParts {
    let plan = provider().select_plan(request).unwrap().plan;
    provider()
        .format_range_parts(DateTimeRangeRequest { plan, start, end })
        .unwrap()
}
fn values(parts: &DateTimeParts) -> Vec<(&str, &str)> {
    parts
        .parts
        .iter()
        .map(|part| (part.kind.as_str(), part.value.as_str()))
        .collect()
}
