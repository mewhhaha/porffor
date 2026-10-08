use super::*;
use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::{NumberProfiles, NumberingSystemOption};
use crate::{
    decode_relative_request, encode_relative_request, FiniteRelativeNumber, RelativeHostOp,
    RelativeNumeric, RelativeRequest, RelativeStyle, RelativeTimeConfiguration,
    RelativeTimeDataImage, RelativeTimeError, RelativeUnit,
};
use std::sync::Arc;

fn setup() -> (CustomProfileId, LocaleDataImage, NumberProfilesDataImage) {
    let id = CustomProfileId::parse("actual-relative-projection").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let numbers = NumberProfilesDataImage::for_profile(profile, &locale).unwrap();
    (id, locale, numbers)
}

fn requested(names: &[&str]) -> Vec<LocaleId> {
    names
        .iter()
        .map(|name| LocaleId::parse(*name).unwrap())
        .collect()
}

fn request(name: &str) -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}

fn configuration(
    profiles: &RelativeProfiles,
    numbers: &Arc<NumberProfiles>,
    name: &str,
    style: RelativeStyle,
    numeric: RelativeNumeric,
) -> RelativeTimeConfiguration {
    let resolved = profiles
        .resolve_locale(&request(name), numbers, &PartitionLimits::HOST_ABI)
        .unwrap();
    RelativeTimeConfiguration::new(resolved, style, numeric, numbers).unwrap()
}

fn text(
    profiles: &RelativeProfiles,
    configuration: &RelativeTimeConfiguration,
    value: f64,
    unit: RelativeUnit,
) -> String {
    profiles
        .format_parts(
            configuration,
            FiniteRelativeNumber::new(value).unwrap(),
            unit,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap()
        .to_text()
        .unwrap()
}

fn envelope(image: &RelativeTimeDataImage) -> DataImageEnvelope {
    DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::RelativeTime,
        super::super::RELATIVE_TIME_IMAGE_MARKERS,
    )
    .unwrap()
}

#[test]
fn actual_selected_rows_preserve_all_fields_and_only_publish_the_selected_catalogue() {
    let (id, locale, numbers) = setup();
    let profile = IntlDataProfile::Custom(id.clone());
    let full = RelativeTimeDataImage::for_profile(profile, &locale, &numbers).unwrap();
    let image = RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["ar", "fr", "pl"]),
        &locale,
        &numbers,
    )
    .unwrap();
    let framed = envelope(&image);
    let (descriptor, blob) = split_payload(framed.blob()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
    let actual = raw["locales"].as_array().unwrap();
    assert_eq!(
        actual
            .iter()
            .map(|row| row["locale"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["ar", "en-US", "fr", "pl"]
    );
    assert!(actual
        .iter()
        .all(|row| row["fields"].as_array().unwrap().len() == 24));
    assert!(blob.len() < PINNED_PROFILE.len());
    assert!(image.bytes().len() < full.bytes().len());
    assert_eq!(descriptor.public_locales, ["ar", "en-US", "fr", "pl"]);
    assert_eq!(descriptor.number_associations.len(), 4);
    let selected = image.profiles();
    let baseline = full.profiles();
    let number_profiles = numbers.profiles();
    assert_eq!(selected.available_locales(), ["ar", "en-US", "fr", "pl"]);
    let requests =
        ["hi", "en", "fr-FR", "pl-PL"].map(|name| CanonicalLocaleId::from_data(name).unwrap());
    assert_eq!(
        selected
            .supported_locales(&requests, LocaleMatcher::Lookup, &PartitionLimits::HOST_ABI)
            .unwrap()
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["fr-FR", "pl-PL"]
    );
    assert_eq!(
        selected
            .resolve_locale(&request("hi"), &number_profiles, &PartitionLimits::HOST_ABI)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    for &name in selected.available_locales() {
        for &style in RelativeStyle::ALL {
            for numeric in [RelativeNumeric::Always, RelativeNumeric::Auto] {
                let selected_configuration =
                    configuration(&selected, &number_profiles, name, style, numeric);
                let full_configuration =
                    configuration(&baseline, &number_profiles, name, style, numeric);
                for &unit in RelativeUnit::ALL {
                    for value in [-2.0, -1.0, -0.0, 0.0, 1.0, 1.25, 2.0, 3.0, 11.0] {
                        assert_eq!(
                            text(&selected, &selected_configuration, value, unit),
                            text(&baseline, &full_configuration, value, unit)
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        text(
            &selected,
            &configuration(
                &selected,
                &number_profiles,
                "pl-PL",
                RelativeStyle::Long,
                RelativeNumeric::Auto
            ),
            -1.0,
            RelativeUnit::Day
        ),
        "wczoraj"
    );
    assert_eq!(
        text(
            &selected,
            &configuration(
                &selected,
                &number_profiles,
                "fr",
                RelativeStyle::Long,
                RelativeNumeric::Auto
            ),
            -1.0,
            RelativeUnit::Day
        ),
        "hier"
    );
    // The larger Number catalogue may negotiate nu, but does not grant public
    // RelativeTime support to one of its unselected template locales.
    let resolved = selected
        .resolve_locale(
            &NumberLocaleRequest {
                requested: vec![CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap()]
                    .into_boxed_slice(),
                ..request("en-US")
            },
            &number_profiles,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
    assert_eq!(resolved.numbering_system(), "arab");
    let arab = RelativeTimeConfiguration::new(
        resolved,
        RelativeStyle::Long,
        RelativeNumeric::Always,
        &number_profiles,
    )
    .unwrap();
    assert!(text(&selected, &arab, 12.0, RelativeUnit::Day).contains("١٢"));
    let resolved = selected
        .resolve_locale(
            &NumberLocaleRequest {
                requested: vec![CanonicalLocaleId::from_data("en-US-u-nu-arab").unwrap()]
                    .into_boxed_slice(),
                numbering_system: Some(NumberingSystemOption::parse("latn").unwrap()),
                ..request("en-US")
            },
            &number_profiles,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
    assert_eq!(resolved.numbering_system(), "latn");
    // Full Minimal and named Custom producers still emit the original JSON.
    assert_eq!(envelope(&full).blob(), PINNED_PROFILE);
    assert_eq!(
        envelope(&crate::embedded_relative_time_data_image().unwrap()).blob(),
        PINNED_PROFILE
    );
}

#[test]
fn projection_uses_selected_canonicalization_and_rejects_invalid_or_mixed_foundations() {
    let (id, locale, numbers) = setup();
    let first = RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["PL", "fr", "en-us"]),
        &locale,
        &numbers,
    )
    .unwrap();
    let second = RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["en-US", "fr", "pl"]),
        &locale,
        &numbers,
    )
    .unwrap();
    assert_eq!(first.bytes(), second.bytes());
    let third = RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["fr", "pl"]),
        &locale,
        &numbers,
    )
    .unwrap();
    assert_eq!(
        first.bytes(),
        third.bytes(),
        "en-US is required once, without request leakage"
    );
    for invalid in [&[][..], &["fr", "FR"][..], &["es"][..], &["fr-FR"][..]] {
        assert!(RelativeTimeDataImage::for_custom_projection(
            &id,
            &requested(invalid),
            &locale,
            &numbers
        )
        .is_err());
    }
    let minimal_locale = crate::embedded_locale_data_image().unwrap();
    let minimal_numbers = crate::embedded_number_profiles_data_image().unwrap();
    assert!(RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["fr"]),
        &minimal_locale,
        &numbers
    )
    .is_err());
    assert!(RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["fr"]),
        &locale,
        &minimal_numbers
    )
    .is_err());
    assert!(RelativeTimeDataImage::for_custom_projection(
        &CustomProfileId::parse("other-id").unwrap(),
        &requested(&["fr"]),
        &locale,
        &numbers
    )
    .is_err());
    let relabeled = DataImageEnvelope::encode(
        DataImageComponent::RelativeTime,
        &IntlDataProfile::Minimal,
        super::super::RELATIVE_TIME_IMAGE_MARKERS,
        envelope(&first).blob(),
    )
    .unwrap();
    assert!(
        RelativeTimeDataImage::from_bytes(relabeled, &minimal_locale, &minimal_numbers).is_err()
    );
}

#[test]
fn self_consistent_projection_changes_cannot_replace_pinned_rows_or_closure_metadata() {
    let (id, locale, numbers) = setup();
    let profile = IntlDataProfile::Custom(id.clone());
    let image = RelativeTimeDataImage::for_custom_projection(
        &id,
        &requested(&["fr", "pl"]),
        &locale,
        &numbers,
    )
    .unwrap();
    let framed = envelope(&image);
    for damage in 0..11 {
        let (mut descriptor, blob) = split_payload(framed.blob()).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
        match damage {
            0 => raw["locales"][0]["fields"][0]["past"][0] = "replacement {0}".into(),
            1 => {
                raw["locales"][0]["fields"].as_array_mut().unwrap().pop();
            }
            2 => {
                raw["locales"].as_array_mut().unwrap().remove(0);
            }
            3 => descriptor.full_profile_sha256[0] ^= 1,
            4 => descriptor.locale_image_sha256[0] ^= 1,
            5 => descriptor.number_image_sha256[0] ^= 1,
            6 => descriptor.number_associations[0].default_numbering_system = "arab".into(),
            7 => descriptor.custom_id = "other-id".into(),
            8 => descriptor.default_locale = "fr".into(),
            9 => {
                descriptor.public_locales.push("ar".into());
            }
            10 => descriptor.public_locales.reverse(),
            _ => unreachable!(),
        }
        let payload = frame_payload(
            &descriptor,
            &serde_json::to_vec(&canonical_objects(raw)).unwrap(),
        )
        .unwrap();
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::RelativeTime,
            &profile,
            super::super::RELATIVE_TIME_IMAGE_MARKERS,
            &payload,
        )
        .unwrap();
        assert!(
            RelativeTimeDataImage::from_bytes(bytes, &locale, &numbers).is_err(),
            "damage {damage}"
        );
    }
    let mut oversized = framed.blob().to_vec();
    oversized[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut overflow = framed.blob().to_vec();
    overflow[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    let mut trailing = framed.blob().to_vec();
    trailing.push(0);
    for payload in [oversized, overflow, trailing, framed.blob()[..19].to_vec()] {
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::RelativeTime,
            &profile,
            super::super::RELATIVE_TIME_IMAGE_MARKERS,
            &payload,
        )
        .unwrap();
        assert!(RelativeTimeDataImage::from_bytes(bytes, &locale, &numbers).is_err());
    }
}

#[test]
fn projected_templates_retain_exact_number_owner_and_wire_remints_only_selected_rows() {
    let (id, locale, numbers) = setup();
    let profile = IntlDataProfile::Custom(id.clone());
    let first =
        RelativeTimeDataImage::for_custom_projection(&id, &requested(&["fr"]), &locale, &numbers)
            .unwrap();
    let second = RelativeTimeDataImage::from_bytes(first.bytes(), &locale, &numbers).unwrap();
    let full = RelativeTimeDataImage::for_profile(profile, &locale, &numbers).unwrap();
    let number_profiles = numbers.profiles();
    let weak_number = Arc::downgrade(&number_profiles);
    let first_profiles = first.profiles();
    let second_profiles = second.profiles();
    let retained = configuration(
        &first_profiles,
        &number_profiles,
        "fr",
        RelativeStyle::Long,
        RelativeNumeric::Auto,
    );
    let value = FiniteRelativeNumber::new(-1.0).unwrap();
    assert_eq!(first.digest(), second.digest());
    assert_eq!(
        second_profiles.format_parts(
            &retained,
            value,
            RelativeUnit::Day,
            &PartitionLimits::HOST_ABI
        ),
        Err(RelativeTimeError::InvalidLocale)
    );
    let original = RelativeRequest::Parts {
        configuration: retained.clone(),
        value,
        unit: RelativeUnit::Day,
    };
    let decoded = decode_relative_request(
        RelativeHostOp::FormatRelativeTimeParts,
        &encode_relative_request(&original).unwrap(),
        &second_profiles,
        &number_profiles,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert_ne!(original, decoded);
    let RelativeRequest::Parts {
        configuration: selected_configuration,
        value,
        unit,
    } = decoded
    else {
        panic!("parts frame")
    };
    assert_eq!(
        second_profiles
            .format_parts(
                &selected_configuration,
                value,
                unit,
                &PartitionLimits::HOST_ABI
            )
            .unwrap()
            .to_text()
            .unwrap(),
        "hier"
    );
    let foreign = RelativeRequest::Parts {
        configuration: configuration(
            &full.profiles(),
            &number_profiles,
            "ar",
            RelativeStyle::Long,
            RelativeNumeric::Auto,
        ),
        value,
        unit,
    };
    assert!(decode_relative_request(
        RelativeHostOp::FormatRelativeTimeParts,
        &encode_relative_request(&foreign).unwrap(),
        &second_profiles,
        &number_profiles,
        &PartitionLimits::HOST_ABI
    )
    .is_err());
    assert!(first.uses_number_profiles(&number_profiles));
    let independently_admitted_numbers = NumberProfilesDataImage::from_bytes(
        numbers.bytes(),
        &locale,
        &crate::ListDataImage::for_profile(locale.profile().clone(), &locale).unwrap(),
    )
    .unwrap();
    assert_eq!(independently_admitted_numbers.digest(), numbers.digest());
    assert!(!first.uses_number_profiles(&independently_admitted_numbers.profiles()));
    assert!(first_profiles
        .resolve_locale(
            &request("fr"),
            &independently_admitted_numbers.profiles(),
            &PartitionLimits::HOST_ABI
        )
        .is_err());
    drop(independently_admitted_numbers);
    drop(foreign);
    drop(original);
    drop(full);
    drop(selected_configuration);
    drop(first);
    drop(second);
    drop(second_profiles);
    drop(numbers);
    drop(locale);
    drop(number_profiles);
    assert!(weak_number.upgrade().is_some());
    assert_eq!(
        text(&first_profiles, &retained, -1.0, RelativeUnit::Day),
        "hier"
    );
    drop(retained);
    assert!(
        weak_number.upgrade().is_some(),
        "catalogue owns the Number foundation"
    );
    drop(first_profiles);
    assert!(weak_number.upgrade().is_none());
}
