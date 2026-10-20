use super::*;
use profile::LocaleCalendarsProfile;
use sha2::{Digest as _, Sha256};
const PROFILE: &[u8] = include_bytes!("../../../data/locale-calendars-cldr-47/profile.json");
fn resolved(tag: &str) -> Vec<String> {
    crate::embedded_native_locale_information_data_image()
        .unwrap()
        .resolve_calendars(
            LocaleCalendarsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()).unwrap(),
        )
        .unwrap()
        .names()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn admitted(
    raw: &serde_json::Value,
) -> Result<LocaleCalendarsProfile, LocaleCalendarsProfileError> {
    let bytes = serde_json::to_vec(raw).unwrap();
    LocaleCalendarsProfile::from_bytes(&bytes, Sha256::digest(&bytes).into())
}
#[test]
fn complete_primary_calendar_preferences_and_effective_regions_are_admitted() {
    let raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    assert_eq!(raw["selectors"].as_array().unwrap().len(), 52);
    assert_eq!(raw["regions"].as_array().unwrap().len(), 292);
    let image = crate::embedded_native_locale_information_data_image().unwrap();
    let profile = image.calendars_profile();
    assert!(profile.calendars_for_region("en", "AQ").is_some());
    assert!(profile.calendars_for_region("en", "XY").is_none());
}
#[test]
fn regional_calendar_order_is_preserved_after_actual_provider_filtering() {
    assert_eq!(resolved("th-TH"), ["buddhist", "gregory"]);
    assert_eq!(
        resolved("fa-IR"),
        ["persian", "gregory", "islamic-civil", "islamic-tbla"]
    );
    assert_eq!(resolved("en-SA"), ["gregory", "islamic-umalqura"]);
    assert_eq!(resolved("en-CN"), ["gregory", "chinese"]);
}
#[test]
fn only_checked_consumed_datetime_calendars_enter_default_results() {
    let image = crate::embedded_date_time_data_image().unwrap();
    let provider = image.provider();
    assert_eq!(
        provider.available_calendars().unwrap(),
        DateTimeCalendar::ALL
    );
    for tag in ["en-IR", "en-SA", "en-001", "abcde"] {
        for name in resolved(tag) {
            assert!(DateTimeCalendar::parse(&name).is_some());
        }
    }
}
#[test]
fn every_present_calendar_keyword_is_refused_by_default_request() {
    for value in ["gregory", "foobar", "gregory-abc", ""] {
        let tag = if value.is_empty() {
            "en-u-ca".to_owned()
        } else {
            format!("en-u-ca-{value}")
        };
        assert!(matches!(
            LocaleCalendarsRequest::new(CanonicalLocaleId::from_data(tag).unwrap()),
            Err(LocaleCalendarsError::ExplicitCalendar)
        ));
    }
}
#[test]
fn rg_override_precedes_base_region_but_unknown_override_does_not_mask_it() {
    assert_eq!(resolved("en-US-u-rg-thzzzz"), ["buddhist", "gregory"]);
    assert_eq!(resolved("th-TH-u-rg-xyzzzz"), ["buddhist", "gregory"]);
    assert_eq!(resolved("th-TH-u-rg-aqzzzz"), ["gregory"]);
}
#[test]
fn sd_selects_region_only_when_explicit_base_region_is_absent() {
    assert_eq!(resolved("en-u-sd-th10"), ["buddhist", "gregory"]);
    assert_eq!(resolved("en-US-u-sd-th10"), ["gregory"]);
    assert_eq!(resolved("en-US-u-rg-th10"), ["buddhist", "gregory"]);
    assert_eq!(resolved("en-US-u-rg-th10-extra"), ["gregory"]);
}
#[test]
fn likely_subtags_world_fallback_and_unrelated_keywords_preserve_shared_region_owner() {
    assert_eq!(resolved("th"), ["buddhist", "gregory"]);
    assert_eq!(resolved("und-001"), ["gregory"]);
    assert_eq!(resolved("en-XY"), ["gregory"]);
    assert_eq!(
        resolved("en-US-u-co-phonebk-fw-mon-hc-h24-nu-arab"),
        ["gregory"]
    );
}
#[test]
fn transform_and_private_text_cannot_supply_calendar_or_region() {
    assert_eq!(resolved("en-US-x-ca-buddhist-rg-thzzzz"), ["gregory"]);
    assert_eq!(resolved("en-US-t-th-th"), ["gregory"]);
}
#[test]
fn calendar_result_rejects_empty_or_duplicate_provider_values() {
    assert!(LocaleCalendars::checked(Vec::new()).is_err());
    assert!(LocaleCalendars::checked(vec![DateTimeCalendar::Gregorian; 2]).is_err());
}
#[test]
fn profile_admission_rejects_digest_schema_revision_and_unknown_fields() {
    assert!(LocaleCalendarsProfile::from_bytes(PROFILE, [0; 32]).is_err());
    let base: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    for (field, value) in [
        ("schema", serde_json::json!(2)),
        ("cldr_release", serde_json::json!("48.0.0")),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut raw = base.clone();
        raw[field] = value;
        assert!(admitted(&raw).is_err());
    }
}
#[test]
fn profile_admission_rejects_unknown_empty_duplicate_and_unsorted_rows() {
    let base: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    for values in [
        serde_json::json!([]),
        serde_json::json!(["foobar"]),
        serde_json::json!(["gregory", "gregory"]),
    ] {
        let mut raw = base.clone();
        raw["regions"][0]["calendars"] = values;
        assert!(admitted(&raw).is_err());
    }
    let mut raw = base;
    raw["regions"].as_array_mut().unwrap().swap(0, 1);
    assert!(admitted(&raw).is_err());
}
#[test]
fn typed_calendar_operation_reaches_pinned_defaults() {
    use crate::{
        EmbeddedIntlProvider, IntlOperation, IntlOperationProvider, IntlService,
        LocaleCalendarsOperation,
    };
    assert!(IntlService::Locale
        .required_capabilities()
        .contains_all(LocaleCalendarsOperation::HOST_OP.required_capabilities()));
    let provider = EmbeddedIntlProvider::new().unwrap();
    let result =
        <EmbeddedIntlProvider as IntlOperationProvider<LocaleCalendarsOperation>>::execute(
            &provider,
            LocaleCalendarsRequest::new(CanonicalLocaleId::from_data("th-TH").unwrap()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        result
            .names()
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>(),
        ["buddhist", "gregory"]
    );
}
