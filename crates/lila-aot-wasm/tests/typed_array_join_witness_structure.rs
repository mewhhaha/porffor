const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");

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
fn typed_array_join_uses_one_validated_method_entry_witness() {
    let join = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_typed_array_prototype_join_builtin(",
        "fn emit_array_join_generic_from_locals(",
    );

    assert_eq!(
        join.matches("emit_load_typed_array_private_state(").count(),
        1,
        "join must load one immutable private view record"
    );
    assert_eq!(
        join.matches("TypedArrayViewLocals::new(").count(),
        1,
        "join must construct one immutable view projection"
    );
    assert_eq!(
        join.matches("emit_typed_array_witness(").count(),
        1,
        "join must create one live buffer witness"
    );
    assert_eq!(
        join.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1,
        "join must select the throwing method-entry projection"
    );

    assert!(!join.contains("emit_validate_typed_array_current_byte_length("));
    assert!(!join.contains("emit_throw_runtime_error("));
    assert!(!join.contains("TYPE_ERROR_NAME"));
    assert!(!join.contains("Instruction::I64DivU"));
    for private_view_slot in [
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
    ] {
        assert!(
            !join.contains(private_view_slot),
            "join must not reconstruct the private view through {private_view_slot}"
        );
    }

    let brand_check = join
        .find("TypedArray.prototype.join requires a TypedArray")
        .expect("join must retain its receiver-brand check");
    let witness = join
        .find("emit_typed_array_witness(")
        .expect("join must emit its buffer witness");
    let separator = join
        .find("emit_builtin_arg_to_locals(0")
        .expect("join must load its separator");
    assert!(
        brand_check < witness,
        "brand validation must precede private-state use"
    );
    assert!(
        witness < separator,
        "buffer validation must precede separator coercion"
    );
}
