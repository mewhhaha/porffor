use super::*;
use crate::CanonicalLocaleId;
use sha2::{Digest as _, Sha256};

fn direction(tag: &str) -> Option<LocaleTextDirection> {
    crate::embedded_native_locale_information_data_image()
        .unwrap()
        .resolve_text(LocaleTextInfoRequest::new(
            CanonicalLocaleId::from_data(tag).unwrap(),
        ))
        .expect("genuine canonical locale direction")
}

#[test]
fn full_pinned_script_profile_preserves_all_known_and_unknown_directions() {
    let image = crate::embedded_native_locale_information_data_image().unwrap();
    let profile = image.text_profile();
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/locale-text-cldr-47/profile.json"
    ))
    .unwrap();
    let mut counts = [0; 3];
    for row in raw["scripts"].as_array().unwrap() {
        let expected = match row["direction"].as_str() {
            Some("ltr") => {
                counts[0] += 1;
                Some(LocaleTextDirection::LeftToRight)
            }
            Some("rtl") => {
                counts[1] += 1;
                Some(LocaleTextDirection::RightToLeft)
            }
            None => {
                assert!(row["direction"].is_null());
                counts[2] += 1;
                None
            }
            Some(other) => panic!("unexpected direction {other}"),
        };
        assert_eq!(profile.direction(row["script"].as_str().unwrap()), expected);
    }
    assert_eq!(counts, [137, 36, 4]);
    assert_eq!(profile.direction("Qaaa"), None);
}

#[test]
fn explicit_script_precedes_language_and_likely_subtag_inference() {
    use LocaleTextDirection::{LeftToRight, RightToLeft};
    for (tag, expected) in [
        ("ar-Latn", LeftToRight),
        ("en-Arab", RightToLeft),
        ("sr-Hebr", RightToLeft),
        ("en-Adlm", RightToLeft),
    ] {
        assert_eq!(direction(tag), Some(expected), "{tag}");
    }
}

#[test]
fn absent_script_uses_genuine_pinned_likely_subtags() {
    use LocaleTextDirection::{LeftToRight, RightToLeft};
    for (tag, expected) in [
        ("en", LeftToRight),
        ("zh", LeftToRight),
        ("ar", RightToLeft),
        ("he", RightToLeft),
        ("fa", RightToLeft),
        ("ur", RightToLeft),
    ] {
        assert_eq!(direction(tag), Some(expected), "{tag}");
    }
}

#[test]
fn explicit_unknown_and_non_general_scripts_never_default_or_maximize() {
    for script in ["Brai", "Zyyy", "Zinh", "Zzzz", "Qaaa"] {
        for language in ["en", "ar"] {
            assert_eq!(direction(&format!("{language}-{script}")), None);
        }
    }
}

#[test]
fn transform_extensions_and_private_use_cannot_supply_base_script() {
    assert_eq!(
        direction("en-t-ar-arab"),
        Some(LocaleTextDirection::LeftToRight)
    );
    assert_eq!(
        direction("ar-t-en-latn"),
        Some(LocaleTextDirection::RightToLeft)
    );
    assert_eq!(
        direction("en-x-arab"),
        Some(LocaleTextDirection::LeftToRight)
    );
    assert_eq!(
        direction("ar-x-latn"),
        Some(LocaleTextDirection::RightToLeft)
    );
    assert_eq!(
        direction("en-x-u-rg-arzzzz"),
        Some(LocaleTextDirection::LeftToRight)
    );
}

#[test]
fn regional_and_week_keywords_do_not_change_text_direction() {
    for tag in [
        "en-u-rg-arzzzz",
        "en-u-sd-irsal",
        "en-u-fw-mon-rg-arzzzz",
        "en-US-u-rg-ilzzzz",
    ] {
        assert_eq!(
            direction(tag),
            Some(LocaleTextDirection::LeftToRight),
            "{tag}"
        );
    }
    assert_eq!(
        direction("ar-u-rg-uszzzz-fw-fri"),
        Some(LocaleTextDirection::RightToLeft)
    );
}

#[test]
fn reserved_language_never_uses_the_und_parser_placeholder_for_inference() {
    for tag in ["abcde", "abcdefgh", "abcde-IL", "abcde-u-rg-ilzzzz"] {
        assert_eq!(direction(tag), None, "{tag}");
    }
    assert_eq!(
        direction("abcde-Hebr"),
        Some(LocaleTextDirection::RightToLeft)
    );
    assert_eq!(
        direction("abcdefgh-Latn"),
        Some(LocaleTextDirection::LeftToRight)
    );
}

#[test]
fn genuine_script_aliases_are_canonicalized_before_data_lookup() {
    assert_eq!(direction("en-Qaai"), None);
    assert_eq!(direction("en-Qaac"), None);
}

fn mutated(
    change: impl FnOnce(&mut serde_json::Value),
) -> Result<profile::LocaleTextProfile, LocaleTextProfileError> {
    let mut raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/locale-text-cldr-47/profile.json"
    ))
    .unwrap();
    change(&mut raw);
    let bytes = serde_json::to_vec(&raw).unwrap();
    profile::LocaleTextProfile::from_bytes(&bytes, Sha256::digest(&bytes).into())
}

#[test]
fn profile_admission_rejects_missing_fields_bad_domains_order_and_revision() {
    assert!(matches!(
        mutated(|v| {
            v["scripts"][0].as_object_mut().unwrap().remove("direction");
        }),
        Err(LocaleTextProfileError::Encoding)
    ));
    assert!(matches!(
        mutated(|v| v["scripts"][0]["direction"] = "vertical".into()),
        Err(LocaleTextProfileError::Encoding)
    ));
    assert!(matches!(
        mutated(|v| v["scripts"][0]["script"] = "abcd".into()),
        Err(LocaleTextProfileError::Script)
    ));
    assert!(matches!(
        mutated(|v| {
            v["scripts"].as_array_mut().unwrap().swap(0, 1);
        }),
        Err(LocaleTextProfileError::ScriptOrder)
    ));
    assert!(matches!(
        mutated(|v| {
            v["scripts"].as_array_mut().unwrap().remove(0);
        }),
        Err(LocaleTextProfileError::Coverage)
    ));
    assert!(matches!(
        mutated(|v| v["schema"] = 2.into()),
        Err(LocaleTextProfileError::Schema)
    ));
    assert!(matches!(
        mutated(|v| v["cldr_release"] = "48.0.0".into()),
        Err(LocaleTextProfileError::Revision)
    ));
}

#[test]
fn exact_profile_digest_is_checked_before_schema_and_fields() {
    let bytes = include_bytes!("../../../data/locale-text-cldr-47/profile.json");
    assert!(matches!(
        profile::LocaleTextProfile::from_bytes(bytes, [0; 32]),
        Err(LocaleTextProfileError::Digest)
    ));
}
