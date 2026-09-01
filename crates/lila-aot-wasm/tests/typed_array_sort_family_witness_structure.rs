const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const COMPAREFN_WIRING: &str = r#"
    self.emit_builtin_arg_to_locals(0, compare_payload_local, compare_tag_local, function);
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(has_compare_local));
    function.instruction(&Instruction::LocalGet(compare_tag_local));
    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_is_callable_i32(compare_tag_local, compare_payload_local, function)?;
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::LocalSet(has_compare_local));
    function.instruction(&Instruction::Else);
    self.emit_throw_current_function_realm_type_error(
        "value is not callable",
        self.result_local,
        self.result_tag_local,
        function,
    )?;
    self.emit_return_current_completion(function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
"#;

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

const ELEMENT_KIND_WIRING: &str = r#"
    self.load_i64_to_local_from_offset(
        receiver_payload_local,
        HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        receiver_element_kind_local,
        function,
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

fn assert_comparefn_then_validated_method_entry(
    label: &str,
    receiver_error: &str,
    body: &str,
) -> usize {
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
    assert_eq!(
        body.matches("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET").count(),
        1,
        "{label} must retain one separate element-kind load"
    );
    assert_eq!(
        body.matches("emit_typed_array_stable_sort(").count(),
        1,
        "{label} must delegate exactly once to the shared stable sorter"
    );

    assert!(!body.contains("emit_validate_typed_array_current_byte_length("));
    assert!(!body.contains("emit_throw_runtime_error("));
    assert!(!body.contains("TYPE_ERROR_NAME"));
    assert!(
        !body.contains("Instruction::I64DivU"),
        "{label} must consume witness length without byte division"
    );
    assert_eq!(
        body.matches("Instruction::LocalSet(receiver_length_local)")
            .count(),
        0,
        "{label} must not overwrite the witness-produced element length"
    );
    for direct_buffer_observation in [
        "emit_load_array_buffer_byte_length",
        "emit_load_array_buffer_data",
        "HEAP_ARRAY_BUFFER_",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
    ] {
        assert!(
            !body.contains(direct_buffer_observation),
            "{label} must not bypass its witness through {direct_buffer_observation}"
        );
    }
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

    let comparefn = unique_normalized_position(
        &normalized_body,
        COMPAREFN_WIRING,
        &format!("{label} exact comparator-admissibility wiring"),
    );
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
    let element_kind = unique_normalized_position(
        &normalized_body,
        ELEMENT_KIND_WIRING,
        &format!("{label} exact element-kind wiring"),
    );
    let witness = unique_normalized_position(
        &normalized_body,
        WITNESS_WIRING,
        &format!("{label} exact validated-witness wiring"),
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
    let brand = unique_normalized_position(
        &normalized_body,
        &guarded_private_state,
        &format!("{label} completed brand guard before private-state use"),
    );

    assert!(
        comparefn < brand
            && brand < private_state
            && private_state < view
            && view < element_kind
            && element_kind < witness,
        "{label} must validate comparefn, reject the wrong brand, load one view, retain element kind, then consume one buffer witness"
    );

    witness
}

#[test]
fn typed_array_sort_family_uses_one_validated_method_entry_witness() {
    let sort = bounded(
        STANDARD_SOURCE,
        "fn compile_typed_array_prototype_sort_builtin(",
        "fn emit_typed_array_stable_sort(",
    );
    let to_sorted = bounded(
        STANDARD_SOURCE,
        "fn compile_typed_array_prototype_to_sorted_builtin(",
        "fn compile_typed_array_prototype_with_builtin(",
    );

    assert_eq!(sort.matches("receiver_length_local").count(), 4);
    assert_eq!(sort.matches("receiver_element_kind_local").count(), 4);
    assert_eq!(to_sorted.matches("receiver_length_local").count(), 6);
    assert_eq!(to_sorted.matches("receiver_element_kind_local").count(), 5);
    assert_eq!(to_sorted.matches("constructor_payload_local").count(), 7);
    assert_eq!(to_sorted.matches("result_payload_local").count(), 6);

    let sort_witness = assert_comparefn_then_validated_method_entry(
        "sort",
        "TypedArray.prototype.sort requires TypedArray",
        sort,
    );
    let normalized_sort = without_whitespace(sort);
    let sort_call = unique_normalized_position(
        &normalized_sort,
        r#"
        self.emit_typed_array_stable_sort(
            receiver_payload_local,
            receiver_tag_local,
            receiver_length_local,
            receiver_element_kind_local,
            compare_payload_local,
            compare_tag_local,
            has_compare_local,
            function,
        )?;
        "#,
        "sort receiver-target stable-sort wiring",
    );
    let sort_result = unique_normalized_position(
        &normalized_sort,
        r#"
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        "#,
        "sort receiver-result wiring",
    );
    assert!(
        sort_witness < sort_call && sort_call < sort_result,
        "sort must validate once, sort the captured receiver range in place, then return that receiver"
    );
    assert_eq!(
        sort.matches("typed_array_constructor_bytes_per_element_entries()")
            .count(),
        0,
        "sort must not allocate a replacement TypedArray"
    );

    let to_sorted_witness = assert_comparefn_then_validated_method_entry(
        "toSorted",
        "TypedArray.prototype.toSorted requires TypedArray",
        to_sorted,
    );
    assert_eq!(
        to_sorted
            .matches("typed_array_constructor_bytes_per_element_entries()")
            .count(),
        1,
        "toSorted must retain intrinsic same-kind constructor selection"
    );
    assert_eq!(
        to_sorted
            .matches("emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        1,
        "toSorted must retain one source read in its emitted copy loop"
    );
    assert_eq!(
        to_sorted
            .matches("emit_typed_array_element_write_from_locals(")
            .count(),
        1,
        "toSorted must retain one result write in its emitted copy loop"
    );

    let normalized_to_sorted = without_whitespace(to_sorted);
    let constructor_selection = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        for (constructor, _) in typed_array_constructor_bytes_per_element_entries() {
            let constructor_global_index = standard_builtin_constructor_global_index(constructor)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing typed array constructor global `{}`",
                        constructor.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::LocalGet(receiver_element_kind_local));
            function.instruction(&Instruction::I64Const(
                typed_array_element_kind(constructor) as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(constructor_payload_local));
            function.instruction(&Instruction::End);
        }
        "#,
        "toSorted receiver-kind constructor selection",
    );
    let allocation = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        self.emit_function_handle_construct_with_argv(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        "#,
        "toSorted same-kind result allocation and capture",
    );
    assert_eq!(
        to_sorted
            .matches("Instruction::LocalSet(result_payload_local)")
            .count(),
        1,
        "toSorted must capture exactly one distinct result payload"
    );
    assert_eq!(
        to_sorted
            .matches("Instruction::LocalSet(result_tag_local)")
            .count(),
        1,
        "toSorted must capture exactly one distinct result tag"
    );
    assert_eq!(
        to_sorted
            .matches("Instruction::LocalSet(constructor_payload_local)")
            .count(),
        2,
        "toSorted must initialize and select its constructor without a later override"
    );
    let source_read = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            copy_index_local,
            copy_payload_local,
            copy_tag_local,
            function,
        )?;
        "#,
        "toSorted source-read wiring",
    );
    let result_write = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        self.emit_typed_array_element_write_from_locals(
            result_payload_local,
            copy_index_local,
            copy_payload_local,
            copy_tag_local,
            function,
        )?;
        "#,
        "toSorted result-write wiring",
    );
    let result_sort = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        self.emit_typed_array_stable_sort(
            result_payload_local,
            result_tag_local,
            receiver_length_local,
            receiver_element_kind_local,
            compare_payload_local,
            compare_tag_local,
            has_compare_local,
            function,
        )?;
        "#,
        "toSorted result-target stable-sort wiring",
    );
    let result_publish = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        "#,
        "toSorted distinct-result wiring",
    );

    let complete_copy_before_sort = unique_normalized_position(
        &normalized_to_sorted,
        r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(receiver_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            copy_index_local,
            copy_payload_local,
            copy_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_typed_array_element_write_from_locals(
            result_payload_local,
            copy_index_local,
            copy_payload_local,
            copy_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_typed_array_stable_sort(
            result_payload_local,
            result_tag_local,
            receiver_length_local,
            receiver_element_kind_local,
            compare_payload_local,
            compare_tag_local,
            has_compare_local,
            function,
        )?;
        "#,
        "toSorted full captured-range copy before result sort",
    );

    assert!(
        to_sorted_witness < constructor_selection
            && constructor_selection < allocation
            && allocation < complete_copy_before_sort
            && complete_copy_before_sort < source_read
            && source_read < result_write
            && result_write < result_sort
            && result_sort < result_publish,
        "toSorted must validate, allocate the same-kind result, copy the complete source, sort the result, then return it"
    );
}
