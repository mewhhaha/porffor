use super::*;
use crate::{
    CanonicalLocaleId, CustomProfileId, ResolveTimeZoneRequest, TimeZoneEpochSeconds, TimeZoneId,
    TimeZoneNameStyle, TimeZoneSelection,
};

fn request(identifier: &str, epoch: i64, style: TimeZoneNameStyle) -> ResolveTimeZoneRequest {
    ResolveTimeZoneRequest::new(
        TimeZoneSelection::Named(TimeZoneId::parse(identifier).unwrap()),
        TimeZoneEpochSeconds::new(epoch).unwrap(),
        Some(style),
        CanonicalLocaleId::from_data("en-US").unwrap(),
    )
}

#[test]
fn actual_name_consumer_retains_iana_and_native_rows_after_image_and_bytes_drop() {
    let named = NamedTimeZoneDataImage::from_bytes(
        crate::embedded_named_time_zone_data_image()
            .unwrap()
            .bytes(),
    )
    .unwrap();
    let image = TimeZoneNamesDataImage::for_profile(IntlDataProfile::Minimal, &named).unwrap();
    let bytes: Arc<[u8]> = image.bytes().as_ref().to_vec().into();
    let decoded = TimeZoneNamesDataImage::from_bytes(bytes.clone(), &named).unwrap();
    let names = decoded.names();
    let weak_named = Arc::downgrade(&named.zones());
    drop(image);
    drop(decoded);
    drop(named);
    drop(bytes);
    assert!(weak_named.upgrade().is_some());
    for (epoch, offset, expected) in [
        (1_609_459_200, -28_800, "Pacific Standard Time"),
        (1_625_097_600, -25_200, "Pacific Daylight Time"),
    ] {
        let result = names
            .resolve(request("US/Pacific", epoch, TimeZoneNameStyle::Long))
            .unwrap();
        assert_eq!(result.offset_seconds(), offset);
        assert_eq!(result.display_name(), Some(expected));
    }
    let newer = names
        .resolve(request(
            "America/Coyhaique",
            1_745_000_000,
            TimeZoneNameStyle::Long,
        ))
        .unwrap();
    assert_eq!(newer.offset_seconds(), -10_800);
    assert_eq!(newer.display_name(), Some("GMT-03:00"));
    drop(names);
    assert!(weak_named.upgrade().is_none());
}

#[test]
fn equal_named_digest_cannot_substitute_a_foreign_actual_foundation() {
    let first = crate::embedded_named_time_zone_data_image().unwrap();
    let second = NamedTimeZoneDataImage::from_bytes(first.bytes()).unwrap();
    let image = TimeZoneNamesDataImage::for_profile(IntlDataProfile::Minimal, &first).unwrap();
    let rebound = TimeZoneNamesDataImage::from_bytes(image.bytes(), &second).unwrap();
    assert_eq!(first.digest(), second.digest());
    assert_eq!(image.digest(), rebound.digest());
    assert!(image.uses_named_zones(&first.zones()));
    assert!(!image.uses_named_zones(&second.zones()));
    assert!(!rebound.uses_named_zones(&first.zones()));
    assert!(rebound.uses_named_zones(&second.zones()));
    assert_eq!(
        rebound
            .names()
            .resolve(request(
                "Europe/London",
                1_625_097_600,
                TimeZoneNameStyle::Long
            ))
            .unwrap()
            .display_name(),
        Some("British Summer Time")
    );
}

#[test]
fn mixed_profiles_and_checksum_consistent_altered_native_rows_reject() {
    let minimal = crate::embedded_named_time_zone_data_image().unwrap();
    let image = TimeZoneNamesDataImage::for_profile(IntlDataProfile::Minimal, &minimal).unwrap();
    let custom = NamedTimeZoneDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("names-selected").unwrap(),
    ))
    .unwrap();
    assert!(TimeZoneNamesDataImage::from_bytes(image.bytes(), &custom).is_err());
    assert!(TimeZoneNamesDataImage::for_profile(IntlDataProfile::Minimal, &custom).is_err());
    let mut altered: serde_json::Value = serde_json::from_slice(PINNED_PAYLOAD).unwrap();
    altered["rows"]["zones"][0]["city"] = "Changed city".into();
    let payload = serde_json::to_vec(&altered).unwrap();
    let bytes = DataImageEnvelope::encode(
        DataImageComponent::TimeZoneNames,
        &IntlDataProfile::Minimal,
        TIME_ZONE_NAMES_IMAGE_MARKERS,
        &payload,
    )
    .unwrap();
    assert!(matches!(
        TimeZoneNamesDataImage::from_bytes(bytes, &minimal),
        Err(IntlDataImageError::Consumer(_))
    ));
    let admitted = TimeZoneNamesDataImage::for_profile(custom.profile().clone(), &custom).unwrap();
    assert_ne!(image.digest(), admitted.digest());
    assert_eq!(
        admitted
            .names()
            .resolve(request(
                "US/Eastern",
                1_609_459_200,
                TimeZoneNameStyle::Long
            ))
            .unwrap()
            .display_name(),
        Some("Eastern Standard Time")
    );
    assert!(matches!(
        TimeZoneNamesDataImage::for_profile(IntlDataProfile::Conformance, &minimal),
        Err(IntlDataImageError::Consumer(_))
    ));
}

#[test]
fn selected_name_rows_prune_actual_zone_and_metazone_data_without_changing_selected_styles() {
    let id = CustomProfileId::parse("selected-localized-zone-names").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let full_named = NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let full = TimeZoneNamesDataImage::for_profile(profile.clone(), &full_named).unwrap();
    let named = NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[TimeZoneId::parse("US/Eastern").unwrap()],
    )
    .unwrap();
    let image = TimeZoneNamesDataImage::for_profile(profile.clone(), &named).unwrap();
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::TimeZoneNames,
        TIME_ZONE_NAMES_IMAGE_MARKERS,
    )
    .unwrap();
    let catalogue = projection::admit(envelope.blob(), &profile, &named).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(catalogue.bytes()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(PINNED_PAYLOAD).unwrap();
    for field in ["locales", "fallback"] {
        assert_eq!(raw[field], original[field]);
    }
    assert_eq!(raw["rows"]["patterns"], original["rows"]["patterns"]);
    for row in raw["rows"]["zones"].as_array().unwrap() {
        let source = original["rows"]["zones"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["identifier"] == row["identifier"])
            .unwrap();
        assert_eq!(row, source);
    }
    for row in raw["rows"]["metazones"].as_array().unwrap() {
        let source = original["rows"]["metazones"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["identifier"] == row["identifier"])
            .unwrap();
        assert_eq!(row, source);
    }
    assert!(
        raw["rows"]["zones"].as_array().unwrap().len()
            < original["rows"]["zones"].as_array().unwrap().len()
    );
    assert!(image.bytes().len() < full.bytes().len());
    for style in TimeZoneNameStyle::ALL {
        for epoch in [1_609_459_200, 1_625_097_600, 4_102_444_800] {
            assert_eq!(
                image
                    .names_ref()
                    .resolve(request("US/Eastern", epoch, style))
                    .unwrap(),
                full.names_ref()
                    .resolve(request("US/Eastern", epoch, style))
                    .unwrap()
            );
        }
    }
    assert!(matches!(
        image
            .names_ref()
            .resolve(request("Europe/Paris", 0, TimeZoneNameStyle::Long)),
        Err(crate::TimeZoneResolveError::UnavailableNamedIdentifier(_))
    ));
    assert!(TimeZoneNamesDataImage::from_bytes(full.bytes(), &named).is_err());
    assert!(TimeZoneNamesDataImage::from_bytes(image.bytes(), &full_named).is_err());
    let foreign = NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[TimeZoneId::parse("Europe/Paris").unwrap()],
    )
    .unwrap();
    assert!(TimeZoneNamesDataImage::from_bytes(image.bytes(), &foreign).is_err());
    let retained = image.names();
    let weak = Arc::downgrade(&named.zones());
    drop(image);
    drop(named);
    assert!(weak.upgrade().is_some());
    assert_eq!(
        retained
            .resolve(request(
                "US/Eastern",
                1_609_459_200,
                TimeZoneNameStyle::Long
            ))
            .unwrap()
            .display_name(),
        Some("Eastern Standard Time")
    );
}
