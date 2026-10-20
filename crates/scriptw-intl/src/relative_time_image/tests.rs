use super::*;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{
    resolve_number_locale, NumberFormatKernelError, NumberLocaleRequest, PartitionLimits,
};
use crate::{
    decode_relative_request, encode_relative_request, CanonicalLocaleId, CustomProfileId,
    FiniteRelativeNumber, RelativeHostOp, RelativeNumeric, RelativeRequest, RelativeStyle,
    RelativeTimeConfiguration, RelativeTimeError, RelativeUnit, ResolvedRelativeTimeLocale,
};

fn request(locale: &str) -> NumberLocaleRequest {
    NumberLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: None,
    }
}
fn configuration(
    profiles: &RelativeProfiles,
    numbers: &Arc<NumberProfiles>,
    numeric: RelativeNumeric,
) -> RelativeTimeConfiguration {
    let locale = profiles
        .resolve_locale(&request("en"), numbers, &PartitionLimits::HOST_ABI)
        .unwrap();
    RelativeTimeConfiguration::new(locale, RelativeStyle::Long, numeric, numbers).unwrap()
}

#[test]
fn image_consumers_retain_templates_and_number_owner_after_handles_drop() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = NumberProfilesDataImage::from_bytes(
        crate::embedded_number_profiles_data_image()
            .unwrap()
            .bytes(),
        &locale,
        &crate::embedded_list_data_image().unwrap(),
    )
    .unwrap();
    let weak_number = Arc::downgrade(&numbers.profiles());
    let bytes: Arc<[u8]> = embedded_relative_time_data_image()
        .unwrap()
        .bytes()
        .as_ref()
        .to_vec()
        .into();
    let image = RelativeTimeDataImage::from_bytes(bytes.clone(), &locale, &numbers).unwrap();
    let profiles = image.profiles();
    let retained = configuration(&profiles, &numbers.profiles(), RelativeNumeric::Always);
    drop(bytes);
    drop(image);
    drop(numbers);
    drop(locale);
    assert!(weak_number.upgrade().is_some());
    assert_eq!(profiles.available_locales().len(), 14);
    assert_eq!(
        profiles
            .format_parts(
                &retained,
                FiniteRelativeNumber::new(-12.345).unwrap(),
                RelativeUnit::Day,
                &PartitionLimits::HOST_ABI,
            )
            .unwrap()
            .to_text()
            .unwrap(),
        "12.345 days ago"
    );
    drop(retained);
    // At this point the catalogue itself keeps the selected Number owner alive.
    assert!(weak_number.upgrade().is_some());
    let numbers = weak_number.upgrade().unwrap();
    for locale in profiles.available_locales() {
        let resolved = profiles
            .resolve_locale(&request(locale), &numbers, &PartitionLimits::HOST_ABI)
            .unwrap();
        for style in RelativeStyle::ALL {
            let configuration = RelativeTimeConfiguration::new(
                resolved.clone(),
                *style,
                RelativeNumeric::Always,
                &numbers,
            )
            .unwrap();
            for unit in RelativeUnit::ALL {
                assert!(!profiles
                    .format_parts(
                        &configuration,
                        FiniteRelativeNumber::new(2.0).unwrap(),
                        *unit,
                        &PartitionLimits::HOST_ABI,
                    )
                    .unwrap()
                    .parts()
                    .is_empty());
            }
        }
    }
}

#[test]
fn same_number_owner_cannot_exchange_foreign_auto_templates_and_wire_remints_selected() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let first = embedded_relative_time_data_image().unwrap();
    let second = RelativeTimeDataImage::from_bytes(first.bytes(), &locale, &numbers).unwrap();
    assert_eq!(first.digest(), second.digest());
    assert!(first.uses_number_profiles(&numbers.profiles()));
    assert!(second.uses_number_profiles(&numbers.profiles()));
    let first_profiles = first.profiles();
    let second_profiles = second.profiles();
    let retained = configuration(&first_profiles, &numbers.profiles(), RelativeNumeric::Auto);
    let value = FiniteRelativeNumber::new(-1.0).unwrap();
    assert_eq!(
        first_profiles
            .format_parts(
                &retained,
                value,
                RelativeUnit::Day,
                &PartitionLimits::HOST_ABI
            )
            .unwrap()
            .to_text()
            .unwrap(),
        "yesterday"
    );
    assert_eq!(
        second_profiles.format_parts(
            &retained,
            value,
            RelativeUnit::Day,
            &PartitionLimits::HOST_ABI,
        ),
        Err(RelativeTimeError::InvalidLocale)
    );
    let original = RelativeRequest::Parts {
        configuration: retained,
        value,
        unit: RelativeUnit::Day,
    };
    let bytes = encode_relative_request(&original).unwrap();
    let decoded = decode_relative_request(
        RelativeHostOp::FormatRelativeTimeParts,
        &bytes,
        &second_profiles,
        &numbers.profiles(),
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert_ne!(original, decoded);
    let RelativeRequest::Parts {
        configuration,
        value,
        unit,
    } = decoded
    else {
        panic!("parts frame")
    };
    assert_eq!(
        second_profiles
            .format_parts(&configuration, value, unit, &PartitionLimits::HOST_ABI)
            .unwrap()
            .to_text()
            .unwrap(),
        "yesterday"
    );
}

#[test]
fn self_consistent_altered_native_templates_cannot_claim_pinned_image_identity() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let numbers = crate::embedded_number_profiles_data_image().unwrap();
    let mut altered = PINNED_PROFILE.to_vec();
    let position = altered
        .windows(b"yesterday".len())
        .position(|bytes| bytes == b"yesterday")
        .unwrap();
    altered[position] = b'Y';
    for payload in [altered.as_slice(), b"{}".as_slice()] {
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::RelativeTime,
            &IntlDataProfile::Minimal,
            RELATIVE_TIME_IMAGE_MARKERS,
            payload,
        )
        .unwrap();
        assert!(matches!(
            RelativeTimeDataImage::from_bytes(bytes, &locale, &numbers),
            Err(IntlDataImageError::Consumer(_))
        ));
    }
}

#[test]
fn mixed_foundations_and_foreign_number_proofs_fail_before_consumer_publication() {
    let minimal = embedded_relative_time_data_image().unwrap();
    let minimal_locale = crate::embedded_locale_data_image().unwrap();
    let minimal_numbers = crate::embedded_number_profiles_data_image().unwrap();
    let profile = IntlDataProfile::Custom(CustomProfileId::parse("relative-native-owner").unwrap());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let numbers = NumberProfilesDataImage::for_profile(profile.clone(), &locale).unwrap();
    assert!(RelativeTimeDataImage::from_bytes(minimal.bytes(), &locale, &numbers).is_err());
    assert!(
        RelativeTimeDataImage::for_profile(profile.clone(), &locale, &minimal_numbers).is_err()
    );
    assert!(
        RelativeTimeDataImage::for_profile(profile.clone(), &minimal_locale, &numbers).is_err()
    );
    let custom = RelativeTimeDataImage::for_profile(profile, &locale, &numbers).unwrap();
    assert_ne!(minimal.digest(), custom.digest());
    assert!(custom.uses_number_profiles(&numbers.profiles()));
    assert!(!custom.uses_number_profiles(&minimal_numbers.profiles()));
    let foreign = resolve_number_locale(&request("en"), &minimal_numbers.profiles()).unwrap();
    assert_eq!(
        ResolvedRelativeTimeLocale::from_resolved(foreign, &custom.profiles()),
        Err(RelativeTimeError::Number(
            NumberFormatKernelError::InvalidResolvedLocale
        ))
    );
    assert_eq!(
        custom.profiles().resolve_locale(
            &request("en"),
            &minimal_numbers.profiles(),
            &PartitionLimits::HOST_ABI,
        ),
        Err(RelativeTimeError::Number(
            NumberFormatKernelError::InvalidResolvedLocale
        ))
    );
    let configuration = configuration(
        &custom.profiles(),
        &numbers.profiles(),
        RelativeNumeric::Auto,
    );
    assert_eq!(
        custom
            .profiles()
            .format_parts(
                &configuration,
                FiniteRelativeNumber::new(-1.0).unwrap(),
                RelativeUnit::Day,
                &PartitionLimits::HOST_ABI,
            )
            .unwrap()
            .to_text()
            .unwrap(),
        "yesterday"
    );
    assert!(matches!(
        RelativeTimeDataImage::for_profile(IntlDataProfile::Conformance, &locale, &numbers),
        Err(IntlDataImageError::Consumer(_))
    ));
}
