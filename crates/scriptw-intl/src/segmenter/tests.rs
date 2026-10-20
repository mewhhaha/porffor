use super::*;
use std::sync::OnceLock;

fn profiles() -> &'static SegmenterProfiles {
    static PROFILES: OnceLock<SegmenterProfiles> = OnceLock::new();
    PROFILES.get_or_init(|| SegmenterProfiles::load().unwrap())
}
fn request(locale: &str, granularity: SegmenterGranularity, input: &[u16]) -> SegmentUtf16Request {
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(
            profiles()
                .admit(CanonicalLocaleId::from_data(locale).unwrap())
                .unwrap(),
            granularity,
        ),
        input.into(),
    )
    .unwrap()
}
fn ends(result: &SegmenterResult) -> Vec<u32> {
    std::iter::once(0)
        .chain(result.boundaries().iter().map(|b| b.end()))
        .collect()
}
fn expected(locale: &str, granularity: SegmenterGranularity, input: &str, boundaries: &[u32]) {
    let input: Vec<_> = input.encode_utf16().collect();
    let result = segment_utf16(&request(locale, granularity, &input)).unwrap();
    assert_eq!(result.input(), input);
    assert_eq!(ends(&result), boundaries);
}
fn corpus(granularity: SegmenterGranularity, locale: &str, file: &str, expected_count: usize) {
    let mut count = 0;
    for (line_number, line) in file.lines().enumerate() {
        let body = line.split('#').next().unwrap().trim();
        if body.is_empty() {
            continue;
        }
        let mut input = Vec::new();
        let mut wanted = Vec::new();
        for (position, token) in body.split_whitespace().enumerate() {
            if position % 2 == 0 {
                match token {
                    "÷" => wanted.push(input.len() as u32),
                    "×" => {}
                    _ => panic!("corpus syntax at line {}", line_number + 1),
                }
            } else {
                let scalar = char::from_u32(u32::from_str_radix(token, 16).unwrap()).unwrap();
                let mut pair = [0; 2];
                input.extend_from_slice(scalar.encode_utf16(&mut pair));
            }
        }
        let result = segment_utf16(&request(locale, granularity, &input)).unwrap();
        assert_eq!(
            ends(&result),
            wanted,
            "Unicode16 corpus line {}: {}",
            line_number + 1,
            line
        );
        count += 1;
    }
    assert_eq!(count, expected_count);
}
#[test]
fn unicode16_all_1093_grapheme_vectors_keep_original_utf16_indices() {
    corpus(
        SegmenterGranularity::Grapheme,
        "en",
        include_str!("../../data/segmenter-icu-2/corpora/GraphemeBreakTest.txt"),
        1093,
    );
}
#[test]
fn unicode16_all_1826_word_vectors_use_genuine_swedish_rule_locale() {
    // The locked upstream spec_test.rs uses Swedish for the UAX29 corpus;
    // invariant English intentionally has a different colon classification.
    corpus(
        SegmenterGranularity::Word,
        "sv",
        include_str!("../../data/segmenter-icu-2/corpora/WordBreakTest.txt"),
        1826,
    );
}
#[test]
fn unicode16_all_512_sentence_vectors() {
    corpus(
        SegmenterGranularity::Sentence,
        "en",
        include_str!("../../data/segmenter-icu-2/corpora/SentenceBreakTest.txt"),
        512,
    );
}
#[test]
fn every_proposed_locale_constructs_each_genuine_granularity() {
    let actual: Vec<_> = profiles()
        .available_locales()
        .map(CanonicalLocaleId::as_str)
        .collect();
    assert_eq!(
        actual,
        [
            "ar",
            "ar-EG",
            "de",
            "el",
            "en",
            "en-US",
            "fi",
            "fr",
            "hi",
            "it",
            "ja",
            "ko",
            "sr",
            "sv",
            "zh",
            "zh-Hans",
            "zh-Hans-CN"
        ]
    );
    for locale in actual {
        for granularity in SegmenterGranularity::ALL {
            let result = segment_utf16(&request(locale, granularity, &[0x41])).unwrap();
            assert_eq!(ends(&result), [0, 1]);
            assert_eq!(
                result.boundaries()[0].is_word_like(),
                (granularity == SegmenterGranularity::Word).then_some(true)
            );
        }
    }
}
#[test]
fn supported_requests_preserve_serbian_script_region_and_chinese_tags() {
    let requested = ["sr-Thai-RS", "de", "zh-CN", "xyz", "sr-u-ca-hebrew"]
        .map(|s| CanonicalLocaleId::from_data(s).unwrap());
    for matcher in [LocaleMatcher::Lookup, LocaleMatcher::BestFit] {
        let req = SegmenterLocaleRequest {
            requested: requested.clone().into(),
            matcher,
        };
        let supported = profiles().supported_locales(&req);
        assert_eq!(
            supported
                .locales
                .iter()
                .map(CanonicalLocaleId::as_str)
                .collect::<Vec<_>>(),
            ["sr-Thai-RS", "de", "zh-CN", "sr-u-ca-hebrew"]
        );
        assert_eq!(profiles().resolve(&req).unwrap().resolved().as_str(), "sr");
    }
}
#[test]
fn default_and_first_supported_resolution_do_not_invent_locale_data() {
    let req = SegmenterLocaleRequest {
        requested: ["xyz", "ar"]
            .map(|s| CanonicalLocaleId::from_data(s).unwrap())
            .into(),
        matcher: LocaleMatcher::Lookup,
    };
    assert_eq!(profiles().resolve(&req).unwrap().resolved().as_str(), "ar");
    let req = SegmenterLocaleRequest {
        requested: [CanonicalLocaleId::from_data("xyz").unwrap()].into(),
        matcher: LocaleMatcher::BestFit,
    };
    assert_eq!(
        profiles().resolve(&req).unwrap().resolved().as_str(),
        "en-US"
    );
    assert!(profiles().supported_locales(&req).locales.is_empty());
    assert!(profiles()
        .admit(CanonicalLocaleId::from_data("th").unwrap())
        .is_err());
}
#[test]
fn swedish_word_and_greek_sentence_overrides_match_locked_upstream_vectors() {
    expected("sv", SegmenterGranularity::Word, "hello:world", &[0, 11]);
    expected(
        "en",
        SegmenterGranularity::Word,
        "hello:world",
        &[0, 5, 6, 11],
    );
    expected(
        "el",
        SegmenterGranularity::Sentence,
        "hello; world",
        &[0, 7, 12],
    );
    expected(
        "en",
        SegmenterGranularity::Sentence,
        "hello; world",
        &[0, 12],
    );
}
#[test]
fn genuine_auto_thai_and_burmese_models_match_locked_upstream_utf16_vectors() {
    expected(
        "en",
        SegmenterGranularity::Word,
        "ภาษาไทยภาษาไทย",
        &[0, 4, 7, 11, 14],
    );
    expected(
        "en",
        SegmenterGranularity::Word,
        "aภาษาไทยภาษาไทยb",
        &[0, 1, 5, 8, 12, 15, 16],
    );
    expected(
        "en",
        SegmenterGranularity::Word,
        "မြန်မာစာမြန်မာစာမြန်မာစာ",
        &[0, 8, 16, 22, 24],
    );
}
#[test]
fn genuine_auto_cjk_dictionary_uses_code_units_not_utf8_offsets() {
    expected("ja", SegmenterGranularity::Word, "うなぎうなじ", &[0, 3, 6]);
    expected(
        "zh",
        SegmenterGranularity::Word,
        "Welcome龟山岛龟山岛Welcome",
        &[0, 7, 10, 13, 20],
    );
}
#[test]
fn word_like_status_belongs_to_the_preceding_segment_and_is_absent_for_other_granularities() {
    let input: Vec<_> = "Hi, 123!".encode_utf16().collect();
    let result = segment_utf16(&request("en", SegmenterGranularity::Word, &input)).unwrap();
    assert_eq!(ends(&result), [0, 2, 3, 4, 7, 8]);
    assert_eq!(
        result
            .boundaries()
            .iter()
            .map(|r| r.is_word_like())
            .collect::<Vec<_>>(),
        [
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false)
        ]
    );
    for granularity in [
        SegmenterGranularity::Grapheme,
        SegmenterGranularity::Sentence,
    ] {
        let result = segment_utf16(&request("en", granularity, &input)).unwrap();
        assert!(result
            .boundaries()
            .iter()
            .all(|row| row.is_word_like().is_none()));
    }
}
#[test]
fn graphemes_preserve_combining_zwj_supplementary_and_isolated_surrogate_units() {
    expected(
        "en",
        SegmenterGranularity::Grapheme,
        "a\u{301}👩\u{200d}🚀",
        &[0, 2, 7],
    );
    // The isolated high/low surrogates remain individual code units. The valid
    // pair at 1..3 is one scalar and must never be split by a partition.
    let input = [0xd800, 0xd83d, 0xde00, 0xdc00, 0x61];
    let result = segment_utf16(&request("en", SegmenterGranularity::Grapheme, &input)).unwrap();
    assert_eq!(result.input(), input);
    assert_eq!(ends(&result), [0, 1, 3, 4, 5]);
    assert_eq!(
        result.units(*result.containing(2).unwrap()).unwrap(),
        [0xd83d, 0xde00]
    );
}
#[test]
fn empty_and_containing_bounds_have_no_synthetic_segment() {
    for granularity in SegmenterGranularity::ALL {
        let result = segment_utf16(&request("en", granularity, &[])).unwrap();
        assert!(result.boundaries().is_empty());
        assert!(result.containing(0).is_none());
    }
    let result = segment_utf16(&request(
        "en",
        SegmenterGranularity::Grapheme,
        &[0x41, 0x301, 0x42],
    ))
    .unwrap();
    assert_eq!(result.containing(0).unwrap().start(), 0);
    assert_eq!(result.containing(1).unwrap().end(), 2);
    assert_eq!(result.containing(2).unwrap().start(), 2);
    assert!(result.containing(3).is_none());
    assert!(result.containing(u32::MAX).is_none());
}
#[test]
fn checked_locale_and_result_owners_survive_catalogue_and_request_drop() {
    let catalog = SegmenterProfiles::load().unwrap();
    let configuration = CheckedSegmenterConfiguration::new(
        catalog
            .admit(CanonicalLocaleId::from_data("sr").unwrap())
            .unwrap(),
        SegmenterGranularity::Grapheme,
    );
    drop(catalog);
    let req =
        SegmentUtf16Request::new(configuration, vec![0xd800, 0x41].into_boxed_slice()).unwrap();
    let result = segment_utf16(&req).unwrap();
    drop(req);
    assert_eq!(result.input(), [0xd800, 0x41]);
    assert_eq!(ends(&result), [0, 1, 2]);
}
