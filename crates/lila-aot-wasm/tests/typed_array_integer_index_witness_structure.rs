const BINARY_DATA_SOURCE: &str = include_str!("../src/builtins/binary_data.rs");

const PRIVATE_STATE_WIRING: &str = r#"
        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
"#;

const VIEW_WIRING: &str = r#"
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
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

fn valid_integer_index_body() -> &'static str {
    BINARY_DATA_SOURCE
        .split_once("pub(crate) fn emit_typed_array_valid_integer_index_i32(")
        .expect("missing TypedArray integer-index validity emitter")
        .1
        .split_once("pub(crate) fn emit_validate_typed_array_current_byte_length(")
        .expect("missing validator after TypedArray integer-index validity emitter")
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

fn local_sequence<'a>(body: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(prefix)?.strip_suffix(suffix))
        .collect()
}

#[test]
fn integer_index_validity_uses_one_live_property_witness() {
    let body = valid_integer_index_body();

    assert!(
        !body.contains("typed_array_tag_local"),
        "integer-index validity must not claim an unused TypedArray tag input"
    );

    for (needle, expected, label) in [
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable view"),
        ("emit_typed_array_witness(", 1, "live buffer witness"),
        (
            "TypedArrayWitnessUse::IntegerIndexedProperty",
            1,
            "integer-indexed projection",
        ),
        (
            "Instruction::LocalSet(index_local)",
            1,
            "integer index output",
        ),
        (
            "Instruction::LocalSet(result_local)",
            1,
            "absent-result initialization",
        ),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            expected,
            "integer-index validity must have exactly {expected} {label}"
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
        "Instruction::I64DivU",
        "emit_throw_runtime_error(",
        "emit_throw_current_function_realm_type_error(",
    ] {
        assert!(
            !body.contains(forbidden),
            "integer-index validity must not bypass its non-throwing witness through {forbidden}"
        );
    }

    let normalized = without_whitespace(body);
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact immutable-view wiring");
    let witness =
        unique_normalized_position(&normalized, WITNESS_WIRING, "exact property-witness wiring");
    assert!(
        private_state < view && view < witness,
        "integer-index validity must load one view before consuming its live witness"
    );
}

#[test]
fn numeric_index_classification_precedes_buffer_observation() {
    let body = valid_integer_index_body();
    let normalized = without_whitespace(body);

    assert_eq!(
        body.matches("Instruction::BrIf(0)").count(),
        4,
        "the four invalid numeric-index exits must remain inside the shared block"
    );

    let initialization = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        "#,
        "absent-result initialization",
    );
    let non_numeric = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        "#,
        "non-numeric sentinel exit",
    );
    let fractional = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::BrIf(0));
        "#,
        "fractional-index exit",
    );
    let negative = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::BrIf(0));
        "#,
        "negative-index exit",
    );
    let too_large = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            18_446_744_073_709_551_616.0,
        )));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::BrIf(0));
        "#,
        "unrepresentable-index exit",
    );
    let conversion = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        "#,
        "integer-index conversion",
    );
    let witness = unique_normalized_position(&normalized, WITNESS_WIRING, "live property witness");

    assert!(
        initialization < non_numeric
            && non_numeric < fractional
            && fractional < negative
            && negative < too_large
            && too_large < conversion
            && conversion < witness,
        "numeric-index classification must complete before any backing-store observation"
    );

    let reservations = local_sequence(body, "let ", " = self.reserve_temp_local();");
    let releases = local_sequence(body, "self.release_temp_local(", ");");
    let mut expected_releases = reservations.clone();
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "integer-index validity must release its view locals in reverse reservation order"
    );
}
