use super::*;

#[test]
fn independent_modern_fields_keep_local_order_names_and_related_year_parts() {
    for (locale, expected) in [
        ("en-US", "1/25/2020"),
        ("ar-EG", "٢٥\u{200f}/١\u{200f}/٢٠٢٠"),
        ("zh-Hans-CN", "2020年1月25日"),
    ] {
        let selected = provider()
            .select_plan(request(
                locale,
                DateTimeStyleSelection::Components(date_components()),
            ))
            .unwrap();
        assert_eq!(selected.components, date_components(), "{locale}");
        let parts = provider()
            .format_parts(
                DateTimeFormatRequest {
                    plan: selected.plan,
                    input: date(2020, 1, 25),
                },
                zones(),
            )
            .unwrap();
        assert_eq!(parts.to_formatted_string(), expected, "{locale}");
        if locale == "zh-Hans-CN" {
            assert_eq!(
                values(&parts),
                [
                    ("year", "2020"),
                    ("literal", "年"),
                    ("month", "1"),
                    ("literal", "月"),
                    ("day", "25"),
                    ("literal", "日"),
                ]
            );
        }
    }
    // New Year and leap-month dates are independent HKO/calendar-domain
    // goldens. Labels, order and d=hanidays are the pinned CLDR47 full pattern.
    let selection = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
    );
    let parts = format(
        request("zh-u-ca-chinese", selection.clone()),
        date(2024, 2, 10),
    );
    assert_eq!(
        values(&parts),
        [
            ("relatedYear", "2024"),
            ("yearName", "甲辰"),
            ("literal", "年"),
            ("month", "正月"),
            ("day", "初一"),
            ("weekday", "星期六")
        ]
    );
    let leap = format(request("zh-u-ca-chinese", selection), date(2023, 3, 22));
    assert_eq!(leap.to_formatted_string(), "2023癸卯年闰二月初一星期三");
    assert!(!leap
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Year));
}

#[test]
fn far_domain_plain_dates_use_exact_iso_fields_and_selected_digits() {
    for (iso, expected_year) in [((-271821, 4, 19), "٢٧١٨٢٢"), ((275760, 9, 13), "٢٧٥٧٦٠")]
    {
        let parts = format(
            request(
                "ar-EG",
                DateTimeStyleSelection::Components(date_components()),
            ),
            date(iso.0, iso.1, iso.2),
        );
        assert_eq!(
            parts
                .parts
                .iter()
                .find(|part| part.kind == DateTimePartKind::Year)
                .unwrap()
                .value,
            expected_year
        );
        assert!(!parts
            .to_formatted_string()
            .chars()
            .any(|character| character.is_ascii_digit()));
    }
    let lunar = request(
        "zh-u-ca-chinese",
        DateTimeStyleSelection::Components(date_components()),
    );
    for input in [date(-271821, 4, 19), date(275760, 9, 13)] {
        let parts = format(lunar.clone(), input);
        assert!(parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::RelatedYear));
        assert!(parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::Month));
        assert!(parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::Day));
    }
}

#[test]
fn selected_widths_preserve_locale_padding_and_honor_two_digit_requests() {
    let input = DateTimeInput::Plain(
        DateTimePlainInput::new(
            DateTimeValueKind::PlainTime,
            DateTimeIsoFields {
                year: 1970,
                month: 1,
                day: 1,
                hour: 14,
                minute: 3,
                second: 9,
                nanosecond: 987_654_321,
            },
        )
        .unwrap(),
    );
    let result = provider()
        .select_plan(request(
            "en-US",
            DateTimeStyleSelection::Components(time_components()),
        ))
        .unwrap();
    assert_eq!(result.components.hour, Some(DateTimeNumericWidth::Numeric));
    assert_eq!(
        result.components.minute,
        Some(DateTimeNumericWidth::TwoDigit)
    );
    assert_eq!(
        result.components.second,
        Some(DateTimeNumericWidth::TwoDigit)
    );
    let parts = provider()
        .format_parts(
            DateTimeFormatRequest {
                plan: result.plan,
                input,
            },
            zones(),
        )
        .unwrap();
    assert_eq!(parts.to_formatted_string(), "2:03:09 PM");
    let fields = DateTimeComponents {
        hour: Some(DateTimeNumericWidth::TwoDigit),
        fractional_second_digits: Some(DateTimeFractionalDigits::new(2).unwrap()),
        ..time_components()
    };
    let parts = format(
        request("en-US", DateTimeStyleSelection::Components(fields)),
        input,
    );
    assert_eq!(parts.to_formatted_string(), "02:03:09.98 PM");
    let parts = format(
        request(
            "zh-Hans",
            DateTimeStyleSelection::Components(time_components()),
        ),
        input,
    );
    assert_eq!(parts.to_formatted_string(), "14:03:09");
}

#[test]
fn pattern_numbering_override_keeps_unrelated_numeric_fields_positional() {
    let parts = format(
        request(
            "zh-u-ca-chinese-nu-arab",
            DateTimeStyleSelection::Styles(
                DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
            ),
        ),
        date(2024, 2, 10),
    );
    assert_eq!(
        parts
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::RelatedYear)
            .unwrap()
            .value,
        "٢٠٢٤"
    );
    assert_eq!(
        parts
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::Day)
            .unwrap()
            .value,
        "初一"
    );
    assert_eq!(
        parts
            .parts
            .iter()
            .find(|part| part.kind == DateTimePartKind::YearName)
            .unwrap()
            .value,
        "甲辰"
    );
}

#[test]
fn exact_negative_subseconds_and_named_transition_snapshots_are_localized() {
    let fields = DateTimeComponents {
        fractional_second_digits: Some(DateTimeFractionalDigits::new(3).unwrap()),
        ..time_components()
    };
    let mut input = request("ar-EG-u-hc-h23", DateTimeStyleSelection::Components(fields));
    input.time_zone =
        TimeZoneSelection::FixedOffset(crate::FixedTimeZoneOffset::from_seconds(0).unwrap());
    let parts = format(input, instant(-1, 999_999_999));
    for (kind, value) in [
        (DateTimePartKind::Hour, "٢٣"),
        (DateTimePartKind::Minute, "٥٩"),
        (DateTimePartKind::Second, "٥٩"),
        (DateTimePartKind::FractionalSecond, "٩٩٩"),
    ] {
        assert_eq!(
            parts
                .parts
                .iter()
                .find(|part| part.kind == kind)
                .unwrap()
                .value,
            value
        );
    }
    let mut input = request(
        "ar-EG-u-hc-h23",
        DateTimeStyleSelection::Components(DateTimeComponents {
            time_zone_name: Some(crate::TimeZoneNameStyle::LongOffset),
            ..time_components()
        }),
    );
    input.time_zone = TimeZoneSelection::Named(TimeZoneId::parse("America/New_York").unwrap());
    for (seconds, hour, zone) in [
        (1_710_053_999, "٠١", "غرينتش-٠٥:٠٠"),
        (1_710_054_000, "٠٣", "غرينتش-٠٤:٠٠"),
    ] {
        let parts = format(input.clone(), instant(seconds, 0));
        assert_eq!(
            parts
                .parts
                .iter()
                .find(|part| part.kind == DateTimePartKind::Hour)
                .unwrap()
                .value,
            hour
        );
        assert_eq!(
            parts
                .parts
                .iter()
                .find(|part| part.kind == DateTimePartKind::TimeZoneName)
                .unwrap()
                .value,
            zone
        );
    }
}

#[test]
fn supplied_ascii_patterns_preserve_non_latin_digits_and_exact_parts() {
    for (locale, expected) in [
        ("en-US", "02:35:06 AM"),
        ("en-US-u-nu-arab", "٠٢:٣٥:٠٦ AM"),
        ("en-US-u-nu-deva", "०२:३५:०६ AM"),
        ("en-US-u-nu-hanidec", "〇二:三五:〇六 AM"),
    ] {
        let fields = DateTimeComponents {
            hour: Some(DateTimeNumericWidth::TwoDigit),
            minute: Some(DateTimeNumericWidth::TwoDigit),
            second: Some(DateTimeNumericWidth::TwoDigit),
            ..Default::default()
        };
        let parts = format(
            request(locale, DateTimeStyleSelection::Components(fields)),
            instant(9_306, 0),
        );
        assert_eq!(parts.to_formatted_string(), expected, "{locale}");
        assert_eq!(parts.parts.len(), 7);
        assert_eq!(parts.parts[5].kind, DateTimePartKind::Literal);
        assert_eq!(parts.parts[5].value, " ");
    }
    let parts = format(
        request(
            "en-US",
            DateTimeStyleSelection::Styles(
                DateTimeStyles::new(None, Some(DateTimeStyle::Medium)).unwrap(),
            ),
        ),
        instant(9_306, 0),
    );
    assert_eq!(parts.to_formatted_string(), "2:35:06 AM");
}

#[test]
fn context_free_midnight_uses_the_general_period_and_preserves_exact_noon() {
    for (width, noon) in [
        (DateTimeTextWidth::Short, "noon"),
        (DateTimeTextWidth::Long, "noon"),
        (DateTimeTextWidth::Narrow, "n"),
    ] {
        for (seconds, nanos, expected) in [
            (0, 0, "in the morning"),
            (0, 1, "in the morning"),
            (43_199, 999_999_999, "in the morning"),
            (43_200, 0, noon),
            (43_200, 1, "in the afternoon"),
            (64_800, 0, "in the evening"),
            (75_600, 0, "at night"),
        ] {
            let fields = DateTimeComponents {
                day_period: Some(width),
                ..Default::default()
            };
            let parts = format(
                request("en", DateTimeStyleSelection::Components(fields)),
                instant(seconds, nanos),
            );
            assert_eq!(
                values(&parts),
                [("dayPeriod", expected)],
                "{width:?}/{seconds}/{nanos}"
            );
        }
    }
    // The lower-level LDML b domain falls back to AM rather than to a flexible
    // name; both b and B leave the ordinary a domain unchanged.
    use super::super::{
        pattern::{Field, NameWidth, Pattern, PeriodKind},
        plan, render,
    };
    let input = request("en", DateTimeStyleSelection::Components(time_components()));
    let selected = plan::select(&provider().profile, &input).unwrap();
    for (kind, seconds, expected) in [
        (PeriodKind::AmPm, 0, "AM"),
        (PeriodKind::NoonMidnight, 0, "AM"),
        (PeriodKind::NoonMidnight, 43_200, "noon"),
        (PeriodKind::Flexible, 0, "in the morning"),
    ] {
        let prepared = render::prepare(&selected, instant(seconds, 0), zones()).unwrap();
        let pattern = Pattern::single(Field::DayPeriod {
            kind,
            width: NameWidth::Abbreviated,
        });
        let parts = render::pattern(&provider().profile, &selected, &pattern, &prepared).unwrap();
        assert_eq!(values(&parts), [("dayPeriod", expected)]);
    }
}

#[test]
fn numeric_chinese_dates_preserve_related_year_and_style_year_names() {
    for (seconds, year, month, day) in [
        (-2_208_988_800, "1899", "12", "1"),
        (946_684_800, "1999", "11", "25"),
        (4_102_444_800, "2099", "11", "21"),
    ] {
        let parts = format(
            request(
                "en-US-u-ca-chinese",
                DateTimeStyleSelection::Components(date_components()),
            ),
            instant(seconds, 0),
        );
        let fields = parts
            .parts
            .iter()
            .filter(|part| part.kind != DateTimePartKind::Literal)
            .map(|part| (part.kind.as_str(), part.value.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            [("month", month), ("day", day), ("relatedYear", year)]
        );
    }
    for locale in ["en-US-u-ca-chinese", "zh-u-ca-chinese"] {
        let parts = format(
            request(
                locale,
                DateTimeStyleSelection::Styles(
                    DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
                ),
            ),
            instant(946_684_800, 0),
        );
        assert!(parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::RelatedYear && part.value == "1999"));
        assert!(parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::YearName && !part.value.is_empty()));
        assert!(!parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::Year));
    }
}
