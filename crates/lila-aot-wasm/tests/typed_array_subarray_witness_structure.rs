const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_subarray_buffer_witness.js");

const PRIVATE_STATE_WIRING: &str = r#"
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    buffer_payload_local,
                    byte_offset_local,
                    stored_byte_length_local,
                    bytes_per_element_local,
                    function,
                );
"#;

const VIEW_WIRING: &str = r#"
                let typed_array_view = TypedArrayViewLocals::new(
                    receiver_payload_local,
                    buffer_payload_local,
                    byte_offset_local,
                    stored_byte_length_local,
                    bytes_per_element_local,
                );
"#;

const WITNESS_WIRING: &str = r#"
                self.emit_typed_array_witness(
                    &typed_array_view,
                    TypedArrayWitnessUse::ArrayLikeLengthSnapshot { length_local },
                    function,
                )?;
"#;

const TRACKING_RESULT_ARITY: &str = r#"
                function.instruction(&Instruction::LocalGet(length_tracking_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(end_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::LocalSet(argc_local));
                function.instruction(&Instruction::End);
"#;

fn subarray_arm() -> &'static str {
    STANDARD_SOURCE
        .split_once("StandardBuiltinId::TypedArrayPrototypeSubarray => {")
        .expect("missing TypedArray.prototype.subarray builtin arm")
        .1
        .split_once("StandardBuiltinId::DateNow => {")
        .expect("missing TypedArray.prototype.subarray builtin boundary")
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_normalized_position(body: &str, snippet: &str, label: &str) -> usize {
    let snippet = without_whitespace(snippet);
    assert_eq!(
        body.matches(snippet.as_str()).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(snippet.as_str())
        .unwrap_or_else(|| panic!("missing normalized sentinel: {label}"))
}

#[test]
fn subarray_uses_one_non_throwing_length_witness_before_argument_coercion() {
    let arm = subarray_arm();

    for (needle, expected, role) in [
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable view"),
        ("emit_typed_array_witness(", 1, "buffer witness"),
        (
            "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
            1,
            "non-throwing length projection",
        ),
        (
            "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
            1,
            "result arity metadata load",
        ),
        (
            "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
            2,
            "source and result element-kind loads",
        ),
    ] {
        assert_eq!(
            arm.matches(needle).count(),
            expected,
            "TypedArray.prototype.subarray must have exactly {expected} {role}"
        );
    }

    for forbidden in [
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "TypedArrayWitnessUse::ValidatedMethodEntry",
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "TypedArrayWitnessUse::Accessor",
        "Instruction::I64DivU",
        "Instruction::LocalSet(length_local)",
        "Instruction::LocalSet(buffer_payload_local)",
        "Instruction::LocalSet(byte_offset_local)",
        "Instruction::LocalSet(stored_byte_length_local)",
        "Instruction::LocalSet(bytes_per_element_local)",
    ] {
        assert!(
            !arm.contains(forbidden),
            "TypedArray.prototype.subarray must not bypass or overwrite its witness through {forbidden}"
        );
    }

    let normalized = without_whitespace(arm);
    let brand_error = normalized
        .find("TypedArray.prototype.subarrayrequiresTypedArray")
        .expect("missing receiver-brand error");
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact immutable-view wiring");
    let source_element_kind = normalized
        .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,element_kind_local")
        .expect("missing source element-kind load");
    let tracking = normalized
        .find("HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,length_tracking_local")
        .expect("missing length-tracking metadata load");
    let witness = unique_normalized_position(
        &normalized,
        WITNESS_WIRING,
        "exact array-like length witness wiring",
    );
    let begin = normalized
        .find("emit_builtin_arg_to_locals(0,begin_payload_local,begin_tag_local,function)")
        .expect("missing begin argument load");
    let end = normalized
        .find("emit_builtin_arg_to_locals(1,end_payload_local,end_tag_local,function)")
        .expect("missing end argument load");
    let species = normalized
        .find("property_key_symbol_payload(\"Symbol.species\")")
        .expect("missing species lookup");
    let tracking_result_arity = unique_normalized_position(
        &normalized,
        TRACKING_RESULT_ARITY,
        "length-tracking result arity",
    );
    let construct = normalized
        .find("emit_function_handle_construct_with_argv(")
        .expect("missing species construction");
    let result_element_kind = normalized
        .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,result_element_kind_local")
        .expect("missing result element-kind load");

    assert!(
        brand_error < private_state
            && private_state < view
            && view < source_element_kind
            && source_element_kind < tracking
            && tracking < witness
            && witness < begin
            && begin < end
            && end < species
            && species < tracking_result_arity
            && tracking_result_arity < construct
            && construct < result_element_kind,
        "subarray must snapshot length before coercion, then preserve species construction and result-kind validation order"
    );

    let release_order = without_whitespace(
        r#"
                self.release_temp_local(length_local);
                self.release_temp_local(length_tracking_local);
                self.release_temp_local(element_kind_local);
                self.release_temp_local(bytes_per_element_local);
                self.release_temp_local(stored_byte_length_local);
                self.release_temp_local(byte_offset_local);
                self.release_temp_local(buffer_tag_local);
                self.release_temp_local(buffer_payload_local);
                self.release_temp_local(typed_array_brand_local);
"#,
    );
    assert_eq!(
        normalized.matches(&release_order).count(),
        1,
        "subarray view locals must retain reverse-order release"
    );
}

#[test]
fn focused_cli_fixture_pins_subarray_snapshot_result_shape_and_error_realm() {
    let test_body = CLI_TESTS
        .split_once("fn run_wasm_backend_subarray_uses_non_throwing_typed_array_buffer_witness()")
        .expect("missing focused TypedArray.prototype.subarray CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray.prototype.subarray CLI test")
        .0;
    assert!(test_body.contains("wasm_typedarray_subarray_buffer_witness.js"));
    assert!(test_body.contains("number(967"));

    for marker in [
        "fixed result out of bounds",
        "fixed result regrown",
        "tracking odd-byte shrink floor",
        "tracking odd-byte growth floor",
        "explicit end creates fixed result",
        "begin,end,species",
        "out-of-bounds zero length snapshot",
        "__lilaDetachArrayBuffer(detachedBuffer)",
        "detached coercion order",
        "custom detached species reached",
        "entry constructor owns detached error",
        "BigInt element kind",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}

#[test]
fn typed_array_prototype_installs_the_witnessed_subarray_builtin() {
    let installation = BOOTSTRAP_SOURCE
        .split_once("let subarray_meta = self")
        .expect("missing TypedArray.prototype.subarray installation")
        .1
        .split_once("let slice_meta = self")
        .expect("missing boundary after TypedArray.prototype.subarray installation")
        .0;

    assert_eq!(
        installation
            .matches("StandardBuiltinId::TypedArrayPrototypeSubarray.function_id()")
            .count(),
        1,
        "the public subarray property must resolve to the witnessed builtin ID"
    );
    assert_eq!(
        installation
            .matches("typed_array_prototype_local,\n            \"subarray\",\n            subarray_meta")
            .count(),
        1,
        "the witnessed builtin must remain installed as TypedArray.prototype.subarray"
    );
}
