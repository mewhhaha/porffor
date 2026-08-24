const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_concat_typed_array_buffer_witness.js");

const PRIVATE_STATE_WIRING: &str = r#"
        self.emit_load_typed_array_private_state(
            item_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
"#;

const VIEW_WIRING: &str = r#"
        let typed_array_view = TypedArrayViewLocals::new(
            item_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
"#;

const WITNESS_WIRING: &str = r#"
        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::IntegerIndexedProperty {
                index_local,
                result_local,
            },
            function,
        )?;
"#;

fn body_between(start: &str, end: &str) -> &'static str {
    ARRAY_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing Array builtin owner: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing Array builtin boundary: {end}"))
        .0
}

fn predicate_body() -> &'static str {
    body_between(
        "pub(crate) fn emit_concat_typed_array_has_index_i32(",
        "pub(crate) fn compile_array_prototype_concat_builtin(",
    )
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
fn concat_typed_array_has_index_uses_one_non_throwing_property_witness() {
    let body = predicate_body();

    for (needle, expected, role) in [
        ("reserve_temp_local()", 4, "private-view temporary"),
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable view"),
        ("emit_typed_array_witness(", 1, "buffer witness"),
        (
            "TypedArrayWitnessUse::IntegerIndexedProperty",
            1,
            "integer-indexed projection",
        ),
        (
            "Instruction::LocalSet(result_local)",
            1,
            "absent-result initialization",
        ),
        (
            "Instruction::LocalSet(typed_array_like_local)",
            2,
            "receiver classification write",
        ),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            expected,
            "concat TypedArray HasProperty must have exactly {expected} {role}"
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
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64Mul",
        "Instruction::I64DivU",
        "emit_throw_runtime_error(",
        "emit_throw_current_function_realm_type_error(",
        "key_local",
        "present_local",
        "slot_payload_local",
        "slot_tag_local",
    ] {
        assert!(
            !body.contains(forbidden),
            "concat TypedArray HasProperty must not bypass its witness through {forbidden}"
        );
    }

    let normalized = without_whitespace(body);
    let absent_result = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        "#,
        "absent-result initialization",
    );
    let ordinary_receiver = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));
        "#,
        "non-TypedArray classification",
    );
    let brand = normalized
        .find("emit_is_typed_array_i32(item_payload_local,item_tag_local,function)")
        .expect("missing TypedArray brand predicate");
    let typed_receiver = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));
        "#,
        "TypedArray classification",
    );
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact view wiring");
    let witness = unique_normalized_position(
        &normalized,
        WITNESS_WIRING,
        "exact integer-indexed witness wiring",
    );
    assert!(
        absent_result < ordinary_receiver
            && ordinary_receiver < brand
            && brand < typed_receiver
            && typed_receiver < private_state
            && private_state < view
            && view < witness,
        "concat HasProperty must initialize absent/fallback results before observing one branded TypedArray view"
    );

    let releases: Vec<_> = body
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("self.release_temp_local(")?
                .strip_suffix(");")
        })
        .collect();
    assert_eq!(
        releases,
        [
            "bytes_per_element_local",
            "stored_byte_length_local",
            "byte_offset_local",
            "buffer_payload_local",
        ],
        "concat HasProperty must release its immutable-view locals in reverse order"
    );
}

#[test]
fn concat_and_array_slice_remain_the_only_predicate_consumers() {
    assert_eq!(
        ARRAY_SOURCE
            .matches("emit_concat_typed_array_has_index_i32(")
            .count(),
        3,
        "the predicate must have one definition and its two established consumers"
    );

    let concat = body_between(
        "pub(crate) fn compile_array_prototype_concat_builtin(",
        "pub(crate) fn compile_array_prototype_flat_map_builtin(",
    );
    let slice = body_between(
        "pub(crate) fn compile_array_prototype_slice_builtin(",
        "pub(crate) fn compile_array_prototype_splice_builtin(",
    );
    assert_eq!(
        concat
            .matches("emit_concat_typed_array_has_index_i32(")
            .count(),
        1,
        "Array.prototype.concat must retain its TypedArray HasProperty consumer"
    );
    assert_eq!(
        slice
            .matches("emit_concat_typed_array_has_index_i32(")
            .count(),
        1,
        "Array.prototype.slice must retain its shared TypedArray HasProperty consumer"
    );
}

#[test]
fn focused_cli_fixture_pins_concat_typed_array_property_presence() {
    let test_body = CLI_TESTS
        .split_once("fn run_wasm_backend_checks_concat_typedarray_indices_through_buffer_witness()")
        .expect("missing focused concat TypedArray witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused concat TypedArray witness CLI test")
        .0;
    assert!(test_body.contains("wasm_array_concat_typed_array_buffer_witness.js"));
    assert!(test_body.contains("number(944"));

    for marker in [
        "Object.defineProperty(view, \"length\"",
        "view[Symbol.isConcatSpreadable] = true",
        "__lilaDetachArrayBuffer(detachedBuffer)",
        "detached",
        "fixed in bounds",
        "fixed out of bounds",
        "fixed regrown",
        "tracking partial shrink",
        "tracking partial growth",
        "tracking offset out of bounds",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
