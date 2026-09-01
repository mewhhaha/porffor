const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_constructed_target_witness.js");

fn constructed_target_validation() -> &'static str {
    OBJECTS_SOURCE
        .split_once("    pub(crate) fn emit_validate_typed_array_from_constructed_target(")
        .expect("missing constructed TypedArray target validator")
        .1
        .split_once("    /// Applies the integer typed-array conversion modulo 2^32.")
        .expect("missing boundary after constructed TypedArray target validator")
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
fn constructed_target_validation_uses_one_validated_method_entry_witness() {
    let body = constructed_target_validation();

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
    ] {
        assert_eq!(
            body.matches(needle).count(),
            1,
            "constructed target validation must contain exactly one {label}"
        );
    }

    for forbidden in [
        "emit_throw_runtime_error(",
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
            "constructed target validation must not bypass its witness through {forbidden}"
        );
    }
    assert_eq!(
        body.matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2,
        "both validator-owned TypeErrors must use the executing builtin's Realm"
    );

    let brand = unique_position(
        body,
        "OBJECT_INTERNAL_BRAND_TYPED_ARRAY",
        "constructed target brand check",
    );
    let private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "immutable private-state load",
    );
    let view = unique_position(body, "TypedArrayViewLocals::new(", "immutable view");
    let witness = unique_position(
        body,
        "TypedArrayWitnessUse::ValidatedMethodEntry",
        "validated-method-entry witness",
    );
    let requested_length = unique_position(
        body,
        "Instruction::LocalGet(requested_length_payload_local)",
        "requested-length conversion",
    );
    let capacity = unique_position(
        body,
        "Instruction::LocalGet(capacity_local)",
        "witness-produced capacity comparison",
    );
    let too_small = unique_position(
        body,
        "Constructed typed array is too small",
        "too-small target error",
    );

    assert!(
        brand < private_state
            && private_state < view
            && view < witness
            && witness < requested_length
            && requested_length < capacity
            && capacity < too_small
    );
}

#[test]
fn focused_cli_fixture_pins_constructed_target_error_realms() {
    let test = CLI_TESTS
        .split_once(
            "fn run_wasm_backend_validates_constructed_typedarray_targets_with_live_witness()",
        )
        .expect("missing focused constructed TypedArray target CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused constructed TypedArray target CLI test")
        .0;

    assert!(test.contains("wasm_typedarray_constructed_target_witness.js"));
    assert!(test.contains("boolean(true)"));
    for marker in [
        "borrowed slice detached species target",
        "borrowed slice out-of-bounds species target",
        "borrowed slice non-TypedArray species target",
        "borrowed slice undersized species target",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing constructed-target CLI control: {marker}"
        );
    }
}
