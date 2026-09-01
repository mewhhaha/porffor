const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_set_buffer_witness.js");

fn set_compiler() -> &'static str {
    STANDARD_SOURCE
        .split_once("    fn compile_typed_array_prototype_set_builtin(")
        .expect("missing TypedArray.prototype.set compiler")
        .1
        .split_once("    fn emit_iterator_zip_keyed(")
        .expect("missing boundary after TypedArray.prototype.set compiler")
        .0
}

fn positions(source: &str, needle: &str) -> Vec<usize> {
    source
        .match_indices(needle)
        .map(|(index, _)| index)
        .collect()
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
fn set_uses_two_receiver_witnesses_and_one_typed_source_witness() {
    let body = set_compiler();

    assert_eq!(
        body.matches("emit_throw_current_function_realm_range_error(")
            .count(),
        4
    );
    assert!(!body.contains("emit_throw_runtime_error("));

    assert_eq!(
        body.matches("emit_load_typed_array_private_state(").count(),
        2
    );
    assert_eq!(body.matches("TypedArrayViewLocals::new(").count(), 2);
    assert_eq!(body.matches("emit_typed_array_witness(").count(), 3);
    assert_eq!(
        body.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        3
    );
    assert_eq!(
        body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
        2
    );

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
        "receiver_data_ptr_local",
        "source_data_ptr_local",
        "Instruction::I64DivU",
    ] {
        assert!(
            !body.contains(forbidden),
            "TypedArray.prototype.set must not bypass its witnesses through {forbidden}"
        );
    }

    let receiver_view = unique_position(body, "let receiver_view", "immutable receiver view");
    let source_view = unique_position(body, "let source_view", "immutable source view");
    let witnesses = positions(body, "TypedArrayWitnessUse::ValidatedMethodEntry");
    let offset_argument = unique_position(
        body,
        "emit_builtin_arg_to_locals(1, offset_payload_local, offset_tag_local, function)",
        "offset argument acquisition",
    );
    let offset_coercion = unique_position(
        body,
        "emit_to_index_i64_from_value_locals(",
        "offset coercion",
    );
    let source_argument = unique_position(
        body,
        "emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function)",
        "source argument acquisition",
    );
    let source_brand = body[source_argument..]
        .find("OBJECT_INTERNAL_BRAND_TYPED_ARRAY")
        .map(|position| source_argument + position)
        .expect("missing TypedArray source brand check");
    let content_type = unique_position(
        body,
        "TypedArray.prototype.set source and target content types differ",
        "content-type check",
    );

    assert_eq!(witnesses.len(), 3);
    assert!(
        receiver_view < witnesses[0]
            && witnesses[0] < offset_argument
            && offset_argument < offset_coercion
            && offset_coercion < witnesses[1]
            && witnesses[1] < source_argument
            && source_argument < source_brand
            && source_brand < source_view
            && source_view < witnesses[2]
            && witnesses[2] < content_type
    );
}

#[test]
fn typed_source_overlap_is_staged_before_target_writes() {
    let body = set_compiler();
    let source_witnesses = positions(body, "TypedArrayWitnessUse::ValidatedMethodEntry");
    let allocation = unique_position(
        body,
        "self.emit_heap_alloc_from_local(temporary_size_local, function)?",
        "temporary overlap snapshot allocation",
    );
    let reads = positions(body, "emit_typed_array_or_object_index_read_from_locals(");
    let writes = positions(body, "emit_typed_array_element_write_from_locals(");
    let back_edges = positions(body, "Instruction::Br(0)");

    assert_eq!(source_witnesses.len(), 3);
    assert_eq!(reads.len(), 2);
    assert_eq!(writes.len(), 2);
    assert_eq!(back_edges.len(), 3);
    assert!(
        source_witnesses[2] < allocation
            && allocation < reads[0]
            && reads[0] < back_edges[0]
            && back_edges[0] < writes[0]
            && writes[0] < back_edges[1]
            && back_edges[1] < reads[1]
            && reads[1] < writes[1]
    );
}

#[test]
fn focused_cli_fixture_pins_set_witness_boundaries() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_revalidates_typedarray_set_buffer_witnesses()")
        .expect("missing focused TypedArray.prototype.set witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray.prototype.set witness CLI test")
        .0;

    assert!(test.contains("wasm_typedarray_set_buffer_witness.js"));
    assert!(test.contains("boolean(true)"));
    for marker in [
        "entry detach skips offset coercion",
        "post-offset growth uses refreshed length",
        "post-offset shrink uses refreshed length",
        "post-offset detachment",
        "post-offset fixed out-of-bounds",
        "detached TypedArray source",
        "out-of-bounds TypedArray source",
        "odd-byte source length floor",
        "borrowed set typed source exceeds target",
        "borrowed set typed source exceeds target suffix",
        "borrowed set array-like source exceeds target",
        "borrowed set array-like source exceeds target suffix",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing TypedArray.prototype.set CLI control: {marker}"
        );
    }
}
