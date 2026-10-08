use super::*;
use profile::LocaleHourCyclesProfile;
use sha2::{Digest as _, Sha256};

const PROFILE: &[u8] = include_bytes!("../../../data/locale-hour-cycles-cldr-47/profile.json");

fn resolved(tag: &str) -> Vec<&'static str> {
    crate::embedded_native_locale_information_data_image()
        .unwrap()
        .resolve_hour_cycles(
            LocaleHourCyclesRequest::new(
                CanonicalLocaleId::from_data(tag).expect("canonical request"),
            )
            .expect("no explicit hour-cycle keyword"),
        )
        .expect("validated pinned hour-cycle data")
        .cycles()
        .iter()
        .map(|cycle| cycle.as_str())
        .collect()
}

fn admitted(
    raw: &serde_json::Value,
) -> Result<LocaleHourCyclesProfile, LocaleHourCyclesProfileError> {
    let bytes = serde_json::to_vec(raw).expect("JSON fixture");
    LocaleHourCyclesProfile::from_bytes(&bytes, Sha256::digest(&bytes).into())
}

#[test]
fn complete_primary_selectors_and_effective_region_rows_are_admitted() {
    let raw: serde_json::Value = serde_json::from_slice(PROFILE).expect("generated JSON");
    assert_eq!(raw["selectors"].as_array().unwrap().len(), 275);
    assert_eq!(raw["regions"].as_array().unwrap().len(), 292);
    let image = crate::embedded_native_locale_information_data_image().unwrap();
    let profile = image.hour_cycles_profile();
    assert!(profile.cycles_for_region("en", "AQ").is_some());
    assert!(profile.cycles_for_region("en", "XY").is_none());
}

#[test]
fn checked_record_rejects_empty_and_duplicate_cycles_without_sorting() {
    assert_eq!(
        LocaleHourCycles::checked(Vec::new()).unwrap_err(),
        LocaleHourCyclesProfileError::Cycles
    );
    assert_eq!(
        LocaleHourCycles::checked(vec![DateTimeHourCycle::H12; 2]).unwrap_err(),
        LocaleHourCyclesProfileError::Cycles
    );
    let cycles = LocaleHourCycles::checked(vec![
        DateTimeHourCycle::H24,
        DateTimeHourCycle::H11,
        DateTimeHourCycle::H23,
        DateTimeHourCycle::H12,
    ])
    .unwrap();
    assert_eq!(
        cycles.cycles(),
        [
            DateTimeHourCycle::H24,
            DateTimeHourCycle::H11,
            DateTimeHourCycle::H23,
            DateTimeHourCycle::H12
        ]
    );
}

#[test]
fn allowed_order_survives_projection_and_duplicate_day_period_symbols() {
    assert_eq!(resolved("en-CD"), ["h12", "h23"]);
    assert_eq!(resolved("en-JP"), ["h23", "h11", "h12"]);
    assert_eq!(resolved("en-US"), ["h12", "h23"]);
}

#[test]
fn native_default_request_rejects_every_present_hour_cycle_keyword() {
    for value in ["h11", "h12", "h23", "h24", "foobar", "h11-abc", ""] {
        let tag = if value.is_empty() {
            String::from("en-US-u-hc")
        } else {
            format!("en-US-u-hc-{value}")
        };
        let locale =
            CanonicalLocaleId::from_data(tag.as_str()).expect("canonical open keyword value");
        assert!(matches!(
            LocaleHourCyclesRequest::new(locale),
            Err(LocaleHourCyclesError::ExplicitHourCycle)
        ));
    }
}

#[test]
fn language_region_time_data_precedes_regional_time_data() {
    assert_eq!(resolved("fr-CA"), ["h23", "h12"]);
    assert_eq!(resolved("en-CA"), ["h12", "h23"]);
    assert_eq!(resolved("fr-US-u-rg-cazzzz"), ["h23", "h12"]);
}

#[test]
fn recognized_sparse_override_uses_generated_world_row_and_unknown_falls_through() {
    assert_eq!(resolved("en-US-u-rg-aqzzzz"), ["h23", "h12"]);
    assert_eq!(resolved("en-US-u-rg-xyzzzz"), ["h12", "h23"]);
    assert_eq!(resolved("en-XY"), ["h23"]);
}

#[test]
fn subdivision_base_and_explicit_region_precedence_use_shared_owner() {
    assert_eq!(resolved("en-u-sd-gbsct"), ["h23", "h12"]);
    assert_eq!(resolved("en-US-u-sd-gbsct"), ["h12", "h23"]);
    assert_eq!(resolved("en-US-u-rg-gbfoo"), ["h23", "h12"]);
    assert_eq!(resolved("en-US-u-rg-gbfoo-extra"), ["h12", "h23"]);
}

#[test]
fn transform_and_private_text_never_become_base_or_hour_cycle_preferences() {
    assert_eq!(resolved("en-t-fr-ca-x-hc-h24-rg-gbzzzz"), ["h12", "h23"]);
    assert_eq!(resolved("en-US-x-u-hc-h24"), ["h12", "h23"]);
}

#[test]
fn genuine_subdivision_aliases_and_reserved_languages_preserve_locale_owner() {
    assert_eq!(resolved("en-US-u-rg-fi01"), ["h23"]);
    assert_eq!(resolved("foobar"), ["h23", "h12"]);
}

#[test]
fn unrelated_week_calendar_and_numbering_keywords_do_not_select_cycles() {
    assert_eq!(
        resolved("en-US-u-ca-islamic-fw-mon-nu-arab"),
        ["h12", "h23"]
    );
    assert_eq!(
        resolved("en-US-u-ca-islamic-fw-mon-nu-arab-rg-aqzzzz"),
        ["h23", "h12"]
    );
}

#[test]
fn exact_profile_digest_is_required_before_schema_admission() {
    let mut changed = PROFILE.to_vec();
    changed.push(b' ');
    assert_eq!(
        LocaleHourCyclesProfile::from_bytes(&changed, LOCALE_HOUR_CYCLES_DATA_SHA256).err(),
        Some(LocaleHourCyclesProfileError::Digest)
    );
}

#[test]
fn malformed_schema_revision_and_unknown_or_missing_fields_fail_closed() {
    let base: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    for (field, value, error) in [
        (
            "schema",
            serde_json::json!(2),
            LocaleHourCyclesProfileError::Schema,
        ),
        (
            "cldr_release",
            serde_json::json!("48.0.0"),
            LocaleHourCyclesProfileError::Revision,
        ),
        (
            "primary_source_sha256",
            serde_json::json!("00"),
            LocaleHourCyclesProfileError::Revision,
        ),
        (
            "unknown",
            serde_json::json!(true),
            LocaleHourCyclesProfileError::Encoding,
        ),
    ] {
        let mut raw = base.clone();
        raw[field] = value;
        assert_eq!(admitted(&raw).err(), Some(error));
    }
    let mut raw = base;
    raw.as_object_mut().unwrap().remove("regions");
    assert_eq!(
        admitted(&raw).err(),
        Some(LocaleHourCyclesProfileError::Encoding)
    );
}

#[test]
fn malformed_order_spelling_coverage_and_cycle_lists_are_rejected() {
    let base: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    let mut raw = base.clone();
    raw["selectors"].as_array_mut().unwrap().swap(0, 1);
    assert_eq!(
        admitted(&raw).err(),
        Some(LocaleHourCyclesProfileError::Order)
    );
    let mut raw = base.clone();
    raw["regions"][0]["region"] = serde_json::json!("world");
    assert_eq!(
        admitted(&raw).err(),
        Some(LocaleHourCyclesProfileError::Region)
    );
    let mut raw = base.clone();
    raw["selectors"][0]["cycles"] = serde_json::json!(["h12", "h12"]);
    assert_eq!(
        admitted(&raw).err(),
        Some(LocaleHourCyclesProfileError::Cycles)
    );
    let mut raw = base;
    raw["regions"].as_array_mut().unwrap().pop();
    assert_eq!(
        admitted(&raw).err(),
        Some(LocaleHourCyclesProfileError::Coverage)
    );
}

#[test]
fn request_consumes_the_unchanged_canonical_locale_owner() {
    let locale = CanonicalLocaleId::from_data("en-US-u-ca-islamic-fw-mon").unwrap();
    assert_eq!(
        LocaleHourCyclesRequest::new(locale.clone())
            .unwrap()
            .into_locale(),
        locale
    );
}
