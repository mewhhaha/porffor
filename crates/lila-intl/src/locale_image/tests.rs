use super::*;
use crate::CustomProfileId;

#[test]
fn pinned_image_owns_real_alias_likely_subtag_and_parent_fallback_data() {
    let image = embedded_locale_data_image().unwrap();
    let second = embedded_locale_data_image().unwrap();
    assert!(Arc::ptr_eq(&image.0, &second.0));
    let mut locale: icu_locale::Locale = "iw-IL".parse().unwrap();
    image.canonicalizer().canonicalize(&mut locale);
    assert_eq!(locale.to_string(), "he-IL");
    let mut locale: icu_locale::Locale = "en".parse().unwrap();
    image.expander().maximize(&mut locale.id);
    assert_eq!(locale.to_string(), "en-Latn-US");
    let fallbacker = image.fallbacker();
    let mut fallback = fallbacker
        .for_config(Default::default())
        .fallback_for("en-US".parse().unwrap());
    assert_eq!(fallback.get().to_string(), "en-US");
    fallback.step();
    assert_eq!(fallback.get().to_string(), "en");
    assert!(!image.aliases().get().language_variants.is_empty());
}
#[test]
fn custom_names_bind_same_exact_pinned_data_without_claiming_general_selection() {
    let minimal = embedded_locale_data_image().unwrap();
    let profile =
        IntlDataProfile::Custom(CustomProfileId::parse("pinned-component-control").unwrap());
    let custom = LocaleDataImage::for_profile(profile.clone()).unwrap();
    assert_eq!(custom.profile(), &profile);
    assert_ne!(minimal.digest(), custom.digest());
    let loaded = LocaleDataImage::from_bytes(custom.bytes()).unwrap();
    assert_eq!(loaded.digest(), custom.digest());
    let mut locale: icu_locale::Locale = "sh".parse().unwrap();
    loaded.canonicalizer().canonicalize(&mut locale);
    assert_eq!(locale.to_string(), "sr-Latn");
    let complete = LocaleDataImage::for_profile(IntlDataProfile::Conformance).unwrap();
    assert_eq!(complete.profile(), &IntlDataProfile::Conformance);
    assert_ne!(complete.digest(), minimal.digest());
}
#[test]
fn structurally_valid_foreign_payload_cannot_self_assert_pinned_versions_or_fallback() {
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::LocaleTransforms,
        &IntlDataProfile::Minimal,
        &MARKERS,
        b"missing genuine ICU marker data",
    )
    .unwrap();
    assert!(matches!(
        LocaleDataImage::from_bytes(bytes),
        Err(IntlDataImageError::Consumer(_))
    ));
    let mut bytes = embedded_locale_data_image().unwrap().bytes().to_vec();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert!(matches!(
        LocaleDataImage::from_bytes(bytes.into()),
        Err(IntlDataImageError::Digest)
    ));
}

#[test]
fn keyword_authority_is_actual_image_data_and_survives_the_source_image() {
    let source =
        LocaleDataImage::from_bytes(embedded_locale_data_image().unwrap().bytes()).unwrap();
    let aliases = source.keyword_aliases();
    drop(source);
    let mut locale: icu_locale::Locale = "en-u-ca-islamicc-kn-yes".parse().unwrap();
    aliases.canonicalize_unicode_keywords(&mut locale);
    assert_eq!(locale.to_string(), "en-u-ca-islamic-civil-kn");
    let mut locale: icu_locale::Locale = "en-u-ca-islamicc-foo".parse().unwrap();
    aliases.canonicalize_unicode_keywords(&mut locale);
    assert_eq!(locale.to_string(), "en-u-ca-islamicc-foo");
}

#[test]
fn genuine_icu_blob_without_the_native_keyword_authority_is_incomplete() {
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::LocaleTransforms,
        &IntlDataProfile::Minimal,
        &MARKERS,
        PINNED_BLOB,
    )
    .unwrap();
    assert!(matches!(
        LocaleDataImage::from_bytes(bytes),
        Err(IntlDataImageError::Consumer(_))
    ));
    let mut payload = selected_payload().unwrap();
    let last = payload.len() - 1;
    payload[last] ^= 1;
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::LocaleTransforms,
        &IntlDataProfile::Minimal,
        &MARKERS,
        &payload,
    )
    .unwrap();
    assert!(matches!(
        LocaleDataImage::from_bytes(bytes),
        Err(IntlDataImageError::Consumer(_))
    ));
}
