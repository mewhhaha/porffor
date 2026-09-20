use super::*;

#[test]
fn cldr_intervals_own_shared_fields_and_keep_reversed_endpoint_identity() {
    let fields = DateTimeComponents {
        month: Some(DateTimeMonthWidth::Short),
        ..date_components()
    };
    let input = request("en-US", DateTimeStyleSelection::Components(fields));
    let result = range(input.clone(), date(2020, 1, 10), date(2020, 1, 12));
    let parts = result
        .parts
        .iter()
        .map(|part| {
            (
                part.kind.as_str(),
                part.value.as_str(),
                part.source.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        parts,
        [
            ("month", "Jan", "shared"),
            ("literal", " ", "shared"),
            ("day", "10", "startRange"),
            ("literal", "\u{2009}–\u{2009}", "shared"),
            ("day", "12", "endRange"),
            ("literal", ", ", "shared"),
            ("year", "2020", "shared")
        ]
    );
    assert_eq!(
        result.to_formatted_string(),
        "Jan 10\u{2009}–\u{2009}12, 2020"
    );
    let result = range(input, date(2020, 1, 12), date(2020, 1, 10));
    assert_eq!(
        result.to_formatted_string(),
        "Jan 12\u{2009}–\u{2009}10, 2020"
    );
    assert_eq!(
        result
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::Day)
            .unwrap()
            .source,
        DateTimeRangeSource::StartRange
    );
}

#[test]
fn narrow_name_collisions_do_not_replace_calendar_field_comparison() {
    let input = request(
        "en-US",
        DateTimeStyleSelection::Components(DateTimeComponents {
            month: Some(DateTimeMonthWidth::Narrow),
            ..Default::default()
        }),
    );
    let result = range(input, date(2020, 1, 1), date(2020, 6, 1));
    let months = result
        .parts
        .iter()
        .filter(|part| part.kind == DateTimePartKind::Month)
        .map(|part| (part.value.as_str(), part.source))
        .collect::<Vec<_>>();
    assert_eq!(
        months,
        [
            ("J", DateTimeRangeSource::StartRange),
            ("J", DateTimeRangeSource::EndRange)
        ]
    );
}

#[test]
fn arabic_intervals_localize_numeric_fields_and_keep_separators_literal() {
    let fields = DateTimeComponents {
        month: Some(DateTimeMonthWidth::Short),
        ..date_components()
    };
    let result = range(
        request("ar-EG", DateTimeStyleSelection::Components(fields)),
        date(2020, 1, 10),
        date(2020, 1, 12),
    );
    assert_eq!(result.to_formatted_string(), "١٠–١٢ يناير ٢٠٢٠");
    assert_eq!(
        result
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::Month)
            .unwrap()
            .source,
        DateTimeRangeSource::Shared
    );
    assert_eq!(
        result
            .parts
            .iter()
            .filter(|part| part.kind == DateTimePartKind::Day)
            .map(|part| part.source)
            .collect::<Vec<_>>(),
        [
            DateTimeRangeSource::StartRange,
            DateTimeRangeSource::EndRange
        ]
    );
}

#[test]
fn subprecision_differences_collapse_but_visible_fraction_changes_do_not() {
    let fields = DateTimeComponents {
        fractional_second_digits: Some(DateTimeFractionalDigits::new(2).unwrap()),
        ..time_components()
    };
    let input = request("en-US", DateTimeStyleSelection::Components(fields));
    let collapsed = range(
        input.clone(),
        instant(0, 121_000_000),
        instant(0, 129_999_999),
    );
    assert!(collapsed
        .parts
        .iter()
        .all(|part| part.source == DateTimeRangeSource::Shared));
    let result = range(input, instant(0, 120_000_000), instant(0, 130_000_000));
    assert_eq!(
        result
            .parts
            .iter()
            .filter(|part| part.kind == DateTimePartKind::FractionalSecond)
            .map(|part| (part.value.as_str(), part.source))
            .collect::<Vec<_>>(),
        [
            ("12", DateTimeRangeSource::StartRange),
            ("13", DateTimeRangeSource::EndRange)
        ]
    );
}

#[test]
fn mixed_patterns_use_the_standard_range_connector() {
    let selection = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Long), Some(DateTimeStyle::Short)).unwrap(),
    );
    let input = request("en-US", selection);
    let single = format(input.clone(), instant(0, 0));
    assert!(single.to_formatted_string().contains(" at "));
    let result = range(input, instant(0, 0), instant(3_600, 0));
    assert!(!result.to_formatted_string().contains(" at "));
    assert_eq!(
        result
            .parts
            .iter()
            .filter(|part| part.kind == DateTimePartKind::Year)
            .count(),
        1
    );
    assert_eq!(
        result
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::Year)
            .unwrap()
            .source,
        DateTimeRangeSource::Shared
    );
}

#[test]
fn intervals_preserve_related_years_and_cyclic_names_for_both_endpoint_orders() {
    for locale in ["en-US-u-ca-chinese", "zh-u-ca-chinese"] {
        for (left, left_year, right, right_year) in [
            (-2_208_988_800, "1899", 946_684_800, "1999"),
            (946_684_800, "1999", -2_208_988_800, "1899"),
            (946_684_800, "1999", 4_102_444_800, "2099"),
        ] {
            for selection in [
                DateTimeStyleSelection::Components(date_components()),
                DateTimeStyleSelection::Styles(
                    DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
                ),
            ] {
                let input = request(locale, selection);
                let start = format(input.clone(), instant(left, 0));
                let end = format(input.clone(), instant(right, 0));
                let result = range(input, instant(left, 0), instant(right, 0));
                assert!(!result
                    .parts
                    .iter()
                    .any(|part| part.kind == DateTimePartKind::Year));
                for (expected, source, scalar) in [
                    (left_year, DateTimeRangeSource::StartRange, &start),
                    (right_year, DateTimeRangeSource::EndRange, &end),
                ] {
                    assert!(result
                        .parts
                        .iter()
                        .any(|part| part.kind == DateTimePartKind::RelatedYear
                            && part.value == expected
                            && part.source == source));
                    if let Some(name) = scalar
                        .parts
                        .iter()
                        .find(|part| part.kind == DateTimePartKind::YearName)
                    {
                        assert!(result
                            .parts
                            .iter()
                            .any(|part| part.kind == DateTimePartKind::YearName
                                && part.value == name.value
                                && part.source == source));
                    }
                }
            }
        }
    }
}
