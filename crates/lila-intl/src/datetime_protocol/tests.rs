use super::*;

fn locale() -> DateTimeLocaleResult {
    DateTimeLocaleResult {
        locale: CanonicalLocaleId::from_data("zh-u-ca-chinese").unwrap(),
        data_locale: CanonicalLocaleId::from_data("zh").unwrap(),
        calendar: DateTimeCalendar::Chinese,
        numbering_system: DateTimeKeyword::parse("latn").unwrap(),
        hour_cycle: DateTimeHourCycle::H23,
    }
}

fn exact(seconds: i64, nanosecond: u32) -> DateTimeInput {
    DateTimeInput::Exact(
        DateTimeExactInput::new(DateTimeValueKind::Instant, seconds, nanosecond).unwrap(),
    )
}

#[test]
fn request_boundaries_reject_truncation_trailing_bytes_and_other_operations() {
    let request = DateTimeLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("ar-EG-u-nu-latn").unwrap()],
        matcher: DateTimeLocaleMatcher::Lookup,
        calendar: Some(DateTimeKeyword::parse("chinese").unwrap()),
        numbering_system: None,
        hour_cycle: DateTimeHourCyclePreference::TwelveHour,
    };
    let bytes = request.encode().unwrap();
    assert_eq!(DateTimeLocaleRequest::decode(&bytes).unwrap(), request);
    for length in 0..bytes.len() {
        assert!(
            DateTimeLocaleRequest::decode(&bytes[..length]).is_err(),
            "prefix {length}"
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(DateTimeLocaleRequest::decode(&trailing).is_err());
    assert!(DateTimeSupportedLocalesRequest::decode(&bytes).is_err());
    let mut future = bytes;
    future[..8].copy_from_slice(&(DATE_TIME_WIRE_VERSION + 1).to_le_bytes());
    assert!(DateTimeLocaleRequest::decode(&future).is_err());
}

#[test]
fn impossible_counts_are_rejected_before_reserving_memory() {
    let mut bytes = DateTimeSupportedLocalesResult {
        locales: Vec::new(),
    }
    .encode()
    .unwrap();
    bytes[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(
        DateTimeSupportedLocalesResult::decode(&bytes),
        Err(DateTimeWireError::Malformed(
            "list count exceeds remaining fields"
        ),)
    );
    let mut bytes = DateTimeParts { parts: Vec::new() }.encode().unwrap();
    bytes[16..24].copy_from_slice(&1_u64.to_le_bytes());
    assert!(DateTimeParts::decode(&bytes).is_err());
}

#[test]
fn empty_requested_locales_remain_distinct_from_supported_locale_results() {
    let request = DateTimeSupportedLocalesRequest {
        requested: Vec::new(),
        matcher: DateTimeLocaleMatcher::BestFit,
    };
    assert_eq!(
        DateTimeSupportedLocalesRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
    let response = DateTimeSupportedLocalesResult {
        locales: vec![CanonicalLocaleId::from_data("en-US-u-ca-chinese").unwrap()],
    };
    assert_eq!(
        DateTimeSupportedLocalesResult::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    assert!(DateTimeLocaleResult::decode(&response.encode().unwrap()).is_err());
}

#[test]
fn unknown_option_codes_do_not_turn_into_absent_options() {
    let mut bytes = DateTimeLocaleRequest {
        requested: Vec::new(),
        matcher: DateTimeLocaleMatcher::Lookup,
        calendar: None,
        numbering_system: None,
        hour_cycle: DateTimeHourCyclePreference::Default,
    }
    .encode()
    .unwrap();
    for offset in [16, 24] {
        let original: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
        bytes[offset..offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(DateTimeLocaleRequest::decode(&bytes).is_err());
        bytes[offset..offset + 8].copy_from_slice(&original);
    }
    assert!(DateTimeNumericWidth::from_wire_code(0).is_none());
    assert!(DateTimeTextWidth::from_wire_code(4).is_none());
}

#[test]
fn normalized_negative_nanoseconds_cross_without_number_rounding() {
    let request = DateTimeFormatRequest {
        plan: EncodedDateTimePlan::from_bytes(vec![7, 3, 9]),
        input: exact(-1, 999_999_999),
    };
    let bytes = request.encode().unwrap();
    assert_eq!(
        bytes.len(),
        DATE_TIME_WIRE_HEADER_BYTES as usize + DATE_TIME_INPUT_BYTES as usize + 11
    );
    assert_eq!(DateTimeFormatRequest::decode(&bytes).unwrap(), request);
    assert_eq!(i64::from_le_bytes(bytes[24..32].try_into().unwrap()), -1);
    assert_eq!(
        u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
        999_999_999
    );
}

#[test]
fn input_kind_changes_cannot_reinterpret_reserved_words() {
    let request = DateTimeFormatRequest {
        plan: EncodedDateTimePlan::from_bytes(Vec::new()),
        input: exact(10, 0),
    };
    let mut bytes = request.encode().unwrap();
    bytes[40..48].copy_from_slice(&2026_u64.to_le_bytes());
    assert!(DateTimeFormatRequest::decode(&bytes).is_err());
    bytes[16..24].copy_from_slice(&DateTimeValueKind::PlainDate.wire_code().to_le_bytes());
    assert!(DateTimeFormatRequest::decode(&bytes).is_err());
    bytes[16..24].copy_from_slice(&99_u64.to_le_bytes());
    assert!(DateTimeFormatRequest::decode(&bytes).is_err());
}

#[test]
fn plain_calendar_fields_preserve_terminal_dates_without_time_clip() {
    let request = DateTimeFormatRequest {
        plan: EncodedDateTimePlan::from_bytes(vec![0]),
        input: DateTimeInput::Plain(
            DateTimePlainInput::new(
                DateTimeValueKind::PlainDate,
                DateTimeIsoFields {
                    year: -271_821,
                    month: 4,
                    day: 19,
                    hour: 12,
                    minute: 0,
                    second: 0,
                    nanosecond: 0,
                },
            )
            .unwrap(),
        ),
    };
    assert_eq!(
        DateTimeFormatRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
}

#[test]
fn reversed_range_preserves_endpoint_order_and_full_precision() {
    let request = DateTimeRangeRequest {
        plan: EncodedDateTimePlan::from_bytes(vec![4, 2]),
        start: exact(12, 4),
        end: exact(-12, 999_999_999),
    };
    assert_eq!(
        DateTimeRangeRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
}

#[test]
fn style_selection_and_component_selection_have_disjoint_wire_forms() {
    for selection in [
        DateTimeStyleSelection::Components(DateTimeComponents {
            year: Some(DateTimeNumericWidth::Numeric),
            month: Some(DateTimeMonthWidth::Long),
            time_zone_name: Some(TimeZoneNameStyle::LongGeneric),
            ..Default::default()
        }),
        DateTimeStyleSelection::Styles(
            DateTimeStyles::new(Some(DateTimeStyle::Long), None).unwrap(),
        ),
    ] {
        let request = DateTimePlanRequest {
            locale: locale(),
            time_zone: TimeZoneSelection::FixedOffset(
                FixedTimeZoneOffset::from_seconds(-3600).unwrap(),
            ),
            matcher: DateTimeFormatMatcher::Basic,
            required: DateTimeRequired::Date,
            defaults: DateTimeDefaults::Date,
            selection,
        };
        assert_eq!(
            DateTimePlanRequest::decode(&request.encode().unwrap()).unwrap(),
            request
        );
    }
    let mut reader = DateTimeWireReader(&[0; 88]);
    assert_eq!(reader.components().unwrap(), DateTimeComponents::default());
    let mut invalid = [0; 88];
    invalid[72..80].copy_from_slice(&4_u64.to_le_bytes());
    assert!(DateTimeWireReader(&invalid).components().is_err());
}

#[test]
fn selected_plan_keeps_reported_locale_and_actual_fields() {
    let response = DateTimePlanResult {
        locale: locale(),
        time_zone: TimeZoneSelection::Named(TimeZoneId::parse("Asia/Shanghai").unwrap()),
        components: DateTimeComponents {
            year: Some(DateTimeNumericWidth::Numeric),
            ..Default::default()
        },
        styles: None,
        available_formats: DateTimeFormatAvailability::from_available_kinds(
            DateTimeValueKind::ALL.iter().copied(),
        ),
        plan: EncodedDateTimePlan::from_bytes(vec![1, 9, 2, 6]),
    };
    assert_eq!(
        DateTimePlanResult::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    let mut bytes = response.encode().unwrap();
    let availability_offset = bytes.len() - response.plan.as_bytes().len() - 16;
    bytes[availability_offset..availability_offset + 8].copy_from_slice(&32_u64.to_le_bytes());
    assert!(DateTimePlanResult::decode(&bytes).is_err());
    let locale = locale();
    assert_eq!(
        DateTimeLocaleResult::decode(&locale.encode().unwrap()).unwrap(),
        locale
    );
}

#[test]
fn parts_keep_related_and_cyclic_years_and_literal_range_ownership() {
    let parts = DateTimeRangeParts {
        parts: vec![
            DateTimeRangePart {
                kind: DateTimePartKind::RelatedYear,
                value: "٢٠٢٦".into(),
                source: DateTimeRangeSource::Shared,
            },
            DateTimeRangePart {
                kind: DateTimePartKind::YearName,
                value: "丙午".into(),
                source: DateTimeRangeSource::Shared,
            },
            DateTimeRangePart {
                kind: DateTimePartKind::Literal,
                value: "年\u{200f} – ".into(),
                source: DateTimeRangeSource::EndRange,
            },
        ],
    };
    let decoded = DateTimeRangeParts::decode(&parts.encode().unwrap()).unwrap();
    assert_eq!(decoded, parts);
    assert_eq!(decoded.to_formatted_string(), "٢٠٢٦丙午年\u{200f} – ");
    let scalar = DateTimeParts {
        parts: decoded
            .parts
            .iter()
            .map(|part| DateTimePart {
                kind: part.kind,
                value: part.value.clone(),
            })
            .collect(),
    };
    assert_eq!(
        DateTimeParts::decode(&scalar.encode().unwrap()).unwrap(),
        scalar
    );
    assert_eq!(scalar.to_formatted_string(), decoded.to_formatted_string());
}

#[test]
fn malformed_utf8_and_excessive_string_lengths_are_rejected() {
    let mut bytes = DateTimeParts {
        parts: vec![DateTimePart {
            kind: DateTimePartKind::Literal,
            value: "x".into(),
        }],
    }
    .encode()
    .unwrap();
    bytes[40] = 0xff;
    assert_eq!(
        DateTimeParts::decode(&bytes),
        Err(DateTimeWireError::Malformed("ill-formed UTF-8"))
    );
    bytes[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(
        DateTimeParts::decode(&bytes),
        Err(DateTimeWireError::Malformed(
            "blob exceeds the Wasm32 host span"
        ))
    );
}

#[test]
fn buddhist_locale_response_has_a_closed_wire_code_and_rejects_future_codes() {
    let mut selected = locale();
    selected.calendar = DateTimeCalendar::Buddhist;
    selected.locale = CanonicalLocaleId::from_data("en-US-u-ca-buddhist").unwrap();
    selected.data_locale = CanonicalLocaleId::from_data("en-US").unwrap();
    let encoded = selected.encode().unwrap();
    assert_eq!(DateTimeLocaleResult::decode(&encoded).unwrap(), selected);
    assert_eq!(DateTimeCalendar::Buddhist.wire_code(), 4);
    let calendar_offset = DATE_TIME_WIRE_HEADER_BYTES as usize
        + 8
        + selected.locale.as_str().len()
        + 8
        + selected.data_locale.as_str().len();
    for invalid in [0_u64, 17, u64::MAX] {
        let mut malformed = encoded.clone();
        malformed[calendar_offset..calendar_offset + 8].copy_from_slice(&invalid.to_le_bytes());
        assert!(DateTimeLocaleResult::decode(&malformed).is_err());
    }
    assert!(DateTimeCalendar::from_wire_code(17).is_none());
    assert!(DateTimeCalendar::from_wire_code(0).is_none());
}

#[test]
fn every_admitted_calendar_crosses_the_response_wire_without_tag_collapse() {
    let mut codes = std::collections::BTreeSet::new();
    for &calendar in DateTimeCalendar::ALL {
        let mut selected = locale();
        selected.calendar = calendar;
        selected.locale =
            CanonicalLocaleId::from_data(format!("en-u-ca-{}", calendar.as_str())).unwrap();
        selected.data_locale = CanonicalLocaleId::from_data("en").unwrap();
        let bytes = selected.encode().unwrap();
        assert_eq!(DateTimeLocaleResult::decode(&bytes).unwrap(), selected);
        assert!(
            codes.insert(calendar.wire_code()),
            "duplicate calendar wire tag"
        );
        assert_eq!(
            DateTimeCalendar::from_wire_code(calendar.wire_code()),
            Some(calendar)
        );
    }
    assert_eq!(codes, (1..=16).collect());
    assert_ne!(
        DateTimeCalendar::Gregorian.wire_code(),
        DateTimeCalendar::Iso8601.wire_code()
    );
    for invalid in [0, 17, u64::MAX] {
        assert!(DateTimeCalendar::from_wire_code(invalid).is_none());
    }
}
