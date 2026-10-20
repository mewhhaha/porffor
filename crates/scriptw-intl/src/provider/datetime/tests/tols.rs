use super::*;

#[test]
fn tols_date_and_buddhist_parts_use_actual_supplement_digits() {
    for (locale, year) in [
        ("en-US-u-nu-tols", "𑷢𑷠𑷢𑷤"),
        ("en-US-u-ca-buddhist-nu-tols", "𑷢𑷥𑷦𑷧"),
    ] {
        let result = format(
            request(
                locale,
                DateTimeStyleSelection::Components(date_components()),
            ),
            date(2024, 2, 29),
        );
        let values = values(&result);
        assert!(values.contains(&("year", year)));
        assert!(values.contains(&("month", "𑷢")));
        assert!(values.contains(&("day", "𑷢𑷩")));
    }
    let selected = provider()
        .resolve_locale(locale_request(&["en-US-u-nu-tols"]))
        .unwrap();
    assert_eq!(selected.numbering_system.as_str(), "tols");
    assert_eq!(selected.locale.as_str(), "en-US-u-nu-tols");
}

#[test]
fn tols_fractional_seconds_and_range_sources_keep_localized_utf16_digits() {
    let fields = DateTimeComponents {
        fractional_second_digits: Some(DateTimeFractionalDigits::new(3).unwrap()),
        ..time_components()
    };
    let request = request(
        "en-US-u-nu-tols",
        DateTimeStyleSelection::Components(fields),
    );
    let start = instant(0, 123_000_000);
    let end = instant(0, 456_000_000);
    let scalar = format(request.clone(), start);
    assert!(values(&scalar).contains(&("fractionalSecond", "𑷡𑷢𑷣")));
    let result = range(request, start, end);
    assert!(result
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::FractionalSecond
            && part.value == "𑷡𑷢𑷣"
            && part.source == DateTimeRangeSource::StartRange));
    assert!(result
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::FractionalSecond
            && part.value == "𑷤𑷥𑷦"
            && part.source == DateTimeRangeSource::EndRange));
}

#[test]
fn unreviewed_numbering_supplement_metadata_cannot_publish_a_datetime_profile() {
    let source = include_str!("../generated/profile.json");
    let mut raw: serde_json::Value = serde_json::from_str(source).unwrap();
    raw["numbering_supplement"]["cldr_commit"] = serde_json::Value::String("unreviewed".into());
    assert!(super::super::profile::Profile::from_json(&raw.to_string()).is_err());
}
