use super::*;

#[test]
fn selected_transitions_keep_all_actual_alias_records_and_full_identity_country_authority() {
    let id = CustomProfileId::parse("selected-zone-records").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let full = NamedTimeZoneDataImage::for_profile(profile.clone()).unwrap();
    let image = NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[TimeZoneId::parse("us/eastern").unwrap()],
    )
    .unwrap();
    let same = NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[
            TimeZoneId::parse("America/New_York").unwrap(),
            TimeZoneId::parse("UTC").unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(image.bytes(), same.bytes());
    assert!(image.bytes().len() < full.bytes().len());
    let envelope = DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::NamedTimeZones,
        NAMED_TIME_ZONE_IMAGE_MARKERS,
    )
    .unwrap();
    let admitted = admit(envelope.blob(), &profile).unwrap();
    let original = read_native(PINNED_PAYLOAD).unwrap();
    assert_eq!(admitted.native.catalogue, original.catalogue);
    assert_eq!(admitted.native.zone_tab, original.zone_tab);
    assert_eq!(admitted.native.regions, original.regions);
    let expected = original
        .records
        .iter()
        .copied()
        .filter(|(name, _)| {
            let identity = full
                .zones_ref()
                .lookup(&TimeZoneId::parse(*name).unwrap())
                .unwrap();
            ["America/New_York", "UTC"].contains(&identity.primary_identifier())
        })
        .collect::<Vec<_>>();
    assert_eq!(admitted.records(), expected);
    assert_eq!(
        image.zones_ref().identifiers().collect::<Vec<_>>(),
        full.zones_ref().identifiers().collect::<Vec<_>>()
    );
    let paris = image
        .zones_ref()
        .lookup(&TimeZoneId::parse("europe/paris").unwrap())
        .unwrap();
    assert_eq!(paris.primary_identifier(), "Europe/Paris");
    assert!(matches!(
        image
            .zones_ref()
            .transition(&paris, crate::TimeZoneEpochSeconds::new(0).unwrap()),
        Err(crate::NamedTimeZoneDataError::UnavailableIdentifier(_))
    ));
    let request = crate::provider::locale_time_zones::LocaleTimeZonesRequest::new(
        crate::CanonicalLocaleId::from_data("fr-FR").unwrap(),
    )
    .unwrap();
    assert_eq!(
        crate::provider::locale_time_zones::resolve_locale_time_zones(
            request.clone(),
            image.country_profiles_ref()
        )
        .unwrap(),
        crate::provider::locale_time_zones::resolve_locale_time_zones(
            request,
            full.country_profiles_ref()
        )
        .unwrap()
    );
    let countries = image.country_profiles();
    let weak = Arc::downgrade(&image.zones());
    drop(image);
    drop(full);
    drop(same);
    let retained = weak
        .upgrade()
        .expect("full country authority retains selected transition owner");
    assert!(countries.uses_named_zones(&retained));
    let alias = retained
        .lookup(&TimeZoneId::parse("US/Eastern").unwrap())
        .unwrap();
    assert_eq!(
        retained
            .transition(
                &alias,
                crate::TimeZoneEpochSeconds::new(1_609_459_200).unwrap()
            )
            .unwrap()
            .offset_seconds(),
        -18_000
    );
}

#[test]
fn projected_iana_admission_rederives_descriptor_and_every_retained_tzif_byte() {
    let id = CustomProfileId::parse("zone-record-damage").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let requested = [TimeZoneId::parse("America/New_York").unwrap()];
    let payload = produce(&id, &requested).unwrap();
    let changes: &[fn(&mut Descriptor, &mut Vec<u8>)] = &[
        |descriptor, _| descriptor.schema = 2,
        |descriptor, _| descriptor.custom_id = "foreign-zone-records".into(),
        |descriptor, _| descriptor.full_payload_sha256[0] ^= 1,
        |descriptor, _| descriptor.named_time_zones.reverse(),
        |descriptor, _| descriptor.named_time_zones.push("US/Eastern".into()),
        |descriptor, _| descriptor.named_time_zones.retain(|name| name != "UTC"),
        |_, native| {
            let last = native.len() - 1;
            native[last] ^= 1;
        },
        |_, native| {
            native.pop();
        },
    ];
    for change in changes {
        let mut reader = Reader {
            bytes: &payload,
            position: 8,
        };
        let mut descriptor: Descriptor = serde_json::from_slice(reader.bytes().unwrap()).unwrap();
        let mut native = reader.bytes().unwrap().to_vec();
        change(&mut descriptor, &mut native);
        let mut altered = MAGIC.to_vec();
        put(&mut altered, &serde_json::to_vec(&descriptor).unwrap()).unwrap();
        put(&mut altered, &native).unwrap();
        let framed = DataImageEnvelope::encode(
            DataImageComponent::NamedTimeZones,
            &profile,
            NAMED_TIME_ZONE_IMAGE_MARKERS,
            &altered,
        )
        .unwrap();
        assert!(NamedTimeZoneDataImage::from_bytes(framed).is_err());
    }
    for names in [
        vec![],
        vec!["America/New_York", "US/Eastern"],
        vec!["+01:00"],
        vec!["Unknown/Zone"],
    ] {
        let parsed = names
            .into_iter()
            .map(|name| TimeZoneId::parse(name).unwrap())
            .collect::<Vec<_>>();
        assert!(NamedTimeZoneDataImage::for_custom_projection(&id, &parsed).is_err());
    }
}
