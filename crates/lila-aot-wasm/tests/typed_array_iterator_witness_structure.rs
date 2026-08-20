const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const ITERATORS_SOURCE: &str = include_str!("../src/builtins/iterators.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

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
fn typed_array_iterator_boundaries_share_the_live_buffer_witness() {
    let creation = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::ArrayPrototypeKeys\n            | StandardBuiltinId::ArrayPrototypeEntries",
        "StandardBuiltinId::ArrayIteratorIdentity => {",
    );
    let step = bounded(
        ITERATORS_SOURCE,
        "pub(crate) fn emit_typed_array_iterator_next_from_locals(",
        "pub(crate) fn emit_iterator_result_object_from_locals(",
    );

    for (label, body) in [("creation", creation), ("step", step)] {
        assert_eq!(
            body.matches("emit_load_typed_array_private_state(").count(),
            1,
            "{label} must load the private view record once"
        );
        assert_eq!(
            body.matches("TypedArrayViewLocals::new(").count(),
            1,
            "{label} must construct one immutable view projection"
        );
        assert_eq!(
            body.matches("emit_typed_array_witness(").count(),
            1,
            "{label} must create one live buffer witness"
        );
        assert_eq!(
            body.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
                .count(),
            1,
            "{label} must select the throwing method-entry projection"
        );
        assert!(!body.contains("emit_validate_typed_array_current_byte_length("));
        for private_view_slot in [
            "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
            "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        ] {
            assert!(
                !body.contains(private_view_slot),
                "{label} must not reconstruct the private view through {private_view_slot}"
            );
        }
    }
}

#[test]
fn validated_witness_errors_use_the_current_function_realm() {
    let validation = bounded(
        BINARY_DATA_SOURCE,
        "match use_ {\n            TypedArrayWitnessUse::ValidatedMethodEntry { .. } => {",
        "            TypedArrayWitnessUse::ArrayLikeLengthSnapshot { .. }",
    );
    assert_eq!(
        validation
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2,
        "detached and out-of-bounds failures must both use the function Realm"
    );
    assert!(!validation.contains("emit_throw_runtime_error("));
    assert!(!validation.contains("TYPE_ERROR_NAME"));
}
