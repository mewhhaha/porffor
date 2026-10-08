use super::*;

#[test]
fn service_manifest_v6_publishes_exact_frames_and_preserves_dependent_number_owner() {
    let profile = CustomIntlProfile::from_manifest_json(r#"{"schema_version":6,"custom_id":"relative-only","services":["RelativeTimeFormat"],"locale_filters":{"relative_time":["fr"]}}"#).unwrap();
    let owner = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(profile));
    let selected = owner.selected().unwrap();
    let components: Vec<_> = selected
        .component_sections()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        components,
        [
            IntlDataComponent::Locale.section_name(),
            IntlDataComponent::List.section_name(),
            IntlDataComponent::Number.section_name(),
            IntlDataComponent::RelativeTime.section_name()
        ]
    );
    let requested = selected.identity().profile().services();
    assert!(requested.contains(IntlService::RelativeTimeFormat));
    assert!(!requested.contains(IntlService::NumberFormat));
    assert!(!requested.contains(IntlService::PluralRules));
    assert!(!requested.contains(IntlService::ListFormat));
    assert!(selected.provider.number_profiles().is_some());
    let provider = SelectedIntlDataBundle::from_export_bytes(&selected.export_bytes().unwrap())
        .unwrap()
        .into_provider();
    let kernel = crate::IntlKernel::new(selected.identity().clone(), provider).unwrap();
    assert!(kernel.operation::<crate::FormatNumberParts>().is_err());
    assert!(kernel.operation::<crate::ResolveNumberLocale>().is_err());
    assert!(selected
        .supported_values(SupportedValuesKey::NumberingSystem)
        .is_ok());
    assert!(matches!(
        selected.supported_values(SupportedValuesKey::Calendar),
        Err(SupportedValuesSetupError::UnavailableData(
            SupportedValuesKey::Calendar
        ))
    ));
    assert!(selected
        .supported_values(SupportedValuesKey::Currency)
        .is_ok());
    let bytes = selected.export_bytes().unwrap();
    assert_eq!(&bytes[..8], b"LILAB002");
    let admitted = SelectedIntlDataBundle::from_export_bytes(&bytes).unwrap();
    assert_eq!(admitted.identity(), selected.identity());
    assert_eq!(admitted.export_bytes().unwrap(), bytes);
}

#[test]
fn sparse_admission_rejects_missing_extra_reordered_and_permission_modified_frames() {
    let profile = CustomIntlProfile::for_services(
        CustomProfileId::parse("list-only").unwrap(),
        &["ListFormat"],
    )
    .unwrap();
    let owner = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(profile));
    let selected = owner.selected().unwrap();
    let selection = selected.service_selection().unwrap().clone();
    let frames: Vec<_> = selection
        .components()
        .iter()
        .zip(
            selected
                .component_sections()
                .into_iter()
                .map(|(_, bytes)| bytes),
        )
        .collect();
    assert_eq!(frames.len(), 2);
    let mut missing = frames.clone();
    missing.pop();
    assert!(
        SelectedIntlDataBundle::from_component_sections(Some(selection.clone()), missing).is_err()
    );
    let mut extra = frames.clone();
    extra.push(frames[1].clone());
    assert!(
        SelectedIntlDataBundle::from_component_sections(Some(selection.clone()), extra).is_err()
    );
    let mut reordered = frames.clone();
    reordered.reverse();
    assert!(SelectedIntlDataBundle::from_component_sections(Some(selection), reordered).is_err());
    let collator =
        CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(IntlService::Collator))
            .unwrap();
    assert!(SelectedIntlDataBundle::from_component_sections(Some(collator), frames).is_err());
    let bytes = selected.export_bytes().unwrap();
    for wire in [
        0u16,
        1u16 << 15,
        CheckedIntlServiceSelection::new(IntlServiceSet::EMPTY.with(IntlService::Collator))
            .unwrap()
            .wire(),
    ] {
        let mut damaged = bytes.to_vec();
        damaged[14..16].copy_from_slice(&wire.to_le_bytes());
        assert!(SelectedIntlDataBundle::from_export_bytes(&damaged).is_err());
    }
}

#[test]
fn service_manifest_rejects_unknown_duplicates_null_and_unused_physical_dimensions() {
    for json in [
        r#"{"schema_version":6,"custom_id":"bad","services":[]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":null}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat","ListFormat"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["listformat"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat"],"currency_codes":["EUR"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["NumberFormat"],"numbering_systems":["deva"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat"],"locale_filters":{"date_time":["fr"]}}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat"],"services":["Collator"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat"],"unknown":true}"#,
    ] {
        assert!(
            CustomIntlProfile::from_manifest_json(json).is_err(),
            "{json}"
        );
    }
    let id = CustomProfileId::parse("composition").unwrap();
    assert!(CustomIntlProfile::for_services(id.clone(), &["ListFormat"])
        .unwrap()
        .with_date_time_calendars(&["chinese"])
        .is_err());
    assert!(CustomIntlProfile::for_currency_codes(id, &["EUR"])
        .unwrap()
        .with_services(&["NumberFormat"])
        .is_err());
}

#[test]
fn complete_legacy_export_and_explicit_all_preserve_frames_with_distinct_framing() {
    let id = CustomProfileId::parse("complete-services").unwrap();
    let full_owner = IntlDataSelection::new(IntlCompilationProfile::Custom(id.clone()));
    let services: Vec<_> = IntlService::ALL
        .iter()
        .map(|service| service.name())
        .collect();
    let explicit_owner = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::for_services(id, &services).unwrap(),
    ));
    let full = full_owner.selected().unwrap();
    let explicit = explicit_owner.selected().unwrap();
    assert_eq!(full.identity(), explicit.identity());
    assert_eq!(full.component_sections(), explicit.component_sections());
    assert_eq!(&full.export_bytes().unwrap()[..8], b"LILAB001");
    assert_eq!(&explicit.export_bytes().unwrap()[..8], b"LILAB002");
    assert_eq!(
        SelectedIntlDataBundle::from_export_bytes(&explicit.export_bytes().unwrap())
            .unwrap()
            .component_sections(),
        full.component_sections()
    );
}
