use super::*;
use crate::display_names::{
    CheckedDisplayNamesConfiguration, DisplayNameRequest, DisplayNameResult,
    DisplayNamesDateTimeField, DisplayNamesError, DisplayNamesFallback,
    DisplayNamesLanguageDisplay, DisplayNamesLocaleRequest, DisplayNamesSelection,
    DisplayNamesStyle, DisplayNamesType,
};
use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::options::LocaleMatcher;
use crate::{
    decode_display_name_request, embedded_display_names_data_image, embedded_locale_data_image,
    encode_display_name_request, DisplayNamesDataImage,
};
use std::sync::Arc;

fn setup() -> (CustomProfileId, LocaleDataImage) {
    let id = CustomProfileId::parse("actual-display-names-projection").unwrap();
    let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(id.clone())).unwrap();
    (id, locale)
}

fn requested(names: &[&str]) -> Vec<LocaleId> {
    names
        .iter()
        .map(|name| LocaleId::parse(*name).unwrap())
        .collect()
}

fn envelope(image: &DisplayNamesDataImage) -> DataImageEnvelope {
    DataImageEnvelope::decode(
        image.bytes(),
        DataImageComponent::DisplayNames,
        super::super::DISPLAY_NAMES_IMAGE_MARKERS,
    )
    .unwrap()
}

fn request(
    image: &DisplayNamesDataImage,
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language: DisplayNamesLanguageDisplay,
    code: &[u16],
) -> DisplayNameRequest {
    DisplayNameRequest::new(
        CheckedDisplayNamesConfiguration::new(
            image
                .profiles()
                .admit(CanonicalLocaleId::from_data(locale).unwrap())
                .unwrap(),
            DisplayNamesSelection::from_options(kind, language),
            style,
            fallback,
        ),
        code.into(),
    )
    .unwrap()
}

fn result(
    image: &DisplayNamesDataImage,
    locale: &str,
    kind: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language: DisplayNamesLanguageDisplay,
    code: &str,
) -> Result<DisplayNameResult, DisplayNamesError> {
    image.display_name(&request(
        image,
        locale,
        kind,
        style,
        fallback,
        language,
        &code.encode_utf16().collect::<Vec<_>>(),
    ))
}

#[test]
fn selected_rows_keep_complete_reachable_pools_and_the_real_fallback_catalogue() {
    let (id, locale) = setup();
    let full =
        DisplayNamesDataImage::for_profile(IntlDataProfile::Custom(id.clone()), &locale).unwrap();
    let image =
        DisplayNamesDataImage::for_custom_projection(&id, &requested(&["ar", "fr"]), &locale)
            .unwrap();
    let frame = envelope(&image);
    let (descriptor, blob) = split_payload(frame.blob()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
    let original: serde_json::Value = serde_json::from_slice(DISPLAY_NAMES_PROFILE).unwrap();
    let rows = raw["locales"].as_array().unwrap();
    let pools = raw["name_pool"].as_array().unwrap();
    assert_eq!(descriptor.public_locales, ["ar", "en-US", "fr"]);
    assert_eq!(pools.len(), 37);
    assert_eq!(pools.len(), descriptor.source_pool_indices.len());
    assert!(descriptor
        .source_pool_indices
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    for (pool, &source) in pools.iter().zip(&descriptor.source_pool_indices) {
        // Every retained pool contains the full original typed code/name table.
        assert_eq!(pool, &original["name_pool"][source as usize]);
    }
    let mut used = BTreeSet::new();
    for row in rows {
        let full_row = original["locales"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["locale"] == row["locale"])
            .unwrap();
        assert_eq!(row["locale_pattern"], full_row["locale_pattern"]);
        assert_eq!(row["locale_separator"], full_row["locale_separator"]);
        for &style in STYLES {
            for &domain in DOMAINS {
                let index = row["styles"][style][domain].as_u64().unwrap() as usize;
                let source = full_row["styles"][style][domain].as_u64().unwrap() as usize;
                assert_eq!(pools[index], original["name_pool"][source]);
                used.insert(index);
            }
        }
    }
    assert_eq!(
        used.into_iter().collect::<Vec<_>>(),
        (0..pools.len()).collect::<Vec<_>>()
    );
    assert!(blob.len() < DISPLAY_NAMES_PROFILE.len());
    assert!(image.bytes().len() < full.bytes().len());
    let profiles = image.profiles();
    assert_eq!(
        profiles
            .available_locales()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["ar", "en-US", "fr"]
    );
    let lookup = DisplayNamesLocaleRequest {
        requested: requested(&["hi", "en", "fr-FR", "ar-EG", "ja", "en-US-u-ca-buddhist"])
            .iter()
            .map(|locale| CanonicalLocaleId::from_data(locale.as_str()).unwrap())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    };
    assert_eq!(
        profiles
            .supported_locales(&lookup)
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["fr-FR", "ar-EG", "en-US-u-ca-buddhist"]
    );
    for (name, expected) in [("hi", "en-US"), ("fr-FR", "fr"), ("ar-EG", "ar")] {
        assert_eq!(
            profiles
                .resolve(&DisplayNamesLocaleRequest {
                    requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
                    matcher: LocaleMatcher::Lookup,
                })
                .unwrap()
                .resolved()
                .as_str(),
            expected
        );
    }
    assert!(profiles
        .admit(CanonicalLocaleId::from_data("hi").unwrap())
        .is_err());
    for name in ["ar", "en-US", "fr"] {
        for &style in DisplayNamesStyle::ALL {
            for fallback in [DisplayNamesFallback::Code, DisplayNamesFallback::None] {
                for language in [
                    DisplayNamesLanguageDisplay::Dialect,
                    DisplayNamesLanguageDisplay::Standard,
                ] {
                    for (kind, codes) in [
                        (
                            DisplayNamesType::Language,
                            &["fr", "es-Cyrl-MX", "sl-rozaj", "IW", "ZZZZZZZZ-aBcD-aB"][..],
                        ),
                        (DisplayNamesType::Region, &["us", "DE", "ZZ"][..]),
                        (DisplayNamesType::Script, &["Hans", "Cyrl", "Zzzz"][..]),
                        (DisplayNamesType::Currency, &["USD", "EUR", "zzz"][..]),
                        (
                            DisplayNamesType::Calendar,
                            &["gregory", "ethiopic-amete-alem", "123-ABC"][..],
                        ),
                    ] {
                        for code in codes {
                            assert_eq!(
                                result(&image, name, kind, style, fallback, language, code),
                                result(&full, name, kind, style, fallback, language, code)
                            );
                        }
                    }
                    for &field in DisplayNamesDateTimeField::ALL {
                        assert_eq!(
                            result(
                                &image,
                                name,
                                DisplayNamesType::DateTimeField,
                                style,
                                fallback,
                                language,
                                field.name()
                            ),
                            result(
                                &full,
                                name,
                                DisplayNamesType::DateTimeField,
                                style,
                                fallback,
                                language,
                                field.name()
                            )
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn canonical_selection_keeps_full_legacy_bytes_and_refuses_absent_or_mixed_foundations() {
    let (id, locale) = setup();
    let first =
        DisplayNamesDataImage::for_custom_projection(&id, &requested(&["fr", "ar"]), &locale)
            .unwrap();
    let reordered = DisplayNamesDataImage::for_custom_projection(
        &id,
        &requested(&["AR", "en-us", "FR"]),
        &locale,
    )
    .unwrap();
    assert_eq!(first.bytes().as_ref(), reordered.bytes().as_ref());
    for names in [
        &[][..],
        &["fr", "FR"][..],
        &["fr-FR"][..],
        &["iw"][..],
        &["xx"][..],
    ] {
        assert!(
            DisplayNamesDataImage::for_custom_projection(&id, &requested(names), &locale).is_err()
        );
    }
    let minimal_locale = embedded_locale_data_image().unwrap();
    let minimal = embedded_display_names_data_image().unwrap();
    assert_eq!(envelope(&minimal).blob(), DISPLAY_NAMES_PROFILE);
    let full =
        DisplayNamesDataImage::for_profile(IntlDataProfile::Custom(id.clone()), &locale).unwrap();
    assert_eq!(envelope(&full).blob(), DISPLAY_NAMES_PROFILE);
    assert_eq!(minimal.profiles().available_locales().len(), 13);
    assert_eq!(full.profiles().available_locales().len(), 13);
    let all = full
        .profiles()
        .available_locales()
        .map(|locale| LocaleId::parse(locale.as_str()).unwrap())
        .collect::<Vec<_>>();
    let maximum = DisplayNamesDataImage::for_custom_projection(&id, &all, &locale).unwrap();
    let maximum_frame = envelope(&maximum);
    let (_, maximum_blob) = split_payload(maximum_frame.blob()).unwrap();
    let maximum_raw: serde_json::Value = serde_json::from_slice(maximum_blob).unwrap();
    assert_eq!(maximum.profiles().available_locales().len(), 13);
    assert_eq!(maximum_raw["name_pool"].as_array().unwrap().len(), 105);
    let other = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("different-display-foundation").unwrap(),
    ))
    .unwrap();
    for foundation in [&minimal_locale, &other] {
        assert!(
            DisplayNamesDataImage::for_custom_projection(&id, &requested(&["fr"]), foundation)
                .is_err()
        );
        assert!(DisplayNamesDataImage::from_bytes(first.bytes(), foundation).is_err());
    }
    let selected_frame = envelope(&first);
    let relabeled = DataImageEnvelope::encode(
        DataImageComponent::DisplayNames,
        &IntlDataProfile::Minimal,
        super::super::DISPLAY_NAMES_IMAGE_MARKERS,
        selected_frame.blob(),
    )
    .unwrap();
    assert!(DisplayNamesDataImage::from_bytes(relabeled, &minimal_locale).is_err());
}

fn reject_payload(payload: &[u8], id: &CustomProfileId, locale: &LocaleDataImage) {
    // The envelope itself refuses an empty payload; that is also a rejection.
    let Ok(bytes) = DataImageEnvelope::encode(
        DataImageComponent::DisplayNames,
        &IntlDataProfile::Custom(id.clone()),
        super::super::DISPLAY_NAMES_IMAGE_MARKERS,
        payload,
    ) else {
        assert!(payload.is_empty());
        return;
    };
    assert!(DisplayNamesDataImage::from_bytes(bytes, locale).is_err());
}

#[test]
fn self_consistent_changed_rows_pools_and_provenance_cannot_claim_the_projection() {
    let (id, locale) = setup();
    let image =
        DisplayNamesDataImage::for_custom_projection(&id, &requested(&["fr"]), &locale).unwrap();
    let frame = envelope(&image);
    let damage: &[fn(&mut Descriptor, &mut serde_json::Value)] = &[
        |d, _| d.schema += 1,
        |d, _| d.custom_id = "different-id".into(),
        |d, _| d.default_locale = "fr".into(),
        |d, _| d.full_profile_sha256[0] ^= 1,
        |d, _| d.locale_image_sha256[0] ^= 1,
        |d, _| d.public_locales.reverse(),
        |d, _| {
            d.public_locales.remove(0);
        },
        |d, _| d.public_locales.push("hi".into()),
        |d, _| d.source_pool_indices[0] = 104,
        |_, raw| raw["source_manifest_sha256"] = "changed-source".into(),
        |_, raw| raw["bcp47_manifest_sha256"] = "changed-keywords".into(),
        |_, raw| raw["locales"][0]["locale_pattern"] = "{1}-{0}".into(),
        |_, raw| raw["name_pool"][0]["entries"][0][1] = "replacement name".into(),
        |_, raw| {
            raw["locales"].as_array_mut().unwrap().pop();
        },
        |_, raw| {
            let value = raw["locales"][0]["styles"]["long"]["region"].clone();
            raw["locales"][0]["styles"]["long"]["script"] = value;
        },
        |_, raw| {
            let extra = raw["name_pool"][0].clone();
            raw["name_pool"].as_array_mut().unwrap().push(extra);
        },
        |d, raw| {
            let count = raw["name_pool"].as_array().unwrap().len();
            raw["name_pool"].as_array_mut().unwrap().reverse();
            d.source_pool_indices.reverse();
            for row in raw["locales"].as_array_mut().unwrap() {
                for &style in STYLES {
                    for &domain in DOMAINS {
                        let old = row["styles"][style][domain].as_u64().unwrap() as usize;
                        row["styles"][style][domain] =
                            serde_json::Value::from((count - 1 - old) as u32);
                    }
                }
            }
        },
    ];
    for change in damage {
        let (mut descriptor, blob) = split_payload(frame.blob()).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
        change(&mut descriptor, &mut raw);
        let changed_blob = serde_json::to_vec(&canonical_objects(raw)).unwrap();
        let payload = frame_payload(&descriptor, &changed_blob).unwrap();
        // Each outer frame has a valid newly computed digest. Admission must
        // rederive the real closure rather than trust this internal manifest.
        reject_payload(&payload, &id, &locale);
    }
    for end in [0, 8, HEADER_BYTES - 1, frame.blob().len() - 1] {
        reject_payload(&frame.blob()[..end], &id, &locale);
    }
    let mut extended = frame.blob().to_vec();
    extended.push(0);
    reject_payload(&extended, &id, &locale);
    for (range, extent) in [
        (8..12, u32::MAX.to_le_bytes().to_vec()),
        (12..20, u64::MAX.to_le_bytes().to_vec()),
    ] {
        let mut changed = frame.blob().to_vec();
        changed[range].copy_from_slice(&extent);
        reject_payload(&changed, &id, &locale);
    }
}

#[test]
fn selected_wire_remints_templates_and_retains_the_actual_image_after_inputs_drop() {
    let (id, locale) = setup();
    let original =
        DisplayNamesDataImage::for_custom_projection(&id, &requested(&["fr"]), &locale).unwrap();
    let bytes: Arc<[u8]> = original.bytes().as_ref().to_vec().into();
    let selected = DisplayNamesDataImage::from_bytes(bytes.clone(), &locale).unwrap();
    let cloned = selected.clone();
    let profiles = selected.profiles();
    let weak = Arc::downgrade(&profiles);
    assert!(Arc::ptr_eq(&profiles, &cloned.profiles()));
    let foreign = request(
        &original,
        "fr",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &[0xd800],
    );
    assert_eq!(
        original.display_name(&foreign),
        Err(DisplayNamesError::InvalidCode)
    );
    assert_eq!(
        selected.display_name(&foreign),
        Err(DisplayNamesError::InvalidResolvedLocale)
    );
    let foreign = request(
        &original,
        "fr",
        DisplayNamesType::Language,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &"IW".encode_utf16().collect::<Vec<_>>(),
    );
    assert_eq!(
        selected.display_name(&foreign),
        Err(DisplayNamesError::InvalidResolvedLocale)
    );
    let wire = encode_display_name_request(&foreign).unwrap();
    let reminted = decode_display_name_request(&wire, &profiles).unwrap();
    let expected = original.display_name(&foreign).unwrap();
    let full = DisplayNamesDataImage::for_profile(IntlDataProfile::Custom(id), &locale).unwrap();
    let excluded = request(
        &full,
        "hi",
        DisplayNamesType::Region,
        DisplayNamesStyle::Narrow,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &"US".encode_utf16().collect::<Vec<_>>(),
    );
    assert!(decode_display_name_request(
        &encode_display_name_request(&excluded).unwrap(),
        &profiles
    )
    .is_err());
    drop(full);
    drop(original);
    drop(bytes);
    drop(locale);
    drop(selected);
    drop(profiles);
    assert!(weak.upgrade().is_some());
    assert_eq!(cloned.display_name(&reminted).unwrap(), expected);
    let reserved = request(
        &cloned,
        "en-US",
        DisplayNamesType::Language,
        DisplayNamesStyle::Short,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &"ZZZZZZZZ-aBcD-aB".encode_utf16().collect::<Vec<_>>(),
    );
    assert_eq!(
        cloned.display_name(&reserved).unwrap().name(),
        Some("zzzzzzzz-Abcd-AB")
    );
    drop(cloned);
    assert!(weak.upgrade().is_none());
}

fn currencies(codes: &[&str]) -> Vec<CurrencyCode> {
    codes
        .iter()
        .map(|code| CurrencyCode::parse(code).unwrap())
        .collect()
}

#[test]
fn currency_projection_filters_real_pools_and_retains_other_domains_and_fallbacks() {
    let (id, locale) = setup();
    let full =
        DisplayNamesDataImage::for_profile(IntlDataProfile::Custom(id.clone()), &locale).unwrap();
    let image = DisplayNamesDataImage::for_custom_data_projection(
        &id,
        Some(&requested(&["fr", "ar"])),
        &currencies(&["USD", "EUR"]),
        &locale,
    )
    .unwrap();
    let same = DisplayNamesDataImage::for_custom_data_projection(
        &id,
        Some(&requested(&["ar", "fr"])),
        &currencies(&["eur", "usd"]),
        &locale,
    )
    .unwrap();
    assert_eq!(image.bytes().as_ref(), same.bytes().as_ref());
    assert_eq!(image.currency_codes().unwrap(), currencies(&["EUR", "USD"]));
    let frame = envelope(&image);
    let (descriptor, blob) = split_payload(frame.blob()).unwrap();
    assert_eq!(descriptor.schema, 2);
    assert_eq!(descriptor.currency_codes.unwrap(), ["EUR", "USD"]);
    assert_eq!(descriptor.public_locales, ["ar", "en-US", "fr"]);
    let raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
    let original: serde_json::Value = serde_json::from_slice(DISPLAY_NAMES_PROFILE).unwrap();
    let pools = raw["name_pool"].as_array().unwrap();
    let encoded = pools
        .iter()
        .map(|pool| serde_json::to_vec(pool).unwrap())
        .collect::<Vec<_>>();
    assert!(encoded.windows(2).all(|pair| pair[0] < pair[1]));
    let mut used = BTreeSet::new();
    for row in raw["locales"].as_array().unwrap() {
        let source = original["locales"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["locale"] == row["locale"])
            .unwrap();
        for &style in STYLES {
            for &domain in DOMAINS {
                let index = row["styles"][style][domain].as_u64().unwrap() as usize;
                let source_index = source["styles"][style][domain].as_u64().unwrap() as usize;
                used.insert(index);
                let mut expected = original["name_pool"][source_index].clone();
                if domain == "currency" {
                    expected["entries"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|entry| matches!(entry[0].as_str(), Some("EUR" | "USD")));
                }
                assert_eq!(pools[index], expected);
            }
        }
    }
    assert_eq!(
        used.into_iter().collect::<Vec<_>>(),
        (0..pools.len()).collect::<Vec<_>>()
    );
    assert!(image.bytes().len() < full.bytes().len());
    for name in ["ar", "en-US", "fr"] {
        for &style in DisplayNamesStyle::ALL {
            for fallback in [DisplayNamesFallback::Code, DisplayNamesFallback::None] {
                for (kind, codes) in [
                    (DisplayNamesType::Currency, &["EUR", "USD"][..]),
                    (DisplayNamesType::Language, &["IW", "fr", "es-Cyrl-MX"][..]),
                    (DisplayNamesType::Region, &["US", "DE"][..]),
                    (DisplayNamesType::Calendar, &["gregory", "buddhist"][..]),
                ] {
                    for code in codes {
                        assert_eq!(
                            result(
                                &image,
                                name,
                                kind,
                                style,
                                fallback,
                                DisplayNamesLanguageDisplay::Dialect,
                                code
                            ),
                            result(
                                &full,
                                name,
                                kind,
                                style,
                                fallback,
                                DisplayNamesLanguageDisplay::Dialect,
                                code
                            )
                        );
                    }
                }
            }
            assert_eq!(
                result(
                    &image,
                    name,
                    DisplayNamesType::Currency,
                    style,
                    DisplayNamesFallback::Code,
                    DisplayNamesLanguageDisplay::Dialect,
                    "JPY"
                )
                .unwrap()
                .name(),
                Some("JPY")
            );
            assert_eq!(
                result(
                    &image,
                    name,
                    DisplayNamesType::Currency,
                    style,
                    DisplayNamesFallback::None,
                    DisplayNamesLanguageDisplay::Dialect,
                    "JPY"
                )
                .unwrap()
                .name(),
                None
            );
        }
    }
}

#[test]
fn full_locale_currency_projection_admits_only_rederived_empty_currency_pools() {
    let (id, locale) = setup();
    let original: serde_json::Value = serde_json::from_slice(DISPLAY_NAMES_PROFILE).unwrap();
    let rows = original["locales"].as_array().unwrap();
    let pools = original["name_pool"].as_array().unwrap();
    let default = rows.iter().find(|row| row["locale"] == "en-US").unwrap();
    let source = pool_reference(default, "long", "currency", pools.len()).unwrap();
    // Select a genuine en-US code absent from at least one actual locale/width
    // pool. The source absence independently requires an empty selected pool.
    let (code, absent_locale, absent_style) = pools[source]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|entry| {
            let code = entry[0].as_str().unwrap();
            rows.iter().find_map(|row| {
                STYLES.iter().find_map(|&style| {
                    let index = pool_reference(row, style, "currency", pools.len()).unwrap();
                    (!pools[index]["entries"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|entry| entry[0].as_str() == Some(code)))
                    .then_some((code, row["locale"].as_str().unwrap(), style))
                })
            })
        })
        .expect("pinned localized currency pools have an absent en-US code");
    let image =
        DisplayNamesDataImage::for_custom_data_projection(&id, None, &currencies(&[code]), &locale)
            .unwrap();
    assert_eq!(image.profiles().available_locales().len(), rows.len());
    let frame = envelope(&image);
    let (_, blob) = split_payload(frame.blob()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(blob).unwrap();
    let row = raw["locales"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["locale"].as_str() == Some(absent_locale))
        .unwrap();
    let index = pool_reference(
        row,
        absent_style,
        "currency",
        raw["name_pool"].as_array().unwrap().len(),
    )
    .unwrap();
    assert!(raw["name_pool"][index]["entries"]
        .as_array()
        .unwrap()
        .is_empty());
    let decoded = DisplayNamesDataImage::from_bytes(image.bytes(), &locale).unwrap();
    let style = match absent_style {
        "long" => DisplayNamesStyle::Long,
        "short" => DisplayNamesStyle::Short,
        "narrow" => DisplayNamesStyle::Narrow,
        _ => unreachable!(),
    };
    assert_eq!(
        result(
            &decoded,
            absent_locale,
            DisplayNamesType::Currency,
            style,
            DisplayNamesFallback::Code,
            DisplayNamesLanguageDisplay::Dialect,
            code
        )
        .unwrap()
        .name(),
        Some(code)
    );
    assert_eq!(
        result(
            &decoded,
            absent_locale,
            DisplayNamesType::Currency,
            style,
            DisplayNamesFallback::None,
            DisplayNamesLanguageDisplay::Dialect,
            code
        )
        .unwrap()
        .name(),
        None
    );
    let retained = request(
        &decoded,
        "en-US",
        DisplayNamesType::Currency,
        DisplayNamesStyle::Long,
        DisplayNamesFallback::Code,
        DisplayNamesLanguageDisplay::Dialect,
        &code.encode_utf16().collect::<Vec<_>>(),
    );
    let wire = encode_display_name_request(&retained).unwrap();
    let reminted = decode_display_name_request(&wire, &image.profiles()).unwrap();
    assert_eq!(
        image.display_name(&retained),
        Err(DisplayNamesError::InvalidResolvedLocale)
    );
    let expected = decoded.display_name(&retained).unwrap();
    drop(decoded);
    drop(locale);
    assert_eq!(image.display_name(&reminted).unwrap(), expected);
}

#[test]
fn currency_descriptor_and_replacement_pool_cannot_self_authorize_changed_data() {
    let (id, locale) = setup();
    for codes in [vec![], currencies(&["USD", "usd"]), currencies(&["ZZZ"])] {
        assert!(
            DisplayNamesDataImage::for_custom_data_projection(&id, None, &codes, &locale).is_err()
        );
    }
    let image = DisplayNamesDataImage::for_custom_data_projection(
        &id,
        Some(&requested(&["fr"])),
        &currencies(&["EUR", "USD"]),
        &locale,
    )
    .unwrap();
    let frame = envelope(&image);
    let changes: &[fn(&mut Descriptor, &mut serde_json::Value)] = &[
        |d, _| d.schema = 1,
        |d, _| d.currency_codes = None,
        |d, _| d.currency_codes.as_mut().unwrap().reverse(),
        |d, _| d.currency_codes.as_mut().unwrap()[0] = "eur".into(),
        |d, _| d.currency_codes.as_mut().unwrap().push("USD".into()),
        |d, _| d.currency_codes.as_mut().unwrap().push("JPY".into()),
        |d, _| d.locale_image_sha256[0] ^= 1,
        |_, raw| {
            let pool = raw["name_pool"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|pool| {
                    pool["kind"] == "currency" && !pool["entries"].as_array().unwrap().is_empty()
                })
                .unwrap();
            pool["entries"][0][1] = "forged localized name".into();
        },
        |_, raw| {
            let pool = raw["name_pool"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|pool| pool["kind"] == "currency")
                .unwrap();
            pool["entries"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!(["JPY", "forged extra currency"]));
        },
    ];
    for change in changes {
        let (mut descriptor, blob) = split_payload(frame.blob()).unwrap();
        let mut raw = serde_json::from_slice(blob).unwrap();
        change(&mut descriptor, &mut raw);
        reject_payload(
            &frame_payload(
                &descriptor,
                &serde_json::to_vec(&canonical_objects(raw)).unwrap(),
            )
            .unwrap(),
            &id,
            &locale,
        );
    }
    let other = LocaleDataImage::for_profile(IntlDataProfile::Custom(
        CustomProfileId::parse("different-currency-foundation").unwrap(),
    ))
    .unwrap();
    assert!(DisplayNamesDataImage::from_bytes(image.bytes(), &other).is_err());
}
