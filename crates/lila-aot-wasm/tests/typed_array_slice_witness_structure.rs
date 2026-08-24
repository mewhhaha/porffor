const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const PRIVATE_STATE_WIRING: &str = r#"
    self.emit_load_typed_array_private_state(
        receiver_payload_local,
        source_buffer_payload_local,
        source_byte_offset_local,
        source_stored_byte_length_local,
        source_bytes_per_element_local,
        function,
    );
"#;

const VIEW_WIRING: &str = r#"
    let source_view = TypedArrayViewLocals::new(
        receiver_payload_local,
        source_buffer_payload_local,
        source_byte_offset_local,
        source_stored_byte_length_local,
        source_bytes_per_element_local,
    );
"#;

const RECEIVER_BRAND_LOAD_WIRING: &str = r#"
    self.load_i64_to_local_from_offset(
        receiver_payload_local,
        HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
        typed_array_brand_local,
        function,
    );
"#;

const SOURCE_ELEMENT_KIND_WIRING: &str = r#"
    self.load_i64_to_local_from_offset(
        receiver_payload_local,
        HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        source_element_kind_local,
        function,
    );
"#;

const ENTRY_WITNESS_WIRING: &str = r#"
    self.emit_typed_array_witness(
        &source_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: source_length_local,
        },
        function,
    )?;
"#;

const POSITIVE_COUNT_WITNESS_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Else);
    self.emit_typed_array_witness(
        &source_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: current_source_length_local,
        },
        function,
    )?;
"#;

const START_WIRING: &str = r#"
    self.emit_builtin_arg_to_locals(0, start_payload_local, start_tag_local, function);
    self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
    function.instruction(&Instruction::LocalSet(start_payload_local));
    self.emit_return_current_completion_if_throw(function);
    self.emit_to_integer_or_infinity_number_payload_from_number_payload(
        start_payload_local,
        start_payload_local,
        function,
    );
    self.emit_array_slice_clamped_index(
        start_payload_local,
        source_length_local,
        start_index_local,
        function,
    );
"#;

const OPTIONAL_END_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(source_length_local));
    function.instruction(&Instruction::LocalSet(end_index_local));
    function.instruction(&Instruction::LocalGet(self.argc_param_local()));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_builtin_arg_to_locals(1, end_payload_local, end_tag_local, function);
    function.instruction(&Instruction::LocalGet(end_tag_local));
    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_value_to_number_payload(end_tag_local, end_payload_local, function)?;
    function.instruction(&Instruction::LocalSet(end_payload_local));
    self.emit_return_current_completion_if_throw(function);
    self.emit_to_integer_or_infinity_number_payload_from_number_payload(
        end_payload_local,
        end_payload_local,
        function,
    );
    self.emit_array_slice_clamped_index(
        end_payload_local,
        source_length_local,
        end_index_local,
        function,
    );
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
"#;

const INITIAL_COUNT_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::End);
"#;

const TARGET_CREATION_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::F64ConvertI64U);
    function.instruction(&Instruction::I64ReinterpretF64);
    function.instruction(&Instruction::LocalSet(count_payload_local));
    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
    function.instruction(&Instruction::LocalSet(number_tag_local));
    self.emit_pre_evaluated_arg_vector(
        &[(count_payload_local, number_tag_local)],
        argc_local,
        argv_local,
        function,
    )?;
    self.emit_function_or_proxy_construct_with_argv(
        constructor_payload_local,
        constructor_tag_local,
        constructor_payload_local,
        constructor_tag_local,
        argc_local,
        argv_local,
        target_payload_local,
        target_tag_local,
        function,
    )?;
    self.emit_return_current_completion_if_throw(function);
    self.set_completion_kind(CompletionKind::Normal, function);
    self.emit_validate_typed_array_from_constructed_target(
        target_payload_local,
        target_tag_local,
        count_payload_local,
        function,
    )?;
"#;

const CURRENT_LENGTH_CAP_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::LocalGet(current_source_length_local));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(current_source_length_local));
    function.instruction(&Instruction::LocalSet(end_index_local));
    function.instruction(&Instruction::End);
"#;

const CONTENT_TYPE_WIRING: &str = r#"
    self.load_i64_to_local_from_offset(
        target_payload_local,
        HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
        target_element_kind_local,
        function,
    );
    function.instruction(&Instruction::LocalGet(source_element_kind_local));
    function.instruction(&Instruction::I64Const(10));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::LocalGet(target_element_kind_local));
    function.instruction(&Instruction::I64Const(10));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::I32Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_throw_current_function_realm_type_error(
        "TypedArray.prototype.slice species content type differs",
        self.result_local,
        self.result_tag_local,
        function,
    )?;
    self.emit_return_current_completion(function);
    function.instruction(&Instruction::End);
"#;

const COPIED_ELEMENT_COUNT_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(copied_element_count_local));
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(copied_element_count_local));
    function.instruction(&Instruction::End);
"#;

const COPY_ADDRESS_WIRING: &str = r#"
    self.emit_load_array_buffer_data(
        source_buffer_payload_local,
        source_buffer_pointer_local,
        function,
    );
    self.load_i64_to_local_from_offset(
        target_payload_local,
        HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
        target_buffer_payload_local,
        function,
    );
    self.emit_load_array_buffer_data(
        target_buffer_payload_local,
        target_buffer_pointer_local,
        function,
    );
    self.load_i64_to_local_from_offset(
        target_payload_local,
        HEAP_TYPED_ARRAY_BYTE_OFFSET,
        target_byte_offset_local,
        function,
    );
    function.instruction(&Instruction::LocalGet(source_buffer_pointer_local));
    function.instruction(&Instruction::LocalGet(source_byte_offset_local));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::LocalGet(source_bytes_per_element_local));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(source_address_local));
    function.instruction(&Instruction::LocalGet(target_buffer_pointer_local));
    function.instruction(&Instruction::LocalGet(target_byte_offset_local));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(target_address_local));
"#;

const SAME_TYPE_SELECTION_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(source_element_kind_local));
    function.instruction(&Instruction::LocalGet(target_element_kind_local));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
"#;

const BYTE_COPY_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(copied_element_count_local));
    function.instruction(&Instruction::LocalGet(source_bytes_per_element_local));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::LocalSet(copied_byte_count_local));
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(copy_index_local));
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::LocalGet(copied_byte_count_local));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));
    function.instruction(&Instruction::LocalGet(target_address_local));
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(source_address_local));
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Load8U(self.buffer_memarg8(0)));
    function.instruction(&Instruction::I32Store8(self.buffer_memarg8(0)));
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(copy_index_local));
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
"#;

const INDEXED_COPY_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::LocalSet(copy_index_local));
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::LocalGet(end_index_local));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));
    self.emit_typed_array_or_object_index_read_from_locals(
        receiver_payload_local,
        receiver_tag_local,
        copy_index_local,
        element_payload_local,
        element_tag_local,
        function,
    )?;
    function.instruction(&Instruction::LocalGet(copy_index_local));
    function.instruction(&Instruction::LocalGet(start_index_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(target_index_local));
    self.emit_typed_array_element_write_from_locals(
        target_payload_local,
        target_tag_local,
        target_index_local,
        element_payload_local,
        element_tag_local,
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
"#;

const RESULT_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(target_payload_local));
    function.instruction(&Instruction::LocalSet(self.result_local));
    function.instruction(&Instruction::LocalGet(target_tag_local));
    function.instruction(&Instruction::LocalSet(self.result_tag_local));
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

fn slice_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_typed_array_prototype_slice_builtin(",
        "pub(crate) fn compile_typed_array_prototype_map_builtin(",
    )
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn normalized_code(source: &str) -> String {
    source
        .lines()
        .flat_map(|line| line.split_once("//").map_or(line, |(code, _)| code).chars())
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

fn unique_position(body: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        body.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(needle)
        .unwrap_or_else(|| panic!("missing sentinel: {label}"))
}

fn unique_normalized_position(body: &str, snippet: &str, label: &str) -> usize {
    let body = without_whitespace(body);
    let snippet = without_whitespace(snippet);
    unique_position(&body, &snippet, label)
}

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle)
        .map(|(position, _)| position)
        .collect()
}

fn exact_identifier_mentions(source: &str, identifier: &str) -> usize {
    let is_boundary = |ch: Option<char>| match ch {
        Some(ch) => !ch.is_alphanumeric() && ch != '_',
        None => true,
    };

    source
        .match_indices(identifier)
        .filter(|(start, _)| {
            let end = *start + identifier.len();
            is_boundary(source[..*start].chars().next_back())
                && is_boundary(source[end..].chars().next())
        })
        .count()
}

fn matching_control_end(body: &str, opening_position: usize) -> usize {
    let mut events = Vec::new();
    for opener in [
        "function.instruction(&Instruction::If(",
        "function.instruction(&Instruction::Block(",
        "function.instruction(&Instruction::Loop(",
    ] {
        for (position, _) in body[opening_position..].match_indices(opener) {
            events.push((opening_position + position, 1_i32));
        }
    }
    for (position, _) in
        body[opening_position..].match_indices("function.instruction(&Instruction::End);")
    {
        events.push((opening_position + position, -1_i32));
    }
    events.sort_unstable_by_key(|(position, _)| *position);

    assert_eq!(
        events.first().map(|(position, _)| *position),
        Some(opening_position),
        "the requested structured-control opener must begin the scan"
    );
    let mut depth = 0_i32;
    for (position, delta) in events {
        depth += delta;
        assert!(
            depth >= 0,
            "structured-control depth must not underflow while finding the matching end"
        );
        if depth == 0 {
            return position;
        }
    }
    panic!("structured-control opener must have a matching end");
}

fn local_sequence<'a>(body: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(prefix)?.strip_suffix(suffix))
        .collect()
}

fn assert_temp_lifetime(body: &str) {
    let reservations = local_sequence(body, "let ", " = self.reserve_temp_local();");
    let releases = local_sequence(body, "self.release_temp_local(", ");");

    assert_eq!(
        body.matches("reserve_temp_local()").count(),
        reservations.len(),
        "slice reservations must keep the reviewed binding shape"
    );
    assert_eq!(
        body.matches("release_temp_local(").count(),
        releases.len(),
        "slice releases must keep the reviewed call shape"
    );

    let mut unique = reservations.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        reservations.len(),
        "slice must reserve every temporary exactly once"
    );

    let mut expected_releases = reservations;
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "slice must release temporaries in reverse reservation order"
    );
}

#[test]
fn slice_uses_one_immutable_source_view_and_two_validated_observations() {
    let body = slice_body();

    for (needle, expected, label) in [
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable source view"),
        ("emit_typed_array_witness(", 2, "source buffer witness"),
        (
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            2,
            "validated source observation",
        ),
        (
            "source_view",
            3,
            "one source-view producer and two consumers",
        ),
        ("emit_load_array_buffer_data(", 2, "fresh copy-data load"),
        (
            "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
            2,
            "source and target element-kind load",
        ),
        (
            "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
            1,
            "target viewed-buffer load",
        ),
        ("HEAP_TYPED_ARRAY_BYTE_OFFSET", 1, "target byte-offset load"),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            expected,
            "slice must have exactly {expected} {label}"
        );
    }

    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
        "Instruction::LocalSet(source_length_local)",
        "Instruction::LocalSet(current_source_length_local)",
        "Instruction::LocalSet(source_buffer_payload_local)",
        "Instruction::LocalSet(source_byte_offset_local)",
        "Instruction::LocalSet(source_stored_byte_length_local)",
        "Instruction::LocalSet(source_bytes_per_element_local)",
        "Instruction::LocalTee(count_local)",
        "Instruction::LocalTee(end_index_local)",
        "Instruction::LocalTee(copied_element_count_local)",
        "Instruction::LocalTee(copied_byte_count_local)",
        "Instruction::LocalTee(source_address_local)",
        "Instruction::LocalTee(target_address_local)",
        "load_i64_from_offset(",
        "Instruction::I32Load(",
        "Instruction::I32Load8S",
        "Instruction::I32Load16",
        "Instruction::I64Load",
        "Instruction::F32Load",
        "Instruction::F64Load",
        "Instruction::V128Load",
    ] {
        assert!(
            !body.contains(forbidden),
            "slice must not bypass or overwrite its source view through {forbidden}"
        );
    }

    let normalized = without_whitespace(body);
    assert_eq!(
        body.matches("load_i64_to_local_from_offset(").count(),
        5,
        "slice must retain exactly its brand, source/target element-kind and target copy-offset loads"
    );
    assert_eq!(
        normalized
            .matches("self.load_i64_to_local_from_offset(receiver_payload_local,")
            .count(),
        2,
        "slice may load only the receiver brand and immutable element kind directly"
    );
    assert_eq!(
        body.matches("self.this_payload_local").count(),
        1,
        "slice must bind the receiver payload once without a second alias source"
    );
    assert_eq!(
        body.matches("let receiver_").count(),
        2,
        "slice may declare only its receiver payload and tag bindings"
    );
    for numeric_prefix in ["-", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        assert!(
            !normalized.contains(&format!(
                "self.load_i64_to_local_from_offset(receiver_payload_local,{numeric_prefix}"
            )),
            "slice must not reconstruct receiver private state from numeric offsets"
        );
    }
    let brand_load = unique_normalized_position(
        body,
        RECEIVER_BRAND_LOAD_WIRING,
        "direct receiver-brand load",
    );
    let brand = unique_position(
        &normalized,
        &without_whitespace(
            r#"
            function.instruction(&Instruction::LocalGet(typed_array_brand_local));
            function.instruction(&Instruction::I64Const(
                OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
            ));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "TypedArray.prototype.slice requires a TypedArray",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            "#,
        ),
        "completed receiver-brand rejection",
    );
    let private_state = unique_normalized_position(body, PRIVATE_STATE_WIRING, "private state");
    let view = unique_normalized_position(body, VIEW_WIRING, "immutable source view");
    let source_element_kind = unique_normalized_position(
        body,
        SOURCE_ELEMENT_KIND_WIRING,
        "direct source element-kind load",
    );
    for (snippet, label) in [
        (
            r#"
            self.load_i64_to_local_from_offset(
                target_payload_local,
                HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
                target_element_kind_local,
                function,
            );
            "#,
            "target element-kind load",
        ),
        (
            r#"
            self.load_i64_to_local_from_offset(
                target_payload_local,
                HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
                target_buffer_payload_local,
                function,
            );
            "#,
            "target viewed-buffer load",
        ),
        (
            r#"
            self.load_i64_to_local_from_offset(
                target_payload_local,
                HEAP_TYPED_ARRAY_BYTE_OFFSET,
                target_byte_offset_local,
                function,
            );
            "#,
            "target byte-offset load",
        ),
    ] {
        unique_normalized_position(body, snippet, label);
    }
    let entry = unique_normalized_position(body, ENTRY_WITNESS_WIRING, "entry witness");
    let late = unique_normalized_position(
        body,
        POSITIVE_COUNT_WITNESS_WIRING,
        "conditional post-species witness",
    );
    assert!(
        brand_load < brand
            && brand < private_state
            && private_state < view
            && view < source_element_kind
            && source_element_kind < entry
            && entry < late,
        "slice must reject the receiver, capture one immutable view, load element kind and consume two fresh witnesses in order"
    );

    assert_eq!(
        normalized
            .matches("length_local:source_length_local")
            .count(),
        1,
        "only the entry witness may publish the captured source length"
    );
    assert_eq!(
        normalized
            .matches("length_local:current_source_length_local")
            .count(),
        1,
        "only the conditional witness may publish the current source length"
    );

    for (local, expected) in [
        ("source_buffer_payload_local", 5),
        ("source_byte_offset_local", 5),
        ("source_stored_byte_length_local", 4),
        ("source_bytes_per_element_local", 6),
        ("source_length_local", 6),
        ("current_source_length_local", 5),
        ("receiver_payload_local", 8),
        ("source_element_kind_local", 6),
    ] {
        assert_eq!(
            exact_identifier_mentions(body, local),
            expected,
            "{local} must retain its reviewed immutable producer/consumer census"
        );
    }
}

#[test]
fn slice_preserves_entry_coercion_species_and_conditional_revalidation_order() {
    let body = slice_body();
    let normalized = without_whitespace(body);

    assert_eq!(
        exact_identifier_mentions(body, "end_index_local"),
        11,
        "end_index_local must retain its reviewed normalization, cap, copy and lifetime census"
    );
    assert_eq!(
        normalized
            .matches("Instruction::LocalSet(end_index_local)")
            .count(),
        2,
        "only entry-length initialization and the exact current-length cap may directly write end_index_local"
    );
    assert_eq!(
        normalized
            .matches("Instruction::LocalTee(end_index_local)")
            .count(),
        0,
        "end_index_local must not admit a hidden LocalTee overwrite"
    );

    for (needle, expected, label) in [
        ("emit_object_read(", 2, "constructor/species property read"),
        (
            "emit_function_or_proxy_construct_with_argv(",
            1,
            "species target construction",
        ),
        (
            "emit_validate_typed_array_from_constructed_target(",
            1,
            "constructed-target validation",
        ),
        (
            "emit_typed_array_or_object_index_read_from_locals(",
            1,
            "different-type indexed read",
        ),
        (
            "emit_typed_array_element_write_from_locals(",
            1,
            "different-type indexed write",
        ),
        ("emit_load_array_buffer_data(", 2, "same-type data load"),
        ("Instruction::I32Load8U", 1, "same-type byte load"),
        ("Instruction::I32Store8", 1, "same-type byte store"),
        ("Instruction::I64Mul", 2, "copy address/count multiply"),
        ("Instruction::Loop(BlockType::Empty)", 2, "copy loop"),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            expected,
            "slice must have exactly {expected} {label}"
        );
    }

    let entry = unique_normalized_position(body, ENTRY_WITNESS_WIRING, "entry witness");
    let start = unique_normalized_position(body, START_WIRING, "start coercion and clamp");
    let end =
        unique_normalized_position(body, OPTIONAL_END_WIRING, "optional end coercion and clamp");
    let count = unique_normalized_position(body, INITIAL_COUNT_WIRING, "original count");
    let constructor_key = unique_position(
        &normalized,
        "function.instruction(&Instruction::I64Const(self.strings.payload(\"constructor\")));function.instruction(&Instruction::LocalSet(constructor_key_local));",
        "constructor property key",
    );
    let constructor_read = unique_position(
        &normalized,
        "self.emit_object_read(receiver_payload_local,receiver_tag_local,receiver_payload_local,receiver_tag_local,constructor_key_local,constructor_property_payload_local,constructor_property_tag_local,function,)?;",
        "constructor observation",
    );
    let species_key = unique_position(
        &normalized,
        "function.instruction(&Instruction::I64Const(self.strings.property_key_symbol_payload(\"Symbol.species\"),));function.instruction(&Instruction::LocalSet(constructor_key_local));",
        "Symbol.species property key",
    );
    let species_read = unique_position(
        &normalized,
        "self.emit_object_read(constructor_property_payload_local,constructor_property_tag_local,constructor_property_payload_local,constructor_property_tag_local,constructor_key_local,species_payload_local,species_tag_local,function,)?;",
        "species observation",
    );
    let construct = unique_position(
        &normalized,
        "self.emit_function_or_proxy_construct_with_argv(constructor_payload_local,constructor_tag_local,constructor_payload_local,constructor_tag_local,argc_local,argv_local,target_payload_local,target_tag_local,function,)?;",
        "species target construction",
    );
    let target_creation = unique_normalized_position(
        body,
        TARGET_CREATION_WIRING,
        "count-to-validated-target dataflow",
    );
    let content_type =
        unique_normalized_position(body, CONTENT_TYPE_WIRING, "content-type comparison");
    let late = unique_normalized_position(
        body,
        POSITIVE_COUNT_WITNESS_WIRING,
        "conditional post-species witness",
    );
    assert!(
        entry < start
            && start < end
            && end < count
            && count < constructor_key
            && constructor_key < constructor_read
            && constructor_read < species_key
            && species_key < species_read
            && species_read < target_creation
            && target_creation <= construct
            && construct < content_type
            && content_type < late,
        "slice must capture its entry length, coerce start/end, construct and validate the target, reject content mismatch, then conditionally revalidate the source"
    );

    assert_eq!(
        body.matches("emit_validate_typed_array_from_constructed_target(")
            .count(),
        1,
        "slice must retain one separate constructed-target validation"
    );

    let positive_branch = unique_position(
        &normalized,
        "function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::Else);",
        "zero-count skip and positive-count branch",
    );
    let positive_if = normalized[positive_branch..]
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .map(|position| positive_branch + position)
        .expect("the count guard must open a Wasm if");
    let positive_else = normalized[positive_if..]
        .find("function.instruction(&Instruction::Else);")
        .map(|position| positive_if + position)
        .expect("the count guard must retain an empty zero-count arm");
    let if_instruction = "function.instruction(&Instruction::If(BlockType::Empty));";
    assert_eq!(
        &normalized[positive_if + if_instruction.len()..positive_else],
        "",
        "the zero-count arm must contain no witness, data load, address setup or copy operation"
    );
    let positive_end = matching_control_end(&normalized, positive_if);
    let current_cap =
        unique_normalized_position(body, CURRENT_LENGTH_CAP_WIRING, "current-length end cap");
    let copied_count = unique_normalized_position(
        body,
        COPIED_ELEMENT_COUNT_WIRING,
        "late copied-element count",
    );
    let cap_wiring = without_whitespace(CURRENT_LENGTH_CAP_WIRING);
    let copied_count_wiring = without_whitespace(COPIED_ELEMENT_COUNT_WIRING);
    let cap_to_copied_count = format!("{cap_wiring}{copied_count_wiring}");
    let adjacent_cap = unique_position(
        &normalized,
        &cap_to_copied_count,
        "current-length cap immediately followed by copied-element count",
    );
    assert_eq!(
        adjacent_cap, current_cap,
        "the reviewed current-length cap must own the cap-to-count adjacency"
    );
    assert_eq!(
        copied_count,
        current_cap + cap_wiring.len(),
        "no post-cap end-index overwrite may fit before copied-count derivation"
    );
    let data = positions(&normalized, "self.emit_load_array_buffer_data(");
    let result = unique_normalized_position(body, RESULT_WIRING, "target result publication");
    assert_eq!(
        data.len(),
        2,
        "slice must reload source and target data once"
    );
    assert!(
        positive_if < positive_else
            && positive_else < late
            && late < current_cap
            && current_cap < copied_count
            && data
                .iter()
                .all(|position| copied_count < *position && *position < positive_end)
            && positive_end < result,
        "the late witness, copied-count calculation and copy setup must remain inside the positive-count arm, with result publication after it"
    );

    let current_length_reads = positions(
        &normalized,
        "function.instruction(&Instruction::LocalGet(current_source_length_local));",
    );
    assert_eq!(
        current_length_reads.len(),
        2,
        "the capped end calculation must read the current source length twice"
    );
    assert!(
        current_length_reads
            .iter()
            .all(|position| current_cap <= *position && *position < copied_count),
        "current source length may only cap the selected range after target validation"
    );
}

#[test]
fn slice_keeps_original_element_and_byte_counts_in_distinct_domains() {
    let body = slice_body();
    let normalized = without_whitespace(body);

    for (local, expected) in [
        ("count_local", 6),
        ("copied_element_count_local", 5),
        ("copied_byte_count_local", 4),
    ] {
        assert_eq!(
            exact_identifier_mentions(body, local),
            expected,
            "{local} must retain its reviewed producer/consumer census"
        );
    }

    let count_sets = positions(
        &normalized,
        "function.instruction(&Instruction::LocalSet(count_local));",
    );
    assert_eq!(
        count_sets.len(),
        2,
        "the original count must only receive zero and its positive entry-derived value"
    );
    let count_argument = unique_position(
        &normalized,
        "function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::F64ConvertI64U);",
        "species constructor count argument",
    );
    assert!(
        count_sets.iter().all(|position| *position < count_argument),
        "the original count must not be overwritten after becoming the species argument"
    );
    let target_creation = unique_normalized_position(
        body,
        TARGET_CREATION_WIRING,
        "count-to-validated-target dataflow",
    );
    assert_eq!(
        count_argument, target_creation,
        "target creation must begin by converting the immutable original count"
    );

    let copied_count = unique_normalized_position(
        body,
        COPIED_ELEMENT_COUNT_WIRING,
        "late copied-element count",
    );
    let copied_element_sets = positions(
        &normalized,
        "function.instruction(&Instruction::LocalSet(copied_element_count_local));",
    );
    assert_eq!(
        copied_element_sets.len(),
        2,
        "the copied element count must only receive zero and its positive late value"
    );
    let byte_copy = unique_normalized_position(body, BYTE_COPY_WIRING, "ascending byte copy");
    assert!(
        copied_element_sets
            .iter()
            .all(|position| copied_count <= *position && *position < byte_copy),
        "the copied element count must remain in element units after its late calculation"
    );
    assert_eq!(
        normalized
            .matches("Instruction::LocalSet(copied_byte_count_local)")
            .count(),
        1,
        "the same-type path must derive one byte count"
    );
    assert_eq!(
        normalized
            .matches("Instruction::LocalGet(copied_byte_count_local)")
            .count(),
        1,
        "only the byte loop may consume the byte count"
    );
    assert_eq!(
        normalized
            .matches("Instruction::LocalGet(copied_element_count_local)")
            .count(),
        1,
        "only the byte-count derivation may consume the late element count"
    );

    let addresses = unique_normalized_position(body, COPY_ADDRESS_WIRING, "copy addresses");
    let same_type = unique_position(
        &normalized,
        "function.instruction(&Instruction::LocalGet(source_element_kind_local));function.instruction(&Instruction::LocalGet(target_element_kind_local));function.instruction(&Instruction::I64Eq);function.instruction(&Instruction::If(BlockType::Empty));",
        "same-element-type branch",
    );
    let same_type_if = normalized[same_type..]
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .map(|position| same_type + position)
        .expect("same-type selection must open a Wasm if");
    let indexed_else = normalized[same_type_if..]
        .find("function.instruction(&Instruction::Else);")
        .map(|position| same_type_if + position)
        .expect("same-type selection must retain an indexed else arm");
    let same_type_end = matching_control_end(&normalized, same_type_if);
    let indexed_copy = unique_normalized_position(body, INDEXED_COPY_WIRING, "indexed copy");
    assert!(
        same_type_if < addresses
            && addresses < byte_copy
            && byte_copy < indexed_else
            && indexed_else < indexed_copy
            && indexed_copy < same_type_end,
        "same-type address and byte copying must precede the distinct indexed conversion arm"
    );
    assert!(
        !normalized[indexed_else..same_type_end].contains("copied_byte_count_local"),
        "the different-element-type path must never consume a byte count"
    );

    for local in ["source_address_local", "target_address_local"] {
        assert_eq!(
            normalized
                .matches(&format!("Instruction::LocalSet({local})"))
                .count(),
            1,
            "{local} must have exactly one address derivation"
        );
        assert_eq!(
            normalized
                .matches(&format!("Instruction::LocalTee({local})"))
                .count(),
            0,
            "{local} must not be overwritten through LocalTee"
        );
        assert_eq!(
            exact_identifier_mentions(body, local),
            4,
            "{local} must occur only in its reservation, derivation, byte-loop read and release"
        );
    }

    for (needle, expected, label) in [
        ("self.emit_load_array_buffer_data(", 2, "backing-data load"),
        (
            "Instruction::LocalSet(source_address_local)",
            1,
            "source-address derivation",
        ),
        (
            "Instruction::LocalSet(target_address_local)",
            1,
            "target-address derivation",
        ),
        (
            "Instruction::LocalSet(copied_byte_count_local)",
            1,
            "byte-count derivation",
        ),
        ("Instruction::I32Load8U", 1, "byte load"),
        ("Instruction::I32Store8", 1, "byte store"),
    ] {
        let operation_positions = positions(&normalized, needle);
        assert_eq!(
            operation_positions.len(),
            expected,
            "slice must retain exactly {expected} {label}"
        );
        assert!(
            operation_positions
                .iter()
                .all(|position| same_type_if < *position && *position < indexed_else),
            "every {label} must remain inside the same-type byte arm"
        );
    }

    for (needle, label) in [
        (
            "self.emit_typed_array_or_object_index_read_from_locals(",
            "indexed source read",
        ),
        (
            "Instruction::LocalSet(target_index_local)",
            "source-minus-start target index",
        ),
        (
            "self.emit_typed_array_element_write_from_locals(",
            "indexed target write",
        ),
    ] {
        let position = unique_position(&normalized, needle, label);
        assert!(
            indexed_else < position && position < same_type_end,
            "the {label} must remain inside the different-type indexed arm"
        );
    }

    let code = normalized_code(body);
    let selection = without_whitespace(SAME_TYPE_SELECTION_WIRING);
    let exact_same_type = unique_position(&code, &selection, "complete same-type branch");
    let exact_same_type_if = code[exact_same_type..]
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .map(|position| exact_same_type + position)
        .expect("the exact same-type branch must open a Wasm if");
    let exact_same_type_end = matching_control_end(&code, exact_same_type_if);
    let end_instruction = "function.instruction(&Instruction::End);";
    let mut expected_same_type = selection;
    expected_same_type.push_str(&without_whitespace(COPY_ADDRESS_WIRING));
    expected_same_type.push_str(&without_whitespace(BYTE_COPY_WIRING));
    expected_same_type.push_str("function.instruction(&Instruction::Else);");
    expected_same_type.push_str(&without_whitespace(INDEXED_COPY_WIRING));
    expected_same_type.push_str(end_instruction);
    assert_eq!(
        &code[exact_same_type..exact_same_type_end + end_instruction.len()],
        expected_same_type.as_str(),
        "the complete same-type/else branch must not admit address overwrites or unreviewed copy operations"
    );

    let positive_branch = unique_position(
        &normalized,
        "function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::Else);",
        "positive-count branch",
    );
    let positive_if = normalized[positive_branch..]
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .map(|position| positive_branch + position)
        .expect("the count guard must open a Wasm if");
    let positive_end = matching_control_end(&normalized, positive_if);
    assert!(
        positive_if < same_type_if && same_type_end < positive_end,
        "both copy paths must remain inside the original-positive-count arm"
    );
}

#[test]
fn slice_has_one_dispatch_owner_and_balanced_temporaries() {
    let body = slice_body();
    let dispatcher = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::TypedArrayPrototypeSlice => {",
        "StandardBuiltinId::TypedArrayPrototypeSet => {",
    );

    assert_eq!(
        STANDARD_SOURCE
            .matches("StandardBuiltinId::TypedArrayPrototypeSlice => {")
            .count(),
        1,
        "slice must have exactly one dispatcher arm"
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("fn compile_typed_array_prototype_slice_builtin(")
            .count(),
        1,
        "slice must have exactly one compiler definition"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("self.compile_typed_array_prototype_slice_builtin(function)?;")
            .count(),
        1,
        "the slice dispatcher arm must have exactly one compiler owner"
    );
    assert_eq!(
        without_whitespace(dispatcher),
        "self.compile_typed_array_prototype_slice_builtin(function)?;}",
        "the slice dispatcher arm must map directly to its sole compiler"
    );

    let normalized = without_whitespace(body);
    let result = unique_normalized_position(body, RESULT_WIRING, "target result publication");
    let first_release = normalized
        .find("self.release_temp_local(")
        .expect("slice must release its temporaries");
    assert!(
        result < first_release,
        "slice must publish its constructed target before releasing temporaries"
    );
    assert_temp_lifetime(body);
}
