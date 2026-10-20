//! Adapter regressions derived from the actual pinned ICU foundation8/8.
//! These tests do not assert a universal date range or profile admission.
use icu_calendar::{types::MonthCode as NativeMonthCode, Date};

use super::{
    kind::{CalendarId, EraName},
    month::Month,
    projection::{CalendarProjection, CalendarYear, CodeYear},
};
use crate::datetime::DateTimeIsoFields;

fn iso((year, month, day): (i32, u8, u8)) -> DateTimeIsoFields {
    DateTimeIsoFields {
        year,
        month,
        day,
        hour: 12,
        minute: 0,
        second: 0,
        nanosecond: 0,
    }
}

fn project(calendar: CalendarId, input: (i32, u8, u8)) -> CalendarProjection {
    CalendarProjection::from_iso(calendar, iso(input), super::pinned_kernels()).unwrap()
}

fn code(value: &str) -> NativeMonthCode {
    NativeMonthCode(value.parse().unwrap())
}

fn era(calendar: CalendarId, input: (i32, u8, u8), name: EraName, year: i32) {
    assert_eq!(
        project(calendar, input).year(),
        CalendarYear::Era { name, year }
    );
}

#[test]
fn native_calendar_domain_is_exact_and_has_no_default_or_deprecated_identity() {
    let expected = [
        "buddhist",
        "chinese",
        "coptic",
        "dangi",
        "ethioaa",
        "ethiopic",
        "gregory",
        "hebrew",
        "indian",
        "islamic-civil",
        "islamic-tbla",
        "islamic-umalqura",
        "iso8601",
        "japanese",
        "persian",
        "roc",
    ];
    assert_eq!(CalendarId::ALL.map(CalendarId::as_str), expected);
    for calendar in CalendarId::ALL {
        assert_eq!(
            CalendarId::from_canonical(calendar.as_str()),
            Some(calendar)
        );
        assert_eq!(calendar.new().kind(), calendar.kind());
        let public = crate::datetime::DateTimeCalendar::parse(calendar.as_str()).unwrap();
        assert_eq!(public.as_str(), calendar.as_str());
        assert_eq!(CalendarId::from_admitted(public), calendar);
    }
    for unknown in [
        "",
        "islamic",
        "islamic-rgsa",
        "islamicc",
        "ethiopic-amete-alem",
        "Gregorian",
    ] {
        assert!(CalendarId::from_canonical(unknown).is_none());
    }
    assert_eq!(
        crate::datetime::DateTimeCalendar::ALL.len(),
        CalendarId::ALL.len()
    );
    assert_ne!(CalendarId::Chinese.kind(), CalendarId::Dangi.kind());
    assert_ne!(
        CalendarId::IslamicCivil.kind(),
        CalendarId::IslamicTbla.kind()
    );
    assert_ne!(
        CalendarId::IslamicCivil.kind(),
        CalendarId::IslamicUmalqura.kind()
    );
}

#[test]
fn every_native_projection_preserves_modern_days_and_correct_code_constructor_year() {
    for calendar in CalendarId::ALL {
        for input in [(1970, 1, 1), (2000, 2, 29), (2025, 6, 15)] {
            let value = project(calendar, input);
            let rebuilt = Date::try_new_from_codes(
                None,
                value.code_year().value(),
                value.month().standard().native(),
                value.day(),
                calendar.new(),
            )
            .unwrap()
            .to_iso();
            assert_eq!(
                (
                    rebuilt.extended_year(),
                    rebuilt.month().ordinal,
                    rebuilt.day_of_month().0
                ),
                input
            );
            assert_eq!(value.weekday(), rebuilt.day_of_week() as u8 % 7);
            assert_eq!(
                matches!(value.code_year(), CodeYear::RelatedIso(_)),
                matches!(calendar, CalendarId::Chinese | CalendarId::Dangi)
            );
        }
    }
}

#[test]
fn signed_single_era_years_and_ethiopian_distinct_year_counts_are_preserved() {
    use CalendarId as C;
    for (calendar, input, name, year) in [
        (C::Buddhist, (-600, 6, 15), EraName::Buddhist, -57),
        (C::Coptic, (250, 6, 15), EraName::Coptic, -34),
        (C::Ethioaa, (-5550, 6, 15), EraName::AmeteAlem, -58),
        (C::Hebrew, (-3800, 6, 15), EraName::Hebrew, -40),
        (C::Indian, (70, 6, 15), EraName::Shaka, -8),
        (C::Persian, (600, 6, 15), EraName::Persian, -21),
        (C::Ethiopic, (-6000, 6, 15), EraName::AmeteAlem, -508),
        (C::Ethiopic, (0, 6, 15), EraName::AmeteAlem, 5492),
        (C::Ethiopic, (2025, 6, 15), EraName::AmeteMihret, 2017),
    ] {
        era(calendar, input, name, year);
    }
    let earlier = project(C::Ethiopic, (0, 6, 15));
    assert_eq!(earlier.extended_year(), -8);
    assert_eq!(earlier.code_year(), CodeYear::Extended(-8));
    assert!(EraName::from_native(C::Coptic, "bce").is_err());
    assert!(EraName::from_native(C::Gregory, "default").is_err());
}

#[test]
fn japanese_uses_gregorian_eras_through_1872_and_exact_modern_boundaries() {
    use EraName::*;
    for (input, name, year) in [
        ((-100, 6, 15), BeforeCommon, 101),
        ((0, 1, 1), BeforeCommon, 1),
        ((1, 1, 1), Common, 1),
        ((1868, 10, 22), Common, 1868),
        ((1868, 10, 23), Common, 1868),
        ((1872, 12, 31), Common, 1872),
        ((1873, 1, 1), Meiji, 6),
        ((1912, 7, 29), Meiji, 45),
        ((1912, 7, 30), Taisho, 1),
        ((1926, 12, 24), Taisho, 15),
        ((1926, 12, 25), Showa, 1),
        ((1989, 1, 7), Showa, 64),
        ((1989, 1, 8), Heisei, 1),
        ((2019, 4, 30), Heisei, 31),
        ((2019, 5, 1), Reiwa, 1),
    ] {
        era(CalendarId::Japanese, input, name, year);
    }
    for (input, name, year) in [
        ((-100, 6, 15), BeforeCommon, 101),
        ((0, 1, 1), BeforeCommon, 1),
        ((1, 1, 1), Common, 1),
    ] {
        era(CalendarId::Iso8601, input, name, year);
    }
}

#[test]
fn hebrew_standard_formatting_and_ordinal_months_do_not_collapse() {
    for (input, year, ordinal, standard, formatting, day) in [
        ((2022, 2, 25), 5782, 6, "M05L", "M05L", 24),
        ((2022, 3, 25), 5782, 7, "M06", "M06L", 22),
        ((2021, 2, 25), 5781, 6, "M06", "M06", 13),
        ((2024, 9, 4), 5784, 13, "M12", "M12", 1),
    ] {
        let value = project(CalendarId::Hebrew, input);
        assert_eq!(value.extended_year(), year);
        assert_eq!(value.month().ordinal(), ordinal);
        assert_eq!(value.month().standard().native(), code(standard));
        assert_eq!(value.month().formatting().native(), code(formatting));
        assert_eq!(value.day(), day);
    }
    assert!(!project(CalendarId::Hebrew, (2022, 3, 25))
        .month()
        .standard()
        .is_leap());
    assert!(project(CalendarId::Hebrew, (2022, 3, 25))
        .month()
        .formatting()
        .is_leap());
}

#[test]
fn coptic_and_both_ethiopian_algorithms_keep_the_real_thirteenth_month() {
    use CalendarId as C;
    for (calendar, year, input) in [
        (C::Coptic, 2022, (2306, 9, 13)),
        (C::Coptic, 2023, (2307, 9, 13)),
        (C::Ethioaa, 2022, (-3470, 7, 31)),
        (C::Ethioaa, 2023, (-3469, 7, 31)),
        (C::Ethiopic, 2022, (2030, 9, 10)),
        (C::Ethiopic, 2023, (2031, 9, 10)),
    ] {
        let value = project(calendar, input);
        assert_eq!(value.extended_year(), year);
        assert_eq!(value.month().ordinal(), 13);
        assert_eq!(value.month().standard().native(), code("M13"));
        assert_eq!(value.month().formatting().native(), code("M13"));
        assert_eq!(value.day(), 5);
    }
}

#[test]
fn islamic_epochs_before_hijrah_and_ummalqura_cache_anchors_use_real_algorithms() {
    use CalendarId as C;
    for (calendar, input) in [
        (C::IslamicCivil, (622, 7, 19)),
        (C::IslamicTbla, (622, 7, 18)),
    ] {
        era(calendar, input, EraName::Hijrah, 1);
        let value = project(calendar, input);
        assert_eq!(value.month().standard().native(), code("M01"));
        assert_eq!(value.day(), 1);
    }
    era(C::IslamicCivil, (622, 7, 18), EraName::BeforeHijrah, 1);
    era(C::IslamicTbla, (622, 7, 17), EraName::BeforeHijrah, 1);
    era(C::IslamicUmalqura, (622, 7, 18), EraName::BeforeHijrah, 1);
    for (year, input) in [(1300, (1882, 11, 12)), (1600, (2173, 12, 7))] {
        let value = project(C::IslamicUmalqura, input);
        assert_eq!(value.extended_year(), year);
        assert_eq!(value.month().standard().native(), code("M01"));
        assert_eq!(value.day(), 1);
    }
    for year in [1299, 1300, 1301, 1599, 1600, 1601] {
        let native = Date::try_new_from_codes(None, year, code("M01"), 1, C::IslamicUmalqura.new())
            .unwrap()
            .to_iso();
        let input = (
            native.extended_year(),
            native.month().ordinal,
            native.day_of_month().0,
        );
        assert_eq!(
            project(C::IslamicUmalqura, input).code_year(),
            CodeYear::Extended(year)
        );
    }
}

#[test]
fn chinese_and_dangi_keep_related_iso_years_and_distinct_traditional_counts() {
    for (input, related, cyclic, chinese, dangi) in [
        ((1900, 1, 31), 1900, 37, 4537, 4233),
        ((2000, 1, 31), 1999, 16, 4636, 4332),
        ((2000, 6, 15), 2000, 17, 4637, 4333),
        ((2050, 1, 23), 2050, 7, 4687, 4383),
    ] {
        for (calendar, extended) in [(CalendarId::Chinese, chinese), (CalendarId::Dangi, dangi)] {
            let value = project(calendar, input);
            assert_eq!(
                value.year(),
                CalendarYear::Cyclic {
                    year: cyclic,
                    related_iso: related
                }
            );
            assert_eq!(value.extended_year(), extended);
            assert_eq!(value.code_year(), CodeYear::RelatedIso(related));
        }
    }
    for calendar in [CalendarId::Chinese, CalendarId::Dangi] {
        let leap = project(calendar, (2020, 5, 23));
        assert_eq!(leap.month().standard().native(), code("M04L"));
        assert_eq!(leap.month().formatting().native(), code("M04L"));
        assert_eq!(leap.month().ordinal(), 5);
    }
    // The actual foundation observes distinct month lengths for these algorithms.
    let iso = Date::try_new_iso(2050, 1, 23).unwrap();
    assert_eq!(
        iso.to_calendar(CalendarId::Chinese.new()).days_in_month(),
        29
    );
    assert_eq!(iso.to_calendar(CalendarId::Dangi.new()).days_in_month(), 30);
}

#[test]
fn native_month_boundary_rejects_unknown_codes_and_wrong_calendar_ordinals() {
    let date = Date::try_new_iso(2025, 1, 1)
        .unwrap()
        .to_calendar(CalendarId::Gregory.new());
    for (standard, formatting, ordinal, count) in [
        ("M01X", "M01", 1, 12),
        ("M01", "M01", 13, 12),
        ("M13", "M13", 13, 13),
        ("M01L", "M01L", 2, 12),
    ] {
        let mut native = date.month();
        native.standard_code = code(standard);
        native.formatting_code = code(formatting);
        native.ordinal = ordinal;
        assert!(Month::from_native(CalendarId::Gregory, native, count).is_err());
    }
    assert!(CalendarProjection::from_iso(
        CalendarId::Hebrew,
        iso((2025, 2, 30)),
        super::pinned_kernels()
    )
    .is_err());
}

#[derive(serde::Deserialize)]
struct Sample {
    calendar: String,
    input: (i32, u8, u8),
    extended_year: i32,
    ordinal: u8,
    standard_code: String,
    formatting_code: String,
    day: u8,
}

#[test]
fn eighty_actual_timeclip_temporal_and_yearmonth_samples_keep_observed_fields() {
    let samples: Vec<Sample> = serde_json::from_str(include_str!("sampled-bounds.json")).unwrap();
    assert_eq!(samples.len(), 80);
    for sample in samples {
        let calendar = CalendarId::from_canonical(&sample.calendar).unwrap();
        let value = project(calendar, sample.input);
        assert_eq!(value.extended_year(), sample.extended_year);
        assert_eq!(value.month().ordinal(), sample.ordinal);
        assert_eq!(
            value.month().standard().native(),
            code(&sample.standard_code)
        );
        assert_eq!(
            value.month().formatting().native(),
            code(&sample.formatting_code)
        );
        assert_eq!(value.day(), sample.day);
    }
}
