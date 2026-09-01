const RUNTIME_SOURCE: &str = include_str!("../src/lib.rs");
const ENGINE_SOURCE: &str = include_str!("../../lila-engine/src/lib.rs");
const SPEC_EXEC_SOURCE: &str = include_str!("../../lila-spec-exec/src/lib.rs");
const DIFFERENTIAL_SOURCE: &str = include_str!("../../lila-test262/src/differential.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn observed_bigint_has_one_private_canonical_decimal_representation() {
    assert!(RUNTIME_SOURCE.contains("pub struct ObservedBigInt(Box<str>);"));
    assert!(RUNTIME_SOURCE.contains("BigInt(ObservedBigInt),"));
    assert!(!RUNTIME_SOURCE.contains("BigInt(Box<str>),"));

    let parser = bounded(
        RUNTIME_SOURCE,
        "impl ObservedBigInt {",
        "pub struct InvalidObservedBigInt {",
    );
    assert!(parser.contains("pub fn parse_canonical_decimal("));
    assert!(parser.contains("Result<Self, InvalidObservedBigInt>"));
    assert!(parser.contains("pub fn as_str(&self) -> &str"));

    let error = bounded(
        RUNTIME_SOURCE,
        "pub struct InvalidObservedBigInt {",
        "impl std::error::Error for InvalidObservedBigInt {}",
    );
    assert!(error.contains("decimal: Box<str>,"));
    assert!(error.contains("pub fn decimal(&self) -> &str"));
    assert!(error.contains("is not canonical"));
}

#[test]
fn every_observed_bigint_producer_crosses_the_parser_once() {
    let engine = bounded(
        ENGINE_SOURCE,
        "fn observe_wasmtime_value(",
        "fn decode_wasmtime_heap_bigint(",
    );
    assert_eq!(
        engine
            .matches("ObservedBigInt::parse_canonical_decimal(")
            .count(),
        2
    );

    let spec_exec = bounded(
        SPEC_EXEC_SOURCE,
        "fn observe_js_value(",
        "fn observe_opaque_throw(",
    );
    assert_eq!(
        spec_exec
            .matches("ObservedBigInt::parse_canonical_decimal(")
            .count(),
        1
    );
}

#[test]
fn differential_projection_reads_only_canonical_decimal_text() {
    let projection = bounded(
        DIFFERENTIAL_SOURCE,
        "impl PrimitiveValueObservation {",
        "const fn type_name(&self)",
    );
    assert!(projection.contains("ObservedJsValue::BigInt(decimal)"));
    assert!(projection.contains("decimal: decimal.as_str().to_string(),"));
}
