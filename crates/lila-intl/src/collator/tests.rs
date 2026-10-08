use super::*;
use crate::number_format::options::LocaleMatcher;
use crate::{
    CompareCollator, EmbeddedIntlProvider, IntlKernel, IntlProvider, IntlService,
    ResolveCollatorLocale, SupportedCollatorLocales,
};
fn request(name: &str, usage: CollatorUsage) -> CollatorLocaleRequest {
    CollatorLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        usage,
        collation: None,
        numeric: None,
        case_first: None,
    }
}
fn resolved(name: &str, usage: CollatorUsage) -> ResolvedCollatorLocale {
    embedded_collator_profiles()
        .unwrap()
        .resolve(request(name, usage))
        .unwrap()
}
fn compare(
    locale: ResolvedCollatorLocale,
    sensitivity: CollatorSensitivity,
    ignore: bool,
    left: &[u16],
    right: &[u16],
) -> CollatorOrdering {
    compare_collator(
        CompareCollatorRequest::new(
            CheckedCollatorConfiguration::new(locale, sensitivity, ignore),
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
fn actual_catalogue_certifies_all_candidate_sort_and_search_profiles() {
    let profiles = embedded_collator_profiles().unwrap();
    let candidates = crate::number_format::embedded_number_profiles().unwrap();
    let canonicalizer = crate::embedded_locale_data_image().unwrap().canonicalizer();
    let mut names = std::collections::BTreeSet::new();
    for name in candidates.available_locales() {
        let mut locale: icu_locale::Locale = name.parse().unwrap();
        canonicalizer.canonicalize(&mut locale);
        names.insert(locale.to_string());
    }
    let actual = profiles
        .available_locales()
        .map(|locale| locale.as_str().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    names.insert("en-US".to_owned());
    assert_eq!(names, actual);
    for locale in profiles.available_locales() {
        for usage in CollatorUsage::ALL {
            let selected = profiles.resolve(request(locale.as_str(), *usage)).unwrap();
            assert_eq!(selected.resolved(), locale);
            assert_eq!(selected.usage(), *usage);
            assert_eq!(
                compare(
                    selected,
                    CollatorSensitivity::Variant,
                    false,
                    &units("é"),
                    &units("e\u{0301}")
                ),
                CollatorOrdering::Equal
            );
        }
    }
    assert!(profiles
        .available_collations()
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    assert!(profiles
        .available_collations()
        .iter()
        .all(|co| !["standard", "search", "searchjl"].contains(&co.as_ref())));
    for co in profiles.available_collations() {
        assert!(profiles.has_admitted_sort_collation(co));
    }
    assert!(profiles
        .available_collations()
        .iter()
        .any(|co| co.as_ref() == "phonetic"));
}
#[test]
fn genuine_german_search_and_phonebook_have_distinct_sort_associations() {
    assert_eq!(
        compare(
            resolved("de", CollatorUsage::Sort),
            CollatorSensitivity::Variant,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Greater
    );
    assert_eq!(
        compare(
            resolved("de", CollatorUsage::Search),
            CollatorSensitivity::Variant,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Less
    );
    assert_eq!(
        compare(
            resolved("de", CollatorUsage::Search),
            CollatorSensitivity::Base,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Equal
    );
    let mut req = request("de", CollatorUsage::Sort);
    req.collation = Some(CollatorCollationOption::parse("PHONEBK").unwrap());
    let selected = embedded_collator_profiles().unwrap().resolve(req).unwrap();
    assert_eq!(selected.collation(), Some("phonebk"));
    assert_eq!(
        compare(
            selected,
            CollatorSensitivity::Base,
            false,
            &units("AE"),
            &units("Ä")
        ),
        CollatorOrdering::Equal
    );
    let mut req = request("de-u-co-phonebk", CollatorUsage::Search);
    req.collation = Some(CollatorCollationOption::parse("phonebk").unwrap());
    let search = embedded_collator_profiles().unwrap().resolve(req).unwrap();
    assert_eq!(search.collation(), None);
    assert_eq!(search.resolved().as_str(), "de");
}
#[test]
fn sensitivity_numeric_case_first_and_thai_defaults_are_real_kernel_inputs() {
    for (sensitivity, case_equal, accent_equal) in [
        (CollatorSensitivity::Base, true, true),
        (CollatorSensitivity::Accent, true, false),
        (CollatorSensitivity::Case, false, true),
        (CollatorSensitivity::Variant, false, false),
    ] {
        assert_eq!(
            compare(
                resolved("en", CollatorUsage::Sort),
                sensitivity,
                false,
                &units("a"),
                &units("A")
            ) == CollatorOrdering::Equal,
            case_equal
        );
        assert_eq!(
            compare(
                resolved("en", CollatorUsage::Sort),
                sensitivity,
                false,
                &units("a"),
                &units("ä")
            ) == CollatorOrdering::Equal,
            accent_equal
        );
    }
    for (numeric, wanted) in [
        (false, CollatorOrdering::Greater),
        (true, CollatorOrdering::Less),
    ] {
        let mut req = request("en", CollatorUsage::Sort);
        req.numeric = Some(numeric);
        let locale = embedded_collator_profiles().unwrap().resolve(req).unwrap();
        assert_eq!(
            compare(
                locale,
                CollatorSensitivity::Variant,
                false,
                &units("2"),
                &units("10")
            ),
            wanted
        );
    }
    for (case, wanted) in [
        (CollatorCaseFirst::Lower, CollatorOrdering::Less),
        (CollatorCaseFirst::Upper, CollatorOrdering::Greater),
    ] {
        let mut req = request("en", CollatorUsage::Sort);
        req.case_first = Some(case);
        let locale = embedded_collator_profiles().unwrap().resolve(req).unwrap();
        assert_eq!(
            compare(
                locale,
                CollatorSensitivity::Variant,
                false,
                &units("a"),
                &units("A")
            ),
            wanted
        );
    }
    let thai = resolved("th", CollatorUsage::Sort);
    assert!(thai.default_ignore_punctuation());
    assert_eq!(
        compare(
            thai.clone(),
            CollatorSensitivity::Variant,
            true,
            &units("ab"),
            &units("a-b")
        ),
        CollatorOrdering::Equal
    );
    assert_ne!(
        compare(
            thai,
            CollatorSensitivity::Variant,
            false,
            &units("ab"),
            &units("a-b")
        ),
        CollatorOrdering::Equal
    );
    assert!(!resolved("th", CollatorUsage::Search).default_ignore_punctuation());
}
#[test]
fn utf16_raw_units_preserve_pairs_and_selected_replacement_collisions() {
    for name in ["en", "de", "ko"] {
        for usage in CollatorUsage::ALL {
            for sensitivity in CollatorSensitivity::ALL {
                let locale = resolved(name, *usage);
                for (left, right) in [
                    (vec![0xd800], vec![0xfffd]),
                    (vec![0xdc00], vec![0xfffd]),
                    (vec![0xd800, 0xd800], vec![0xfffd, 0xfffd]),
                    (vec![0x61, 0xd800], vec![0x61, 0xfffd]),
                    (vec![0xd83d, 0xde00], units("😀")),
                    (units("각"), units("\u{1100}\u{1161}\u{11a8}")),
                ] {
                    assert_eq!(
                        compare(locale.clone(), *sensitivity, false, &left, &right),
                        CollatorOrdering::Equal
                    );
                }
                assert_ne!(
                    compare(locale, *sensitivity, false, &units("😀"), &units("�")),
                    CollatorOrdering::Equal
                );
            }
        }
    }
}
#[test]
fn extension_and_option_resolution_preserve_only_supported_additions() {
    let profiles = embedded_collator_profiles().unwrap();
    let req = request("de-u-co-phonebk-kf-upper-kn", CollatorUsage::Sort);
    let locale = profiles.resolve(req.clone()).unwrap();
    assert_eq!(locale.resolved().as_str(), "de-u-co-phonebk-kf-upper-kn");
    assert_eq!(locale.case_first(), CollatorCaseFirst::Upper);
    assert!(locale.numeric());
    let mut overridden = req.clone();
    overridden.numeric = Some(false);
    overridden.case_first = Some(CollatorCaseFirst::Lower);
    let locale = profiles.resolve(overridden).unwrap();
    assert_eq!(locale.resolved().as_str(), "de-u-co-phonebk");
    assert!(!locale.numeric());
    assert_eq!(locale.case_first(), CollatorCaseFirst::Lower);
    let mut unsupported = req;
    unsupported.collation = Some(CollatorCollationOption::parse("foobar").unwrap());
    assert_eq!(
        profiles.resolve(unsupported).unwrap().collation(),
        Some("phonebk")
    );
    let mut compound = request("de", CollatorUsage::Sort);
    compound.collation = Some(CollatorCollationOption::parse("phonebk-true").unwrap());
    assert_eq!(profiles.resolve(compound).unwrap().collation(), None);
    assert_eq!(
        profiles
            .resolve(request("de-u-co-phonebk-true", CollatorUsage::Sort))
            .unwrap()
            .collation(),
        None
    );
    let supported = profiles
        .supported(CollatorSupportedLocalesRequest {
            requested: vec![
                CanonicalLocaleId::from_data("en-u-kn").unwrap(),
                CanonicalLocaleId::from_data("zz-ZZ").unwrap(),
            ]
            .into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(supported.locales.len(), 1);
    assert_eq!(supported.locales[0].as_str(), "en-u-kn");
}
#[test]
fn all_wire_shapes_round_trip_and_compare_keeps_every_utf16_unit() {
    let profiles = embedded_collator_profiles().unwrap();
    let req = request("de-u-co-phonebk", CollatorUsage::Sort);
    assert_eq!(
        CollatorLocaleRequest::decode(&req.encode().unwrap()).unwrap(),
        req
    );
    let locale = profiles.resolve(req.clone()).unwrap();
    let roundtrip =
        ResolvedCollatorLocale::decode(&locale.encode().unwrap(), profiles, &req).unwrap();
    assert_eq!(roundtrip.collation(), Some("phonebk"));
    let req = CompareCollatorRequest::new(
        CheckedCollatorConfiguration::new(roundtrip, CollatorSensitivity::Case, false),
        vec![0xd800, 0xdc00, 0xdc00].into_boxed_slice(),
        vec![0xfffd].into_boxed_slice(),
    )
    .unwrap();
    let decoded = CompareCollatorRequest::decode(&req.encode().unwrap(), profiles).unwrap();
    assert_eq!(decoded.left(), req.left());
    assert_eq!(decoded.right(), req.right());
    for value in CollatorOrdering::ALL {
        assert_eq!(
            CollatorOrdering::decode(&value.encode().unwrap()).unwrap(),
            *value
        );
    }
}
#[test]
fn malformed_or_unrelated_wire_configuration_never_becomes_a_profile() {
    let profiles = embedded_collator_profiles().unwrap();
    let req = request("en", CollatorUsage::Sort);
    let bytes = req.encode().unwrap();
    for end in 0..bytes.len() {
        assert!(CollatorLocaleRequest::decode(&bytes[..end]).is_err());
    }
    let mut wrong = bytes.clone();
    wrong[0] = 2;
    assert!(CollatorLocaleRequest::decode(&wrong).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(CollatorLocaleRequest::decode(&trailing).is_err());
    let mut response = resolved("de", CollatorUsage::Search).encode().unwrap();
    assert!(ResolvedCollatorLocale::decode(&response, profiles, &req).is_err());
    response = CollatorOrdering::Equal.encode().unwrap();
    response[16..24].copy_from_slice(&3u64.to_le_bytes());
    assert!(CollatorOrdering::decode(&response).is_err());
    assert!(profiles
        .admit(
            CanonicalLocaleId::from_data("de").unwrap(),
            CollatorUsage::Sort,
            Some("searchjl"),
            false,
            CollatorCaseFirst::False
        )
        .is_err());
    assert!(profiles
        .admit(
            CanonicalLocaleId::from_data("de-u-kn").unwrap(),
            CollatorUsage::Sort,
            None,
            false,
            CollatorCaseFirst::False
        )
        .is_err());
}
#[test]
fn provider_identity_capability_and_dispatch_attach_all_three_operations() {
    let provider = EmbeddedIntlProvider::new().unwrap();
    assert!(provider
        .identity()
        .profile()
        .services()
        .contains(IntlService::Collator));
    let kernel = IntlKernel::new(provider.identity().clone(), provider).unwrap();
    let locale = kernel
        .operation::<ResolveCollatorLocale>()
        .unwrap()
        .execute(request("de", CollatorUsage::Search))
        .unwrap();
    let selected = kernel
        .operation::<SupportedCollatorLocales>()
        .unwrap()
        .execute(CollatorSupportedLocalesRequest {
            requested: request("de", CollatorUsage::Sort).requested,
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(selected.locales.len(), 1);
    let value = kernel
        .operation::<CompareCollator>()
        .unwrap()
        .execute(
            CompareCollatorRequest::new(
                CheckedCollatorConfiguration::new(locale, CollatorSensitivity::Base, false),
                units("AE").into_boxed_slice(),
                units("Ä").into_boxed_slice(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(value, CollatorOrdering::Equal);
}
