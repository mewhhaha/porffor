const ENCODING_SOURCE: &str = include_str!("../src/heap_value_encodings.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn variants<'a>(source: &'a str, declaration: &str) -> Vec<&'a str> {
    source
        .split_once(declaration)
        .unwrap_or_else(|| panic!("missing enum declaration: {declaration}"))
        .1
        .split_once("\n}")
        .unwrap_or_else(|| panic!("unterminated enum declaration: {declaration}"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn normalized(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn heap_value_and_payload_encodings_are_exact_closed_domains() {
    assert!(!ENCODING_SOURCE.contains(")]\npub(crate) enum HeapValueEncoding"));
    assert_eq!(
        variants(ENCODING_SOURCE, "pub(crate) enum HeapValueEncoding {"),
        [
            "Undefined,",
            "Null,",
            "Boolean,",
            "Number,",
            "String,",
            "Symbol,",
            "Object,",
            "Array,",
            "Function,",
            "Arguments,",
            "BigInt,",
            "Dynamic,",
        ]
    );
    assert_eq!(
        variants(ENCODING_SOURCE, "pub(crate) enum ValuePayloadEncoding {"),
        [
            "Immediate,",
            "BooleanBit,",
            "Ieee754Bits,",
            "HeapPointer,",
            "StaticOrHeapPointer,",
            "I64TemporaryOrHeapPointer,",
            "DynamicTaggedPair,",
        ]
    );
    assert!(!ENCODING_SOURCE.contains("I64Temporary,"));
}

#[test]
fn value_identity_owns_four_exhaustive_projections() {
    let implementation = bounded(
        ENCODING_SOURCE,
        "impl HeapValueEncoding {",
        "pub(crate) const HEAP_VALUE_ENCODINGS",
    );
    assert_eq!(implementation.matches("match self {").count(), 4);
    assert!(!implementation.contains("_ =>"));
    let kind_projection = bounded(
        implementation,
        "pub(crate) const fn kind",
        "pub(crate) const fn payload",
    );
    let payload_projection = bounded(
        implementation,
        "pub(crate) const fn payload",
        "pub(crate) const fn preserves_number_bits",
    );
    let number_bits_projection = bounded(
        implementation,
        "pub(crate) const fn preserves_number_bits",
        "pub(crate) const fn arbitrary_precision_ready",
    );
    let precision_projection = bounded(
        implementation,
        "pub(crate) const fn arbitrary_precision_ready",
        "}\n}",
    );

    for (variant, kind, payload, preserves_number_bits, arbitrary_precision_ready) in [
        ("Undefined", "Undefined", "Immediate", false, true),
        ("Null", "Null", "Immediate", false, true),
        ("Boolean", "Boolean", "BooleanBit", false, true),
        ("Number", "Number", "Ieee754Bits", true, true),
        ("String", "String", "StaticOrHeapPointer", false, true),
        ("Symbol", "Symbol", "StaticOrHeapPointer", false, true),
        ("Object", "Object", "HeapPointer", false, true),
        ("Array", "Array", "HeapPointer", false, true),
        ("Function", "Function", "HeapPointer", false, true),
        ("Arguments", "Arguments", "HeapPointer", false, true),
        (
            "BigInt",
            "BigInt",
            "I64TemporaryOrHeapPointer",
            false,
            false,
        ),
        ("Dynamic", "Dynamic", "DynamicTaggedPair", false, true),
    ] {
        assert!(kind_projection.contains(&format!("Self::{variant} => ValueKind::{kind},")));
        assert!(payload_projection.contains(&format!(
            "Self::{variant} => ValuePayloadEncoding::{payload},"
        )));
        assert!(number_bits_projection
            .contains(&format!("Self::{variant} => {preserves_number_bits},")));
        assert!(precision_projection
            .contains(&format!("Self::{variant} => {arbitrary_precision_ready},")));
    }
}

#[test]
fn heap_value_registry_contains_every_identity_once_in_order() {
    let registry = normalized(bounded(
        ENCODING_SOURCE,
        "pub(crate) const HEAP_VALUE_ENCODINGS",
        "];",
    ));
    assert_eq!(
        registry,
        ":&[HeapValueEncoding]=&[HeapValueEncoding::Undefined,HeapValueEncoding::Null,HeapValueEncoding::Boolean,HeapValueEncoding::Number,HeapValueEncoding::String,HeapValueEncoding::Symbol,HeapValueEncoding::Object,HeapValueEncoding::Array,HeapValueEncoding::Function,HeapValueEncoding::Arguments,HeapValueEncoding::BigInt,HeapValueEncoding::Dynamic,"
    );
}

#[test]
fn heap_owner_consumes_typed_encoding_projections_only() {
    assert!(!HEAP_SOURCE.contains("struct ValueEncodingSlot"));
    assert!(!HEAP_SOURCE.contains("VALUE_ENCODING_SLOTS"));
    assert!(!HEAP_SOURCE.contains("preserves_number_bits:"));
    assert!(!HEAP_SOURCE.contains("arbitrary_precision_ready:"));

    let owner = bounded(
        HEAP_SOURCE,
        "fn assert_value_encodings(",
        "#[test]\n    fn heap_limits_are_stable()",
    );
    assert!(owner.contains("encoding.kind()"));
    assert!(owner.contains("encoding.payload()"));
    assert!(owner.contains("encoding.preserves_number_bits()"));

    let witness = bounded(
        HEAP_SOURCE,
        "fn heap_value_encoding_registry_covers_ecmascript_language_types()",
        "fn linear_side_storage_identities_own_metadata_and_element_semantics()",
    );
    assert!(witness.contains("assert_value_encodings(HEAP_VALUE_ENCODINGS);"));
    assert!(witness.contains("number.preserves_number_bits()"));
    assert!(witness.contains("!bigint.arbitrary_precision_ready()"));
}
