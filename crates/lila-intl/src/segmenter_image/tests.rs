use super::*;
use crate::{
    CanonicalLocaleId, CheckedSegmenterConfiguration, CustomProfileId, SegmentUtf16Request,
    SegmenterError, SegmenterGranularity,
};

fn request(
    profiles: &SegmenterProfiles,
    locale: &str,
    granularity: SegmenterGranularity,
    input: &[u16],
) -> SegmentUtf16Request {
    let selected = profiles
        .admit(CanonicalLocaleId::from_data(locale).unwrap())
        .unwrap();
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(selected, granularity),
        input.into(),
    )
    .unwrap()
}
fn ends(result: &crate::SegmenterResult) -> Vec<u32> {
    std::iter::once(0)
        .chain(result.boundaries().iter().map(|row| row.end()))
        .collect()
}

#[test]
fn retained_image_consumers_use_true_complex_models_and_original_utf16() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let image =
        SegmenterDataImage::from_bytes(embedded_segmenter_data_image().unwrap().bytes(), &locale)
            .unwrap();
    let profiles = image.profiles();
    drop(image);
    let units = [0xd800, 0xd83d, 0xde00, 0xdc00, 0x61];
    let grapheme_request = request(&profiles, "en", SegmenterGranularity::Grapheme, &units);
    let result = profiles.segment(&grapheme_request).unwrap();
    assert_eq!(result.input(), units);
    assert_eq!(ends(&result), [0, 1, 3, 4, 5]);
    assert!(result
        .boundaries()
        .iter()
        .all(|row| row.is_word_like().is_none()));
    let thai: Vec<_> = "ภาษาไทยภาษาไทย".encode_utf16().collect();
    let result = profiles
        .segment(&request(&profiles, "en", SegmenterGranularity::Word, &thai))
        .unwrap();
    assert_eq!(ends(&result), [0, 4, 7, 11, 14]);
    assert!(result
        .boundaries()
        .iter()
        .all(|row| row.is_word_like().is_some()));
    let cjk: Vec<_> = "うなぎうなじ".encode_utf16().collect();
    let result = profiles
        .segment(&request(&profiles, "ja", SegmenterGranularity::Word, &cjk))
        .unwrap();
    assert_eq!(ends(&result), [0, 3, 6]);
    let sentence: Vec<_> = "hello; world".encode_utf16().collect();
    let result = profiles
        .segment(&request(
            &profiles,
            "el",
            SegmenterGranularity::Sentence,
            &sentence,
        ))
        .unwrap();
    assert_eq!(ends(&result), [0, 7, 12]);
}

#[test]
fn same_bytes_do_not_authorize_a_foreign_resolved_profile() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let first = embedded_segmenter_data_image().unwrap();
    let second = SegmenterDataImage::from_bytes(first.bytes(), &locale).unwrap();
    assert_eq!(first.digest(), second.digest());
    for granularity in SegmenterGranularity::ALL {
        let request = request(&first.profiles(), "en", granularity, &[0x41]);
        assert_eq!(ends(&first.profiles().segment(&request).unwrap()), [0, 1]);
        assert!(matches!(
            second.profiles().segment(&request),
            Err(SegmenterError::InvalidResolvedLocale)
        ));
    }
}

#[test]
fn coherent_manifest_cannot_substitute_icu_or_locale_descriptor_bytes() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let (descriptor, _) = split_payload(PINNED_PAYLOAD).unwrap();
    let mut descriptor_value: serde_json::Value = serde_json::from_slice(descriptor).unwrap();
    descriptor_value["locales"] = serde_json::json!(["en-US"]);
    let descriptor = serde_json::to_vec(&descriptor_value).unwrap();
    let mut altered = b"LILASEG1".to_vec();
    altered.extend_from_slice(&u32::try_from(descriptor.len()).unwrap().to_le_bytes());
    altered.extend_from_slice(&descriptor);
    altered.extend_from_slice(split_payload(PINNED_PAYLOAD).unwrap().1);
    for payload in [
        altered.as_slice(),
        b"missing genuine ICU payload".as_slice(),
    ] {
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::Segmenter,
            &IntlDataProfile::Minimal,
            &MARKERS,
            payload,
        )
        .unwrap();
        assert!(matches!(
            SegmenterDataImage::from_bytes(bytes, &locale),
            Err(IntlDataImageError::Consumer(_))
        ));
    }
}

#[test]
fn component_profiles_match_locale_and_full_conformance_remains_unavailable() {
    let minimal = embedded_segmenter_data_image().unwrap();
    let profile =
        IntlDataProfile::Custom(CustomProfileId::parse("segmenter-profile-proof").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    assert!(matches!(
        SegmenterDataImage::from_bytes(minimal.bytes(), &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
    let custom = SegmenterDataImage::for_profile(profile.clone(), &locale).unwrap();
    assert_eq!(custom.profile(), &profile);
    assert_ne!(custom.digest(), minimal.digest());
    let request = request(
        &custom.profiles(),
        "en",
        SegmenterGranularity::Grapheme,
        &[0xd800, 0x41],
    );
    assert_eq!(
        ends(&custom.profiles().segment(&request).unwrap()),
        [0, 1, 2]
    );
    assert!(matches!(
        SegmenterDataImage::for_profile(IntlDataProfile::Conformance, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
}
