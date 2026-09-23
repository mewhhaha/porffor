use super::super::{plan, profile::Profile, render};
use super::*;

#[test]
fn available_formats_preserve_mandatory_kinds_and_reject_unknown_wire_bits() {
    let exact_only = DateTimeFormatAvailability::from_available_kinds([]);
    assert!(exact_only.contains(DateTimeValueKind::Legacy));
    assert!(exact_only.contains(DateTimeValueKind::Instant));
    for kind in [
        DateTimeValueKind::PlainDate,
        DateTimeValueKind::PlainYearMonth,
        DateTimeValueKind::PlainMonthDay,
        DateTimeValueKind::PlainTime,
        DateTimeValueKind::PlainDateTime,
    ] {
        assert!(!exact_only.contains(kind));
        let selected = DateTimeFormatAvailability::from_available_kinds([kind]);
        assert!(selected.contains(kind));
        assert_eq!(
            DateTimeFormatAvailability::from_wire_code(selected.wire_code()),
            Some(selected)
        );
    }
    assert_eq!(
        DateTimeFormatAvailability::from_wire_code(0),
        Some(exact_only)
    );
    assert!(DateTimeFormatAvailability::from_wire_code(31).is_some());
    for invalid in [32, 63, 1 << 63, u64::MAX] {
        assert_eq!(DateTimeFormatAvailability::from_wire_code(invalid), None);
    }
}

#[test]
fn opaque_plans_validate_identity_framing_and_primitive_membership() {
    let input = request(
        "en-US",
        DateTimeStyleSelection::Components(date_components()),
    );
    let encoded = provider()
        .select_plan(input.clone())
        .unwrap()
        .plan
        .into_bytes();
    for bytes in [
        encoded[..39].to_vec(),
        {
            let mut bytes = encoded.clone();
            bytes[8] ^= 1;
            bytes
        },
        {
            let mut bytes = encoded;
            bytes.push(0);
            bytes
        },
    ] {
        let result = provider().format_parts(
            DateTimeFormatRequest {
                plan: EncodedDateTimePlan::from_bytes(bytes),
                input: date(2020, 1, 25),
            },
            zones(),
        );
        assert!(matches!(result, Err(DateTimeFormatError::InvalidPlan(_))));
    }
    let mut bad = input;
    bad.locale.numbering_system = DateTimeKeyword::parse("unknown").unwrap();
    assert!(matches!(
        provider().select_plan(bad),
        Err(DateTimeFormatError::InvalidPlan(_))
    ));
}

#[test]
fn plain_domain_checks_terminal_days_and_reference_fields() {
    let lower = DateTimeIsoFields {
        year: -271821,
        month: 4,
        day: 19,
        hour: 12,
        minute: 0,
        second: 0,
        nanosecond: 0,
    };
    assert!(DateTimePlainInput::new(DateTimeValueKind::PlainDate, lower).is_ok());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainDate,
        DateTimeIsoFields { day: 18, ..lower }
    )
    .is_err());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainDate,
        DateTimeIsoFields { hour: 0, ..lower }
    )
    .is_err());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainYearMonth,
        DateTimeIsoFields { day: 1, ..lower }
    )
    .is_ok());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainDateTime,
        DateTimeIsoFields { hour: 0, ..lower }
    )
    .is_err());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainDateTime,
        DateTimeIsoFields {
            hour: 0,
            nanosecond: 1,
            ..lower
        }
    )
    .is_ok());
    assert!(DateTimePlainInput::new(
        DateTimeValueKind::PlainTime,
        DateTimeIsoFields {
            year: 1970,
            month: 1,
            day: 2,
            ..lower
        }
    )
    .is_err());
}

#[test]
fn corrupted_required_profile_records_fail_at_construction() {
    let source = include_str!("../generated/profile.json");
    let original: serde_json::Value = serde_json::from_str(source).unwrap();
    let mut duplicate_locale = original.clone();
    let duplicate = duplicate_locale["locales"][0].clone();
    duplicate_locale["locales"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(matches!(
        Profile::from_json(&duplicate_locale.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut unknown_field = original.clone();
    unknown_field["locales"][0]["calendars"]["gregorian"]["styles"]["full"]["date"]["tokens"][0]
        ["field"] = "Q".into();
    assert!(matches!(
        Profile::from_json(&unknown_field.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut missing_name = original.clone();
    let names = missing_name["locales"][0]["calendars"]["gregorian"]["names"]
        .as_array_mut()
        .unwrap();
    names.retain(|row| {
        !(row["kind"] == "month"
            && row["context"] == "format"
            && row["width"] == "wide"
            && row["index"] == 1)
    });
    assert!(matches!(
        Profile::from_json(&missing_name.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut missing_era_append = original.clone();
    missing_era_append["locales"][0]["calendars"]["gregorian"]["append_era"] =
        serde_json::Value::Null;
    assert!(matches!(
        Profile::from_json(&missing_era_append.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut unsupported_era_append = original.clone();
    unsupported_era_append["locales"][0]["calendars"]["chinese"]["append_era"] =
        original["locales"][0]["calendars"]["gregorian"]["append_era"].clone();
    assert!(matches!(
        Profile::from_json(&unsupported_era_append.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut repeated_era_slot = original.clone();
    repeated_era_slot["locales"][0]["calendars"]["gregorian"]["append_era"]["tokens"] =
        serde_json::json!([{"placeholder":0},{"placeholder":0}]);
    assert!(matches!(
        Profile::from_json(&repeated_era_slot.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut overlapping = original;
    let periods = overlapping["locales"][0]["day_period_rules"]
        .as_array_mut()
        .unwrap();
    periods.push(serde_json::json!({"type":"morning1", "from":"00:00", "before":"24:00"}));
    assert!(matches!(
        Profile::from_json(&overlapping.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
}

#[test]
fn every_inherited_pattern_has_a_renderable_checked_field_closure() {
    let profile = &provider().profile;
    for locale in &profile.locales {
        for calendar in [DateTimeCalendar::Gregorian, DateTimeCalendar::Chinese] {
            let mut input = request(
                locale.identifier.as_str(),
                DateTimeStyleSelection::Components(date_components()),
            );
            input.locale.calendar = calendar;
            let plan = plan::select(profile, &input).unwrap();
            for epoch in [-8_640_000_000_000, 0, 1_707_523_200, 8_640_000_000_000] {
                let prepared = render::prepare(&plan, instant(epoch, 0), zones()).unwrap();
                let calendar = locale.calendar(calendar);
                let patterns = calendar
                    .available
                    .iter()
                    .chain(
                        calendar
                            .styles
                            .iter()
                            .flat_map(|style| [&style.date, &style.time]),
                    )
                    .chain(calendar.intervals.iter().map(|interval| &interval.pattern));
                for pattern in patterns {
                    let parts = render::pattern(profile, &plan, pattern, &prepared).unwrap();
                    assert!(!parts.parts.is_empty());
                    assert!(parts.parts.iter().all(|part| !part.value.is_empty()));
                }
            }
        }
    }
}

#[test]
fn a_bare_numbering_override_stays_with_its_pattern_when_composed() {
    use super::super::pattern::{Field, Glue, GlueToken, Pattern, Token};
    let input = request(
        "ar-EG-u-hc-h23",
        DateTimeStyleSelection::Components(date_components()),
    );
    let profile = &provider().profile;
    let selected = plan::select(profile, &input).unwrap();
    let prepared = render::prepare(&selected, instant(0, 0), zones()).unwrap();
    let year = Pattern::new(
        vec![Token::Field(Field::Year(1))],
        vec![(None, "latn".into())],
    );
    let hour = Pattern::single(Field::Hour {
        cycle: DateTimeHourCycle::H23,
        width: 1,
    });
    let glue = Glue {
        tokens: vec![
            GlueToken::First,
            GlueToken::Literal("|".into()),
            GlueToken::Second,
        ],
    };
    let combined = glue.combine(&year, &hour).unwrap();
    let parts = render::pattern(profile, &selected, &combined, &prepared).unwrap();
    assert_eq!(
        values(&parts),
        [("year", "1970"), ("literal", "|"), ("hour", "٠")]
    );
}
