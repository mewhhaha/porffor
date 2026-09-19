use super::*;
use crate::provider::time_zone_snapshot::NamedTimeZoneTransition;
use crate::{FixedTimeZoneOffset, NamedTimeZoneIdentity, TimeZoneEpochSeconds};

fn named(
    identifier: &str,
    primary: &str,
    epoch: i64,
    transition: NamedTimeZoneTransition,
    style: TimeZoneNameStyle,
) -> String {
    let identity = NamedTimeZoneIdentity::from_data(identifier, primary).unwrap();
    TimeZoneNames::from_pinned_data()
        .unwrap()
        .format(
            TimeZoneNameInput::Named {
                identity: &identity,
                epoch: TimeZoneEpochSeconds::new(epoch).unwrap(),
                transition,
            },
            style,
            &CanonicalLocaleId::from_data("en-US").unwrap(),
        )
        .unwrap()
}

#[test]
fn all_six_styles_use_the_selected_standard_or_daylight_snapshot() {
    for (epoch, variant, offset, expected) in [
        (
            1_609_459_200,
            TimeZoneVariant::Standard,
            -28_800,
            [
                "PST",
                "Pacific Standard Time",
                "GMT-8",
                "GMT-08:00",
                "PT",
                "Pacific Time",
            ],
        ),
        (
            1_625_097_600,
            TimeZoneVariant::Daylight,
            -25_200,
            [
                "PDT",
                "Pacific Daylight Time",
                "GMT-7",
                "GMT-07:00",
                "PT",
                "Pacific Time",
            ],
        ),
    ] {
        for (style, expected) in TimeZoneNameStyle::ALL.into_iter().zip(expected) {
            assert_eq!(
                named(
                    "US/Pacific",
                    "America/Los_Angeles",
                    epoch,
                    NamedTimeZoneTransition::new(offset, variant),
                    style
                ),
                expected,
                "{} at {epoch}",
                style.spelling()
            );
        }
    }
}

#[test]
fn direct_zone_names_precede_metazones_and_generic_names_ignore_the_variant() {
    for variant in [TimeZoneVariant::Standard, TimeZoneVariant::Daylight] {
        assert_eq!(
            named(
                "Pacific/Honolulu",
                "Pacific/Honolulu",
                1_609_459_200,
                NamedTimeZoneTransition::new(-36_000, variant),
                TimeZoneNameStyle::ShortGeneric
            ),
            "HST"
        );
    }
    assert_eq!(
        named(
            "Pacific/Honolulu",
            "Pacific/Honolulu",
            1_609_459_200,
            NamedTimeZoneTransition::stable_standard(-36_000),
            TimeZoneNameStyle::LongGeneric
        ),
        "Hawaii-Aleutian Time"
    );
}

#[test]
fn metazone_qualification_follows_pinned_ldml_for_generic_and_specific() {
    for (style, expected) in [
        (TimeZoneNameStyle::Long, "Pacific Standard Time (Canada)"),
        (TimeZoneNameStyle::ShortGeneric, "PT (Canada)"),
        (TimeZoneNameStyle::LongGeneric, "Pacific Time (Canada)"),
    ] {
        assert_eq!(
            named(
                "America/Vancouver",
                "America/Vancouver",
                1_609_459_200,
                NamedTimeZoneTransition::new(-28_800, TimeZoneVariant::Standard),
                style
            ),
            expected
        );
    }
    assert_eq!(
        named(
            "America/Phoenix",
            "America/Phoenix",
            1_609_459_200,
            NamedTimeZoneTransition::stable_standard(-25_200),
            TimeZoneNameStyle::LongGeneric
        ),
        "Mountain Time (Phoenix)"
    );
}

#[test]
fn primary_country_identity_precedes_older_cldr_alias_groups() {
    for (alias, primary, epoch, offset, expected) in [
        (
            "Antarctica/South_Pole",
            "Antarctica/McMurdo",
            1_625_097_600,
            43_200,
            "New Zealand Standard Time (Antarctica)",
        ),
        (
            "Atlantic/Jan_Mayen",
            "Arctic/Longyearbyen",
            1_609_459_200,
            3600,
            "Central European Standard Time (Svalbard & Jan Mayen)",
        ),
    ] {
        assert_eq!(
            named(
                alias,
                primary,
                epoch,
                NamedTimeZoneTransition::new(offset, TimeZoneVariant::Standard),
                TimeZoneNameStyle::Long
            ),
            expected
        );
    }
    assert_eq!(
        named(
            "Asia/Calcutta",
            "Asia/Kolkata",
            1_609_459_200,
            NamedTimeZoneTransition::stable_standard(19_800),
            TimeZoneNameStyle::Long
        ),
        "India Standard Time"
    );
}

#[test]
fn generic_standard_fallback_requires_the_complete_stability_proof() {
    // Inject proof states to test this consumer; actual transition-window
    // eligibility is exercised by the independent IANA provider controls.
    for (transition, expected) in [
        (
            NamedTimeZoneTransition::new(0, TimeZoneVariant::Standard),
            "UK Time",
        ),
        (
            NamedTimeZoneTransition::new(3600, TimeZoneVariant::Daylight),
            "UK Time",
        ),
        (
            NamedTimeZoneTransition::stable_standard(0),
            "Greenwich Mean Time (United Kingdom)",
        ),
    ] {
        assert_eq!(
            named(
                "Europe/London",
                "Europe/London",
                1_609_459_200,
                transition,
                TimeZoneNameStyle::LongGeneric
            ),
            expected
        );
    }
    assert_eq!(
        named(
            "Europe/London",
            "Europe/London",
            1_625_097_600,
            NamedTimeZoneTransition::new(3600, TimeZoneVariant::Daylight),
            TimeZoneNameStyle::ShortGeneric
        ),
        "UK Time"
    );
    assert_eq!(
        named(
            "Europe/London",
            "Europe/London",
            1_625_097_600,
            NamedTimeZoneTransition::new(3600, TimeZoneVariant::Daylight),
            TimeZoneNameStyle::Short
        ),
        "GMT+1"
    );
}

#[test]
fn daylight_overrides_and_half_hour_offsets_keep_the_selected_variant() {
    for (identifier, epoch, offset, expected) in [
        ("Europe/London", 1_625_097_600, 3600, "British Summer Time"),
        ("Europe/Dublin", 1_625_097_600, 3600, "Irish Standard Time"),
        (
            "Australia/Lord_Howe",
            1_609_459_200,
            39_600,
            "Lord Howe Daylight Time",
        ),
    ] {
        assert_eq!(
            named(
                identifier,
                identifier,
                epoch,
                NamedTimeZoneTransition::new(offset, TimeZoneVariant::Daylight),
                TimeZoneNameStyle::Long
            ),
            expected
        );
    }
    assert_eq!(
        named(
            "Australia/Lord_Howe",
            "Australia/Lord_Howe",
            1_625_097_600,
            NamedTimeZoneTransition::new(37_800, TimeZoneVariant::Standard),
            TimeZoneNameStyle::ShortOffset
        ),
        "GMT+10:30"
    );
}

#[test]
fn exact_utc_boundaries_choose_half_open_metazone_periods() {
    for (epoch, variant, expected) in [
        (
            688_546_799,
            TimeZoneVariant::Daylight,
            "CDT (Knox, Indiana)",
        ),
        (
            688_546_800,
            TimeZoneVariant::Standard,
            "EST (Knox, Indiana)",
        ),
        (
            688_546_801,
            TimeZoneVariant::Standard,
            "EST (Knox, Indiana)",
        ),
    ] {
        assert_eq!(
            named(
                "America/Indiana/Knox",
                "America/Indiana/Knox",
                epoch,
                NamedTimeZoneTransition::new(-18_000, variant),
                TimeZoneNameStyle::Short
            ),
            expected
        );
    }
}

#[test]
fn repeated_wall_time_retains_each_exact_instant_metazone() {
    let before = 973_400_400 - 1800;
    let after = 973_400_400 + 1800;
    assert_eq!(before - 18_000, after - 21_600);
    assert_eq!(
        named(
            "America/Cambridge_Bay",
            "America/Cambridge_Bay",
            before,
            NamedTimeZoneTransition::new(-18_000, TimeZoneVariant::Standard),
            TimeZoneNameStyle::Short
        ),
        "EST (Cambridge Bay)"
    );
    assert_eq!(
        named(
            "America/Cambridge_Bay",
            "America/Cambridge_Bay",
            after,
            NamedTimeZoneTransition::new(-21_600, TimeZoneVariant::Standard),
            TimeZoneNameStyle::Short
        ),
        "CST (Cambridge Bay)"
    );
}

#[test]
fn out_of_icu_timestamp_range_uses_exact_periods_and_preserves_gaps() {
    for epoch in [-TimeZoneEpochSeconds::LIMIT, TimeZoneEpochSeconds::LIMIT] {
        assert_eq!(
            named(
                "America/New_York",
                "America/New_York",
                epoch,
                NamedTimeZoneTransition::new(-18_000, TimeZoneVariant::Standard),
                TimeZoneNameStyle::Long
            ),
            "Eastern Standard Time"
        );
    }
    for epoch in [1_540_692_000, TimeZoneEpochSeconds::LIMIT] {
        assert_eq!(
            named(
                "Africa/Casablanca",
                "Africa/Casablanca",
                epoch,
                NamedTimeZoneTransition::new(3600, TimeZoneVariant::Standard),
                TimeZoneNameStyle::Long
            ),
            "GMT+01:00"
        );
        assert_eq!(
            named(
                "Africa/Casablanca",
                "Africa/Casablanca",
                epoch,
                NamedTimeZoneTransition::new(3600, TimeZoneVariant::Standard),
                TimeZoneNameStyle::LongGeneric
            ),
            "Morocco Time"
        );
    }
}

#[test]
fn missing_new_iana_metadata_uses_the_same_selected_offset_for_every_style() {
    for style in TimeZoneNameStyle::ALL {
        let expected = match style {
            TimeZoneNameStyle::Short
            | TimeZoneNameStyle::ShortOffset
            | TimeZoneNameStyle::ShortGeneric => "GMT-3",
            TimeZoneNameStyle::Long
            | TimeZoneNameStyle::LongOffset
            | TimeZoneNameStyle::LongGeneric => "GMT-03:00",
        };
        assert_eq!(
            named(
                "America/Coyhaique",
                "America/Coyhaique",
                1_745_000_000,
                NamedTimeZoneTransition::stable_standard(-10_800),
                style
            ),
            expected
        );
    }
}

#[test]
fn historical_seconds_and_full_fixed_offset_domain_are_not_truncated() {
    for (seconds, short, long) in [
        (561, "GMT+0:09:21", "GMT+00:09:21"),
        (-2670, "GMT-0:44:30", "GMT-00:44:30"),
        (-1, "GMT-0:00:01", "GMT-00:00:01"),
        (86_340, "GMT+23:59", "GMT+23:59"),
        (-86_340, "GMT-23:59", "GMT-23:59"),
    ] {
        for (style, expected) in [
            (TimeZoneNameStyle::ShortOffset, short),
            (TimeZoneNameStyle::LongOffset, long),
        ] {
            assert_eq!(
                named(
                    "Europe/Paris",
                    "Europe/Paris",
                    -2_208_988_800,
                    NamedTimeZoneTransition::new(seconds, TimeZoneVariant::Standard),
                    style
                ),
                expected
            );
        }
    }
    let names = TimeZoneNames::from_pinned_data().unwrap();
    let locale = CanonicalLocaleId::from_data("en").unwrap();
    for seconds in [-86_340, 86_340] {
        let input =
            TimeZoneNameInput::FixedOffset(FixedTimeZoneOffset::from_seconds(seconds).unwrap());
        for style in TimeZoneNameStyle::ALL {
            assert_eq!(
                names.format(input, style, &locale).unwrap(),
                if seconds < 0 {
                    "GMT-23:59"
                } else {
                    "GMT+23:59"
                }
            );
        }
    }
}

#[test]
fn numeric_zero_and_named_utc_retain_distinct_non_offset_names() {
    let names = TimeZoneNames::from_pinned_data().unwrap();
    let locale = CanonicalLocaleId::from_data("en-US").unwrap();
    for style in TimeZoneNameStyle::ALL {
        assert_eq!(
            names
                .format(
                    TimeZoneNameInput::FixedOffset(FixedTimeZoneOffset::from_seconds(0).unwrap()),
                    style,
                    &locale
                )
                .unwrap(),
            "GMT"
        );
        let expected = match style {
            TimeZoneNameStyle::Short | TimeZoneNameStyle::ShortGeneric => "UTC",
            TimeZoneNameStyle::Long | TimeZoneNameStyle::LongGeneric => {
                "Coordinated Universal Time"
            }
            TimeZoneNameStyle::ShortOffset | TimeZoneNameStyle::LongOffset => "GMT",
        };
        assert_eq!(
            named(
                "Etc/GMT",
                "UTC",
                0,
                NamedTimeZoneTransition::stable_standard(0),
                style
            ),
            expected
        );
    }
}

#[test]
fn name_locale_matches_formatter_inventory_and_keeps_latin_digit_skeletons() {
    let names = TimeZoneNames::from_pinned_data().unwrap();
    let input = TimeZoneNameInput::FixedOffset(FixedTimeZoneOffset::from_seconds(19_800).unwrap());
    for locale in [
        "en",
        "en-US",
        "en-u-nu-arab",
        "en-US-u-ca-gregory-hc-h23-nu-fullwide",
    ] {
        assert_eq!(
            names
                .format(
                    input,
                    TimeZoneNameStyle::LongOffset,
                    &CanonicalLocaleId::from_data(locale).unwrap()
                )
                .unwrap(),
            "GMT+05:30"
        );
    }
    for locale in [
        "fr",
        "ar",
        "zh",
        "en-GB",
        "en-Latn-US",
        "en-US-x-name",
        "en-US-u-rg-gbzzzz",
    ] {
        let locale = CanonicalLocaleId::from_data(locale).unwrap();
        assert!(matches!(
            names.format(input, TimeZoneNameStyle::LongOffset, &locale),
            Err(TimeZoneResolveError::UnsupportedNameLocale(rejected)) if rejected == locale
        ));
    }
}
