use super::super::{
    calendar::{convert_kind, CalendarId, CalendarYear, EraName},
    names::{MonthYearType, NameKey},
    pattern::{NameContext, NameWidth},
};
use super::*;

fn profile() -> Profile {
    Profile::from_json(include_str!("../generated/profile.json")).unwrap()
}
fn iso(year: i32, month: u8, day: u8) -> DateTimeIsoFields {
    DateTimeIsoFields {
        year,
        month,
        day,
        hour: 12,
        minute: 3,
        second: 4,
        nanosecond: 567_000_000,
    }
}
fn selected<'p>(
    profile: &'p Profile,
    locale: &str,
    kind: CalendarId,
) -> (&'p Locale, &'p super::super::profile::Calendar) {
    let locale = profile
        .locales
        .iter()
        .find(|row| row.identifier.as_str() == locale)
        .unwrap();
    (locale, locale.foundation_calendar(kind).unwrap())
}
fn field(
    profile: &Profile,
    locale: &str,
    kind: CalendarId,
    date: DateTimeIsoFields,
    value: Field,
) -> String {
    let (locale, calendar) = selected(profile, locale, kind);
    let pattern = Pattern::new(vec![Token::Field(value)], vec![]);
    fields::format_fields(
        profile,
        locale,
        calendar,
        "latn",
        &pattern,
        value,
        convert_kind(kind, date).unwrap(),
    )
    .unwrap()
}

#[test]
fn every_genuine_208_profile_renders_its_sourced_fields_with_the_native_calendar() {
    let profile = profile();
    assert_eq!(profile.locales.len(), 13);
    let mut associations = 0;
    for locale in &profile.locales {
        assert_eq!(locale.calendars().count(), 16);
        for (kind, calendar) in locale.calendars() {
            associations += 1;
            let patterns = calendar
                .styles
                .iter()
                .flat_map(|style| [&style.date, &style.time])
                .chain(calendar.available.iter())
                .chain(calendar.intervals.iter().map(|interval| &interval.pattern));
            for pattern in patterns {
                for date in [
                    iso(2024, 7, 1),
                    iso(2020, 5, 23),
                    iso(2022, 2, 25),
                    iso(2022, 3, 25),
                    iso(2024, 9, 4),
                    iso(2019, 5, 1),
                ] {
                    let projected = convert_kind(kind, date).unwrap();
                    for token in &pattern.tokens {
                        if let Token::Field(value) = token {
                            if !matches!(value, Field::ZoneName(_)) {
                                let result = fields::format_fields(
                                    &profile,
                                    locale,
                                    calendar,
                                    &locale.default_numbering,
                                    pattern,
                                    *value,
                                    projected,
                                )
                                .unwrap_or_else(|error| {
                                    panic!(
                                        "{} {} {:?}: {error:?}",
                                        locale.identifier.as_str(),
                                        kind.as_str(),
                                        value
                                    )
                                });
                                assert!(!result.is_empty());
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(associations, 208);
}

#[test]
fn hebrew_common_and_leap_adars_use_real_distinct_names_without_cyclic_wrappers() {
    let profile = profile();
    let value = Field::Month {
        context: NameContext::Format,
        width: 4,
    };
    assert_eq!(
        field(&profile, "en", CalendarId::Hebrew, iso(2022, 2, 25), value),
        "Adar I"
    );
    assert_eq!(
        field(&profile, "en", CalendarId::Hebrew, iso(2022, 3, 25), value),
        "Adar II"
    );
    assert_eq!(
        field(&profile, "en", CalendarId::Hebrew, iso(2021, 2, 25), value),
        "Adar"
    );
    assert_eq!(
        field(&profile, "en", CalendarId::Hebrew, iso(2024, 9, 4), value),
        "Elul"
    );
    for width in [1, 2] {
        let numeric = Field::Month {
            context: NameContext::Format,
            width,
        };
        for (date, expected) in [
            (iso(2022, 2, 25), "Adar I"),
            (iso(2022, 3, 25), "Adar II"),
            (iso(2021, 2, 25), "Adar"),
            (iso(2022, 4, 25), "Nisan"),
            (iso(2021, 3, 25), "Nisan"),
            (iso(2024, 9, 4), "Elul"),
        ] {
            assert_eq!(
                field(&profile, "en", CalendarId::Hebrew, date, numeric),
                expected
            );
        }
    }
}

#[test]
fn coptic_and_both_ethiopian_domains_render_the_real_thirteenth_month() {
    let profile = profile();
    for (kind, date) in [
        (CalendarId::Coptic, iso(2306, 9, 13)),
        (CalendarId::Ethioaa, iso(-3470, 7, 31)),
        (CalendarId::Ethiopic, iso(2030, 9, 10)),
    ] {
        let projected = convert_kind(kind, date).unwrap();
        assert_eq!(projected.month.ordinal(), 13);
        assert!(!projected.month.uses_leap_placeholder(kind));
        assert_eq!(
            field(
                &profile,
                "en",
                kind,
                date,
                Field::Month {
                    context: NameContext::Format,
                    width: 1
                }
            ),
            "13"
        );
        let (_, calendar) = selected(&profile, "en", kind);
        assert_eq!(
            field(
                &profile,
                "en",
                kind,
                date,
                Field::Month {
                    context: NameContext::Format,
                    width: 4
                }
            ),
            calendar
                .names
                .get(NameKey::Month(
                    NameContext::Format,
                    NameWidth::Wide,
                    13,
                    None
                ))
                .unwrap()
        );
    }
}

#[test]
fn japanese_pre1873_names_are_gregorian_and_modern_names_use_native_era_keys() {
    let profile = profile();
    let value = Field::Era(NameWidth::Wide);
    for date in [iso(0, 6, 15), iso(1868, 10, 23), iso(1872, 12, 31)] {
        assert_eq!(
            field(&profile, "en", CalendarId::Japanese, date, value),
            field(&profile, "en", CalendarId::Gregory, date, value)
        );
    }
    assert_eq!(
        field(&profile, "en", CalendarId::Japanese, iso(1873, 1, 1), value),
        "Meiji"
    );
    assert_eq!(
        field(
            &profile,
            "en",
            CalendarId::Japanese,
            iso(2019, 4, 30),
            value
        ),
        "Heisei"
    );
    assert_eq!(
        field(&profile, "en", CalendarId::Japanese, iso(2019, 5, 1), value),
        "Reiwa"
    );
}

#[test]
fn japanese_year_algorithm_checks_full_year_and_uses_real_decimal_fallback() {
    let profile = profile();
    let (locale, calendar) = selected(&profile, "ja", CalendarId::Japanese);
    for width in [1, 2] {
        let value = Field::Year(width);
        let pattern = Pattern::new(
            vec![Token::Field(value)],
            vec![(Some('y'), "jpanyear".into())],
        );
        for (date, expected) in [
            (iso(2019, 5, 1), "元"),
            (iso(2020, 5, 1), "2"),
            (iso(2119, 5, 1), if width == 2 { "1" } else { "101" }),
        ] {
            assert_eq!(
                fields::format_fields(
                    &profile,
                    locale,
                    calendar,
                    "latn",
                    &pattern,
                    value,
                    convert_kind(CalendarId::Japanese, date).unwrap()
                )
                .unwrap(),
                expected
            );
        }
    }
    assert_eq!(
        field(
            &profile,
            "ja",
            CalendarId::Japanese,
            iso(2019, 5, 1),
            Field::Year(2)
        ),
        "01"
    );
}

#[test]
fn numeric_weekday_respects_sourced_first_day_context_width_and_numbering() {
    let profile = profile();
    for (locale, sunday, monday) in [("en", "1", "2"), ("de", "7", "1")] {
        let e = Field::from_ldml('e', 1).unwrap();
        let ee = Field::from_ldml('e', 2).unwrap();
        let cc = Field::from_ldml('c', 2).unwrap();
        assert_eq!(
            field(&profile, locale, CalendarId::Gregory, iso(2024, 7, 7), e),
            sunday
        );
        assert_eq!(
            field(&profile, locale, CalendarId::Gregory, iso(2024, 7, 8), e),
            monday
        );
        assert_eq!(
            field(&profile, locale, CalendarId::Gregory, iso(2024, 7, 7), ee),
            format!("0{sunday}")
        );
        assert_eq!(
            field(&profile, locale, CalendarId::Gregory, iso(2024, 7, 7), cc),
            sunday
        );
        assert_eq!(e.numbering_symbol(), Some('e'));
        assert_eq!(cc.numbering_symbol(), Some('c'));
    }
    let (locale, calendar) = selected(&profile, "ar", CalendarId::Gregory);
    let value = Field::from_ldml('e', 2).unwrap();
    let pattern = Pattern::new(vec![Token::Field(value)], vec![(Some('e'), "arab".into())]);
    assert_eq!(
        fields::format_fields(
            &profile,
            locale,
            calendar,
            "latn",
            &pattern,
            value,
            convert_kind(CalendarId::Gregory, iso(2024, 7, 7)).unwrap()
        )
        .unwrap(),
        "٠٢"
    );
}

#[test]
fn signed_single_era_years_keep_native_sign_until_numeric_formatting() {
    let profile = profile();
    for (kind, date, era, year) in [
        (
            CalendarId::Buddhist,
            iso(-600, 6, 15),
            EraName::Buddhist,
            -57,
        ),
        (CalendarId::Coptic, iso(250, 6, 15), EraName::Coptic, -34),
        (
            CalendarId::Ethioaa,
            iso(-5550, 6, 15),
            EraName::AmeteAlem,
            -58,
        ),
        (CalendarId::Hebrew, iso(-3800, 6, 15), EraName::Hebrew, -40),
        (CalendarId::Indian, iso(70, 6, 15), EraName::Shaka, -8),
        (CalendarId::Persian, iso(600, 6, 15), EraName::Persian, -21),
    ] {
        assert_eq!(
            convert_kind(kind, date).unwrap().year,
            CalendarYear::Era { name: era, year }
        );
        assert_eq!(
            field(&profile, "en", kind, date, Field::Year(1)),
            (1 - year).to_string()
        );
    }
}

#[test]
fn native_profile_constructor_rejects_missing_expanded_names_and_week_data() {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("../generated/profile.json")).unwrap();
    let mut missing = source.clone();
    missing["locales"][0]
        .as_object_mut()
        .unwrap()
        .remove("first_weekday");
    assert!(Profile::from_json(&missing.to_string()).is_err());
    let mut missing = source.clone();
    let pools = missing["calendar_pool"].as_array_mut().unwrap();
    let hebrew = pools
        .iter_mut()
        .find(|row| row["calendar"] == "hebrew")
        .unwrap();
    let before = hebrew["names"].as_array().unwrap().len();
    hebrew["names"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["year_type"] != "leap");
    assert!(hebrew["names"].as_array().unwrap().len() < before);
    assert!(Profile::from_json(&missing.to_string()).is_err());
    let mut missing = source;
    let japanese = missing["calendar_pool"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["calendar"] == "japanese")
        .unwrap();
    japanese["names"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["era_source_calendar"] != "gregorian");
    assert!(Profile::from_json(&missing.to_string()).is_err());
    // This compile-time key contains the actual leap discriminator.
    let _ = NameKey::Month(
        NameContext::Format,
        NameWidth::Wide,
        7,
        Some(MonthYearType::Leap),
    );
}
