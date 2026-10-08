use super::*;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{NumberLocaleRequest, PartitionLimits};
use crate::{
    format_duration_parts, CanonicalLocaleId, CheckedDurationConfiguration, CustomProfileId,
    DurationError, DurationOptions, DurationRecord, DurationStyle,
};

fn request(locale: &str) -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn configuration(
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
    locale: &str,
    style: DurationStyle,
) -> CheckedDurationConfiguration {
    CheckedDurationConfiguration::new(
        profiles.resolve_locale(&request(locale), numbers).unwrap(),
        DurationOptions {
            style,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn dynamic_image_retains_selected_number_list_and_digital_consumers() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = NumberProfilesDataImage::from_bytes(
        crate::embedded_number_profiles_data_image()
            .unwrap()
            .bytes(),
        &locale,
        &crate::embedded_list_data_image().unwrap(),
    )
    .unwrap();
    let lists =
        ListDataImage::from_bytes(crate::embedded_list_data_image().unwrap().bytes(), &locale)
            .unwrap();
    let first = DurationDataImage::for_profile(IntlDataProfile::Minimal, &locale, &numbers, &lists)
        .unwrap();
    let source: Arc<[u8]> = first.bytes().as_ref().to_vec().into();
    let admitted =
        DurationDataImage::from_bytes(source.clone(), &locale, &numbers, &lists).unwrap();
    let profiles = admitted.profiles();
    assert!(admitted.uses_foundations(&numbers.profiles(), &lists.profiles()));
    let digital = configuration(&profiles, &numbers.profiles(), "sr", DurationStyle::Digital);
    let text = configuration(&profiles, &numbers.profiles(), "en", DurationStyle::Long);
    drop(source);
    drop(first);
    drop(admitted);
    drop(numbers);
    drop(lists);
    let record =
        DurationRecord::from_number_fields([0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        format_duration_parts(&digital, &record, &profiles, &PartitionLimits::HOST_ABI)
            .unwrap()
            .to_text()
            .unwrap(),
        "1.02.03"
    );
    let record =
        DurationRecord::from_number_fields([1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        format_duration_parts(&text, &record, &profiles, &PartitionLimits::HOST_ABI)
            .unwrap()
            .to_text()
            .unwrap(),
        "1 year, 2 months"
    );
    assert_eq!(profiles.available_locales().len(), 15);
}

#[test]
fn identical_foundations_do_not_authorize_foreign_empty_numeric_or_text_templates() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let lists = crate::embedded_list_data_image().unwrap();
    let first = crate::embedded_duration_data_image().unwrap();
    let second = DurationDataImage::from_bytes(first.bytes(), &locale, &numbers, &lists).unwrap();
    assert_eq!(first.digest(), second.digest());
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
        let selected = configuration(&first.profiles(), &numbers.profiles(), "en", style);
        let record = DurationRecord::from_number_fields(fields).unwrap();
        assert!(format_duration_parts(
            &selected,
            &record,
            &first.profiles(),
            &PartitionLimits::HOST_ABI
        )
        .is_ok());
        assert_eq!(
            format_duration_parts(
                &selected,
                &record,
                &second.profiles(),
                &PartitionLimits::HOST_ABI
            ),
            Err(DurationError::InvalidLocale)
        );
    }
}

#[test]
fn dependency_digests_do_not_substitute_exact_number_or_list_owners() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let lists = crate::embedded_list_data_image().unwrap();
    let image = crate::embedded_duration_data_image().unwrap();
    let other_numbers =
        NumberProfilesDataImage::from_bytes(numbers.bytes(), &locale, &lists).unwrap();
    let other_lists = ListDataImage::from_bytes(lists.bytes(), &locale).unwrap();
    assert_eq!(numbers.digest(), other_numbers.digest());
    assert_eq!(lists.digest(), other_lists.digest());
    assert!(image.uses_foundations(&numbers.profiles(), &lists.profiles()));
    assert!(!image.uses_foundations(&other_numbers.profiles(), &lists.profiles()));
    assert!(!image.uses_foundations(&numbers.profiles(), &other_lists.profiles()));
    assert!(matches!(
        image
            .profiles()
            .resolve_locale(&request("en"), &other_numbers.profiles()),
        Err(DurationError::InvalidLocale)
    ));
}

#[test]
fn substituted_native_payload_and_mixed_profiles_cannot_claim_duration_admission() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let lists = crate::embedded_list_data_image().unwrap();
    let mut altered = PINNED_PAYLOAD.to_vec();
    altered.push(b' '); // Valid JSON, with a freshly coherent envelope digest.
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::DurationFormat,
        &IntlDataProfile::Minimal,
        MARKERS,
        &altered,
    )
    .unwrap();
    assert!(matches!(
        DurationDataImage::from_bytes(bytes, &locale, &numbers, &lists),
        Err(IntlDataImageError::Consumer(_))
    ));
    let profile =
        IntlDataProfile::Custom(CustomProfileId::parse("duration-foundation-proof").unwrap());
    assert!(matches!(
        DurationDataImage::for_profile(profile.clone(), &locale, &numbers, &lists),
        Err(IntlDataImageError::Consumer(_))
    ));
    let custom_locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let custom_numbers =
        NumberProfilesDataImage::for_profile(profile.clone(), &custom_locale).unwrap();
    let custom_lists = ListDataImage::for_profile(profile.clone(), &custom_locale).unwrap();
    let custom = DurationDataImage::for_profile(
        profile.clone(),
        &custom_locale,
        &custom_numbers,
        &custom_lists,
    )
    .unwrap();
    assert_eq!(custom.profile(), &profile);
    assert!(custom.uses_foundations(&custom_numbers.profiles(), &custom_lists.profiles()));
    assert!(matches!(
        DurationDataImage::from_bytes(custom.bytes(), &custom_locale, &numbers, &custom_lists),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        DurationDataImage::for_profile(IntlDataProfile::Conformance, &locale, &numbers, &lists),
        Err(IntlDataImageError::Consumer(_))
    ));
}
