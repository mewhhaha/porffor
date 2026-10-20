use super::super::{NumberProfilesDataImage, NUMBER_IMAGE_MARKERS};
use super::*;
use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{
    resolve_number_locale, NumberLocaleRequest, PartitionLimits, ResolvedNumberLocale,
};
use crate::plural_rules::{resolve_plural_locale, PluralLocaleRequest, ResolvedPluralLocale};
use crate::{CanonicalLocaleId, DurationDataImage, IntlDataProfile, RelativeTimeDataImage};
use std::sync::Arc;

fn setup() -> (CustomProfileId, LocaleDataImage, ListDataImage) {
    let id = CustomProfileId::parse("coupled-number-projection").unwrap();
    let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(id.clone())).unwrap();
    let lists = ListDataImage::for_profile(locale.profile().clone(), &locale).unwrap();
    (id, locale, lists)
}

#[test]
fn projected_best_fit_uses_the_selected_locale_owner_and_cannot_publish_private_number_rows() {
    let (id, locale, lists) = setup();
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["zh-Hant-TW"]),
        Some(&requested(&["fr"])),
        Some(&requested(&["fr"])),
        &locale,
        &lists,
    )
    .unwrap();
    let reminted = NumberProfilesDataImage::from_bytes(selected.bytes(), &locale, &lists).unwrap();
    for image in [&selected, &reminted] {
        assert!(image.uses_locale(&locale));
        let owner = image.profiles();
        for (requested, expected) in [("zh-TW", "zh-Hant-TW"), ("fr-CA", "en-US")] {
            let result = resolve_number_locale(
                &NumberLocaleRequest {
                    requested: vec![CanonicalLocaleId::from_data(requested).unwrap()]
                        .into_boxed_slice(),
                    matcher: LocaleMatcher::BestFit,
                    numbering_system: None,
                },
                &owner,
            )
            .unwrap();
            assert_eq!(result.formatting().as_str(), expected);
        }
    }
}

#[test]
fn numbering_projection_requires_the_exact_sparse_binary_defaults_and_canonical_descriptor() {
    let (id, locale, lists) = setup();
    let systems = [
        NumberingSystemOption::parse("deva").unwrap(),
        NumberingSystemOption::parse("arab").unwrap(),
    ];
    let image = NumberProfilesDataImage::for_custom_numbering_projection(
        &id,
        Some(&requested(&["fr", "ar-EG"])),
        None,
        &systems,
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let same = NumberProfilesDataImage::for_custom_numbering_projection(
        &id,
        Some(&requested(&["ar-EG", "fr"])),
        None,
        &[systems[1].clone(), systems[0].clone()],
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    assert_eq!(image.bytes(), same.bytes());
    let admitted = envelope(&image);
    let (descriptor, binary) = split_payload(admitted.blob()).unwrap();
    assert_eq!(&binary[..8], b"LNF47\0\x03\0");
    assert!(crate::number_format::NumberProfiles::from_bytes(binary, &locale).is_err());
    let changes: &[fn(&mut Descriptor)] = &[
        |value| value.schema = 2,
        |value| value.numbering_systems = None,
        |value| value.numbering_systems.as_mut().unwrap().reverse(),
        |value| {
            value
                .numbering_systems
                .as_mut()
                .unwrap()
                .push("deva".into())
        },
        |value| value.numbering_systems.as_mut().unwrap()[0] = "roman".into(),
        |value| value.default_numbering[0].1 ^= 1,
        |value| value.list_image_sha256[0] ^= 1,
    ];
    for change in changes {
        let (mut damaged, _) = split_payload(admitted.blob()).unwrap();
        change(&mut damaged);
        let bytes = frame_payload(&damaged, binary).unwrap();
        assert!(NumberProfilesDataImage::from_bytes(
            frame(&bytes, locale.profile()),
            &locale,
            &lists
        )
        .is_err());
    }
    let mut damaged = binary.to_vec();
    *damaged.last_mut().unwrap() ^= 1;
    let bytes = frame_payload(&descriptor, &damaged).unwrap();
    assert!(
        NumberProfilesDataImage::from_bytes(frame(&bytes, locale.profile()), &locale, &lists)
            .is_err()
    );
    for values in [
        vec![],
        vec![systems[0].clone(), systems[0].clone()],
        vec![NumberingSystemOption::parse("roman").unwrap()],
    ] {
        assert!(NumberProfilesDataImage::for_custom_numbering_projection(
            &id, None, None, &values, None, None, &locale, &lists
        )
        .is_err());
    }
}

#[test]
fn currency_descriptor_requires_the_exact_canonical_binary_and_selected_foundations() {
    let (id, locale, lists) = setup();
    let codes = [
        CurrencyCode::parse("EUR").unwrap(),
        CurrencyCode::parse("JPY").unwrap(),
    ];
    let image = NumberProfilesDataImage::for_custom_data_projection(
        &id,
        Some(&requested(&["fr"])),
        &codes,
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let admitted = envelope(&image);
    let (mut descriptor, binary) = split_payload(admitted.blob()).unwrap();
    assert_eq!(descriptor.schema, 2);
    assert_eq!(
        descriptor.currency_codes.as_deref(),
        Some(["EUR".to_owned(), "JPY".to_owned()].as_slice())
    );
    assert_eq!(
        NumberProfilesDataImage::from_bytes(image.bytes(), &locale, &lists)
            .unwrap()
            .currency_codes(),
        Some(codes.as_slice())
    );
    descriptor.currency_codes = Some(vec!["USD".to_owned()]);
    let changed = frame_payload(&descriptor, binary).unwrap();
    assert!(NumberProfilesDataImage::from_bytes(
        frame(&changed, locale.profile()),
        &locale,
        &lists
    )
    .is_err());
    descriptor.currency_codes = Some(vec!["eur".to_owned(), "JPY".to_owned()]);
    let changed = frame_payload(&descriptor, binary).unwrap();
    assert!(NumberProfilesDataImage::from_bytes(
        frame(&changed, locale.profile()),
        &locale,
        &lists
    )
    .is_err());
    let full = NumberProfilesDataImage::for_custom_data_projection(
        &id, None, &codes, None, None, &locale, &lists,
    )
    .unwrap();
    assert_eq!(
        full.profiles().available_locales(),
        crate::number_image::pinned_number_source()
            .unwrap()
            .profiles()
            .available_locales()
    );
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
fn envelope(image: &NumberProfilesDataImage) -> DataImageEnvelope {
    DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::NumberProfiles,
        NUMBER_IMAGE_MARKERS,
    )
    .unwrap()
}
fn frame(payload: &[u8], profile: &IntlDataProfile) -> Arc<[u8]> {
    DataImageEnvelope::encode(
        DataImageComponent::NumberProfiles,
        profile,
        NUMBER_IMAGE_MARKERS,
        payload,
    )
    .unwrap()
}

#[test]
fn public_number_and_plural_catalogue_excludes_actual_hidden_dependency_rows() {
    let (id, locale, lists) = setup();
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["pl", "es"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let profiles = selected.profiles();
    assert_eq!(
        profiles
            .available_locales()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["en-US", "es", "pl"]
    );
    let framed = envelope(&selected);
    let (descriptor, binary) = split_payload(framed.blob()).unwrap();
    let physical = crate::number_format::NumberProfiles::from_bytes(binary, &locale).unwrap();
    assert!(physical
        .available_locales()
        .iter()
        .any(|name| name.as_ref() == "ar"));
    assert!(physical
        .available_locales()
        .iter()
        .any(|name| name.as_ref() == "sr"));
    assert!(!physical
        .available_locales()
        .iter()
        .any(|name| name.as_ref() == "bn-BD"));
    assert!(binary.len() < PINNED_BINARY.len());
    assert_eq!(descriptor.required_relative_time_locales.len(), 14);
    assert_eq!(descriptor.required_duration_locales.len(), 15);
    assert_eq!(profiles.numbering_systems().len(), 78);
    assert!(profiles
        .numbering_systems()
        .iter()
        .any(|name| name.as_ref() == "tols"));
    assert_eq!(
        resolve_number_locale(&request("ar"), &profiles)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    assert_eq!(
        resolve_plural_locale(
            &PluralLocaleRequest {
                requested: request("sr").requested,
                matcher: LocaleMatcher::Lookup
            },
            &profiles,
            &PartitionLimits::HOST_ABI
        )
        .unwrap()
        .resolved()
        .as_str(),
        "en-US"
    );
    assert!(ResolvedNumberLocale::from_resolved(
        CanonicalLocaleId::from_data("ar").unwrap(),
        CanonicalLocaleId::from_data("ar").unwrap(),
        "latn",
        &profiles
    )
    .is_err());
    assert!(ResolvedPluralLocale::from_resolved(
        CanonicalLocaleId::from_data("sr").unwrap(),
        CanonicalLocaleId::from_data("sr").unwrap(),
        &profiles
    )
    .is_err());
}

#[test]
fn dependent_formats_use_the_same_number_arc_without_public_support_leakage() {
    let (id, locale, lists) = setup();
    let full = NumberProfilesDataImage::for_profile(locale.profile().clone(), &locale).unwrap();
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["es", "pl"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let numbers = selected.profiles();
    let full_numbers = full.profiles();
    let relative =
        RelativeTimeDataImage::for_profile(locale.profile().clone(), &locale, &selected).unwrap();
    let baseline_relative =
        RelativeTimeDataImage::for_profile(locale.profile().clone(), &locale, &full).unwrap();
    assert!(relative.uses_number_profiles(&numbers));
    for name in ["ar", "ar-EG", "pl"] {
        for numeric in [crate::RelativeNumeric::Always, crate::RelativeNumeric::Auto] {
            let actual = crate::RelativeTimeConfiguration::new(
                relative
                    .profiles()
                    .resolve_locale(&request(name), &numbers, &PartitionLimits::HOST_ABI)
                    .unwrap(),
                crate::RelativeStyle::Long,
                numeric,
                &numbers,
            )
            .unwrap();
            let expected = crate::RelativeTimeConfiguration::new(
                baseline_relative
                    .profiles()
                    .resolve_locale(&request(name), &full_numbers, &PartitionLimits::HOST_ABI)
                    .unwrap(),
                crate::RelativeStyle::Long,
                numeric,
                &full_numbers,
            )
            .unwrap();
            let value = crate::FiniteRelativeNumber::new(-1.0).unwrap();
            let parts = relative
                .profiles()
                .format_parts(
                    &actual,
                    value,
                    crate::RelativeUnit::Day,
                    &PartitionLimits::HOST_ABI,
                )
                .unwrap();
            assert_eq!(
                parts,
                baseline_relative
                    .profiles()
                    .format_parts(
                        &expected,
                        value,
                        crate::RelativeUnit::Day,
                        &PartitionLimits::HOST_ABI
                    )
                    .unwrap()
            );
            let wire = crate::RelativeRequest::Parts {
                configuration: actual,
                value,
                unit: crate::RelativeUnit::Day,
            };
            let decoded = crate::decode_relative_request(
                crate::RelativeHostOp::FormatRelativeTimeParts,
                &crate::encode_relative_request(&wire).unwrap(),
                &relative.profiles(),
                &numbers,
                &PartitionLimits::HOST_ABI,
            )
            .unwrap();
            assert_eq!(decoded, wire);
        }
    }
    let duration =
        DurationDataImage::for_profile(locale.profile().clone(), &locale, &selected, &lists)
            .unwrap();
    let baseline_duration =
        DurationDataImage::for_profile(locale.profile().clone(), &locale, &full, &lists).unwrap();
    assert!(duration.uses_foundations(&numbers, &lists.profiles()));
    let record = crate::DurationRecord::from_number_fields([
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 456.0, 0.0, 0.0,
    ])
    .unwrap();
    for &style in crate::DurationStyle::ALL {
        let actual = crate::CheckedDurationConfiguration::new(
            duration
                .profiles()
                .resolve_locale(&request("sr"), &numbers)
                .unwrap(),
            crate::DurationOptions {
                style,
                ..Default::default()
            },
        )
        .unwrap();
        let expected = crate::CheckedDurationConfiguration::new(
            baseline_duration
                .profiles()
                .resolve_locale(&request("sr"), &full_numbers)
                .unwrap(),
            crate::DurationOptions {
                style,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            crate::format_duration_parts(
                &actual,
                &record,
                &duration.profiles(),
                &PartitionLimits::HOST_ABI
            )
            .unwrap(),
            crate::format_duration_parts(
                &expected,
                &record,
                &baseline_duration.profiles(),
                &PartitionLimits::HOST_ABI
            )
            .unwrap()
        );
    }
    // Locale.getNumberingSystems remains backed by all original default-nu rows,
    // although their NumberFormat tables have actually been removed.
    for name in full_numbers.available_locales() {
        let request = crate::LocaleNumberingSystemsRequest::new(
            CanonicalLocaleId::from_data(name.as_ref()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            crate::number_format::locale_default_numbering_system(request.clone(), &numbers)
                .unwrap(),
            crate::number_format::locale_default_numbering_system(request, &full_numbers).unwrap()
        );
    }
}

#[test]
fn public_wire_reminting_cannot_import_a_hidden_number_or_plural_locale() {
    let (id, locale, lists) = setup();
    let full = NumberProfilesDataImage::for_profile(locale.profile().clone(), &locale)
        .unwrap()
        .profiles();
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["pl", "es"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap()
    .profiles();
    let hidden_number = resolve_number_locale(&request("ar"), &full).unwrap();
    assert!(ResolvedNumberLocale::decode(&hidden_number.encode().unwrap(), &selected).is_err());
    let hidden_plural = resolve_plural_locale(
        &PluralLocaleRequest {
            requested: request("sr").requested,
            matcher: LocaleMatcher::Lookup,
        },
        &full,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    assert!(ResolvedPluralLocale::decode(&hidden_plural.encode().unwrap(), &selected).is_err());
    let public = resolve_number_locale(&request("es"), &full).unwrap();
    let decoded = ResolvedNumberLocale::decode(&public.encode().unwrap(), &selected).unwrap();
    assert!(decoded.ensure_profiles(&selected).is_ok());
    assert!(decoded.ensure_profiles(&full).is_err());
    let public_plural = resolve_plural_locale(
        &PluralLocaleRequest {
            requested: request("pl").requested,
            matcher: LocaleMatcher::Lookup,
        },
        &full,
        &PartitionLimits::HOST_ABI,
    )
    .unwrap();
    let decoded_plural =
        ResolvedPluralLocale::decode(&public_plural.encode().unwrap(), &selected).unwrap();
    assert_ne!(decoded_plural, public_plural);
    assert_eq!(decoded_plural.data().as_str(), "pl");
}

#[test]
fn selected_dependency_domains_and_list_foundation_are_exact_admission_obligations() {
    let (id, locale, lists) = setup();
    let selected_names = requested(&["fr"]);
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["pl"]),
        Some(&selected_names),
        Some(&selected_names),
        &locale,
        &lists,
    )
    .unwrap();
    let framed = envelope(&selected);
    let (_, binary) = split_payload(framed.blob()).unwrap();
    assert_eq!(
        crate::number_format::NumberProfiles::from_bytes(binary, &locale)
            .unwrap()
            .available_locales()
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>(),
        ["en-US", "fr", "pl"]
    );
    assert!(
        RelativeTimeDataImage::for_custom_projection(&id, &selected_names, &locale, &selected)
            .is_ok()
    );
    assert!(DurationDataImage::for_custom_projection(
        &id,
        &selected_names,
        &locale,
        &selected,
        &lists
    )
    .is_ok());
    assert!(
        RelativeTimeDataImage::for_profile(locale.profile().clone(), &locale, &selected).is_err()
    );
    assert!(DurationDataImage::for_custom_projection(
        &id,
        &requested(&["pl"]),
        &locale,
        &selected,
        &lists
    )
    .is_err());
    let other_lists = ListDataImage::for_custom_projection(
        &crate::CustomListProfile::new(id.clone(), &["ja"]).unwrap(),
        &locale,
    )
    .unwrap();
    assert!(NumberProfilesDataImage::from_bytes(selected.bytes(), &locale, &other_lists).is_err());
    assert!(DurationDataImage::for_custom_projection(
        &id,
        &selected_names,
        &locale,
        &selected,
        &other_lists
    )
    .is_err());
    assert!(NumberProfilesDataImage::from_bytes(
        frame(framed.blob(), &IntlDataProfile::Minimal),
        &crate::embedded_locale_data_image().unwrap(),
        &crate::embedded_list_data_image().unwrap()
    )
    .is_err());
}

#[test]
fn altered_rows_pins_domains_and_default_nu_cannot_claim_pinned_projection() {
    let (id, locale, lists) = setup();
    let selected = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["es", "pl"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let framed = envelope(&selected);
    let (mut descriptor, binary) = split_payload(framed.blob()).unwrap();
    descriptor.full_number_binary_sha256[0] ^= 1;
    assert!(NumberProfilesDataImage::from_bytes(
        frame(
            &frame_payload(&descriptor, binary).unwrap(),
            locale.profile()
        ),
        &locale,
        &lists
    )
    .is_err());
    let (mut descriptor, binary) = split_payload(framed.blob()).unwrap();
    descriptor.default_numbering[0].1 = descriptor.default_numbering[0].1.wrapping_add(1);
    assert!(NumberProfilesDataImage::from_bytes(
        frame(
            &frame_payload(&descriptor, binary).unwrap(),
            locale.profile()
        ),
        &locale,
        &lists
    )
    .is_err());
    let (mut descriptor, binary) = split_payload(framed.blob()).unwrap();
    descriptor
        .required_duration_locales
        .retain(|name| name != "sr");
    assert!(NumberProfilesDataImage::from_bytes(
        frame(
            &frame_payload(&descriptor, binary).unwrap(),
            locale.profile()
        ),
        &locale,
        &lists
    )
    .is_err());
    let (descriptor, binary) = split_payload(framed.blob()).unwrap();
    let mut damaged = binary.to_vec();
    *damaged.last_mut().unwrap() ^= 1;
    assert!(NumberProfilesDataImage::from_bytes(
        frame(
            &frame_payload(&descriptor, &damaged).unwrap(),
            locale.profile()
        ),
        &locale,
        &lists
    )
    .is_err());
}

#[test]
fn catalogue_selection_is_canonical_and_retained_owners_outlive_all_frames() {
    let (id, locale, lists) = setup();
    let first = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["pl", "es"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    let reordered = NumberProfilesDataImage::for_custom_projection(
        &id,
        &requested(&["es", "pl"]),
        None,
        None,
        &locale,
        &lists,
    )
    .unwrap();
    assert_eq!(first.bytes(), reordered.bytes());
    for invalid in [
        Vec::new(),
        requested(&["he", "iw"]),
        requested(&["pl", "pl"]),
        requested(&["zz"]),
        requested(&["es-u-nu-arab"]),
    ] {
        assert!(NumberProfilesDataImage::for_custom_projection(
            &id, &invalid, None, None, &locale, &lists
        )
        .is_err());
    }
    let bytes: Arc<[u8]> = first.bytes().as_ref().to_vec().into();
    let second = NumberProfilesDataImage::from_bytes(bytes.clone(), &locale, &lists).unwrap();
    let relative =
        RelativeTimeDataImage::for_profile(locale.profile().clone(), &locale, &second).unwrap();
    let numbers = second.profiles();
    let weak = Arc::downgrade(&numbers);
    let configuration = crate::RelativeTimeConfiguration::new(
        relative
            .profiles()
            .resolve_locale(&request("ar"), &numbers, &PartitionLimits::HOST_ABI)
            .unwrap(),
        crate::RelativeStyle::Long,
        crate::RelativeNumeric::Always,
        &numbers,
    )
    .unwrap();
    let profiles = relative.profiles();
    drop(numbers);
    drop(second);
    drop(first);
    drop(reordered);
    drop(relative);
    drop(bytes);
    drop(locale);
    drop(lists);
    assert!(weak.upgrade().is_some());
    assert!(!profiles
        .format_parts(
            &configuration,
            crate::FiniteRelativeNumber::new(-2.0).unwrap(),
            crate::RelativeUnit::Day,
            &PartitionLimits::HOST_ABI
        )
        .unwrap()
        .to_text()
        .unwrap()
        .is_empty());
    drop(configuration);
    drop(profiles);
    assert!(weak.upgrade().is_none());
}
