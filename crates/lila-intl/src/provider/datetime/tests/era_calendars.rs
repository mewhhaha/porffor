use super::*;

fn fields(locale: &str, input: DateTimeInput) -> Vec<(String, String)> {
    format(
        request(
            locale,
            DateTimeStyleSelection::Components(date_components()),
        ),
        input,
    )
    .parts
    .into_iter()
    .filter(|part| part.kind != DateTimePartKind::Literal)
    .map(|part| (part.kind.as_str().to_owned(), part.value))
    .collect()
}

fn sorted(mut rows: Vec<(String, String)>) -> Vec<(String, String)> {
    rows.sort();
    rows
}

fn expected(rows: &[(&str, &str)]) -> Vec<(String, String)> {
    sorted(
        rows.iter()
            .map(|(kind, value)| ((*kind).to_owned(), (*value).to_owned()))
            .collect(),
    )
}

#[test]
fn solar_era_calendars_use_pinned_arithmetic_and_cldr_era_names() {
    // 2022-12-24 is 3 Dey 1401 AP, 3 Pausa 1944 Saka, 2565 BE and Minguo 111.
    for (locale, rows) in [
        (
            "en-US-u-ca-buddhist",
            [
                ("era", "BE"),
                ("year", "2565"),
                ("month", "12"),
                ("day", "24"),
            ],
        ),
        (
            "en-US-u-ca-persian",
            [
                ("era", "AP"),
                ("year", "1401"),
                ("month", "10"),
                ("day", "3"),
            ],
        ),
        (
            "en-US-u-ca-indian",
            [
                ("era", "Saka"),
                ("year", "1944"),
                ("month", "10"),
                ("day", "3"),
            ],
        ),
        (
            "en-US-u-ca-roc",
            [
                ("era", "Minguo"),
                ("year", "111"),
                ("month", "12"),
                ("day", "24"),
            ],
        ),
    ] {
        assert_eq!(
            sorted(fields(locale, date(2022, 12, 24))),
            expected(&rows),
            "{locale}"
        );
    }
    // Years before the ROC epoch count backwards in the B.R.O.C. era.
    assert_eq!(
        sorted(fields("en-US-u-ca-roc", date(1900, 1, 1))),
        expected(&[
            ("era", "B.R.O.C."),
            ("year", "12"),
            ("month", "1"),
            ("day", "1")
        ])
    );
}

#[test]
fn resolved_calendars_keep_their_canonical_identifier() {
    for calendar in DateTimeCalendar::ALL {
        let tag = format!("en-US-u-ca-{}", calendar.as_str());
        let result = provider()
            .resolve_locale(locale_request(&[tag.as_str()]))
            .unwrap();
        assert_eq!(result.calendar, *calendar, "{tag}");
    }
    // Region preferences never replace the pinned locale defaults.
    for locale in [
        "en",
        "en-US",
        "ar",
        "ar-EG",
        "zh",
        "zh-Hans",
        "zh-Hans-CN",
        "de",
        "de-DE",
        "ja",
    ] {
        let result = provider()
            .resolve_locale(locale_request(&[locale]))
            .unwrap();
        assert_eq!(result.calendar, DateTimeCalendar::Gregorian, "{locale}");
    }
    let mut request = locale_request(&["en"]);
    request.calendar = Some(DateTimeKeyword::parse("islamicc").unwrap());
    assert_eq!(
        provider().resolve_locale(request).unwrap().calendar,
        DateTimeCalendar::IslamicCivil
    );
}

#[test]
fn islamic_civil_uses_icu_eras_and_retains_localized_ah_names() {
    let selection = DateTimeStyleSelection::Components(DateTimeComponents {
        era: Some(DateTimeTextWidth::Long),
        year: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    });
    let era = |locale, year| {
        format(request(locale, selection.clone()), date(year, 6, 15))
            .parts
            .into_iter()
            .find(|part| part.kind == DateTimePartKind::Era)
            .expect("requested Islamic era")
            .value
    };
    assert_eq!(era("en-u-ca-islamic-civil", 600), "BH");
    assert_eq!(era("en-u-ca-islamic-civil", 2025), "AH");
    assert_eq!(era("ar-u-ca-islamic-civil", 2025), "هـ");
}

#[test]
fn era_boundaries_cover_every_supported_calendar() {
    let selection = DateTimeStyleSelection::Components(DateTimeComponents {
        era: Some(DateTimeTextWidth::Long),
        year: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    });
    let era = |calendar, year| {
        format(
            request(&format!("en-u-ca-{calendar}"), selection.clone()),
            date(year, 6, 15),
        )
        .parts
        .into_iter()
        .find(|part| part.kind == DateTimePartKind::Era)
        .expect("era calendar must produce the requested era")
        .value
    };
    for (calendar, years) in [
        ("gregory", &[-100, 2025][..]),
        ("islamic-tbla", &[600, 2025][..]),
        ("islamic-umalqura", &[600, 2025][..]),
        ("japanese", &[-100, 1850, 1880, 1920, 1930, 1990, 2025][..]),
    ] {
        let values: std::collections::BTreeSet<_> =
            years.iter().map(|year| era(calendar, *year)).collect();
        assert_eq!(values.len(), years.len(), "{calendar}");
        assert!(values.iter().all(|value| !value.is_empty()));
    }
    for (calendar, before) in [("coptic", 250), ("ethioaa", -5550), ("hebrew", -3800)] {
        assert_eq!(era(calendar, before), era(calendar, 2025), "{calendar}");
    }
    assert_eq!(era("ethiopic", -6000), era("ethiopic", 0));
    assert_ne!(era("ethiopic", 0), era("ethiopic", 2025));
}

#[test]
fn hebrew_leap_month_names_distinguish_adar_one_and_two() {
    let selection = DateTimeStyleSelection::Components(DateTimeComponents {
        month: Some(DateTimeMonthWidth::Long),
        ..Default::default()
    });
    let month = |year, month, day| {
        format(
            request("en-u-ca-hebrew", selection.clone()),
            date(year, month, day),
        )
        .parts
        .into_iter()
        .find(|part| part.kind == DateTimePartKind::Month)
        .unwrap()
        .value
    };
    assert_eq!(month(2024, 2, 15), "Adar I");
    assert_eq!(month(2024, 3, 15), "Adar II");
    assert_eq!(month(2023, 3, 1), "Adar");
}

#[test]
fn japanese_first_era_year_uses_the_pinned_year_numbering_rule() {
    let selection = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Full), None).unwrap(),
    );
    let first = format(
        request("ja-u-ca-japanese", selection.clone()),
        date(2019, 5, 1),
    );
    let second = format(request("ja-u-ca-japanese", selection), date(2020, 5, 1));
    assert!(first
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Year && part.value == "元"));
    assert!(second
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Year && part.value == "2"));
}

#[test]
fn japanese_era_switch_uses_the_gregorian_adoption_boundary() {
    let selection = DateTimeStyleSelection::Components(DateTimeComponents {
        era: Some(DateTimeTextWidth::Long),
        year: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    });
    let parts = |year, month, day| {
        format(
            request("en-u-ca-japanese", selection.clone()),
            date(year, month, day),
        )
        .parts
    };
    let before = parts(1872, 12, 31);
    assert!(before
        .iter()
        .any(|p| p.kind == DateTimePartKind::Era && p.value == "CE"));
    assert!(before
        .iter()
        .any(|p| p.kind == DateTimePartKind::Year && p.value == "1872"));
    let after = parts(1873, 1, 1);
    assert!(after
        .iter()
        .any(|p| p.kind == DateTimePartKind::Era && p.value == "Meiji"));
    assert!(after
        .iter()
        .any(|p| p.kind == DateTimePartKind::Year && p.value == "6"));
}
