use super::*;
use crate::provider::locale_time_zones::{resolve_locale_time_zones, LocaleTimeZonesRequest};
use crate::{CanonicalLocaleId, CustomProfileId, TimeZoneEpochSeconds, TimeZoneId};

#[test]
fn country_consumer_retains_actual_iana_owner_after_bytes_and_image_handles_drop() {
    let first = embedded_named_time_zone_data_image().unwrap();
    let bytes: Arc<[u8]> = first.bytes().as_ref().to_vec().into();
    let image = NamedTimeZoneDataImage::from_bytes(bytes.clone()).unwrap();
    let zones = image.zones();
    let countries = image.country_profiles();
    let weak_zones = Arc::downgrade(&zones);
    assert!(countries.uses_named_zones(&zones));
    drop(first);
    drop(bytes);
    drop(image);
    drop(zones);
    let retained = weak_zones
        .upgrade()
        .expect("country catalogue retains its IANA foundation");
    let identity = retained
        .lookup(&TimeZoneId::parse("europe/oslo").unwrap())
        .unwrap();
    assert_eq!(identity.identifier(), "Europe/Oslo");
    assert_eq!(identity.primary_identifier(), "Europe/Oslo");
    let snapshot = retained
        .transition(&identity, TimeZoneEpochSeconds::new(0).unwrap())
        .unwrap();
    assert_eq!(snapshot.offset_seconds(), 3600);
    let names = resolve_locale_time_zones(
        LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data("nb-NO-u-rg-uszzzz").unwrap())
            .unwrap(),
        &countries,
    )
    .unwrap();
    assert!(names
        .names()
        .iter()
        .any(|name| name.as_ref() == "Europe/Oslo"));
    assert!(names
        .names()
        .iter()
        .all(|name| name.as_ref() != "America/New_York"));
}

#[test]
fn identical_image_bytes_have_distinct_retained_iana_country_foundations() {
    let first = embedded_named_time_zone_data_image().unwrap();
    let second = NamedTimeZoneDataImage::from_bytes(first.bytes()).unwrap();
    assert_eq!(first.digest(), second.digest());
    assert!(!Arc::ptr_eq(&first.zones(), &second.zones()));
    assert!(first.country_profiles().uses_named_zones(&first.zones()));
    assert!(!first.country_profiles().uses_named_zones(&second.zones()));
    for image in [first, second] {
        let utc = image
            .zones()
            .lookup(&TimeZoneId::parse("Etc/UTC").unwrap())
            .unwrap();
        assert_eq!(utc.identifier(), "Etc/UTC");
        assert_eq!(utc.primary_identifier(), "UTC");
        for epoch in [-TimeZoneEpochSeconds::LIMIT, 0, TimeZoneEpochSeconds::LIMIT] {
            assert_eq!(
                image
                    .zones()
                    .transition(&utc, TimeZoneEpochSeconds::new(epoch).unwrap())
                    .unwrap()
                    .offset_seconds(),
                0
            );
        }
    }
}

#[test]
fn altered_or_incomplete_iana_payload_rejects_even_with_consistent_outer_checksum() {
    let mut altered = PINNED_PAYLOAD.to_vec();
    let last = altered.len() - 1;
    altered[last] ^= 1;
    for payload in [altered.as_slice(), MAGIC.as_slice()] {
        let bytes = DataImageEnvelope::encode(
            DataImageComponent::NamedTimeZones,
            &IntlDataProfile::Minimal,
            NAMED_TIME_ZONE_IMAGE_MARKERS,
            payload,
        )
        .unwrap();
        assert!(matches!(
            NamedTimeZoneDataImage::from_bytes(bytes),
            Err(IntlDataImageError::Consumer(_))
        ));
    }
}

#[test]
fn named_custom_frame_consumes_actual_country_and_transition_data_without_conformance_claim() {
    let first = embedded_named_time_zone_data_image().unwrap();
    let custom = NamedTimeZoneDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("iana-country-owner").unwrap(),
    ))
    .unwrap();
    assert_ne!(first.digest(), custom.digest());
    let request =
        LocaleTimeZonesRequest::new(CanonicalLocaleId::from_data("en-US").unwrap()).unwrap();
    let country = resolve_locale_time_zones(request, custom.country_profiles_ref()).unwrap();
    assert!(country
        .names()
        .iter()
        .any(|name| name.as_ref() == "America/New_York"));
    let named = custom
        .zones()
        .lookup(&TimeZoneId::parse("US/Eastern").unwrap())
        .unwrap();
    assert_eq!(named.identifier(), "US/Eastern");
    assert_eq!(named.primary_identifier(), "America/New_York");
    let complete = NamedTimeZoneDataImage::for_profile(IntlDataProfile::Conformance).unwrap();
    assert_eq!(complete.profile(), &IntlDataProfile::Conformance);
    assert_eq!(
        complete
            .zones()
            .lookup(&TimeZoneId::parse("US/Eastern").unwrap())
            .unwrap()
            .primary_identifier(),
        named.primary_identifier()
    );
}
