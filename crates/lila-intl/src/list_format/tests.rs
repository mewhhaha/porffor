use super::*;
use crate::{
    EmbeddedIntlProvider, FormatListParts, IntlKernel, IntlProvider, IntlService,
    ResolveListLocale, SupportedListLocales,
};
use icu_list::options::{ListFormatterOptions, ListLength};
use icu_list::{ListFormatter, ListFormatterPreferences};
use writeable::Writeable;

fn locale(name: &str) -> ResolvedListLocale {
    embedded_list_profiles()
        .unwrap()
        .resolve(ListLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data(name).unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap()
}
fn request(
    name: &str,
    kind: ListType,
    style: ListStyle,
    inputs: &[&[u16]],
) -> FormatListPartsRequest {
    FormatListPartsRequest::new(
        CheckedListConfiguration::new(locale(name), kind, style),
        inputs
            .iter()
            .map(|v| v.to_vec().into_boxed_slice())
            .collect(),
    )
    .unwrap()
}
fn render(parts: &ListParts, elements: &[Box<[u16]>]) -> Vec<u16> {
    let mut units = Vec::new();
    for part in parts.parts() {
        match part {
            ListPart::Literal(text) => units.extend_from_slice(text),
            ListPart::Element(i) => units.extend_from_slice(&elements[*i as usize]),
        }
    }
    units
}
fn result(name: &str, kind: ListType, style: ListStyle, inputs: &[&str]) -> (ListParts, String) {
    let elements: Vec<Vec<u16>> = inputs.iter().map(|s| s.encode_utf16().collect()).collect();
    let slices: Vec<&[u16]> = elements.iter().map(Vec::as_slice).collect();
    let request = request(name, kind, style, &slices);
    let parts = format_list_parts(request.clone()).unwrap();
    let text = String::from_utf16(&render(&parts, request.elements())).unwrap();
    (parts, text)
}
#[test]
fn actual_available_locale_catalogue_loads_and_formats_all_nine_profiles() {
    let profiles = embedded_list_profiles().unwrap();
    let locales: Vec<_> = profiles.available_locales().collect();
    assert!(!locales.is_empty());
    assert!(locales
        .windows(2)
        .all(|pair| pair[0].as_str() < pair[1].as_str()));
    for admitted in locales {
        let resolved = profiles.admit(admitted.clone()).unwrap();
        let parsed: icu_locale::Locale = admitted.as_str().parse().unwrap();
        let prefs: ListFormatterPreferences = parsed.into();
        for &kind in ListType::ALL {
            for &style in ListStyle::ALL {
                let options = ListFormatterOptions::default().with_length(match style {
                    ListStyle::Long => ListLength::Wide,
                    ListStyle::Short => ListLength::Short,
                    ListStyle::Narrow => ListLength::Narrow,
                });
                let independent = match kind {
                    ListType::Conjunction => ListFormatter::try_new_and(prefs, options).unwrap(),
                    ListType::Disjunction => ListFormatter::try_new_or(prefs, options).unwrap(),
                    ListType::Unit => ListFormatter::try_new_unit(prefs, options).unwrap(),
                };
                let request = FormatListPartsRequest::new(
                    CheckedListConfiguration::new(resolved.clone(), kind, style),
                    ["A", "א", "नमस्ते", "D"]
                        .iter()
                        .map(|s| s.encode_utf16().collect::<Vec<_>>().into_boxed_slice())
                        .collect(),
                )
                .unwrap();
                let actual = format_list_parts(request.clone()).unwrap();
                assert_eq!(
                    String::from_utf16(&render(&actual, request.elements())).unwrap(),
                    independent
                        .format(["A", "א", "नमस्ते", "D"].into_iter())
                        .write_to_string()
                );
                assert_eq!(
                    actual
                        .parts()
                        .iter()
                        .filter_map(|p| match p {
                            ListPart::Element(i) => Some(*i),
                            ListPart::Literal(_) => None,
                        })
                        .collect::<Vec<_>>(),
                    [0, 1, 2, 3]
                );
                assert!(!profiles
                    .effective_locale(&resolved, kind, style)
                    .as_str()
                    .is_empty());
            }
        }
    }
}
#[test]
fn english_nine_profiles_use_distinct_pair_and_final_templates() {
    let rows = [
        (
            ListType::Conjunction,
            ListStyle::Long,
            "A and B",
            "A, B, and C",
        ),
        (
            ListType::Conjunction,
            ListStyle::Short,
            "A & B",
            "A, B, & C",
        ),
        (ListType::Conjunction, ListStyle::Narrow, "A, B", "A, B, C"),
        (
            ListType::Disjunction,
            ListStyle::Long,
            "A or B",
            "A, B, or C",
        ),
        (
            ListType::Disjunction,
            ListStyle::Short,
            "A or B",
            "A, B, or C",
        ),
        (
            ListType::Disjunction,
            ListStyle::Narrow,
            "A or B",
            "A, B, or C",
        ),
        (ListType::Unit, ListStyle::Long, "A, B", "A, B, C"),
        (ListType::Unit, ListStyle::Short, "A, B", "A, B, C"),
        (ListType::Unit, ListStyle::Narrow, "A B", "A B C"),
    ];
    for (kind, style, pair, triple) in rows {
        assert_eq!(result("en-US", kind, style, &["A", "B"]).1, pair);
        let (parts, text) = result("en-US", kind, style, &["A", "B", "C"]);
        assert_eq!(text, triple);
        assert_eq!(parts.parts().len(), 5);
    }
}
#[test]
fn spanish_contexts_use_pinned_anchored_conditions_for_pair_and_end() {
    for word in ["iglesia", "HI", "hi", "hilo"] {
        assert_eq!(
            result(
                "es",
                ListType::Conjunction,
                ListStyle::Long,
                &["agua", word]
            )
            .1,
            format!("agua e {word}")
        );
    }
    for word in ["hielo", "hiato", "HIA", "otro", ""] {
        assert_eq!(
            result(
                "es",
                ListType::Conjunction,
                ListStyle::Long,
                &["agua", word]
            )
            .1,
            format!("agua y {word}")
        );
    }
    for word in [
        "oso",
        "HOJA",
        "8",
        "80",
        "11",
        "11.000",
        "11 000",
        "11\u{202f}000",
        "11,2",
    ] {
        assert_eq!(
            result(
                "es",
                ListType::Disjunction,
                ListStyle::Long,
                &["siete", word]
            )
            .1,
            format!("siete u {word}")
        );
    }
    for word in ["110", "1100", "211000", "algo", ""] {
        assert_eq!(
            result(
                "es",
                ListType::Disjunction,
                ListStyle::Long,
                &["siete", word]
            )
            .1,
            format!("siete o {word}")
        );
    }
    assert_eq!(
        result(
            "es",
            ListType::Conjunction,
            ListStyle::Long,
            &["agua", "pan", "iglesia"]
        )
        .1,
        "agua, pan e iglesia"
    );
    assert_eq!(
        result(
            "es",
            ListType::Disjunction,
            ListStyle::Long,
            &["uno", "dos", "8"]
        )
        .1,
        "uno, dos u 8"
    );
}
#[test]
fn hebrew_condition_projection_never_replaces_original_utf16_elements() {
    for (right, joiner) in [
        (vec![0x05d1], " ו"),
        (vec![], " ו"),
        (vec![0x41], " ו‑"),
        (vec![0xd800], " ו‑"),
        (vec![0xd83d, 0xde00], " ו‑"),
    ] {
        let left = [0x05d0];
        let req = request(
            "he",
            ListType::Conjunction,
            ListStyle::Long,
            &[&left, &right],
        );
        let actual = format_list_parts(req.clone()).unwrap();
        assert_eq!(
            actual.parts(),
            [
                ListPart::Element(0),
                ListPart::Literal(joiner.encode_utf16().collect()),
                ListPart::Element(1)
            ]
        );
        let mut expected = left.to_vec();
        expected.extend(joiner.encode_utf16());
        expected.extend(&right);
        assert_eq!(render(&actual, req.elements()), expected);
    }
}
#[test]
fn empty_and_duplicate_elements_remain_distinct_indexed_parts() {
    for n in 0..=4 {
        let inputs = vec![&[][..]; n];
        let req = request("en-US", ListType::Conjunction, ListStyle::Long, &inputs);
        let parts = format_list_parts(req).unwrap();
        assert_eq!(
            parts
                .parts()
                .iter()
                .filter_map(|p| if let ListPart::Element(i) = p {
                    Some(*i)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            (0..n as u32).collect::<Vec<_>>()
        );
        assert!(parts.parts().iter().all(|p| match p {
            ListPart::Literal(s) => !s.is_empty(),
            ListPart::Element(_) => true,
        }));
    }
    let inputs = [0xd800, 0, 0xdc00, 0xd83d, 0xde00];
    let req = request(
        "en-US",
        ListType::Unit,
        ListStyle::Narrow,
        &[&inputs, &inputs],
    );
    let parts = format_list_parts(req.clone()).unwrap();
    let mut expected = inputs.to_vec();
    expected.push(0x20);
    expected.extend(inputs);
    assert_eq!(render(&parts, req.elements()), expected);
}
#[test]
fn admitted_aliases_extensions_and_supported_order_use_the_actual_catalogue() {
    let profiles = embedded_list_profiles().unwrap();
    assert_eq!(locale("en-US-u-ca-chinese").resolved().as_str(), "en-US");
    assert_eq!(locale("zz").resolved().as_str(), "en-US");
    assert!(profiles
        .admit(CanonicalLocaleId::from_data("zz").unwrap())
        .is_err());
    let result = profiles
        .supported(ListSupportedLocalesRequest {
            requested: ["es-MX-u-nu-latn", "zz", "he"]
                .iter()
                .map(|s| CanonicalLocaleId::from_data(*s).unwrap())
                .collect(),
            matcher: LocaleMatcher::BestFit,
        })
        .unwrap();
    assert_eq!(
        result
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
        ["es-MX-u-nu-latn", "he"]
    );
    let en = locale("en-US");
    assert_eq!(
        profiles
            .effective_locale(&en, ListType::Conjunction, ListStyle::Long)
            .as_str(),
        "en"
    );
}
#[test]
fn list_wire_roundtrips_raw_units_and_all_domains_with_complete_consumption() {
    let profiles = embedded_list_profiles().unwrap();
    for &kind in ListType::ALL {
        for &style in ListStyle::ALL {
            let units = [0xd800, 0, 0xdc00, 0xd83d, 0xde00];
            let req = request("en-US", kind, style, &[&units, &[]]);
            let bytes = req.encode().unwrap();
            let decoded = FormatListPartsRequest::decode(&bytes, profiles).unwrap();
            assert_eq!(decoded.elements(), req.elements());
            assert_eq!(
                decoded.configuration().wire_words(),
                req.configuration().wire_words()
            );
            let parts = format_list_parts(decoded).unwrap();
            assert_eq!(
                ListParts::decode(&parts.encode().unwrap(), 2).unwrap(),
                parts
            );
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(FormatListPartsRequest::decode(&trailing, profiles).is_err());
            let mut truncated = bytes;
            truncated.pop();
            assert!(FormatListPartsRequest::decode(&truncated, profiles).is_err());
        }
    }
}
#[test]
fn wire_rejects_wrong_headers_extents_domains_and_independent_indices() {
    let profiles = embedded_list_profiles().unwrap();
    let req = request(
        "en-US",
        ListType::Conjunction,
        ListStyle::Long,
        &[&[0xd800]],
    );
    let bytes = req.encode().unwrap();
    let config = 16 + 8 + 5;
    for (offset, value) in [
        (0usize, 2u64),
        (8, 43),
        (config, 0),
        (config + 8, 4),
        (config + 16, u64::MAX),
        (config + 24, u64::MAX),
    ] {
        let mut bad = bytes.clone();
        bad[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        assert!(FormatListPartsRequest::decode(&bad, profiles).is_err());
    }
    let parts = format_list_parts(req).unwrap();
    let valid = parts.encode().unwrap();
    assert!(ListParts::decode(&valid, 0).is_err());
    let mut wrong = valid.clone();
    wrong[32..40].copy_from_slice(&1u64.to_le_bytes());
    assert!(ListParts::decode(&wrong, 1).is_err());
    assert!(ListParts::checked(vec![ListPart::Element(0), ListPart::Element(0)], 2).is_err());
    assert!(ListParts::checked(vec![ListPart::Literal(Box::new([]))], 0).is_err());
}
#[test]
fn kernel_advertises_and_executes_only_attached_list_operations() {
    let provider = EmbeddedIntlProvider::new().unwrap();
    let identity = provider.identity().clone();
    let kernel = IntlKernel::new(identity, provider).unwrap();
    let resolved = kernel
        .operation::<ResolveListLocale>()
        .unwrap()
        .execute(ListLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap();
    assert!(kernel
        .identity()
        .profile()
        .services()
        .contains(IntlService::ListFormat));
    let request = FormatListPartsRequest::new(
        CheckedListConfiguration::new(resolved, ListType::Conjunction, ListStyle::Long),
        ["A", "B"]
            .iter()
            .map(|s| s.encode_utf16().collect::<Vec<_>>().into_boxed_slice())
            .collect(),
    )
    .unwrap();
    let parts = kernel
        .operation::<FormatListParts>()
        .unwrap()
        .execute(request)
        .unwrap();
    assert_eq!(parts.parts().len(), 3);
    let supported = kernel
        .operation::<SupportedListLocales>()
        .unwrap()
        .execute(ListSupportedLocalesRequest {
            requested: vec![CanonicalLocaleId::from_data("es").unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
        })
        .unwrap();
    assert_eq!(supported.locales.len(), 1);
}
