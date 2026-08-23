const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const PRIVATE_STATE_WIRING: &str = r#"
    self.emit_load_typed_array_private_state(
        receiver_payload_local,
        receiver_buffer_local,
        receiver_byte_offset_local,
        receiver_byte_length_local,
        receiver_bytes_per_element_local,
        function,
    );
"#;

const VIEW_WIRING: &str = r#"
    let receiver_view = TypedArrayViewLocals::new(
        receiver_payload_local,
        receiver_buffer_local,
        receiver_byte_offset_local,
        receiver_byte_length_local,
        receiver_bytes_per_element_local,
    );
"#;

const WITNESS_WIRING: &str = r#"
    self.emit_typed_array_witness(
        &receiver_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: receiver_length_local,
        },
        function,
    )?;
"#;

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
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

fn assert_validated_method_entry_witness(label: &str, receiver_error: &str, body: &str) {
    let normalized_body = without_whitespace(body);

    assert_eq!(
        body.matches("emit_load_typed_array_private_state(").count(),
        1,
        "{label} must load one immutable private view record"
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
    assert!(!body.contains("emit_throw_runtime_error("));
    assert!(!body.contains("TYPE_ERROR_NAME"));
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

    let private_state = unique_normalized_position(
        &normalized_body,
        PRIVATE_STATE_WIRING,
        &format!("{label} exact private-state wiring"),
    );
    let view = unique_normalized_position(
        &normalized_body,
        VIEW_WIRING,
        &format!("{label} exact immutable-view wiring"),
    );
    let witness = unique_normalized_position(
        &normalized_body,
        WITNESS_WIRING,
        &format!("{label} exact validated-witness wiring"),
    );
    assert!(
        private_state < view && view < witness,
        "{label} must load private state, construct its view, then validate its buffer"
    );

    let brand_throw = format!(
        r#"
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "{receiver_error}",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        "#
    );
    let guarded_private_state = format!("{brand_throw}{PRIVATE_STATE_WIRING}");
    assert_eq!(
        normalized_body
            .matches(without_whitespace(&guarded_private_state).as_str())
            .count(),
        1,
        "{label} must throw and return for a failed brand check before private-state use"
    );
}

#[test]
fn typed_array_reverse_family_uses_one_validated_method_entry_witness() {
    let reverse = bounded(
        STANDARD_SOURCE,
        "fn compile_typed_array_prototype_reverse_builtin(",
        "fn compile_typed_array_prototype_copy_within_builtin(",
    );
    let to_reversed = bounded(
        STANDARD_SOURCE,
        "fn compile_typed_array_prototype_to_reversed_builtin(",
        "fn compile_typed_array_prototype_sort_builtin(",
    );

    assert_validated_method_entry_witness(
        "reverse",
        "TypedArray.prototype.reverse requires TypedArray",
        reverse,
    );
    assert_validated_method_entry_witness(
        "toReversed",
        "TypedArray.prototype.toReversed requires TypedArray",
        to_reversed,
    );

    assert_eq!(
        reverse.matches("Instruction::I64DivU").count(),
        1,
        "reverse's only unsigned division must compute the midpoint"
    );
    assert_eq!(
        to_reversed.matches("Instruction::I64DivU").count(),
        0,
        "toReversed must not derive element length through byte division"
    );
    let normalized_reverse = without_whitespace(reverse);
    unique_normalized_position(
        &normalized_reverse,
        r#"
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(middle_local));
        "#,
        "reverse midpoint division wiring",
    );

    assert_eq!(
        reverse
            .matches("emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        2,
        "reverse must retain its two indexed reads per emitted loop body"
    );
    assert_eq!(
        reverse
            .matches("emit_typed_array_element_write_from_locals(")
            .count(),
        2,
        "reverse must retain its two indexed writes per emitted loop body"
    );
    assert_eq!(
        reverse
            .matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET")
            .count(),
        0,
        "reverse must remain element-kind agnostic"
    );

    let lower_read = unique_normalized_position(
        &normalized_reverse,
        r#"
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            lower_index_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        "#,
        "reverse lower-index read wiring",
    );
    let upper_read = unique_normalized_position(
        &normalized_reverse,
        r#"
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            upper_index_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        "#,
        "reverse upper-index read wiring",
    );
    let lower_write = unique_normalized_position(
        &normalized_reverse,
        r#"
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            lower_index_local,
            upper_payload_local,
            upper_tag_local,
            function,
        )?;
        "#,
        "reverse lower-index write wiring",
    );
    let upper_write = unique_normalized_position(
        &normalized_reverse,
        r#"
        self.emit_typed_array_element_write_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            upper_index_local,
            lower_payload_local,
            lower_tag_local,
            function,
        )?;
        "#,
        "reverse upper-index write wiring",
    );
    assert!(
        lower_read < upper_read && upper_read < lower_write && lower_write < upper_write,
        "reverse must read both source values before writing the swapped pair"
    );

    assert_eq!(
        to_reversed
            .matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET")
            .count(),
        1,
        "toReversed must retain one element-kind load for same-kind allocation"
    );
    assert_eq!(
        to_reversed
            .matches("typed_array_constructor_bytes_per_element_entries()")
            .count(),
        1,
        "toReversed must retain intrinsic same-kind constructor selection"
    );
    assert_eq!(
        to_reversed
            .matches("emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        1,
        "toReversed must retain its reverse-order source read"
    );
    assert_eq!(
        to_reversed
            .matches("emit_typed_array_element_write_from_locals(")
            .count(),
        1,
        "toReversed must retain its result write"
    );

    let to_reversed = without_whitespace(to_reversed);
    let source_read = unique_normalized_position(
        &to_reversed,
        r#"
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            from_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        "#,
        "toReversed source read wiring",
    );
    let result_write = unique_normalized_position(
        &to_reversed,
        r#"
        self.emit_typed_array_element_write_from_locals(
            result_payload_local,
            result_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        "#,
        "toReversed result write wiring",
    );
    assert!(
        source_read < result_write,
        "toReversed must read the reverse source index before writing the result index"
    );
}
