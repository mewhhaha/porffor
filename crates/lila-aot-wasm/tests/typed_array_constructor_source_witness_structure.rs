const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_typedarray_constructor_source_buffer_witness.js"
);

fn typed_array_constructor_arm() -> &'static str {
    STANDARD_SOURCE
        .split_once("            StandardBuiltinId::Float64ArrayConstructor")
        .expect("missing TypedArray constructor builtin arm")
        .1
        .split_once("            StandardBuiltinId::DataViewPrototypeGetUint8")
        .expect("missing boundary after TypedArray constructor builtin arm")
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
fn typed_array_source_uses_one_validated_method_entry_witness() {
    let body = typed_array_constructor_arm();

    for (needle, label) in [
        (
            "emit_load_typed_array_private_state(",
            "immutable source private-state load",
        ),
        ("TypedArrayViewLocals::new(", "immutable source view"),
        ("emit_typed_array_witness(", "source buffer witness"),
        (
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            "source validated-method-entry projection",
        ),
        (
            "HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
            "target element-kind load",
        ),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            1,
            "TypedArray construction must contain exactly one {label}"
        );
    }

    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "source_data_ptr_local",
    ] {
        assert!(
            !body.contains(forbidden),
            "TypedArray construction must not bypass its source witness through {forbidden}"
        );
        assert!(
            !BINARY_DATA_SOURCE.contains(forbidden),
            "the retired raw current-length helper must be absent from binary_data.rs: {forbidden}"
        );
    }

    for local in [
        "source_buffer_local",
        "source_byte_offset_local",
        "source_stored_byte_length_local",
        "source_bytes_per_element_local",
    ] {
        assert!(
            body.contains(local),
            "the immutable source view must own {local}"
        );
    }

    for offset in [
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
    ] {
        let offset_position = unique_position(body, offset, "target private-state offset");
        let store_position = body[..offset_position]
            .rfind("self.store_i64_local_at_offset(")
            .unwrap_or_else(|| panic!("{offset} must belong to a target private-state store"));
        let store = &body[store_position..offset_position];
        assert!(
            store.contains("typed_array_object_local") && store.len() < 200,
            "{offset} must be used only to initialize the constructed target"
        );
    }

    assert_eq!(
        body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
        2
    );
    let source_kind = unique_position(
        body,
        "arg_payload_local,\n                    HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,\n                    source_element_kind_local,",
        "validated source content-kind load",
    );
    let target_kind = unique_position(
        body,
        "typed_array_object_local,\n                    HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,\n                    element_kind_local,",
        "constructed target content-kind store",
    );
    assert!(source_kind < target_kind);

    let unsigned_divide = unique_position(
        body,
        "Instruction::I64DivU",
        "backing-memory page-count division",
    );
    let page_size = body[..unsigned_divide]
        .rfind("WASM_PAGE_SIZE as i64")
        .expect("the sole unsigned division must use the Wasm page size");
    assert!(
        unsigned_divide - page_size < 300,
        "the constructor must not divide a raw TypedArray byte length"
    );
}

#[test]
fn source_snapshot_precedes_target_allocation_and_copy() {
    let body = typed_array_constructor_arm();
    let selected_prototype = unique_position(
        body,
        "Instruction::LocalSet(selected_prototype_payload_local)",
        "selected result prototype",
    );
    assert_eq!(
        body.matches("OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64")
            .count(),
        2,
        "the constructor validates the typed source, then selects its post-allocation copy path"
    );
    let source_brand = body
        .find("OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64")
        .expect("missing TypedArray source brand branch");
    let source_private_state = unique_position(
        body,
        "emit_load_typed_array_private_state(",
        "source private-state load",
    );
    let source_view = unique_position(body, "let source_view", "immutable source view");
    let source_witness = unique_position(
        body,
        "TypedArrayWitnessUse::ValidatedMethodEntry",
        "source length snapshot",
    );
    let content_type = unique_position(
        body,
        "TypedArray constructor source and target content types differ",
        "source and target content-type validation",
    );
    let generator_throw_probe = unique_position(
        body,
        "self.strings.payload(LILA_GENERATOR_THROW_SLOT)",
        "synthetic generator-throw probe",
    );
    let typed_array_branch = &body[source_witness..generator_throw_probe];
    let non_typed_array_branch = typed_array_branch
        .rfind("Instruction::Else")
        .map(|position| source_witness + position)
        .expect("the generator-throw probe must follow the non-TypedArray branch boundary");
    assert!(
        typed_array_branch[..non_typed_array_branch - source_witness]
            .contains("Instruction::LocalSet(byte_offset_local)"),
        "the TypedArray branch must finish its target offset before the non-TypedArray boundary"
    );
    let iterator_probe = body[generator_throw_probe..]
        .find("self.strings.property_key_symbol_payload(\"Symbol.iterator\")")
        .map(|position| generator_throw_probe + position)
        .expect("the non-TypedArray generator-throw probe must precede iterator lookup");
    let target_allocation = unique_position(
        body,
        "let buffer_memory_alloc = self.functions.shared_memory_alloc_function_index()",
        "target backing-store allocation",
    );
    let copy_branch = body
        .rfind("OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64")
        .expect("missing post-allocation typed-source copy branch");
    let raw_copy = unique_position(
        body,
        "emit_typed_array_copy_bytes_in_order(",
        "same-kind raw-byte copy",
    );
    let indexed_read = unique_position(
        body,
        "emit_typed_array_or_object_index_read_from_locals(",
        "source indexed read",
    );
    let conversions: Vec<_> = body
        .match_indices("emit_value_to_typed_array_element_payload(")
        .map(|(position, _)| position)
        .collect();
    let target_materialization = unique_position(
        body,
        "Instruction::LocalSet(typed_array_object_local)",
        "target TypedArray materialization",
    );

    assert_eq!(conversions.len(), 2);
    assert!(
        selected_prototype < source_brand
            && source_brand < source_private_state
            && source_private_state < source_view
            && source_view < source_witness
            && source_witness < non_typed_array_branch
            && non_typed_array_branch < generator_throw_probe
            && generator_throw_probe < iterator_probe
            && source_witness < content_type
            && content_type < target_allocation
            && target_allocation < copy_branch
            && copy_branch < raw_copy
            && raw_copy < indexed_read
            && indexed_read < conversions[1]
            && conversions[1] < target_materialization
    );
}

#[test]
fn focused_cli_fixture_pins_constructor_source_witness_behavior() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_validates_typedarray_constructor_source_buffer_witness()")
        .expect("missing focused TypedArray constructor source-witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray constructor source-witness CLI test")
        .0;

    assert!(test.contains("wasm_typedarray_constructor_source_buffer_witness.js"));
    assert!(test.contains("boolean(true)"));
    for marker in [
        "constructor detached source",
        "constructor out-of-bounds source",
        "odd-byte source length snapshot",
        "fixed source regrowth",
        "TypedArray source skips generator throw slot",
        "Number source into BigInt target",
        "BigInt source into Number target",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing TypedArray constructor source-witness CLI control: {marker}"
        );
    }
}
