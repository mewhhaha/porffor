use lila_intl::*;

const LOCALES: [&str; 13] = [
    "en",
    "en-US",
    "ar",
    "ar-EG",
    "zh",
    "zh-Hans",
    "zh-Hans-CN",
    "de",
    "fr",
    "it",
    "ja",
    "ko",
    "hi",
];

fn resolve(provider: &EmbeddedIntlProvider, locale: &str, calendar: &str) -> DateTimeLocaleResult {
    <EmbeddedIntlProvider as IntlOperationProvider<ResolveDateTimeLocale>>::execute(
        provider,
        DateTimeLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data(locale).unwrap()],
            matcher: DateTimeLocaleMatcher::Lookup,
            calendar: Some(DateTimeKeyword::parse(calendar).unwrap()),
            numbering_system: Some(DateTimeKeyword::parse("latn").unwrap()),
            hour_cycle: DateTimeHourCyclePreference::Default,
        },
    )
    .unwrap()
}

fn parts(
    provider: &EmbeddedIntlProvider,
    locale: DateTimeLocaleResult,
    iso: (i32, u8, u8),
    month: DateTimeMonthWidth,
) -> DateTimeParts {
    let request = DateTimePlanRequest {
        locale,
        time_zone: TimeZoneSelection::Named(TimeZoneId::parse("UTC").unwrap()),
        selection: DateTimeStyleSelection::Components(DateTimeComponents {
            year: Some(DateTimeNumericWidth::Numeric),
            month: Some(month),
            day: Some(DateTimeNumericWidth::Numeric),
            ..Default::default()
        }),
        matcher: DateTimeFormatMatcher::Basic,
        required: DateTimeRequired::Date,
        defaults: DateTimeDefaults::Date,
    };
    assert_eq!(
        DateTimePlanRequest::decode(&request.encode().unwrap()).unwrap(),
        request
    );
    let plan = <EmbeddedIntlProvider as IntlOperationProvider<SelectDateTimeFormat>>::execute(
        provider, request,
    )
    .unwrap();
    assert_eq!(
        DateTimePlanResult::decode(&plan.encode().unwrap()).unwrap(),
        plan
    );
    let input = DateTimeInput::Plain(
        DateTimePlainInput::new(
            DateTimeValueKind::PlainDate,
            DateTimeIsoFields {
                year: iso.0,
                month: iso.1,
                day: iso.2,
                hour: 12,
                minute: 0,
                second: 0,
                nanosecond: 0,
            },
        )
        .unwrap(),
    );
    <EmbeddedIntlProvider as IntlOperationProvider<FormatDateTimeParts>>::execute(
        provider,
        DateTimeFormatRequest {
            plan: plan.plan,
            input,
        },
    )
    .unwrap()
}

fn value(parts: &DateTimeParts, kind: DateTimePartKind) -> &str {
    let mut values = parts.parts.iter().filter(|part| part.kind == kind);
    let selected = values
        .next()
        .expect("required independently expected field");
    assert!(values.next().is_none(), "duplicate single-date field");
    &selected.value
}

#[test]
fn all208_public_associations_select_and_format_without_calendar_tag_fallback() {
    let provider = EmbeddedIntlProvider::new().unwrap();
    let mut associations = 0;
    for locale in LOCALES {
        for &calendar in DateTimeCalendar::ALL {
            let selected = resolve(&provider, locale, calendar.as_str());
            assert_eq!(
                selected.calendar,
                calendar,
                "{locale} {}",
                calendar.as_str()
            );
            assert_eq!(
                DateTimeLocaleResult::decode(&selected.encode().unwrap()).unwrap(),
                selected
            );
            let result = parts(
                &provider,
                selected,
                (1970, 1, 2),
                DateTimeMonthWidth::Numeric,
            );
            assert!(result
                .parts
                .iter()
                .any(|part| part.kind == DateTimePartKind::Month));
            assert!(result
                .parts
                .iter()
                .any(|part| part.kind == DateTimePartKind::Day));
            assert_eq!(
                DateTimeParts::decode(&result.encode().unwrap()).unwrap(),
                result
            );
            associations += 1;
        }
    }
    assert_eq!(associations, 208);
    let request = DateTimeSupportedLocalesRequest {
        requested: LOCALES
            .into_iter()
            .map(|locale| CanonicalLocaleId::from_data(locale).unwrap())
            .collect(),
        matcher: DateTimeLocaleMatcher::Lookup,
    };
    let supported =
        <EmbeddedIntlProvider as IntlOperationProvider<SupportedDateTimeLocales>>::execute(
            &provider,
            request.clone(),
        )
        .unwrap();
    assert_eq!(supported.locales, request.requested);
}

#[test]
fn independently_pinned_calendar_dates_reach_public_parts_with_genuine_names() {
    let provider = EmbeddedIntlProvider::new().unwrap();
    // Hebrew vectors are ICU4X's authored ISO vectors, already verified by the
    // retained eight-control foundation. Names are genuine CLDR47 en/root.
    for (iso, year, month, day) in [
        ((2022, 2, 25), "5782", "Adar I", "24"),
        ((2022, 3, 25), "5782", "Adar II", "22"),
        ((2021, 2, 25), "5781", "Adar", "13"),
    ] {
        let result = parts(
            &provider,
            resolve(&provider, "en", "hebrew"),
            iso,
            DateTimeMonthWidth::Long,
        );
        assert_eq!(value(&result, DateTimePartKind::Year), year);
        assert_eq!(value(&result, DateTimePartKind::Month), month);
        assert_eq!(value(&result, DateTimePartKind::Day), day);
    }
    // Exact independent vendor examples: coptic.rs module documentation,
    // ethiopian.rs::test_leap_year, hijri.rs::test_ummalqura_regression.
    for (calendar, iso, year, month, day) in [
        ("coptic", (1970, 1, 2), "1686", "4", "24"),
        ("ethiopic", (2023, 9, 11), "2015", "13", "6"),
        ("islamic-umalqura", (2011, 4, 4), "1432", "4", "30"),
    ] {
        let result = parts(
            &provider,
            resolve(&provider, "en", calendar),
            iso,
            DateTimeMonthWidth::Numeric,
        );
        assert_eq!(value(&result, DateTimePartKind::Year), year);
        assert_eq!(value(&result, DateTimePartKind::Month), month);
        assert_eq!(value(&result, DateTimePartKind::Day), day);
    }
}

#[test]
fn genuine_whole_value_aliases_canonicalize_options_without_entering_the_catalogue() {
    let provider = EmbeddedIntlProvider::new().unwrap();
    for (alias, expected) in [
        ("islamicc", DateTimeCalendar::IslamicCivil),
        ("ethiopic-amete-alem", DateTimeCalendar::Ethioaa),
    ] {
        assert_eq!(resolve(&provider, "en", alias).calendar, expected);
    }
    let catalogue = embedded_supported_values(SupportedValuesKey::Calendar).unwrap();
    assert!(catalogue.values().iter().all(|value| ![
        "islamicc",
        "ethiopic-amete-alem",
        "gregorian"
    ]
    .contains(&value.as_ref())));
    // A compound keyword is a distinct complete value, not an alias prefix.
    assert_eq!(
        resolve(&provider, "en", "islamicc-foo").calendar,
        DateTimeCalendar::Gregorian
    );
}
