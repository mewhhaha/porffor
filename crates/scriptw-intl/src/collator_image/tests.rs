use super::*;
use crate::collator::{
    CheckedCollatorConfiguration, CollatorLocaleRequest, CollatorOperationError, CollatorOrdering,
    CollatorSensitivity, CollatorUsage, CompareCollatorRequest,
};
use crate::number_format::options::LocaleMatcher;
use crate::{CanonicalLocaleId, CustomProfileId};

fn request(locale: &str, usage: CollatorUsage, numeric: bool) -> CollatorLocaleRequest {
    CollatorLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(locale).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        usage,
        collation: None,
        numeric: Some(numeric),
        case_first: None,
    }
}
fn compare(
    profiles: &CollatorProfiles,
    locale: &str,
    usage: CollatorUsage,
    numeric: bool,
    ignore: bool,
    left: &[u16],
    right: &[u16],
) -> CollatorOrdering {
    let selected = profiles.resolve(request(locale, usage, numeric)).unwrap();
    profiles
        .compare(
            CompareCollatorRequest::new(
                CheckedCollatorConfiguration::new(selected, CollatorSensitivity::Variant, ignore),
                left.into(),
                right.into(),
            )
            .unwrap(),
        )
        .unwrap()
}
fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

#[test]
fn metadata_inventory_support_matches_actual_icu_fallback() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let provider = CollatorImageProvider::checked(
        PINNED_BLOB,
        &locale,
        CollatorLocaleInventory::from_bytes(PINNED_LOCALES).unwrap(),
    )
    .unwrap();
    for (name, attributes, supported) in [
        ("de-AT", "phonebk", true),
        ("de-CH", "phonebk", true),
        ("en-US", "phonebk", false),
        ("en-US", "", true),
        ("en-US", "search", true),
        ("qaa", "emoji", true),
        ("qaa", "eor", true),
        ("qaa", "foobar", false),
    ] {
        let id = DataIdentifierCow::from_owned(
            DataMarkerAttributes::try_from_str(attributes)
                .unwrap()
                .to_owned(),
            name.parse().unwrap(),
        );
        assert_eq!(provider.supports_metadata_id(&id), supported, "{id}");
        let actual = DataProvider::<CollationMetadataV1>::load(
            &provider,
            DataRequest {
                id: id.as_borrowed(),
                metadata: Default::default(),
            },
        );
        match actual {
            Ok(_) => assert!(supported, "unexpected real metadata for {id}"),
            Err(error) => {
                assert!(!supported, "missing expected metadata for {id}: {error}");
                assert_eq!(error.kind, DataErrorKind::IdentifierNotFound);
            }
        }
    }
}

#[test]
fn pinned_image_consumes_search_fallback_nfd_and_conditional_numeric_data() {
    let image = embedded_collator_data_image().unwrap();
    let profiles = image.profiles();
    assert_eq!(
        image.locale_digest(),
        crate::embedded_locale_data_image().unwrap().digest()
    );
    assert_eq!(
        compare(
            &profiles,
            "de-CH",
            CollatorUsage::Sort,
            false,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Greater
    );
    assert_eq!(
        compare(
            &profiles,
            "de-CH",
            CollatorUsage::Search,
            false,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Less
    );
    assert_eq!(
        compare(
            &profiles,
            "de-CH",
            CollatorUsage::Sort,
            false,
            false,
            &units("é"),
            &units("e\u{0301}")
        ),
        CollatorOrdering::Equal
    );
    assert_eq!(
        compare(
            &profiles,
            "de-CH",
            CollatorUsage::Sort,
            true,
            true,
            &units("a-2"),
            &units("a10")
        ),
        CollatorOrdering::Less
    );
    assert_eq!(
        compare(
            &profiles,
            "en-US",
            CollatorUsage::Sort,
            false,
            false,
            &[0xD800],
            &[0xFFFD]
        ),
        CollatorOrdering::Equal
    );
}

#[test]
fn identical_image_bytes_do_not_admit_a_foreign_resolved_configuration() {
    let first = embedded_collator_data_image().unwrap();
    let locale = crate::embedded_locale_data_image().unwrap();
    let second = CollatorDataImage::from_bytes(first.bytes(), &locale).unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.digest(), second.digest());
    let selected = first
        .profiles()
        .resolve(request("de-CH", CollatorUsage::Sort, true))
        .unwrap();
    let request = CompareCollatorRequest::new(
        CheckedCollatorConfiguration::new(selected, CollatorSensitivity::Base, false),
        units("é").into_boxed_slice(),
        units("e").into_boxed_slice(),
    )
    .unwrap();
    assert_eq!(
        second.profiles().compare(request),
        Err(CollatorOperationError::InvalidConfiguration)
    );
}

#[test]
fn self_consistent_manifest_and_digest_cannot_substitute_other_payload_bytes() {
    let locale = crate::embedded_locale_data_image().unwrap();
    let altered = DataImageEnvelope::encode(
        DataImageComponent::Collator,
        &IntlDataProfile::Minimal,
        &MARKERS,
        b"other payload",
    )
    .unwrap();
    assert!(matches!(
        CollatorDataImage::from_bytes(altered, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));

    // A well-framed ICU image cannot silently narrow the admitted locale domain.
    let mut descriptor: serde_json::Value = serde_json::from_slice(PINNED_LOCALES).unwrap();
    descriptor["locales"].as_array_mut().unwrap().remove(0);
    let descriptor = serde_json::to_vec(&descriptor).unwrap();
    let mut payload = MAGIC.to_vec();
    for part in [descriptor.as_slice(), PINNED_BLOB] {
        payload.extend_from_slice(&u64::try_from(part.len()).unwrap().to_le_bytes());
        payload.extend_from_slice(part);
    }
    let altered = DataImageEnvelope::encode(
        DataImageComponent::Collator,
        &IntlDataProfile::Minimal,
        &MARKERS,
        &payload,
    )
    .unwrap();
    assert!(matches!(
        CollatorDataImage::from_bytes(altered, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
}

#[test]
fn selected_locale_profile_must_match_and_conformance_is_not_minted() {
    let image = embedded_collator_data_image().unwrap();
    let custom = IntlDataProfile::Custom(CustomProfileId::parse("collator-profile-proof").unwrap());
    let locale = LocaleDataImage::for_profile(custom).unwrap();
    assert!(matches!(
        CollatorDataImage::from_bytes(image.bytes(), &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
    assert!(matches!(
        CollatorDataImage::for_profile(IntlDataProfile::Conformance, &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
}
