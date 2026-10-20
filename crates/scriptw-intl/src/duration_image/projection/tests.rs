use super::*;
use crate::duration_wire::{DurationPartitionRequest, DurationWireRequest};
use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::{NumberProfiles, PartitionLimits};
use crate::{
    decode_duration_request, encode_duration_request, execute_duration_request,
    format_duration_parts, CheckedDurationConfiguration, CustomListProfile, DurationError,
    DurationOptions, DurationRecord, DurationStyle, DurationSupportedLocalesRequest,
};
use std::sync::Arc;

fn setup() -> (
    CustomProfileId,
    LocaleDataImage,
    NumberProfilesDataImage,
    ListDataImage,
) {
    let id = CustomProfileId::parse("actual-duration-projection").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let numbers = NumberProfilesDataImage::for_profile(profile.clone(), &locale).unwrap();
    let lists = ListDataImage::for_profile(profile, &locale).unwrap();
    (id, locale, numbers, lists)
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
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
    name: &str,
    style: DurationStyle,
) -> CheckedDurationConfiguration {
    CheckedDurationConfiguration::new(
        profiles.resolve_locale(&request(name), numbers).unwrap(),
        DurationOptions {
            style,
            ..Default::default()
        },
    )
    .unwrap()
}
fn envelope(image: &DurationDataImage) -> DataImageEnvelope {
    DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DurationFormat,
        super::super::MARKERS,
    )
    .unwrap()
}
fn frame(payload: &[u8], profile: &IntlDataProfile) -> Arc<[u8]> {
    DataImageEnvelope::encode(
        DataImageComponent::DurationFormat,
        profile,
        super::super::MARKERS,
        payload,
    )
    .unwrap()
}

#[test]
fn complete_selected_rows_shrink_and_keep_all_styles_units_and_actual_fallback() {
    let (id, locale, numbers, lists) = setup();
    let full = DurationDataImage::for_profile(
        IntlDataProfile::Custom(id.clone()),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap();
    let image = DurationDataImage::for_custom_projection(
        &id,
        &requested(&["fr", "sr"]),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap();
    let framed = envelope(&image);
    let (descriptor, blob) = split_payload(framed.blob()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
    let source: serde_json::Value = serde_json::from_slice(PINNED_PAYLOAD).unwrap();
    let rows = raw["locales"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["locale"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["en-US", "fr", "sr"]
    );
    for row in rows {
        assert_eq!(row["units"].as_array().unwrap().len(), 30);
        assert_eq!(row["lists"].as_array().unwrap().len(), 3);
        assert_eq!(
            Some(row),
            source["locales"]
                .as_array()
                .unwrap()
                .iter()
                .find(|original| original["locale"] == row["locale"])
        );
    }
    assert!(blob.len() < PINNED_PAYLOAD.len());
    assert!(image.bytes().len() < full.bytes().len());
    assert_eq!(descriptor.public_locales, ["en-US", "fr", "sr"]);
    assert_eq!(descriptor.associations.len(), 3);
    let selected = image.profiles();
    let baseline = full.profiles();
    let number_profiles = numbers.profiles();
    assert_eq!(
        selected
            .available_locales()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["en-US", "fr", "sr"]
    );
    assert_eq!(
        selected
            .supported_locales(DurationSupportedLocalesRequest {
                requested: ["ja", "en", "fr-FR", "sr-RS"]
                    .map(|name| CanonicalLocaleId::from_data(name).unwrap())
                    .into(),
                matcher: LocaleMatcher::Lookup,
            })
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["fr-FR", "sr-RS"]
    );
    assert_eq!(
        selected
            .resolve_locale(&request("ja"), &number_profiles)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    let records = [
        [0.0; 10],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        [-1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 456.0, 789.0, 123.0],
    ];
    for name in ["en-US", "fr", "sr"] {
        for &style in DurationStyle::ALL {
            let projected = configuration(&selected, &number_profiles, name, style);
            let original = configuration(&baseline, &number_profiles, name, style);
            for fields in records {
                let record = DurationRecord::from_number_fields(fields).unwrap();
                assert_eq!(
                    format_duration_parts(
                        &projected,
                        &record,
                        &selected,
                        &PartitionLimits::HOST_ABI
                    )
                    .unwrap(),
                    format_duration_parts(
                        &original,
                        &record,
                        &baseline,
                        &PartitionLimits::HOST_ABI
                    )
                    .unwrap(),
                );
            }
        }
    }
}

#[test]
fn canonical_selection_rejects_duplicates_unknown_rows_and_replacement_data() {
    let (id, locale, numbers, lists) = setup();
    let profile = IntlDataProfile::Custom(id.clone());
    let image = DurationDataImage::for_custom_projection(
        &id,
        &requested(&["sr", "en-us", "fr"]),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap();
    let reordered = DurationDataImage::for_custom_projection(
        &id,
        &requested(&["fr", "sr"]),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap();
    assert_eq!(image.bytes(), reordered.bytes());
    for names in [
        &[][..],
        &["fr", "FR"][..],
        &["sr-Latn"][..],
        &["en-US-u-nu-arab"][..],
    ] {
        assert!(DurationDataImage::for_custom_projection(
            &id,
            &requested(names),
            &locale,
            &numbers,
            &lists
        )
        .is_err());
    }
    let many = vec![LocaleId::parse("fr").unwrap(); 16];
    assert!(
        DurationDataImage::for_custom_projection(&id, &many, &locale, &numbers, &lists).is_err()
    );
    let framed = envelope(&image);
    let (descriptor, blob) = split_payload(framed.blob()).unwrap();
    for field in ["digital", "units", "lists"] {
        let mut raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
        match field {
            "digital" => {
                raw["locales"][0]["digital"]["hms"]["pattern"] = serde_json::json!("H-mm-ss")
            }
            "units" => {
                raw["locales"][0]["units"].as_array_mut().unwrap().remove(0);
            }
            "lists" => {
                raw["locales"][0]["lists"][0]["patterns"][0] = serde_json::json!("{1} and {0}")
            }
            _ => unreachable!(),
        }
        let blob = serde_json::to_vec(&canonical_objects(raw)).unwrap();
        let payload = frame_payload(&descriptor, &blob).unwrap();
        assert!(DurationDataImage::from_bytes(
            frame(&payload, &profile),
            &locale,
            &numbers,
            &lists
        )
        .is_err());
    }
    let mut wrong_association = descriptor;
    wrong_association.associations[0].list_locale = "fr".into();
    let payload = frame_payload(&wrong_association, blob).unwrap();
    assert!(
        DurationDataImage::from_bytes(frame(&payload, &profile), &locale, &numbers, &lists)
            .is_err()
    );
    let (mut missing_default, blob) = split_payload(framed.blob()).unwrap();
    missing_default.public_locales.remove(0);
    let payload = frame_payload(&missing_default, blob).unwrap();
    assert!(
        DurationDataImage::from_bytes(frame(&payload, &profile), &locale, &numbers, &lists)
            .is_err()
    );
    let mut trailing = framed.blob().to_vec();
    trailing.push(0);
    assert!(
        DurationDataImage::from_bytes(frame(&trailing, &profile), &locale, &numbers, &lists)
            .is_err()
    );
    let mut extent = framed.blob().to_vec();
    extent[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        DurationDataImage::from_bytes(frame(&extent, &profile), &locale, &numbers, &lists).is_err()
    );
    let minimal_locale = crate::embedded_locale_data_image().unwrap();
    let minimal_numbers = crate::embedded_number_profiles_data_image().unwrap();
    let minimal_lists = crate::embedded_list_data_image().unwrap();
    assert!(DurationDataImage::from_bytes(
        frame(framed.blob(), &IntlDataProfile::Minimal),
        &minimal_locale,
        &minimal_numbers,
        &minimal_lists
    )
    .is_err());
    assert!(DurationDataImage::for_custom_projection(
        &id,
        &requested(&["fr"]),
        &minimal_locale,
        &numbers,
        &lists
    )
    .is_err());
}

#[test]
fn private_list_dependencies_retained_lifetime_and_wire_use_exact_selected_owners() {
    let (id, locale, numbers, _) = setup();
    let list_selection = CustomListProfile::new(id.clone(), &["ja"]).unwrap();
    let lists = ListDataImage::for_custom_projection(&list_selection, &locale).unwrap();
    assert!(!lists
        .profiles()
        .available_locales()
        .any(|name| name.as_str() == "sr"));
    // List's lawful private fifteen-locale Duration closure remains in that SAME
    // actual image. Selecting Duration rows does not claim Number/List shrinking.
    for name in DurationProfiles::required_list_locales() {
        assert!(lists
            .profiles()
            .resolve_duration_locale(&CanonicalLocaleId::from_data(*name).unwrap())
            .is_ok());
    }
    let image = DurationDataImage::for_custom_projection(
        &id,
        &requested(&["sr"]),
        &locale,
        &numbers,
        &lists,
    )
    .unwrap();
    let owned_bytes = image.bytes();
    let selected = image.profiles();
    assert!(image.uses_foundations(&numbers.profiles(), &lists.profiles()));
    let other_numbers =
        NumberProfilesDataImage::from_bytes(numbers.bytes(), &locale, &lists).unwrap();
    let other_lists = ListDataImage::from_bytes(lists.bytes(), &locale).unwrap();
    let other =
        DurationDataImage::from_bytes(owned_bytes.clone(), &locale, &other_numbers, &other_lists)
            .unwrap();
    assert_eq!(image.digest(), other.digest());
    assert!(!image.uses_foundations(&other_numbers.profiles(), &lists.profiles()));
    assert!(!image.uses_foundations(&numbers.profiles(), &other_lists.profiles()));
    let number_profiles = numbers.profiles();
    for (style, fields) in [
        (DurationStyle::Short, [0.0; 10]),
        (
            DurationStyle::Digital,
            [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0],
        ),
        (
            DurationStyle::Long,
            [1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ] {
        let selected_configuration = configuration(&selected, &number_profiles, "sr", style);
        let foreign = DurationWireRequest::Parts(
            DurationPartitionRequest::from_completed_number_fields(selected_configuration, fields)
                .unwrap(),
        );
        assert!(matches!(
            execute_duration_request(
                foreign.clone().into_native(),
                &other.profiles(),
                &other_numbers.profiles(),
                &PartitionLimits::HOST_ABI
            ),
            Err(DurationError::InvalidLocale)
        ));
        let decoded = decode_duration_request(
            foreign.operation(),
            &encode_duration_request(&foreign).unwrap(),
            &other.profiles(),
            &other_numbers.profiles(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        let selected_output = execute_duration_request(
            foreign.into_native(),
            &selected,
            &number_profiles,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        let reminted = execute_duration_request(
            decoded.into_native(),
            &other.profiles(),
            &other_numbers.profiles(),
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        match (selected_output, reminted) {
            (crate::DurationResponse::Parts(a), crate::DurationResponse::Parts(b)) => {
                assert_eq!(a, b)
            }
            _ => panic!("Parts operation changed its response domain"),
        }
    }
    let replacement_lists = ListDataImage::for_custom_projection(
        &CustomListProfile::new(id.clone(), &["fr"]).unwrap(),
        &locale,
    )
    .unwrap();
    assert_ne!(replacement_lists.digest(), lists.digest());
    assert!(
        DurationDataImage::from_bytes(owned_bytes, &locale, &numbers, &replacement_lists).is_err()
    );
    let retained = configuration(&selected, &number_profiles, "sr", DurationStyle::Digital);
    drop(image);
    drop(other);
    drop(locale);
    drop(numbers);
    drop(lists);
    drop(other_numbers);
    drop(other_lists);
    drop(number_profiles);
    let record =
        DurationRecord::from_number_fields([0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        format_duration_parts(&retained, &record, &selected, &PartitionLimits::HOST_ABI)
            .unwrap()
            .to_text()
            .unwrap(),
        "1.02.03"
    );
}
