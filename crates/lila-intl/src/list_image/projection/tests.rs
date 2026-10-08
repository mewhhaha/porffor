use super::*;
use crate::image::DataImageEnvelope;
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{NumberLocaleRequest, PartitionLimits};
use crate::{
    format_duration_parts, CheckedDurationConfiguration, CheckedListConfiguration, CustomProfileId,
    DataImageComponent, DurationDataImage, DurationOptions, DurationRecord, DurationStyle,
    FormatListPartsRequest, ListDataImage, ListFormatOperationError, ListLocaleRequest, ListPart,
    ListStyle, ListSupportedLocalesRequest, ListType, NumberProfilesDataImage, ResolvedListLocale,
};

fn setup() -> (IntlDataProfile, LocaleDataImage, CustomListProfile) {
    let id = CustomProfileId::parse("actual-list-projection").unwrap();
    let profile = IntlDataProfile::Custom(id.clone());
    let locale = LocaleDataImage::for_profile(profile.clone()).unwrap();
    let selection = CustomListProfile::new(id, &["es", "he"]).unwrap();
    (profile, locale, selection)
}

fn canonical(name: &str) -> CanonicalLocaleId {
    CanonicalLocaleId::from_data(name).unwrap()
}

fn request(
    locale: ResolvedListLocale,
    kind: ListType,
    style: ListStyle,
    values: &[Vec<u16>],
) -> FormatListPartsRequest {
    FormatListPartsRequest::new(
        CheckedListConfiguration::new(locale, kind, style),
        values
            .iter()
            .map(|value| value.clone().into_boxed_slice())
            .collect(),
    )
    .unwrap()
}

fn rendered(profiles: &ListProfiles, request: FormatListPartsRequest) -> Vec<u16> {
    let parts = profiles.format_parts(request.clone()).unwrap();
    let mut result = Vec::new();
    for part in parts.parts() {
        match part {
            ListPart::Literal(text) => result.extend_from_slice(text),
            ListPart::Element(index) => {
                result.extend_from_slice(&request.elements()[*index as usize])
            }
        }
    }
    result
}

#[test]
fn projected_payload_formats_all_nine_choices_without_exposing_duration_dependencies() {
    let (profile, locale, selection) = setup();
    let full = ListDataImage::for_profile(profile.clone(), &locale).unwrap();
    let projected = ListDataImage::for_custom_projection(&selection, &locale).unwrap();
    let envelope = DataImageEnvelope::decode(
        projected.bytes(),
        DataImageComponent::ListFormat,
        super::super::LIST_IMAGE_MARKERS,
    )
    .unwrap();
    let (descriptor, blob) = split_payload(envelope.blob()).unwrap();
    assert!(
        blob.len() < PINNED_BLOB.len(),
        "actual raw data must be projected"
    );
    assert_eq!(descriptor.public_locales, ["en-US", "es", "he"]);
    assert_eq!(descriptor.duration_associations.len(), 15);
    for marker in [Marker::And, Marker::Or, Marker::Unit] {
        for width in ["N", "S", "W"] {
            assert!(descriptor
                .rows
                .iter()
                .any(|row| row.marker == marker && row.attributes == width));
        }
    }
    // en-US uses a genuine en physical fallback row; this does not publish en.
    assert!(descriptor.rows.iter().any(|row| row.locale == "en"));
    assert!(!descriptor.public_locales.iter().any(|name| name == "en"));
    let selected = projected.profiles();
    let baseline = full.profiles();
    assert_eq!(
        selected
            .available_locales()
            .map(|locale| locale.as_str())
            .collect::<Vec<_>>(),
        ["en-US", "es", "he"]
    );
    let supported = selected
        .supported(ListSupportedLocalesRequest {
            requested: ["ar", "es", "he", "sr"].map(canonical).into(),
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(
        supported
            .locales
            .iter()
            .map(|locale| locale.as_str())
            .collect::<Vec<_>>(),
        ["es", "he"]
    );
    assert_eq!(
        selected
            .resolve(ListLocaleRequest {
                requested: vec![canonical("sr")].into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
            })
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    for values in [
        ["España", "isla"]
            .map(|value| value.encode_utf16().collect::<Vec<_>>())
            .to_vec(),
        ["Madrid", "Oviedo"]
            .map(|value| value.encode_utf16().collect::<Vec<_>>())
            .to_vec(),
        ["א", "B"]
            .map(|value| value.encode_utf16().collect::<Vec<_>>())
            .to_vec(),
        vec![vec![0xd800], vec![0x0042], vec![0xdfff]],
    ] {
        for name in selected.available_locales() {
            for &kind in ListType::ALL {
                for &style in ListStyle::ALL {
                    assert_eq!(
                        rendered(
                            &selected,
                            request(selected.admit(name.clone()).unwrap(), kind, style, &values)
                        ),
                        rendered(
                            &baseline,
                            request(baseline.admit(name.clone()).unwrap(), kind, style, &values)
                        )
                    );
                }
            }
        }
    }
    let spanish = ["España", "isla"].map(|value| value.encode_utf16().collect::<Vec<_>>());
    assert_eq!(
        String::from_utf16(&rendered(
            &selected,
            request(
                selected.admit(canonical("es")).unwrap(),
                ListType::Conjunction,
                ListStyle::Long,
                &spanish
            )
        ))
        .unwrap(),
        "España e isla"
    );
    let hebrew = ["א", "B"].map(|value| value.encode_utf16().collect::<Vec<_>>());
    assert_eq!(
        String::from_utf16(&rendered(
            &selected,
            request(
                selected.admit(canonical("he")).unwrap(),
                ListType::Conjunction,
                ListStyle::Long,
                &hebrew
            )
        ))
        .unwrap(),
        "א ו‑B"
    );
    // Both wire reminting paths must reject a hidden dependency even though its
    // actual profile remains available to the selected native Duration owner.
    for name in ["ar", "sr"] {
        assert!(selected.resolve_duration_locale(&canonical(name)).is_ok());
        assert!(selected.admit(canonical(name)).is_err());
        let resolved = baseline.admit(canonical(name)).unwrap();
        assert!(ResolvedListLocale::decode(&resolved.encode().unwrap(), &selected).is_err());
        let encoded = request(resolved, ListType::Unit, ListStyle::Long, &spanish)
            .encode()
            .unwrap();
        assert!(FormatListPartsRequest::decode(&encoded, &selected).is_err());
    }
    let admitted = ListDataImage::from_bytes(projected.bytes(), &locale).unwrap();
    let foreign = request(
        selected.admit(canonical("es")).unwrap(),
        ListType::Conjunction,
        ListStyle::Long,
        &spanish,
    );
    assert!(selected.format_parts(foreign.clone()).is_ok());
    assert_eq!(
        admitted.profiles().format_parts(foreign),
        Err(ListFormatOperationError::InvalidResolvedLocale)
    );
}

#[test]
fn full_duration_rows_keep_the_selected_projected_list_owner_and_original_outputs() {
    let (profile, locale, selection) = setup();
    let numbers = NumberProfilesDataImage::for_profile(profile.clone(), &locale).unwrap();
    let lists = ListDataImage::for_custom_projection(&selection, &locale).unwrap();
    let full_lists = ListDataImage::for_profile(profile.clone(), &locale).unwrap();
    let image = DurationDataImage::for_profile(profile.clone(), &locale, &numbers, &lists).unwrap();
    let full = DurationDataImage::for_profile(profile, &locale, &numbers, &full_lists).unwrap();
    assert_eq!(
        image.bytes(),
        full.bytes(),
        "Duration native payload remains exact and complete"
    );
    assert!(image.uses_foundations(&numbers.profiles(), &lists.profiles()));
    assert!(!image.uses_foundations(&numbers.profiles(), &full_lists.profiles()));
    let selected = image.profiles();
    let baseline = full.profiles();
    let number_owner = numbers.profiles();
    let record =
        DurationRecord::from_number_fields([1.0, 2.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0])
            .unwrap();
    let mut retained = Vec::new();
    assert_eq!(selected.available_locales().len(), 15);
    for name in selected.available_locales() {
        for &style in DurationStyle::ALL {
            let request = NumberLocaleRequest {
                requested: vec![name.clone()].into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
                numbering_system: None,
            };
            let options = DurationOptions {
                style,
                ..Default::default()
            };
            let configuration = CheckedDurationConfiguration::new(
                selected.resolve_locale(&request, &number_owner).unwrap(),
                options.clone(),
            )
            .unwrap();
            let original = CheckedDurationConfiguration::new(
                baseline.resolve_locale(&request, &number_owner).unwrap(),
                options,
            )
            .unwrap();
            let expected =
                format_duration_parts(&original, &record, &baseline, &PartitionLimits::HOST_ABI)
                    .unwrap();
            assert_eq!(
                format_duration_parts(
                    &configuration,
                    &record,
                    &selected,
                    &PartitionLimits::HOST_ABI
                )
                .unwrap(),
                expected
            );
            retained.push((configuration, expected));
        }
    }
    drop(image);
    drop(full);
    drop(numbers);
    drop(number_owner);
    drop(lists);
    drop(full_lists);
    drop(baseline);
    drop(locale);
    for (configuration, expected) in retained {
        assert_eq!(
            format_duration_parts(
                &configuration,
                &record,
                &selected,
                &PartitionLimits::HOST_ABI
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn projection_admission_recomputes_real_width_rows_and_refuses_forged_pins() {
    let (profile, locale, selection) = setup();
    let payload = produce(&selection, &locale).unwrap();
    let (mut descriptor, blob) = split_payload(&payload).unwrap();
    descriptor.full_list_blob_sha256[0] ^= 1;
    let forged = frame_payload(&descriptor, blob).unwrap();
    let wrap = |payload: &[u8], profile: &IntlDataProfile| {
        DataImageEnvelope::encode(
            DataImageComponent::ListFormat,
            profile,
            super::super::LIST_IMAGE_MARKERS,
            payload,
        )
        .unwrap()
    };
    assert!(ListDataImage::from_bytes(wrap(&forged, &profile), &locale).is_err());
    let (mut descriptor, _) = split_payload(&payload).unwrap();
    let removed = descriptor
        .rows
        .iter()
        .position(|row| row.marker == Marker::And && row.attributes == "S")
        .unwrap();
    descriptor.rows.remove(removed);
    let rows = descriptor
        .rows
        .iter()
        .map(|row| {
            (
                row.marker,
                DataIdentifierCow::from_owned(
                    DataMarkerAttributes::try_from_string(row.attributes.clone()).unwrap(),
                    row.locale.parse().unwrap(),
                ),
            )
        })
        .collect::<BTreeSet<_>>();
    let full = BlobDataProvider::try_new_from_static_blob(PINNED_BLOB).unwrap();
    let missing_width = export::export_rows(&full, &rows).unwrap();
    let missing = frame_payload(&descriptor, &missing_width).unwrap();
    // This is a real exported subset with a valid frame/digest and one consumed
    // width row omitted, rather than a random corrupt artifact.
    assert!(matches!(
        ListDataImage::from_bytes(wrap(&missing, &profile), &locale),
        Err(IntlDataImageError::Consumer(_))
    ));
    let minimal = crate::embedded_locale_data_image().unwrap();
    assert!(
        ListDataImage::from_bytes(wrap(&payload, &IntlDataProfile::Minimal), &minimal).is_err()
    );
    let unknown = CustomListProfile::new(selection.id().clone(), &["zz"]).unwrap();
    assert!(ListDataImage::for_custom_projection(&unknown, &locale).is_err());
    let aliases = CustomListProfile::new(selection.id().clone(), &["he", "iw"]).unwrap();
    assert!(ListDataImage::for_custom_projection(&aliases, &locale).is_err());
}

#[test]
fn canonical_projection_is_reproducible_and_keeps_full_profiles_byte_exact() {
    let (profile, locale, selection) = setup();
    let first = ListDataImage::for_custom_projection(&selection, &locale).unwrap();
    let aliases = CustomListProfile::new(selection.id().clone(), &["iw", "ES"]).unwrap();
    let second = ListDataImage::for_custom_projection(&aliases, &locale).unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.digest(), second.digest());
    let different = CustomListProfile::new(selection.id().clone(), &["es"]).unwrap();
    let third = ListDataImage::for_custom_projection(&different, &locale).unwrap();
    assert_ne!(first.digest(), third.digest());
    let full = ListDataImage::for_profile(profile, &locale).unwrap();
    let full_envelope = DataImageEnvelope::decode(
        full.bytes(),
        DataImageComponent::ListFormat,
        super::super::LIST_IMAGE_MARKERS,
    )
    .unwrap();
    assert_eq!(full_envelope.blob(), PINNED_BLOB);
    let minimal = crate::embedded_list_data_image().unwrap();
    let minimal_envelope = DataImageEnvelope::decode(
        minimal.bytes(),
        DataImageComponent::ListFormat,
        super::super::LIST_IMAGE_MARKERS,
    )
    .unwrap();
    assert_eq!(minimal_envelope.blob(), PINNED_BLOB);
}
