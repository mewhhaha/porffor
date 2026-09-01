const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_with_buffer_witness.js");

fn with_compiler() -> &'static str {
    STANDARD_SOURCE
        .split_once("    fn compile_typed_array_prototype_with_builtin(")
        .expect("missing TypedArray.prototype.with compiler")
        .1
        .split_once("    fn compile_typed_array_prototype_set_builtin(")
        .expect("missing boundary after TypedArray.prototype.with compiler")
        .0
}

fn unique_position(source: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        source.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    source
        .find(needle)
        .unwrap_or_else(|| panic!("missing {label}"))
}

#[test]
fn with_captures_one_validated_method_entry_before_coercion() {
    let body = with_compiler();

    assert_eq!(
        body.matches("emit_throw_current_function_realm_range_error(")
            .count(),
        1
    );
    assert!(!body.contains("emit_throw_runtime_error("));

    for (needle, label) in [
        (
            "emit_load_typed_array_private_state(",
            "immutable private-state load",
        ),
        ("TypedArrayViewLocals::new(", "immutable view"),
        ("emit_typed_array_witness(", "buffer witness"),
        (
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            "validated-method-entry projection",
        ),
        ("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET", "element-kind load"),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            1,
            "TypedArray.prototype.with must contain exactly one {label}"
        );
    }

    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
    ] {
        assert!(
            !body.contains(forbidden),
            "TypedArray.prototype.with must not bypass its entry witness through {forbidden}"
        );
    }

    let index_argument = unique_position(
        body,
        "emit_builtin_arg_to_locals(0, index_payload_local, index_tag_local, function)",
        "index argument acquisition",
    );
    let value_argument = unique_position(
        body,
        "emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function)",
        "value argument acquisition",
    );
    let brand = unique_position(
        body,
        "OBJECT_INTERNAL_BRAND_TYPED_ARRAY",
        "receiver brand check",
    );
    let private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "immutable private-state load",
    );
    let element_kind = unique_position(
        body,
        "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
        "element-kind load",
    );
    let view = unique_position(body, "TypedArrayViewLocals::new(", "immutable view");
    let witness = unique_position(
        body,
        "TypedArrayWitnessUse::ValidatedMethodEntry",
        "validated-method-entry witness",
    );
    let index_coercion = unique_position(
        body,
        "emit_value_to_number_payload(index_tag_local, index_payload_local, function)",
        "index coercion",
    );
    let value_coercion = unique_position(
        body,
        "emit_atomics_bigint_element_kind_i32(receiver_element_kind_local, function)",
        "value coercion dispatch",
    );
    let live_index = unique_position(
        body,
        "emit_typed_array_valid_integer_index_i32(",
        "post-coercion live index validation",
    );

    assert!(
        index_argument < value_argument
            && value_argument < brand
            && brand < private_state
            && private_state < element_kind
            && element_kind < view
            && view < witness
            && witness < index_coercion
            && index_coercion < value_coercion
            && value_coercion < live_index
    );
}

#[test]
fn focused_cli_fixture_pins_with_entry_witness_behavior() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_validates_typedarray_with_entry_buffer_witness()")
        .expect("missing focused TypedArray.prototype.with witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray.prototype.with witness CLI test")
        .0;

    assert!(test.contains("wasm_typedarray_with_buffer_witness.js"));
    assert!(test.contains("boolean(true)"));
    for marker in [
        "detached receiver error realm",
        "detached receiver skips coercion",
        "out-of-bounds receiver error realm",
        "odd-byte tracking length floor",
        "borrowed with out-of-range error realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing TypedArray.prototype.with CLI control: {marker}"
        );
    }
}
