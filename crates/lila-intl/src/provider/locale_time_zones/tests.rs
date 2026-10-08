use super::*;

fn profiles() -> &'static LocaleTimeZoneProfiles {
    crate::named_time_zone_image::embedded_named_time_zone_data_image_ref()
        .unwrap()
        .country_profiles_ref()
}

fn names(tag: &str) -> Vec<String> {
    resolve_locale_time_zones(
        LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data(tag).unwrap()).unwrap(),
        profiles(),
    )
    .unwrap()
    .names()
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn missing_base_region_is_rejected_even_when_extensions_suggest_a_country() {
    for tag in [
        "en",
        "en-Latn",
        "en-u-rg-uszzzz",
        "en-u-sd-usca",
        "en-t-en-us",
        "en-x-us",
    ] {
        assert!(matches!(
            LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data(tag).unwrap()),
            Err(LocaleTimeZonesError::MissingExplicitRegion)
        ));
    }
}

#[test]
fn request_records_the_explicit_base_region_and_original_owner() {
    let locale =
        CanonicalLocaleId::from_data("en-Latn-US-u-rg-gbzzzz-sd-gbeng-tz-gblon-x-ca").unwrap();
    let request = LocaleTimeZonesRequest::new(locale.clone()).unwrap();
    assert_eq!(request.region(), "US");
    assert_eq!(request.clone().into_locale(), locale);
    assert_eq!(request.into_locale(), locale);
}

#[test]
fn present_world_and_unknown_regions_have_an_empty_list() {
    assert!(names("en-001").is_empty());
    assert!(names("en-ZZ").is_empty());
}

#[test]
fn reserved_language_with_a_base_region_uses_the_same_country_membership() {
    for tag in ["abcde-US", "foobar-US", "abcdefgh-Latn-US"] {
        assert_eq!(names(tag), names("en-US"));
    }
    assert_eq!(names("foobar-JP-u-rg-uszzzz"), names("ja-JP"));
}

#[test]
fn reserved_language_without_a_base_region_stays_absent_despite_extensions() {
    for tag in [
        "abcde",
        "foobar-Latn",
        "abcdefgh-u-rg-uszzzz",
        "abcde-u-sd-usca",
        "foobar-x-us",
    ] {
        assert!(matches!(
            LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data(tag).unwrap()),
            Err(LocaleTimeZonesError::MissingExplicitRegion)
        ));
    }
}

#[test]
fn base_country_is_not_overridden_by_rg_sd_or_timezone_keywords() {
    assert_eq!(names("en-US-u-rg-gbzzzz-sd-gbeng-tz-gblon"), names("en-US"));
    assert_eq!(names("en-GB-u-rg-uszzzz"), names("en-GB"));
}

#[test]
fn country_specific_primary_exceptions_keep_slovakia_separate_from_czechia() {
    assert_eq!(names("sk-SK"), ["Europe/Bratislava"]);
    assert_eq!(names("cs-CZ"), ["Europe/Prague"]);
}

#[test]
fn german_country_membership_keeps_busingen_as_its_own_primary() {
    assert_eq!(names("de-DE"), ["Europe/Berlin", "Europe/Busingen"]);
    assert_eq!(names("sv-AX"), ["Europe/Mariehamn"]);
}

#[test]
fn actual_catalogue_primary_spelling_replaces_links() {
    assert_eq!(names("en-IN"), ["Asia/Kolkata"]);
    assert!(!names("uk-UA").iter().any(|name| name == "Europe/Kiev"));
    assert!(names("uk-UA").iter().any(|name| name == "Europe/Kyiv"));
}

#[test]
fn zone_tab_single_country_membership_keeps_samoa_and_american_samoa_separate() {
    assert_eq!(names("en-WS"), ["Pacific/Apia"]);
    assert_eq!(names("en-AS"), ["Pacific/Pago_Pago"]);
}

#[test]
fn all_country_profiles_are_sorted_unique_canonical_names() {
    let profiles = profiles();
    assert_eq!(profiles.regions.len(), 247);
    for names in profiles.regions.values() {
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(names.iter().all(|name| name.is_ascii() && !name.is_empty()));
    }
    let us = names("en-US");
    for name in ["America/Adak", "America/Los_Angeles", "America/New_York"] {
        assert!(us.iter().any(|actual| actual == name));
    }
}

#[test]
fn malformed_or_unordered_projection_is_rejected() {
    for source in [
        "US\n",
        "us\tAmerica/New_York\n",
        "US\t\n",
        "US\tEurope/Prague\textra\n",
        "US\tAmerica/New_York\nUS\tAmerica/New_York\n",
        "US\tAmerica/New_York\nDE\tEurope/Berlin\n",
        "US\tAmerica/é\n",
    ] {
        assert!(projection(source).is_err(), "{source:?}");
    }
}

#[test]
fn typed_provider_operation_selects_only_the_explicit_country() {
    use crate::{EmbeddedIntlProvider, IntlOperationProvider, LocaleTimeZonesOperation};
    let provider = EmbeddedIntlProvider::new().unwrap();
    let request =
        LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data("sk-SK-u-rg-czzzzz").unwrap())
            .unwrap();
    let actual =
        <EmbeddedIntlProvider as IntlOperationProvider<LocaleTimeZonesOperation>>::execute(
            &provider, request,
        )
        .unwrap();
    assert_eq!(
        actual
            .names()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["Europe/Bratislava"]
    );
}

#[test]
fn locale_profile_requires_country_membership_without_granting_datetime_formatting() {
    use crate::{IntlDataCapability, IntlProfilePlan, IntlService, IntlServiceSet};
    let profile =
        IntlProfilePlan::minimal(IntlServiceSet::EMPTY.with(IntlService::Locale)).unwrap();
    assert!(profile
        .capabilities()
        .contains(IntlDataCapability::TimeZoneRegions));
    assert!(!profile.services().contains(IntlService::DateTimeFormat));
}
