const RUNTIME_ABI_SOURCE: &str = include_str!("../src/runtime_abi.rs");

fn heap_bigint_sign_domain_source() -> &'static str {
    let start = RUNTIME_ABI_SOURCE
        .find("enum HeapBigIntSign {")
        .expect("heap BigInt sign domain should exist");
    let end = RUNTIME_ABI_SOURCE[start..]
        .find("\n}")
        .map(|offset| start + offset + 2)
        .expect("heap BigInt sign domain should be bounded");
    &RUNTIME_ABI_SOURCE[start..end]
}

fn heap_bigint_decoder_source() -> &'static str {
    let start = RUNTIME_ABI_SOURCE
        .find("pub fn decode_heap_bigint_decimal")
        .expect("heap BigInt decoder should exist");
    let end = RUNTIME_ABI_SOURCE[start..]
        .find("\nfn read_record_u64")
        .map(|offset| start + offset)
        .expect("heap BigInt decoder should have a bounded owner");
    &RUNTIME_ABI_SOURCE[start..end]
}

#[test]
fn heap_bigint_sign_is_the_exact_closed_abi_domain() {
    let domain = heap_bigint_sign_domain_source();

    assert!(domain.contains("Negative,"));
    assert!(domain.contains("Zero,"));
    assert!(domain.contains("Positive,"));
    assert_eq!(
        domain
            .lines()
            .filter(|line| line.trim().ends_with(','))
            .count(),
        3
    );
}

#[test]
fn heap_bigint_decoder_parses_once_and_projects_both_sign_meanings_exhaustively() {
    let decoder = heap_bigint_decoder_source();

    for parse_arm in [
        "-1 => HeapBigIntSign::Negative",
        "0 => HeapBigIntSign::Zero",
        "1 => HeapBigIntSign::Positive",
    ] {
        assert!(
            decoder.contains(parse_arm),
            "missing parse arm: {parse_arm}"
        );
    }
    for projection_arm in [
        "HeapBigIntSign::Negative => (Sign::Minus, false)",
        "HeapBigIntSign::Zero => (Sign::NoSign, true)",
        "HeapBigIntSign::Positive => (Sign::Plus, false)",
    ] {
        assert!(
            decoder.contains(projection_arm),
            "missing paired projection arm: {projection_arm}"
        );
    }
    assert_eq!(decoder.matches("match sign {").count(), 1);
    assert!(!decoder.contains("HeapBigIntSign::_"));
    assert!(!decoder.contains("_ => (Sign::"));
}

#[test]
fn heap_bigint_decoder_uses_only_the_closed_sign_for_semantic_decisions() {
    let decoder = heap_bigint_decoder_source();

    assert!(decoder.contains("if magnitude_must_be_zero != magnitude_is_zero"));
    assert!(decoder.contains("BigInt::from_bytes_le(bigint_sign, &limb_bytes)"));
    assert!(!decoder.contains("sign_value =="));
    assert!(!decoder.contains("sign_value !="));
}
