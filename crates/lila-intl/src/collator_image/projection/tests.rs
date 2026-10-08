use super::*;
use crate::collator::{
    CheckedCollatorConfiguration, CollatorCaseFirst, CollatorCollationOption,
    CollatorLocaleRequest, CollatorOperationError, CollatorSensitivity, CollatorUsage,
    CompareCollatorRequest,
};
use crate::number_format::options::LocaleMatcher;
use crate::{CollatorDataImage, IntlDataDigest};

fn locale(id: &str) -> (CustomProfileId, LocaleDataImage) {
    let id = CustomProfileId::parse(id).unwrap();
    let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(id.clone())).unwrap();
    (id, locale)
}
fn selected(names: &[&str]) -> Vec<LocaleId> {
    names
        .iter()
        .map(|name| LocaleId::parse(*name).unwrap())
        .collect()
}
fn request(
    name: &str,
    usage: CollatorUsage,
    collation: Option<&str>,
    numeric: bool,
    case_first: CollatorCaseFirst,
) -> CollatorLocaleRequest {
    CollatorLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        usage,
        collation: collation.map(|value| CollatorCollationOption::parse(value).unwrap()),
        numeric: Some(numeric),
        case_first: Some(case_first),
    }
}
fn comparison(
    profiles: &CollatorProfiles,
    name: &str,
    usage: CollatorUsage,
    collation: Option<&str>,
    numeric: bool,
    case_first: CollatorCaseFirst,
    sensitivity: CollatorSensitivity,
    ignore: bool,
    left: &str,
    right: &str,
) -> CompareCollatorRequest {
    let resolved = profiles
        .resolve(request(name, usage, collation, numeric, case_first))
        .unwrap();
    CompareCollatorRequest::new(
        CheckedCollatorConfiguration::new(resolved, sensitivity, ignore),
        left.encode_utf16().collect::<Vec<_>>().into_boxed_slice(),
        right.encode_utf16().collect::<Vec<_>>().into_boxed_slice(),
    )
    .unwrap()
}

#[test]
fn metadata_inventory_support_does_not_borrow_absent_projection_rows() {
    let (_, locale) = locale("collator-metadata-domain");
    let source = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB).unwrap();
    let keys = BTreeSet::from([RowKey {
        marker: Marker::Metadata,
        id: DataIdentifierCow::default(),
    }]);
    let provider =
        CollatorImageProvider::projected(RawRows::from_pinned(&source, &keys).unwrap(), &locale)
            .unwrap();
    let mut id = DataIdentifierCow::from_locale("de-CH".parse().unwrap());
    assert!(provider.supports_metadata_id(&id));
    for attributes in ["search", "emoji", "eor", "phonebk"] {
        id.marker_attributes = std::borrow::Cow::Owned(
            DataMarkerAttributes::try_from_str(attributes)
                .unwrap()
                .to_owned(),
        );
        assert!(!provider.supports_metadata_id(&id), "{id}");
    }
}

#[test]
fn physical_rows_preserve_complete_selected_sort_search_options_and_private_locale_preferences() {
    let (id, locale) = locale("collator-native-projection");
    let full =
        CollatorDataImage::for_profile(IntlDataProfile::Custom(id.clone()), &locale).unwrap();
    let projected =
        CollatorDataImage::for_custom_projection(&id, &selected(&["sv", "de-CH"]), &locale)
            .unwrap();
    assert!(projected.bytes().len() < full.bytes().len());
    let projection = admit(
        super::super::DataImageEnvelope::decode(
            projected.bytes(),
            super::super::DataImageComponent::Collator,
            &super::super::MARKERS,
        )
        .unwrap()
        .blob(),
        projected.profile(),
        &locale,
    )
    .unwrap();
    let source = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB).unwrap();
    let mut full_rows = 0;
    for marker in [
        icu_collator::provider::CollationRootV1::INFO,
        icu_collator::provider::CollationTailoringV1::INFO,
        icu_collator::provider::CollationMetadataV1::INFO,
        icu_collator::provider::CollationDiacriticsV1::INFO,
        icu_collator::provider::CollationJamoV1::INFO,
        icu_collator::provider::CollationReorderingV1::INFO,
        icu_collator::provider::CollationSpecialPrimariesV1::INFO,
        icu_normalizer::provider::NormalizerNfdDataV1::INFO,
        icu_normalizer::provider::NormalizerNfdTablesV1::INFO,
    ] {
        full_rows += source.iter_ids_for_marker(marker).unwrap().len();
    }
    assert!(projection.rows.iter().count() < full_rows);
    for (key, row) in projection.rows.iter() {
        assert_eq!(
            row.payload.get(),
            source
                .load_data(
                    key.marker.info(),
                    DataRequest {
                        id: key.id.as_borrowed(),
                        metadata: Default::default(),
                    }
                )
                .unwrap()
                .payload
                .get()
        );
    }
    let public = projected.profiles();
    let broad = full.profiles();
    assert_eq!(
        public
            .available_locales()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["de-CH", "en-US", "sv"]
    );
    for name in ["de-CH", "en-US", "sv"] {
        for usage in CollatorUsage::ALL {
            // Default/search and the real German phonebook option traverse the
            // conditional rows under all closed option combinations.
            let choices = if name == "de-CH" && *usage == CollatorUsage::Sort {
                vec![None, Some("phonebk")]
            } else {
                vec![None]
            };
            for collation in choices {
                for numeric in [false, true] {
                    for ignore in [false, true] {
                        for case_first in CollatorCaseFirst::ALL {
                            for sensitivity in CollatorSensitivity::ALL {
                                for (left, right) in
                                    [("AE", "Ä"), ("é", "e\u{0301}"), ("a-2", "a10"), ("z", "å")]
                                {
                                    let actual = comparison(
                                        &public,
                                        name,
                                        *usage,
                                        collation,
                                        numeric,
                                        *case_first,
                                        *sensitivity,
                                        ignore,
                                        left,
                                        right,
                                    );
                                    let expected = comparison(
                                        &broad,
                                        name,
                                        *usage,
                                        collation,
                                        numeric,
                                        *case_first,
                                        *sensitivity,
                                        ignore,
                                        left,
                                        right,
                                    );
                                    assert_eq!(
                                        public.compare(actual).unwrap(),
                                        broad.compare(expected).unwrap()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for name in ["de", "de-DE-fonipa", "zh", "qaa-US"] {
        let name = CanonicalLocaleId::from_data(name).unwrap();
        assert_eq!(
            public.locale_sort_collations(&name).unwrap(),
            broad.locale_sort_collations(&name).unwrap()
        );
    }
    assert_eq!(
        public
            .resolve(request(
                "de",
                CollatorUsage::Sort,
                None,
                false,
                CollatorCaseFirst::False
            ))
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
}

#[test]
fn projected_configuration_keeps_its_owner_and_wire_remints_only_public_profiles() {
    let (id, locale) = locale("collator-owner-projection");
    let first = CollatorDataImage::for_custom_projection(&id, &selected(&["de-CH", "sv"]), &locale)
        .unwrap();
    let second = CollatorDataImage::from_bytes(first.bytes(), &locale).unwrap();
    assert_eq!(first.digest(), second.digest());
    let profiles = first.profiles();
    let request = comparison(
        &profiles,
        "de-CH",
        CollatorUsage::Search,
        None,
        true,
        CollatorCaseFirst::Upper,
        CollatorSensitivity::Variant,
        true,
        "a-2",
        "a10",
    );
    let bytes = request.encode().unwrap();
    assert_eq!(
        second.profiles().compare(request.clone()),
        Err(CollatorOperationError::InvalidConfiguration)
    );
    let rebound = CompareCollatorRequest::decode(&bytes, &second.profiles()).unwrap();
    assert_eq!(
        profiles.compare(request.clone()).unwrap(),
        second.profiles().compare(rebound).unwrap()
    );
    let full = CollatorDataImage::for_profile(IntlDataProfile::Custom(id), &locale).unwrap();
    let hidden = comparison(
        &full.profiles(),
        "de",
        CollatorUsage::Sort,
        None,
        false,
        CollatorCaseFirst::False,
        CollatorSensitivity::Variant,
        false,
        "a",
        "b",
    );
    assert!(CompareCollatorRequest::decode(&hidden.encode().unwrap(), &profiles).is_err());
    let expected = profiles.compare(request.clone()).unwrap();
    drop(first);
    drop(second);
    drop(locale);
    assert_eq!(profiles.compare(request).unwrap(), expected);
}

#[test]
fn exact_projection_recomputation_rejects_row_inventory_foundation_and_manifest_damage() {
    let (id, locale) = locale("collator-damage-projection");
    let projected = project(&id, &selected(&["de-CH", "sv"]), &locale).unwrap();
    let mut bytes = projected.bytes.clone();
    *bytes.last_mut().unwrap() ^= 1;
    assert!(admit(&bytes, &IntlDataProfile::Custom(id.clone()), &locale).is_err());
    let mut bytes = projected.bytes.clone();
    bytes.push(0);
    assert!(admit(&bytes, &IntlDataProfile::Custom(id.clone()), &locale).is_err());
    assert!(admit(&projected.bytes, &IntlDataProfile::Minimal, &locale).is_err());
    assert!(project(&id, &selected(&[]), &locale).is_err());
    assert!(project(&id, &selected(&["qaa"]), &locale).is_err());
    assert!(project(&id, &selected(&["he", "iw"]), &locale).is_err());
    assert!(project(&id, &selected(&["de", "DE"]), &locale).is_err());
    let other = CustomProfileId::parse("collator-other-projection").unwrap();
    assert!(admit(&projected.bytes, &IntlDataProfile::Custom(other), &locale).is_err());
    assert_ne!(locale.digest(), IntlDataDigest::from_sha256([0; 32]));
    let descriptor_len = u32::from_le_bytes(projected.bytes[8..12].try_into().unwrap()) as usize;
    let mut descriptor: Descriptor =
        serde_json::from_slice(&projected.bytes[12..12 + descriptor_len]).unwrap();
    descriptor.locale_digest = "00".repeat(32);
    let description = serde_json::to_vec(&descriptor).unwrap();
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&(description.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&description);
    bytes.extend_from_slice(&projected.bytes[12 + descriptor_len..]);
    assert!(admit(&bytes, &IntlDataProfile::Custom(id), &locale).is_err());
}

#[test]
fn conditional_numeric_shifted_data_is_required_by_the_real_projected_constructor() {
    let (id, locale) = locale("collator-conditional-projection");
    let projected = project(&id, &selected(&["de-CH"]), &locale).unwrap();
    let keys = projected
        .rows
        .iter()
        .filter(|(key, _)| key.marker != Marker::SpecialPrimaries)
        .map(|(key, _)| key.clone())
        .collect();
    let rows = RawRows::from_pinned(
        &BlobDataProvider::try_new_from_static_blob(PINNED_BLOB).unwrap(),
        &keys,
    )
    .unwrap();
    let incomplete =
        Arc::new(super::super::CollatorImageProvider::projected(rows, &locale).unwrap());
    assert!(CollatorProfiles::from_projection(incomplete, &projected.catalogue).is_err());
}
