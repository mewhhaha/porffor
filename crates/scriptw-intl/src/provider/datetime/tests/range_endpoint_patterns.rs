use super::*;

enum ClockKind {
    Time,
    DateTime,
}

fn clock(kind: ClockKind, hour: u8, nanosecond: u32) -> DateTimeInput {
    let (kind, date) = match kind {
        ClockKind::Time => (DateTimeValueKind::PlainTime, (1970, 1, 1)),
        ClockKind::DateTime => (DateTimeValueKind::PlainDateTime, (2021, 8, 4)),
    };
    DateTimeInput::Plain(
        DateTimePlainInput::new(
            kind,
            DateTimeIsoFields {
                year: date.0,
                month: date.1,
                day: date.2,
                hour,
                minute: 30,
                second: 45,
                nanosecond,
            },
        )
        .unwrap(),
    )
}

#[test]
fn genuine_range_endpoint_patterns_preserve_default_literals_and_single_alternates() {
    // CLDR47 en: hms default "h:mm:ss\u{202f}a", ascii "h:mm:ss a";
    // hm/a interval "h:mm\u{202f}a\u{2009}–\u{2009}h:mm\u{202f}a".
    // No hms interval exists, so the genuine default endpoint is required.
    for locale in ["en", "en-US"] {
        for cycle in [DateTimeHourCycle::H11, DateTimeHourCycle::H12] {
            for fraction in [None, Some(DateTimeFractionalDigits::new(3).unwrap())] {
                let mut input = request(
                    locale,
                    DateTimeStyleSelection::Components(DateTimeComponents {
                        fractional_second_digits: fraction,
                        ..time_components()
                    }),
                );
                input.locale.hour_cycle = cycle;
                let start = clock(ClockKind::Time, 0, 123_456_789);
                let end = clock(ClockKind::Time, 23, 123_456_789);
                let hour = if cycle == DateTimeHourCycle::H11 {
                    "0"
                } else {
                    "12"
                };
                let suffix = if fraction.is_some() { ".123" } else { "" };
                let single = format(input.clone(), start);
                assert_eq!(
                    single.to_formatted_string(),
                    format!("{hour}:30:45{suffix} AM")
                );
                let equal = range(input.clone(), start, start);
                assert_eq!(equal.to_formatted_string(), single.to_formatted_string());
                assert!(equal
                    .parts
                    .iter()
                    .all(|part| part.source == DateTimeRangeSource::Shared));
                let result = range(input.clone(), start, end);
                assert_eq!(
                    result.to_formatted_string(),
                    format!(
                        "{hour}:30:45{suffix}\u{202f}AM\u{2009}–\u{2009}11:30:45{suffix}\u{202f}PM"
                    )
                );
                let periods: Vec<_> = result
                    .parts
                    .iter()
                    .enumerate()
                    .filter(|(_, part)| part.kind == DateTimePartKind::DayPeriod)
                    .map(|(index, part)| (&result.parts[index - 1], part))
                    .collect();
                assert_eq!(periods.len(), 2);
                for ((space, period), source) in periods.into_iter().zip([
                    DateTimeRangeSource::StartRange,
                    DateTimeRangeSource::EndRange,
                ]) {
                    assert_eq!(space.kind, DateTimePartKind::Literal);
                    assert_eq!(space.value, "\u{202f}");
                    assert_eq!(space.source, source);
                    assert_eq!(period.source, source);
                }
                let reversed = range(input, end, start);
                assert_eq!(
                    reversed.to_formatted_string(),
                    format!(
                        "11:30:45{suffix}\u{202f}PM\u{2009}–\u{2009}{hour}:30:45{suffix}\u{202f}AM"
                    )
                );
            }
        }
    }
}

#[test]
fn genuine_range_companions_survive_date_glue_styles_and_zone_composition() {
    let start = clock(ClockKind::DateTime, 0, 0);
    let end = clock(ClockKind::DateTime, 23, 0);
    let mut input = request(
        "en-US",
        DateTimeStyleSelection::Components(Default::default()),
    );
    input.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("Pacific/Apia").unwrap());
    let result = range(input, start, end);
    assert_eq!(
        result.to_formatted_string(),
        "8/4/2021, 12:30:45\u{202f}AM\u{2009}–\u{2009}11:30:45\u{202f}PM"
    );
    assert!(result
        .parts
        .iter()
        .filter(|part| matches!(
            part.kind,
            DateTimePartKind::Year | DateTimePartKind::Month | DateTimePartKind::Day
        ))
        .all(|part| part.source == DateTimeRangeSource::Shared));

    for style in [DateTimeStyle::Medium, DateTimeStyle::Short] {
        let input = request(
            "en-US",
            DateTimeStyleSelection::Styles(DateTimeStyles::new(None, Some(style)).unwrap()),
        );
        let first = instant(0, 0);
        let last = instant(86_400, 0);
        let result = range(input, first, last);
        let suffix = if style == DateTimeStyle::Medium {
            ":00"
        } else {
            ""
        };
        assert_eq!(
            result.to_formatted_string(),
            format!("12:00{suffix}\u{202f}AM\u{2009}–\u{2009}12:00{suffix}\u{202f}AM")
        );
    }

    let mut fields = time_components();
    fields.time_zone_name = Some(crate::TimeZoneNameStyle::ShortOffset);
    let input = request("en-US", DateTimeStyleSelection::Components(fields));
    assert_eq!(
        range(input, instant(1845, 0), instant(84645, 0)).to_formatted_string(),
        "12:30:45\u{202f}AM GMT\u{2009}–\u{2009}11:30:45\u{202f}PM GMT"
    );

    let mut input = request(
        "en-US",
        DateTimeStyleSelection::Components(time_components()),
    );
    input.locale.hour_cycle = DateTimeHourCycle::H23;
    assert_eq!(
        range(
            input,
            clock(ClockKind::Time, 0, 0),
            clock(ClockKind::Time, 23, 0)
        )
        .to_formatted_string(),
        "00:30:45\u{2009}–\u{2009}23:30:45"
    );
}

#[test]
fn range_endpoint_companions_reject_changed_domains_and_nested_or_connector_owners() {
    use super::super::profile::Profile;
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../generated/profile.json")).unwrap();
    let index = original["calendar_pool"]
        .as_array()
        .unwrap()
        .iter()
        .position(|record| record["styles"]["short"]["time"]["range_pattern"].is_object())
        .unwrap();
    for mutation in ["field", "numbering", "nested", "connector"] {
        let mut bad = original.clone();
        let calendar = &mut bad["calendar_pool"][index];
        let primary = calendar["styles"]["short"]["time"].clone();
        let companion = &mut calendar["styles"]["short"]["time"]["range_pattern"];
        match mutation {
            "field" => companion["tokens"][0]["field"] = "H".into(),
            "numbering" => {
                companion["numbering_overrides"] =
                    serde_json::json!([{"field":null,"numbering":"arab"}])
            }
            "nested" => companion["range_pattern"] = primary,
            "connector" => calendar["interval_fallback"]["range_pattern"] = primary,
            _ => unreachable!(),
        }
        assert!(
            matches!(
                Profile::from_json(&bad.to_string()),
                Err(DateTimeFormatError::InvalidProfile(_))
            ),
            "{mutation}"
        );
    }
}
