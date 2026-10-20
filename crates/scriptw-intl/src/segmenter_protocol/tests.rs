use super::*;
use std::sync::OnceLock;
fn profiles() -> &'static SegmenterProfiles {
    static PROFILES: OnceLock<SegmenterProfiles> = OnceLock::new();
    PROFILES.get_or_init(|| SegmenterProfiles::load().unwrap())
}
fn req(granularity: SegmenterGranularity, input: &[u16]) -> SegmentUtf16Request {
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(
            profiles()
                .admit(CanonicalLocaleId::from_data("en").unwrap())
                .unwrap(),
            granularity,
        ),
        input.into(),
    )
    .unwrap()
}
fn locale_request() -> SegmenterLocaleRequest {
    SegmenterLocaleRequest {
        requested: ["sr-Thai-RS", "xyz", "de"]
            .map(|s| CanonicalLocaleId::from_data(s).unwrap())
            .into(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn replace_word(frame: &mut [u8], offset: usize, word: u64) {
    frame[offset..offset + 8].copy_from_slice(&word.to_le_bytes());
}
#[test]
fn reserved_three_primitive_tags_and_configuration_word_have_exact_projections() {
    assert_eq!(
        SegmenterWireOperation::ALL.map(SegmenterWireOperation::code),
        [33, 34, 35]
    );
    assert_eq!(SegmenterConfigurationWord::Granularity.offset(), 0);
    for granularity in SegmenterGranularity::ALL {
        assert_eq!(
            req(granularity, &[]).configuration().wire_words(),
            [granularity.wire_code()]
        );
    }
}
#[test]
fn resolve_and_supported_frames_roundtrip_checked_locale_request_associations() {
    let request = locale_request();
    let encoded = encode_resolve_segmenter_locale_request(&request).unwrap();
    assert_eq!(
        decode_resolve_segmenter_locale_request(&encoded).unwrap(),
        request
    );
    let resolved = profiles().resolve(&request).unwrap();
    let encoded = encode_resolve_segmenter_locale_response(&resolved).unwrap();
    assert_eq!(
        decode_resolve_segmenter_locale_response(profiles(), &encoded)
            .unwrap()
            .resolved()
            .as_str(),
        "sr"
    );
    let encoded = encode_supported_segmenter_locales_request(&request).unwrap();
    assert_eq!(
        decode_supported_segmenter_locales_request(&encoded).unwrap(),
        request
    );
    let supported = profiles().supported_locales(&request);
    let encoded = encode_supported_segmenter_locales_response(&supported).unwrap();
    assert_eq!(
        decode_supported_segmenter_locales_response(profiles(), &request, &encoded).unwrap(),
        supported
    );
    let forged = SegmenterSupportedLocalesResult {
        locales: [CanonicalLocaleId::from_data("en").unwrap()].into(),
    };
    assert!(decode_supported_segmenter_locales_response(
        profiles(),
        &request,
        &encode_supported_segmenter_locales_response(&forged).unwrap()
    )
    .is_err());
}
#[test]
fn segment_frames_keep_original_utf16_surrogates_for_every_granularity() {
    for granularity in SegmenterGranularity::ALL {
        let request = req(granularity, &[0xd800, 0x41, 0xd83d, 0xde00, 0xdc00]);
        let encoded = encode_segment_utf16_request(&request).unwrap();
        let decoded = decode_segment_utf16_request(profiles(), &encoded).unwrap();
        assert_eq!(decoded.input(), request.input());
        let original = segment_utf16(&request).unwrap();
        let encoded = encode_segment_utf16_response(&original).unwrap();
        let decoded = decode_segment_utf16_response(&request, &encoded).unwrap();
        assert_eq!(decoded.input(), original.input());
        assert_eq!(decoded.boundaries(), original.boundaries());
    }
}
#[test]
fn malformed_version_operation_direction_truncation_and_trailing_frames_reject() {
    let request = req(SegmenterGranularity::Grapheme, &[0x41]);
    let encoded = encode_segment_utf16_request(&request).unwrap();
    for size in 0..encoded.len() {
        assert!(
            decode_segment_utf16_request(profiles(), &encoded[..size]).is_err(),
            "prefix {size}"
        );
    }
    for (offset, value) in [(0, 2), (8, 67), (8, 69), (8, 71)] {
        let mut bad = encoded.clone();
        replace_word(&mut bad, offset, value);
        assert!(decode_segment_utf16_request(profiles(), &bad).is_err());
    }
    let mut bad = encoded.clone();
    bad.push(0);
    assert!(decode_segment_utf16_request(profiles(), &bad).is_err());
}
#[test]
fn malformed_locale_granularity_counts_and_duplicate_requests_reject() {
    let request = req(SegmenterGranularity::Grapheme, &[]);
    let encoded = encode_segment_utf16_request(&request).unwrap();
    // Header16 + locale length8 + en2 = granularity at26, input count34.
    for (offset, value) in [(26, 0), (26, 4), (34, u64::MAX)] {
        let mut bad = encoded.clone();
        replace_word(&mut bad, offset, value);
        assert!(decode_segment_utf16_request(profiles(), &bad).is_err());
    }
    let mut bad = encoded.clone();
    bad[24] = b'E';
    assert!(decode_segment_utf16_request(profiles(), &bad).is_err());
    let mut bad = encoded.clone();
    bad[24] = 0xff;
    assert!(decode_segment_utf16_request(profiles(), &bad).is_err());
    let duplicate = SegmenterLocaleRequest {
        requested: ["en", "en"]
            .map(|s| CanonicalLocaleId::from_data(s).unwrap())
            .into(),
        matcher: LocaleMatcher::Lookup,
    };
    assert!(decode_resolve_segmenter_locale_request(
        &encode_resolve_segmenter_locale_request(&duplicate).unwrap()
    )
    .is_err());
}
#[test]
fn malformed_response_partition_pair_split_annotation_and_owner_associations_reject() {
    let request = req(SegmenterGranularity::Grapheme, &[0xd83d, 0xde00, 0x41]);
    let good = encode_segment_utf16_response(&segment_utf16(&request).unwrap()).unwrap();
    // Header16 + granularity8 + input length8 + count8; rows are end/annotation.
    for (offset, value) in [
        (16, 2),
        (24, 4),
        (32, u64::MAX),
        (40, 0),
        (40, 1),
        (40, 4),
        (48, 1),
        (48, 3),
        (56, 2),
    ] {
        let mut bad = good.clone();
        replace_word(&mut bad, offset, value);
        assert!(
            decode_segment_utf16_response(&request, &bad).is_err(),
            "offset={offset} value={value}"
        );
    }
    let word_request = req(SegmenterGranularity::Word, &[0x41]);
    let mut bad = encode_segment_utf16_response(&segment_utf16(&word_request).unwrap()).unwrap();
    replace_word(&mut bad, 48, 0);
    assert!(decode_segment_utf16_response(&word_request, &bad).is_err());
}
#[test]
fn empty_response_is_zero_rows_and_cannot_forge_a_synthetic_boundary() {
    let request = req(SegmenterGranularity::Sentence, &[]);
    let good = encode_segment_utf16_response(&segment_utf16(&request).unwrap()).unwrap();
    assert!(decode_segment_utf16_response(&request, &good)
        .unwrap()
        .boundaries()
        .is_empty());
    let mut bad = good;
    replace_word(&mut bad, 32, 1);
    bad.extend_from_slice(&0u64.to_le_bytes());
    bad.extend_from_slice(&0u64.to_le_bytes());
    assert!(decode_segment_utf16_response(&request, &bad).is_err());
}
