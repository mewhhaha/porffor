use super::*;

#[test]
fn locale_resolution_preserves_order_keyword_precedence_and_data_locale() {
    let mut input = locale_request(&["de-DE", "ar-EG-u-ca-chinese-hc-h23-nu-latn", "zh-Hans-CN"]);
    input.calendar = Some(DateTimeKeyword::parse("gregory").unwrap());
    input.numbering_system = Some(DateTimeKeyword::parse("latn").unwrap());
    let result = provider().resolve_locale(input).unwrap();
    assert_eq!(result.locale.as_str(), "ar-EG-u-hc-h23-nu-latn");
    assert_eq!(result.data_locale.as_str(), "ar-EG");
    assert_eq!(result.calendar, DateTimeCalendar::Gregorian);
    assert_eq!(result.numbering_system.as_str(), "latn");
    assert_eq!(result.hour_cycle, DateTimeHourCycle::H23);
    let mut input = locale_request(&["en-US-u-hc-h24"]);
    input.hour_cycle = DateTimeHourCyclePreference::TwelveHour;
    let result = provider().resolve_locale(input).unwrap();
    assert_eq!(result.locale.as_str(), "en-US");
    assert_eq!(result.hour_cycle, DateTimeHourCycle::H12);
}

#[test]
fn available_locale_inventory_and_supported_lists_are_distinct_from_default_fallback() {
    let defaults = provider()
        .resolve_locale(locale_request(&["de-DE"]))
        .unwrap();
    assert_eq!(defaults.data_locale.as_str(), "en-US");
    let result = provider()
        .supported_locales(DateTimeSupportedLocalesRequest {
            requested: locale_request(&["de-DE", "ar-EG-u-nu-arab", "zh-Hans-CN", "en-AU"])
                .requested,
            matcher: DateTimeLocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(
        result
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["ar-EG-u-nu-arab", "zh-Hans-CN", "en-AU"]
    );
    assert_eq!(
        provider()
            .resolve_locale(locale_request(&["ar"]))
            .unwrap()
            .numbering_system
            .as_str(),
        "latn"
    );
    assert_eq!(
        provider()
            .resolve_locale(locale_request(&["ar-EG"]))
            .unwrap()
            .numbering_system
            .as_str(),
        "arab"
    );
}

#[test]
fn original_options_drive_plain_availability_after_legacy_defaulting() {
    let components = DateTimeStyleSelection::Components(DateTimeComponents {
        hour: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    });
    let mut input = request("en-US", components);
    input.required = DateTimeRequired::Date;
    let selected = provider().select_plan(input).unwrap();
    for kind in [
        DateTimeValueKind::Legacy,
        DateTimeValueKind::Instant,
        DateTimeValueKind::PlainTime,
        DateTimeValueKind::PlainDateTime,
    ] {
        assert!(selected.available_formats.contains(kind));
    }
    for kind in [
        DateTimeValueKind::PlainDate,
        DateTimeValueKind::PlainYearMonth,
        DateTimeValueKind::PlainMonthDay,
    ] {
        assert!(!selected.available_formats.contains(kind));
    }
    assert_eq!(
        selected.components.year,
        Some(DateTimeNumericWidth::Numeric)
    );
    let error = provider().format_parts(
        DateTimeFormatRequest {
            plan: selected.plan,
            input: date(2020, 1, 25),
        },
        zones(),
    );
    assert_eq!(error, Err(DateTimeFormatError::UnavailableFormat));

    let input = request(
        "en-US",
        DateTimeStyleSelection::Components(Default::default()),
    );
    let selected = provider().select_plan(input).unwrap();
    assert_eq!(selected.available_formats.wire_code(), 31);
    assert!(selected.components.year.is_some() && selected.components.hour.is_none());
    for kind in [
        DateTimeValueKind::PlainYearMonth,
        DateTimeValueKind::PlainMonthDay,
    ] {
        let parts = provider()
            .format_parts(
                DateTimeFormatRequest {
                    plan: selected.plan.clone(),
                    input: plain(kind, 2020, 1, 25),
                },
                zones(),
            )
            .unwrap();
        let kinds = parts.parts.iter().map(|part| part.kind).collect::<Vec<_>>();
        assert!(kinds.contains(&DateTimePartKind::Month));
        assert_eq!(
            kinds.contains(&DateTimePartKind::Year),
            kind == DateTimeValueKind::PlainYearMonth
        );
        assert_eq!(
            kinds.contains(&DateTimePartKind::Day),
            kind == DateTimeValueKind::PlainMonthDay
        );
    }
}

#[test]
fn style_rejection_and_plain_filtering_follow_the_caller_and_value_kind() {
    let styles = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(None, Some(DateTimeStyle::Short)).unwrap(),
    );
    let selected = provider()
        .select_plan(request("en-US", styles.clone()))
        .unwrap();
    assert_eq!(
        provider().format_parts(
            DateTimeFormatRequest {
                plan: selected.plan,
                input: date(2020, 1, 25)
            },
            zones()
        ),
        Err(DateTimeFormatError::UnavailableFormat)
    );
    let mut input = request("en-US", styles);
    input.required = DateTimeRequired::Date;
    assert!(matches!(
        provider().select_plan(input),
        Err(DateTimeFormatError::InvalidRequest(_))
    ));
    let parts = format(
        request(
            "en-US",
            DateTimeStyleSelection::Styles(
                DateTimeStyles::new(Some(DateTimeStyle::Full), Some(DateTimeStyle::Full)).unwrap(),
            ),
        ),
        plain(DateTimeValueKind::PlainMonthDay, 2020, 1, 25),
    );
    assert_eq!(
        values(&parts),
        [("month", "January"), ("literal", " "), ("day", "25")]
    );
}

#[test]
fn era_only_defaults_preserve_numeric_plain_fields_and_append_localized_era() {
    for width in [
        DateTimeTextWidth::Short,
        DateTimeTextWidth::Long,
        DateTimeTextWidth::Narrow,
    ] {
        let fields = DateTimeStyleSelection::Components(DateTimeComponents {
            era: Some(width),
            ..Default::default()
        });
        for (kind, expected) in [
            (
                DateTimeValueKind::PlainDate,
                vec!["month", "day", "year", "era"],
            ),
            (
                DateTimeValueKind::PlainYearMonth,
                vec!["month", "year", "era"],
            ),
            (DateTimeValueKind::PlainMonthDay, vec!["month", "day"]),
            (
                DateTimeValueKind::PlainTime,
                vec!["hour", "minute", "second", "dayPeriod"],
            ),
        ] {
            let input = if kind == DateTimeValueKind::PlainTime {
                plain(kind, 1970, 1, 1)
            } else {
                plain(kind, 2025, 11, 4)
            };
            let parts = format(request("en", fields.clone()), input);
            let kinds = parts
                .parts
                .iter()
                .filter(|part| part.kind != DateTimePartKind::Literal)
                .map(|part| part.kind.as_str())
                .collect::<Vec<_>>();
            assert_eq!(kinds, expected, "{kind:?}/{width:?}");
            if kind != DateTimeValueKind::PlainTime {
                assert_eq!(parts.parts[0].value, "11", "{kind:?}/{width:?}");
            }
        }
    }
    for (locale, first) in [("ar", DateTimePartKind::Era), ("zh", DateTimePartKind::Era)] {
        let parts = format(
            request(
                locale,
                DateTimeStyleSelection::Components(DateTimeComponents {
                    era: Some(DateTimeTextWidth::Short),
                    year: Some(DateTimeNumericWidth::Numeric),
                    month: Some(DateTimeMonthWidth::Numeric),
                    ..Default::default()
                }),
            ),
            plain(DateTimeValueKind::PlainYearMonth, 2025, 11, 4),
        );
        assert_eq!(parts.parts[0].kind, first, "{locale}");
        assert!(!parts
            .parts
            .iter()
            .any(|part| part.kind == DateTimePartKind::Day));
    }
    let parts = format(
        request(
            "zh-u-ca-chinese",
            DateTimeStyleSelection::Components(DateTimeComponents {
                era: Some(DateTimeTextWidth::Narrow),
                ..Default::default()
            }),
        ),
        plain(DateTimeValueKind::PlainYearMonth, 2025, 11, 4),
    );
    assert!(!parts
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Era));
    assert!(parts
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::RelatedYear));
}
