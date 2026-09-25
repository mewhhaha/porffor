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
    ] {
        let result = provider()
            .resolve_locale(locale_request(&[locale]))
            .unwrap();
        assert_eq!(result.calendar, DateTimeCalendar::Gregorian, "{locale}");
    }
}
