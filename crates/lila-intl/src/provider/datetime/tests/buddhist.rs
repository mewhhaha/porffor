use super::*;

#[test]
fn buddhist_projection_keeps_solar_dates_and_its_single_signed_era() {
    use super::super::calendar::{convert, pinned_kernels, CalendarYear, EraName};
    let kernels = pinned_kernels();
    for (iso_year, month, day, buddhist_year) in [
        (1970, 1, 2, 2513),
        (2024, 2, 29, 2567),
        (-543, 1, 2, 0),
        (-544, 1, 2, -1),
    ] {
        let fields = DateTimeIsoFields {
            year: iso_year,
            month,
            day,
            hour: 12,
            minute: 3,
            second: 4,
            nanosecond: 5,
        };
        let converted = convert(DateTimeCalendar::Buddhist, fields, kernels).unwrap();
        assert_eq!(
            converted.year,
            CalendarYear::Era {
                name: EraName::Buddhist,
                year: buddhist_year
            }
        );
        assert_eq!(
            (
                converted.month.formatting().number(),
                converted.day,
                converted.month.formatting().is_leap()
            ),
            (month, day, false)
        );
        assert_eq!(
            (
                converted.hour,
                converted.minute,
                converted.second,
                converted.nanosecond
            ),
            (12, 3, 4, 5)
        );
        assert_eq!(
            converted.weekday,
            convert(DateTimeCalendar::Gregorian, fields, kernels)
                .unwrap()
                .weekday
        );
    }
}

#[test]
fn buddhist_patterns_keep_pinned_localized_eras_and_nonpositive_year_formatting() {
    let long = DateTimeStyleSelection::Styles(
        DateTimeStyles::new(Some(DateTimeStyle::Long), None).unwrap(),
    );
    for (locale, expected) in [
        ("en-US-u-ca-buddhist", "January 2, 2513 BE"),
        ("zh-u-ca-buddhist", "佛历2513年1月2日"),
    ] {
        assert_eq!(
            format(request(locale, long.clone()), date(1970, 1, 2)).to_formatted_string(),
            expected
        );
    }
    let fields = DateTimeComponents {
        era: Some(DateTimeTextWidth::Long),
        year: Some(DateTimeNumericWidth::Numeric),
        ..Default::default()
    };
    let parts = format(
        request(
            "ar-EG-u-ca-buddhist",
            DateTimeStyleSelection::Components(fields.clone()),
        ),
        date(1970, 1, 2),
    );
    assert!(values(&parts).contains(&("year", "٢٥١٣")));
    assert!(values(&parts).contains(&("era", "التقويم البوذي")));
    for (iso_year, numeric, two_digit) in
        [(-543, "1", "01"), (-544, "2", "02"), (-643, "101", "01")]
    {
        for (width, expected) in [
            (DateTimeNumericWidth::Numeric, numeric),
            (DateTimeNumericWidth::TwoDigit, two_digit),
        ] {
            let fields = DateTimeComponents {
                year: Some(width),
                era: Some(DateTimeTextWidth::Short),
                ..Default::default()
            };
            let parts = format(
                request(
                    "en-US-u-ca-buddhist",
                    DateTimeStyleSelection::Components(fields),
                ),
                date(iso_year, 1, 2),
            );
            assert!(
                values(&parts).contains(&("year", expected)),
                "ISO {iso_year}"
            );
            assert!(values(&parts).contains(&("era", "BE")));
        }
    }
}

#[test]
fn buddhist_locale_extension_and_explicit_override_select_actual_profiles() {
    let selected = provider()
        .resolve_locale(locale_request(&["en-US-u-ca-buddhist"]))
        .unwrap();
    assert_eq!(selected.calendar, DateTimeCalendar::Buddhist);
    assert_eq!(selected.locale.as_str(), "en-US-u-ca-buddhist");
    let mut input = locale_request(&["en-US-u-ca-buddhist"]);
    input.calendar = Some(DateTimeKeyword::parse("gregory").unwrap());
    let selected = provider().resolve_locale(input).unwrap();
    assert_eq!(selected.calendar, DateTimeCalendar::Gregorian);
    assert_eq!(selected.locale.as_str(), "en-US");
    let mut input = locale_request(&["en-US-u-ca-gregory"]);
    input.calendar = Some(DateTimeKeyword::parse("buddhist").unwrap());
    assert_eq!(
        provider().resolve_locale(input).unwrap().calendar,
        DateTimeCalendar::Buddhist
    );
    assert_eq!(
        provider()
            .resolve_locale(locale_request(&["en-US-u-ca-chinese"]))
            .unwrap()
            .calendar,
        DateTimeCalendar::Chinese
    );
}

#[test]
fn buddhist_range_sources_and_exact_terminal_instants_keep_calendar_projection() {
    let selection = DateTimeStyleSelection::Components(date_components());
    let parts = range(
        request("en-US-u-ca-buddhist", selection.clone()),
        date(1970, 1, 2),
        date(1971, 1, 2),
    );
    assert!(parts
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Year
            && part.value == "2513"
            && part.source == DateTimeRangeSource::StartRange));
    assert!(parts
        .parts
        .iter()
        .any(|part| part.kind == DateTimePartKind::Year
            && part.value == "2514"
            && part.source == DateTimeRangeSource::EndRange));
    for (seconds, nanosecond, expected_year, expected_day) in [
        (-1, 999_999_999, "2512", "31"),
        (-8_640_000_000_000, 0, "271279", "20"),
        (8_640_000_000_000, 0, "276303", "13"),
    ] {
        let parts = format(
            request("en-US-u-ca-buddhist", selection.clone()),
            instant(seconds, nanosecond),
        );
        assert!(values(&parts).contains(&("year", expected_year)));
        assert!(values(&parts).contains(&("day", expected_day)));
    }
}

#[test]
fn incomplete_buddhist_era_or_recipe_cannot_publish_a_profile() {
    use super::super::profile::Profile;
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../generated/profile.json")).unwrap();
    let mut incomplete = original.clone();
    let names = calendar_record_mut(&mut incomplete, "buddhist")["names"]
        .as_array_mut()
        .unwrap();
    let before = names.len();
    names.retain(|row| !(row["kind"] == "era" && row["width"] == "wide" && row["index"] == 0));
    assert_eq!(names.len(), before - 1);
    assert!(matches!(
        Profile::from_json(&incomplete.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut extra_era = original.clone();
    calendar_record_mut(&mut extra_era, "buddhist")["names"]
        .as_array_mut().unwrap()
        .push(serde_json::json!({"kind":"era", "context":null, "width":"wide", "index":1, "period":null, "value":"false second era"}));
    assert!(matches!(
        Profile::from_json(&extra_era.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
    let mut incomplete = original;
    incomplete["locales"][0]["calendar_refs"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row[0] != "buddhist");
    assert!(matches!(
        Profile::from_json(&incomplete.to_string()),
        Err(DateTimeFormatError::InvalidProfile(_))
    ));
}
