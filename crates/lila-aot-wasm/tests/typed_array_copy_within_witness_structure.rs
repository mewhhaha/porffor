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

const ENTRY_WITNESS_WIRING: &str = r#"
    self.emit_typed_array_witness(
        &receiver_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: receiver_length_local,
        },
        function,
    )?;
"#;

const POSITIVE_COUNT_WITNESS_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_typed_array_witness(
        &receiver_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: current_length_local,
        },
        function,
    )?;
"#;

const OPTIONAL_END_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(receiver_length_local));
    function.instruction(&Instruction::LocalSet(final_local));
    function.instruction(&Instruction::LocalGet(self.argc_param_local()));
    function.instruction(&Instruction::I64Const(2));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_builtin_arg_to_locals(2, argument_payload_local, argument_tag_local, function);
    function.instruction(&Instruction::LocalGet(argument_tag_local));
    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_value_to_number_payload(argument_tag_local, argument_payload_local, function)?;
    function.instruction(&Instruction::LocalSet(argument_payload_local));
    self.emit_return_current_completion_if_throw(function);
    self.emit_to_integer_or_infinity_number_payload_from_number_payload(
        argument_payload_local,
        argument_payload_local,
        function,
    );
    self.emit_array_slice_clamped_index(
        argument_payload_local,
        receiver_length_local,
        final_local,
        function,
    );
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
"#;

const INITIAL_COUNT_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::LocalGet(final_local));
    function.instruction(&Instruction::LocalGet(from_local));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(final_local));
    function.instruction(&Instruction::LocalGet(from_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::LocalGet(receiver_length_local));
    function.instruction(&Instruction::LocalGet(to_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(available_local));
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
"#;

const DESTINATION_CAP_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(available_local));
    function.instruction(&Instruction::LocalGet(to_local));
    function.instruction(&Instruction::LocalGet(current_length_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(current_length_local));
    function.instruction(&Instruction::LocalGet(to_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(available_local));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::End);
"#;

const SOURCE_CAP_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(available_local));
    function.instruction(&Instruction::LocalGet(from_local));
    function.instruction(&Instruction::LocalGet(current_length_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(current_length_local));
    function.instruction(&Instruction::LocalGet(from_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(available_local));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalGet(count_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(available_local));
    function.instruction(&Instruction::LocalSet(count_local));
    function.instruction(&Instruction::End);
"#;

const OVERLAP_DIRECTION_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::LocalSet(direction_local));
    function.instruction(&Instruction::LocalGet(from_byte_local));
    function.instruction(&Instruction::LocalGet(to_byte_local));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(to_byte_local));
    function.instruction(&Instruction::LocalGet(from_byte_local));
    function.instruction(&Instruction::LocalGet(byte_count_local));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Empty));
"#;

const RECEIVER_RESULT_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(receiver_payload_local));
    function.instruction(&Instruction::LocalSet(self.result_local));
    function.instruction(&Instruction::LocalGet(receiver_tag_local));
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

fn copy_within_body() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "fn compile_typed_array_prototype_copy_within_builtin(",
        "fn compile_typed_array_prototype_to_reversed_builtin(",
    )
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
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
    let snippet = without_whitespace(snippet);
    assert_eq!(
        body.matches(snippet.as_str()).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(snippet.as_str())
        .unwrap_or_else(|| panic!("missing normalized sentinel: {label}"))
}

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle)
        .map(|(position, _)| position)
        .collect()
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
        "copyWithin reservations must keep the reviewed binding shape"
    );
    assert_eq!(
        body.matches("release_temp_local(").count(),
        releases.len(),
        "copyWithin releases must keep the reviewed call shape"
    );

    let mut unique = reservations.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        reservations.len(),
        "copyWithin must reserve every temporary exactly once"
    );

    let mut expected_releases = reservations;
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "copyWithin must release temporaries in reverse reservation order"
    );
}

#[test]
fn copy_within_uses_one_immutable_view_and_two_validated_observations() {
    let body = copy_within_body();

    for (needle, expected, label) in [
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable view"),
        ("emit_typed_array_witness(", 2, "live buffer witness"),
        (
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            2,
            "validated observation",
        ),
        ("receiver_view", 3, "one view producer and two consumers"),
        ("emit_load_array_buffer_data(", 1, "explicit copy-data load"),
    ] {
        assert_eq!(
            body.matches(needle).count(),
            expected,
            "copyWithin must have exactly {expected} {label}"
        );
    }

    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_throw_runtime_error(",
        "TYPE_ERROR_NAME",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
        "Instruction::LocalSet(receiver_length_local)",
        "Instruction::LocalSet(current_length_local)",
        "Instruction::LocalSet(receiver_buffer_local)",
        "Instruction::LocalSet(receiver_byte_offset_local)",
        "Instruction::LocalSet(receiver_byte_length_local)",
        "Instruction::LocalSet(receiver_bytes_per_element_local)",
    ] {
        assert!(
            !body.contains(forbidden),
            "copyWithin must not bypass or overwrite its witnesses through {forbidden}"
        );
    }

    let normalized = without_whitespace(body);
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact immutable-view wiring");
    let entry = unique_normalized_position(
        &normalized,
        ENTRY_WITNESS_WIRING,
        "entry validated observation",
    );
    let late = unique_normalized_position(
        &normalized,
        POSITIVE_COUNT_WITNESS_WIRING,
        "conditional post-coercion validated observation",
    );
    assert!(
        private_state < view && view < entry && entry < late,
        "copyWithin must load one view and consume its two observations in order"
    );

    let brand_throw = r#"
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray.prototype.copyWithin requires TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
    "#;
    unique_normalized_position(
        &normalized,
        &format!("{brand_throw}{PRIVATE_STATE_WIRING}"),
        "completed receiver-brand guard before private-state access",
    );
}

#[test]
fn copy_within_preserves_coercion_revalidation_and_byte_copy_order() {
    let body = copy_within_body();
    let normalized = without_whitespace(body);

    let entry = unique_normalized_position(
        &normalized,
        ENTRY_WITNESS_WIRING,
        "entry validated observation",
    );
    let target = unique_position(
        &normalized,
        "self.emit_builtin_arg_to_locals(0,argument_payload_local,argument_tag_local,function);",
        "target argument load",
    );
    let start = unique_position(
        &normalized,
        "self.emit_builtin_arg_to_locals(1,argument_payload_local,argument_tag_local,function);",
        "start argument load",
    );
    let end = unique_position(
        &normalized,
        "self.emit_builtin_arg_to_locals(2,argument_payload_local,argument_tag_local,function);",
        "end argument load",
    );

    let coercions = positions(
        &normalized,
        "self.emit_value_to_number_payload(argument_tag_local,argument_payload_local,function)?;",
    );
    let integer_conversions = positions(
        &normalized,
        "self.emit_to_integer_or_infinity_number_payload_from_number_payload(argument_payload_local,argument_payload_local,function,);",
    );
    let clamps = [
        unique_position(
            &normalized,
            "self.emit_array_slice_clamped_index(argument_payload_local,receiver_length_local,to_local,function,);",
            "target clamp destination",
        ),
        unique_position(
            &normalized,
            "self.emit_array_slice_clamped_index(argument_payload_local,receiver_length_local,from_local,function,);",
            "start clamp destination",
        ),
        unique_position(
            &normalized,
            "self.emit_array_slice_clamped_index(argument_payload_local,receiver_length_local,final_local,function,);",
            "end clamp destination",
        ),
    ];
    assert_eq!(
        coercions.len(),
        3,
        "target, start and end must each be coerced"
    );
    assert_eq!(
        integer_conversions.len(),
        3,
        "target, start and end must each use ToIntegerOrInfinity"
    );
    let optional_end = unique_normalized_position(
        &normalized,
        OPTIONAL_END_WIRING,
        "optional end presence and undefined guards",
    );

    let initial_count = unique_normalized_position(
        &normalized,
        INITIAL_COUNT_WIRING,
        "captured-length count calculation",
    );
    let late = unique_normalized_position(
        &normalized,
        POSITIVE_COUNT_WITNESS_WIRING,
        "positive-count post-coercion observation",
    );
    let positive_count_if = normalized[late..]
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .map(|position| late + position)
        .expect("positive-count observation must begin inside a Wasm if");
    let positive_count_end = matching_control_end(&normalized, positive_count_if);

    assert!(
        entry < target
            && target < coercions[0]
            && coercions[0] < integer_conversions[0]
            && integer_conversions[0] < clamps[0]
            && clamps[0] < start
            && start < coercions[1]
            && coercions[1] < integer_conversions[1]
            && integer_conversions[1] < clamps[1]
            && clamps[1] < optional_end
            && optional_end < end
            && end < coercions[2]
            && coercions[2] < integer_conversions[2]
            && integer_conversions[2] < clamps[2]
            && clamps[2] < initial_count
            && initial_count < late,
        "copyWithin must validate at entry, coerce target/start/end in order, calculate captured-length count, then conditionally revalidate"
    );

    let current_length_reads = positions(
        &normalized,
        "function.instruction(&Instruction::LocalGet(current_length_local));",
    );
    assert_eq!(
        current_length_reads.len(),
        4,
        "the two current source/destination caps must each read current length twice"
    );
    assert!(
        current_length_reads.iter().all(|position| late < *position),
        "current-length caps must follow the second observation"
    );

    let data = unique_position(
        &normalized,
        "self.emit_load_array_buffer_data(receiver_buffer_local,buffer_data_local,function);",
        "copy-data load",
    );
    let destination_cap = unique_normalized_position(
        &normalized,
        DESTINATION_CAP_WIRING,
        "current destination-availability cap",
    );
    let source_cap = unique_normalized_position(
        &normalized,
        SOURCE_CAP_WIRING,
        "current source-availability cap",
    );
    assert!(
        late < positive_count_if
            && positive_count_if < destination_cap
            && destination_cap < source_cap
            && source_cap < data
            && current_length_reads.iter().all(|position| *position < data),
        "copy data must be loaded only after the destination and source current-length caps"
    );

    let overlap = unique_normalized_position(
        &normalized,
        OVERLAP_DIRECTION_WIRING,
        "overlap-direction selection",
    );
    let backward = unique_normalized_position(
        &normalized,
        r#"
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(direction_local));
        "#,
        "backward-copy direction",
    );
    let byte_load = unique_position(
        &normalized,
        "function.instruction(&Instruction::I32Load8U(self.buffer_memarg8(0)));",
        "bytewise source load",
    );
    let byte_store = unique_position(
        &normalized,
        "function.instruction(&Instruction::I32Store8(self.buffer_memarg8(0)));",
        "bytewise destination store",
    );
    let copy_loop = unique_position(
        &normalized,
        "function.instruction(&Instruction::Loop(BlockType::Empty));",
        "byte-copy loop",
    );
    let copy_loop_end = matching_control_end(&normalized, copy_loop);
    let result = unique_normalized_position(
        &normalized,
        RECEIVER_RESULT_WIRING,
        "original-receiver result wiring",
    );
    assert_eq!(
        body.matches("Instruction::Loop(BlockType::Empty)").count(),
        1,
        "copyWithin must retain one byte-copy loop"
    );
    assert!(
        data < overlap
            && overlap < backward
            && backward < copy_loop
            && copy_loop < byte_load
            && byte_load < byte_store
            && byte_store < copy_loop_end
            && copy_loop_end < positive_count_end
            && positive_count_end < result,
        "copyWithin must keep byte transfer inside its loop and every copy operation inside the positive-count branch, then return the receiver after it"
    );
}

#[test]
fn copy_within_has_one_dispatch_owner_and_balanced_temporaries() {
    let body = copy_within_body();
    let dispatcher = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::TypedArrayPrototypeCopyWithin => {",
        "StandardBuiltinId::TypedArrayPrototypeSort => {",
    );

    assert_eq!(
        STANDARD_SOURCE
            .matches("StandardBuiltinId::TypedArrayPrototypeCopyWithin => {")
            .count(),
        1,
        "copyWithin must have exactly one dispatcher arm"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("fn compile_typed_array_prototype_copy_within_builtin(")
            .count(),
        1,
        "copyWithin must have exactly one compiler definition"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("self.compile_typed_array_prototype_copy_within_builtin(function)?;")
            .count(),
        1,
        "the copyWithin dispatcher arm must have exactly one compiler owner"
    );
    assert_eq!(
        without_whitespace(dispatcher),
        "self.compile_typed_array_prototype_copy_within_builtin(function)?;}",
        "the copyWithin dispatcher arm must map directly to its sole compiler"
    );

    let normalized = without_whitespace(body);
    let result = unique_normalized_position(
        &normalized,
        RECEIVER_RESULT_WIRING,
        "original-receiver result wiring",
    );
    let first_release = normalized
        .find("self.release_temp_local(")
        .expect("copyWithin must release its temporaries");
    assert!(
        result < first_release,
        "copyWithin must publish its receiver before releasing temporaries"
    );
    assert_temp_lifetime(body);
}
