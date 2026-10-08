use super::*;
use crate::{
    CheckedSegmenterConfiguration, SegmentUtf16Request, SegmenterDataImage, SegmenterError,
    SegmenterGranularity,
};

fn foundation() -> (CustomProfileId, LocaleDataImage) {
    let id = CustomProfileId::parse("segmenter-projection").unwrap();
    let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(id.clone())).unwrap();
    (id, locale)
}
fn locales(names: &[&str]) -> Vec<LocaleId> {
    names
        .iter()
        .map(|name| LocaleId::parse(*name).unwrap())
        .collect()
}
fn request(
    profiles: &SegmenterProfiles,
    name: &str,
    granularity: SegmenterGranularity,
    units: &[u16],
) -> SegmentUtf16Request {
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(
            profiles
                .admit(CanonicalLocaleId::from_data(name).unwrap())
                .unwrap(),
            granularity,
        ),
        units.into(),
    )
    .unwrap()
}
fn ends(
    profiles: &SegmenterProfiles,
    name: &str,
    granularity: SegmenterGranularity,
    input: &str,
) -> Vec<u32> {
    let units = input.encode_utf16().collect::<Vec<_>>();
    let result = profiles
        .segment(&request(profiles, name, granularity, &units))
        .unwrap();
    std::iter::once(0)
        .chain(result.boundaries().iter().map(|row| row.end()))
        .collect()
}

#[test]
fn exact_prefix_buffers_global_models_and_unicode_rules_survive_locale_projection() {
    let (id, locale) = foundation();
    let selected = project(&id, &locales(&["fr"]), &locale).unwrap();
    let complete = project(&id, &locales(&["fi", "sv", "el"]), &locale).unwrap();
    assert_eq!(selected.rows.iter().count(), 8);
    assert_eq!(complete.rows.iter().count(), 11);
    assert!(selected.rows.iter().all(|(key, _)| key.marker.is_global()));
    assert!(
        selected
            .rows
            .iter()
            .map(|(_, row)| row.payload.get().len())
            .sum::<usize>()
            < complete
                .rows
                .iter()
                .map(|(_, row)| row.payload.get().len())
                .sum::<usize>()
    );
    let (_, blob) = split_payload(PINNED_PAYLOAD).unwrap();
    let source = BlobDataProvider::try_new_from_blob(blob.into()).unwrap();
    check_prefixes(&source, &selected.rows).unwrap();
    for (key, row) in selected.rows.iter() {
        let full = source
            .load_data(
                key.marker.info(),
                DataRequest {
                    id: key.id.as_borrowed(),
                    metadata: Default::default(),
                },
            )
            .unwrap();
        assert_eq!(row.payload.get(), full.payload.get());
        assert_eq!(row.metadata.checksum, full.metadata.checksum);
    }
    let image = SegmenterDataImage::for_custom_projection(&id, &locales(&["fr"]), &locale).unwrap();
    let profiles = image.profiles();
    assert_eq!(
        profiles
            .available_locales()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["en-US", "fr"]
    );
    assert_eq!(
        ends(
            &profiles,
            "en-US",
            SegmenterGranularity::Word,
            "ภาษาไทยภาษาไทย"
        ),
        [0, 4, 7, 11, 14]
    );
    assert_eq!(
        ends(
            &profiles,
            "fr",
            SegmenterGranularity::Word,
            "မြန်မာစာမြန်မာစာမြန်မာစာ"
        ),
        [0, 8, 16, 22, 24]
    );
    assert_eq!(
        ends(&profiles, "fr", SegmenterGranularity::Word, "うなぎうなじ"),
        [0, 3, 6]
    );
    let raw = [0xd800, 0xd83d, 0xde00, 0xdc00, 0x61];
    let result = profiles
        .segment(&request(
            &profiles,
            "en-US",
            SegmenterGranularity::Grapheme,
            &raw,
        ))
        .unwrap();
    assert_eq!(result.input(), raw);
    assert_eq!(
        result
            .boundaries()
            .iter()
            .map(|row| row.end())
            .collect::<Vec<_>>(),
        [1, 3, 4, 5]
    );
}

#[test]
fn selected_overrides_are_required_and_each_global_omission_rejects_real_constructors() {
    let (id, locale) = foundation();
    let selected = project(&id, &locales(&["sv", "el"]), &locale).unwrap();
    assert_eq!(selected.rows.iter().count(), 10);
    assert!(!selected
        .rows
        .iter()
        .any(|(key, _)| key.id.locale.to_string() == "fi"));
    let image =
        SegmenterDataImage::for_custom_projection(&id, &locales(&["sv", "el"]), &locale).unwrap();
    let profiles = image.profiles();
    assert_eq!(
        ends(&profiles, "sv", SegmenterGranularity::Word, "hello:world"),
        [0, 11]
    );
    assert_eq!(
        ends(
            &profiles,
            "en-US",
            SegmenterGranularity::Word,
            "hello:world"
        ),
        [0, 5, 6, 11]
    );
    assert_eq!(
        ends(
            &profiles,
            "el",
            SegmenterGranularity::Sentence,
            "hello; world"
        ),
        [0, 7, 12]
    );
    assert_eq!(
        ends(
            &profiles,
            "en-US",
            SegmenterGranularity::Sentence,
            "hello; world"
        ),
        [0, 12]
    );
    let keys = selected
        .rows
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in keys {
        let mut omitted = project(&id, &locales(&["sv", "el"]), &locale).unwrap();
        omitted.rows.remove(&key);
        let data = Arc::new(SegmenterImageProvider::projected(omitted.rows, &locale));
        assert!(
            SegmenterProfiles::from_projection(data, &omitted.catalogue).is_err(),
            "{key:?}"
        );
    }
}

#[test]
fn selected_wire_remints_exact_owner_and_retained_profiles_outlive_images() {
    let (id, locale) = foundation();
    let image = SegmenterDataImage::for_custom_projection(&id, &locales(&["fr"]), &locale).unwrap();
    let second = SegmenterDataImage::from_bytes(image.bytes(), &locale).unwrap();
    assert_eq!(image.digest(), second.digest());
    let profiles = image.profiles();
    let foreign = request(
        &profiles,
        "fr",
        SegmenterGranularity::Grapheme,
        &[0x61, 0x301],
    );
    assert!(matches!(
        second.profiles().segment(&foreign),
        Err(SegmenterError::InvalidResolvedLocale)
    ));
    let wire = crate::encode_segment_utf16_request(&foreign).unwrap();
    let reminted = crate::decode_segment_utf16_request(&second.profiles(), &wire).unwrap();
    assert_eq!(
        second.profiles().segment(&reminted).unwrap().boundaries()[0].end(),
        2
    );
    let full = SegmenterDataImage::for_profile(IntlDataProfile::Custom(id), &locale).unwrap();
    let hidden = request(&full.profiles(), "sv", SegmenterGranularity::Word, &[0x61]);
    assert!(crate::decode_segment_utf16_request(
        &profiles,
        &crate::encode_segment_utf16_request(&hidden).unwrap()
    )
    .is_err());
    drop(image);
    drop(second);
    drop(full);
    drop(locale);
    assert_eq!(profiles.segment(&foreign).unwrap().boundaries()[0].end(), 2);
}

#[test]
fn source_catalogue_pins_labels_aliases_and_physical_damage_are_admitted_once() {
    let (id, locale) = foundation();
    for names in [
        &[][..],
        &["fr", "FR"][..],
        &["qaa"][..],
        &["fr-u-ca-gregory"][..],
    ] {
        assert!(SegmenterDataImage::for_custom_projection(&id, &locales(names), &locale).is_err());
    }
    let image = SegmenterDataImage::for_custom_projection(&id, &locales(&["fr"]), &locale).unwrap();
    let payload = project(&id, &locales(&["fr"]), &locale).unwrap().bytes;
    let length = u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;
    let original: serde_json::Value = serde_json::from_slice(&payload[12..12 + length]).unwrap();
    for key in [
        "schema",
        "custom_id",
        "source_payload_sha256",
        "source_descriptor_sha256",
        "locale_digest",
        "default_locale",
        "public_locales",
        "rows",
    ] {
        let mut damaged = original.clone();
        damaged[key] = match key {
            "schema" => serde_json::json!(2),
            "public_locales" => serde_json::json!(["en-US", "sv"]),
            "rows" => serde_json::json!([]),
            _ => serde_json::json!("foreign"),
        };
        let description = serde_json::to_vec(&damaged).unwrap();
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&u32::try_from(description.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(&description);
        bytes.extend_from_slice(&payload[12 + length..]);
        let frame = super::super::DataImageEnvelope::encode(
            super::super::DataImageComponent::Segmenter,
            &IntlDataProfile::Custom(id.clone()),
            &super::super::MARKERS,
            &bytes,
        )
        .unwrap();
        assert!(
            SegmenterDataImage::from_bytes(frame, &locale).is_err(),
            "{key}"
        );
    }
    for bytes in [
        &payload[..payload.len() - 1],
        &[payload.as_slice(), &[0]].concat()[..],
    ] {
        let frame = super::super::DataImageEnvelope::encode(
            super::super::DataImageComponent::Segmenter,
            &IntlDataProfile::Custom(id.clone()),
            &super::super::MARKERS,
            bytes,
        )
        .unwrap();
        assert!(SegmenterDataImage::from_bytes(frame, &locale).is_err());
    }
    let minimal = crate::embedded_locale_data_image().unwrap();
    assert!(SegmenterDataImage::from_bytes(image.bytes(), &minimal).is_err());
    let different = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("other").unwrap(),
    ))
    .unwrap();
    assert!(SegmenterDataImage::from_bytes(image.bytes(), &different).is_err());
}
