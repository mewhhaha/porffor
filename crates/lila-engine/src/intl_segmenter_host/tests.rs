use super::*;
use crate::intl_host_probe::IntlHostProbe;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::*;

fn locale() -> SegmenterLocaleRequest {
    SegmenterLocaleRequest {
        requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
    }
}
fn request() -> SegmentUtf16Request {
    let locale = embedded_segmenter_profiles()
        .unwrap()
        .resolve(&locale())
        .unwrap();
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(locale, SegmenterGranularity::Grapheme),
        vec![0xd800, 0x61, 0xd83d, 0xde00].into_boxed_slice(),
    )
    .unwrap()
}
fn gc_response(probe: &mut IntlHostProbe, operation: IntlHostOp, bytes: &[u8]) -> Vec<u8> {
    probe
        .invoke(operation, bytes)
        .unwrap()
        .expect("accepted Intl request")
}

#[test]
fn all_segmenter_operations_return_owned_gc_responses() {
    let mut probe = IntlHostProbe::new();
    let profiles = embedded_segmenter_profiles().unwrap();
    let response = gc_response(
        &mut probe,
        IntlHostOp::ResolveSegmenterLocale,
        &encode_resolve_segmenter_locale_request(&locale()).unwrap(),
    );
    assert_eq!(
        decode_resolve_segmenter_locale_response(profiles, &response)
            .unwrap()
            .resolved()
            .as_str(),
        "en-US"
    );
    let response = gc_response(
        &mut probe,
        IntlHostOp::SupportedSegmenterLocales,
        &encode_supported_segmenter_locales_request(&locale()).unwrap(),
    );
    assert_eq!(
        decode_supported_segmenter_locales_response(profiles, &locale(), &response)
            .unwrap()
            .locales[0]
            .as_str(),
        "en-US"
    );
    let request = request();
    let response = gc_response(
        &mut probe,
        IntlHostOp::SegmentUtf16,
        &encode_segment_utf16_request(&request).unwrap(),
    );
    let result = decode_segment_utf16_response(&request, &response).unwrap();
    assert_eq!(result.input(), request.input());
    assert_eq!(
        result
            .boundaries()
            .iter()
            .map(|row| row.end())
            .collect::<Vec<_>>(),
        [1, 2, 4]
    );
}
#[test]
fn malformed_segmenter_frames_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let valid = [
        (
            IntlHostOp::ResolveSegmenterLocale,
            encode_resolve_segmenter_locale_request(&locale()).unwrap(),
        ),
        (
            IntlHostOp::SupportedSegmenterLocales,
            encode_supported_segmenter_locales_request(&locale()).unwrap(),
        ),
        (
            IntlHostOp::SegmentUtf16,
            encode_segment_utf16_request(&request()).unwrap(),
        ),
    ];
    for (op, original) in valid {
        let mut bad_version = original.clone();
        bad_version[0] = 2;
        let mut trailing = original.clone();
        trailing.push(0);
        let mut bad_operation = original.clone();
        bad_operation[8] = 0;
        for bytes in [
            bad_version,
            trailing,
            bad_operation,
            original[..15].to_vec(),
        ] {
            assert!(probe.invoke(op, &bytes).is_err());
        }
    }
}
#[test]
fn unadmitted_locale_and_invalid_granularity_fault_at_the_gc_boundary() {
    let mut probe = IntlHostProbe::new();
    let original = encode_segment_utf16_request(&request()).unwrap();
    let mut locale = original.clone();
    locale[24..29].copy_from_slice(b"xx-XX");
    let mut granularity = original;
    granularity[29..37].copy_from_slice(&0_u64.to_le_bytes());
    for bytes in [locale, granularity] {
        assert!(probe.invoke(IntlHostOp::SegmentUtf16, &bytes).is_err());
    }
}
