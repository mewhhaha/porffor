const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const TYPED_BRAND_WIRING: &str = r#"
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "TypedArray find method requires a TypedArray",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
"#;

const TYPED_PRIVATE_STATE_WIRING: &str = r#"
        self.emit_load_typed_array_private_state(
            receiver_payload_local,
            receiver_buffer_local,
            receiver_byte_offset_local,
            receiver_byte_length_local,
            receiver_bytes_per_element_local,
            function,
        );
"#;

const TYPED_VIEW_WIRING: &str = r#"
        let receiver_view = TypedArrayViewLocals::new(
            receiver_payload_local,
            receiver_buffer_local,
            receiver_byte_offset_local,
            receiver_byte_length_local,
            receiver_bytes_per_element_local,
        );
"#;

const TYPED_WITNESS_WIRING: &str = r#"
        self.emit_typed_array_witness(
            &receiver_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: len_local,
            },
            function,
        )?;
"#;

const TYPED_PREDICATE_WIRING: &str = r#"
        let predicate =
            self.emit_validate_find_predicate(predicate_not_callable_message, function)?;
"#;

const VALIDATED_PREDICATE_CALL_WIRING: &str = r#"
        let ValidatedFindPredicateLocals(predicate) = predicate;

        self.emit_pre_evaluated_arg_vector(
            &[
                (element.payload, element.tag),
                (index.payload, index.tag),
                (receiver.payload, receiver.tag),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            predicate.payload,
            predicate.tag,
            this_argument.payload,
            this_argument.tag,
            argc_local,
            argv_local,
            result.payload,
            result.tag,
            function,
        )?;
"#;

const TYPED_LIVE_READ_WIRING: &str = r#"
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
"#;

const TYPED_CALLBACK_WIRING: &str = r#"
        self.emit_call_validated_find_predicate(
            predicate,
            TaggedLocals::new(this_arg_payload_local, this_arg_tag_local),
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            argc_local,
            argv_local,
            TaggedLocals::new(callback_result_payload_local, callback_result_tag_local),
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(
            callback_result_tag_local,
            callback_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_project_find_match(
            projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_advance_find_index(direction, index_local, function);
"#;

const ARRAY_CALLBACK_WIRING: &str = r#"
        self.emit_call_validated_find_predicate(
            predicate,
            TaggedLocals::new(this_arg_payload_local, this_arg_tag_local),
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_number_payload_local, number_tag_local),
            TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            argc_local,
            argv_local,
            TaggedLocals::new(callback_result_payload_local, callback_result_tag_local),
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;

        self.compile_truthy_tagged_i32(
            callback_result_tag_local,
            callback_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_project_find_match(
            projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_number_payload_local, number_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_advance_find_index(direction, index_local, function);
"#;

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
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
fn find_via_predicate_kind_has_exactly_four_inhabitants() {
    let body = ARRAY_SOURCE
        .split_once("pub(crate) enum FindViaPredicateKind {")
        .expect("find kind")
        .1
        .split_once('}')
        .expect("find kind end")
        .0;
    let variants = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["Find,", "FindIndex,", "FindLast,", "FindLastIndex,"]
    );
}

#[test]
fn typed_array_find_brand_error_has_one_source_owner() {
    assert_eq!(
        ARRAY_SOURCE
            .matches(r#""TypedArray find method requires a TypedArray""#)
            .count(),
        1
    );
}

#[test]
fn predicate_witness_has_one_validator_and_one_proxy_aware_consumer() {
    let declaration = ARRAY_SOURCE
        .split_once("struct ValidatedFindPredicateLocals")
        .expect("validated predicate declaration")
        .0
        .rsplit_once("\n\n")
        .expect("attribute boundary")
        .1;
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!ARRAY_SOURCE.contains("impl Copy for ValidatedFindPredicateLocals"));
    assert_eq!(
        ARRAY_SOURCE
            .matches("ValidatedFindPredicateLocals(")
            .count(),
        3
    );

    let validator = ARRAY_SOURCE
        .split_once("fn emit_validate_find_predicate(")
        .expect("validator")
        .1
        .split_once("fn emit_call_validated_find_predicate(")
        .expect("validator end")
        .0;
    assert_eq!(validator.matches("emit_is_callable_i32").count(), 1);
    assert!(!validator.contains("ValueKind::Function"));

    let consumer = ARRAY_SOURCE
        .split_once("fn emit_call_validated_find_predicate(")
        .expect("consumer")
        .1
        .split_once("fn emit_initialize_find_result(")
        .expect("consumer end")
        .0;
    assert_eq!(
        consumer
            .matches("emit_function_or_proxy_call_with_argv_leave_throw_completion")
            .count(),
        1
    );
    assert!(!consumer.contains("emit_function_handle_call_with_argv"));
    unique_normalized_position(
        &without_whitespace(consumer),
        VALIDATED_PREDICATE_CALL_WIRING,
        "validated predicate thisArg and element/index/receiver argument vector",
    );
}

#[test]
fn array_and_typed_array_entries_share_the_closed_four_kind_dispatch() {
    let typed_entry = ARRAY_SOURCE
        .split_once("pub(crate) fn compile_typed_array_prototype_find_builtin(")
        .expect("typed entry")
        .1
        .split_once("pub(crate) fn compile_array_prototype_find_builtin(")
        .expect("typed entry end")
        .0;
    let array_entry = ARRAY_SOURCE
        .split_once("pub(crate) fn compile_array_prototype_find_builtin(")
        .expect("array entry")
        .1
        .split_once("fn emit_array_iteration_to_object(")
        .expect("array entry end")
        .0;
    let normalized_typed_entry = without_whitespace(typed_entry);
    let normalized_array_entry = without_whitespace(array_entry);
    assert_eq!(
        typed_entry
            .matches("emit_load_typed_array_private_state(")
            .count(),
        1
    );
    assert_eq!(typed_entry.matches("TypedArrayViewLocals::new(").count(), 1);
    assert_eq!(typed_entry.matches("emit_typed_array_witness(").count(), 1);
    assert_eq!(
        typed_entry
            .matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert_eq!(typed_entry.matches("receiver_view").count(), 2);
    assert_eq!(typed_entry.matches("len_local").count(), 5);
    for carrier in [
        "receiver_buffer_local",
        "receiver_byte_offset_local",
        "receiver_byte_length_local",
        "receiver_bytes_per_element_local",
    ] {
        assert_eq!(
            typed_entry.matches(carrier).count(),
            4,
            "{carrier} must remain one reserved private-state/view carrier"
        );
    }
    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_ARRAY_BUFFER_",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
        "Instruction::LocalSet(len_local)",
        "emit_throw_runtime_error(",
        "TYPE_ERROR_NAME",
    ] {
        assert!(
            !typed_entry.contains(forbidden),
            "TypedArray find entry must not bypass its witness through {forbidden}"
        );
    }

    let brand_then_entry_witness = [
        TYPED_BRAND_WIRING,
        TYPED_PRIVATE_STATE_WIRING,
        TYPED_VIEW_WIRING,
        TYPED_WITNESS_WIRING,
        TYPED_PREDICATE_WIRING,
    ]
    .concat();
    unique_normalized_position(
        &normalized_typed_entry,
        &brand_then_entry_witness,
        "completed brand guard, private view, entry witness and later predicate validation",
    );

    for boundary in [
        "emit_initialize_find_result(",
        "emit_validate_find_predicate(",
        "emit_initialize_find_index(",
        "emit_typed_array_or_object_index_read_from_locals(",
        "emit_call_validated_find_predicate(",
        "emit_project_find_match(",
        "emit_advance_find_index(",
    ] {
        assert_eq!(
            typed_entry.matches(boundary).count(),
            1,
            "TypedArray find entry must retain one {boundary} boundary"
        );
    }
    let witness = unique_normalized_position(
        &normalized_typed_entry,
        TYPED_WITNESS_WIRING,
        "TypedArray method-entry buffer witness",
    );
    let predicate = normalized_typed_entry
        .find("emit_validate_find_predicate(")
        .expect("predicate validation");
    let initialize_index = normalized_typed_entry
        .find("emit_initialize_find_index(")
        .expect("direction-aware index initialization");
    let typed_iteration_wiring = [TYPED_LIVE_READ_WIRING, TYPED_CALLBACK_WIRING].concat();
    let index_read = unique_normalized_position(
        &normalized_typed_entry,
        &typed_iteration_wiring,
        "TypedArray live read, abrupt propagation, callback, truthiness, projection and advance",
    );
    assert!(
        witness < predicate
            && predicate < initialize_index
            && initialize_index < index_read,
        "TypedArray find entry must preserve witness, predicate, direction, live-read and projection order"
    );
    unique_normalized_position(
        &normalized_array_entry,
        ARRAY_CALLBACK_WIRING,
        "Array callback thisArg/arguments, abrupt propagation, truthiness, projection and advance",
    );

    assert_before(
        array_entry,
        "emit_array_iteration_to_object(",
        "emit_validate_find_predicate(",
    );
    assert_before(
        array_entry,
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
        "emit_validate_find_predicate(",
    );
    assert_before(
        array_entry,
        "emit_to_length_i64_from_value_locals(",
        "emit_validate_find_predicate(",
    );
    for entry in [typed_entry, array_entry] {
        assert_eq!(entry.matches("emit_validate_find_predicate(").count(), 1);
        assert_eq!(
            entry.matches("emit_call_validated_find_predicate(").count(),
            1
        );
        assert!(!entry.contains("emit_function_handle_call_with_argv"));
        assert!(!entry.contains("emit_function_or_proxy_call_with_argv_leave_throw_completion"));
        assert!(!entry.contains("emit_is_callable_i32"));
    }
    for forbidden in [
        "typed_array_only",
        "return_index",
        "reverse",
        "typed_brand_local",
        "typed_buffer_tag_local",
    ] {
        assert!(!array_entry.contains(forbidden));
    }

    let normalized_standard = without_whitespace(STANDARD_SOURCE).replace(",)", ")");
    for (builtin, compiler, variant) in [
        (
            "ArrayPrototypeFind",
            "compile_array_prototype_find_builtin",
            "Find",
        ),
        (
            "ArrayPrototypeFindIndex",
            "compile_array_prototype_find_builtin",
            "FindIndex",
        ),
        (
            "ArrayPrototypeFindLast",
            "compile_array_prototype_find_builtin",
            "FindLast",
        ),
        (
            "ArrayPrototypeFindLastIndex",
            "compile_array_prototype_find_builtin",
            "FindLastIndex",
        ),
        (
            "TypedArrayPrototypeFind",
            "compile_typed_array_prototype_find_builtin",
            "Find",
        ),
        (
            "TypedArrayPrototypeFindIndex",
            "compile_typed_array_prototype_find_builtin",
            "FindIndex",
        ),
        (
            "TypedArrayPrototypeFindLast",
            "compile_typed_array_prototype_find_builtin",
            "FindLast",
        ),
        (
            "TypedArrayPrototypeFindLastIndex",
            "compile_typed_array_prototype_find_builtin",
            "FindLastIndex",
        ),
    ] {
        let mapping = format!(
            "StandardBuiltinId::{builtin}=>{{self.{compiler}(function,FindViaPredicateKind::{variant})?;}}"
        );
        unique_normalized_position(
            &normalized_standard,
            &mapping,
            &format!("StandardBuiltinId::{builtin} -> FindViaPredicateKind::{variant}"),
        );
    }
    assert_eq!(
        STANDARD_SOURCE
            .matches("compile_array_prototype_find_builtin(")
            .count(),
        4
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("compile_typed_array_prototype_find_builtin(")
            .count(),
        4
    );
    for variant in ["Find", "FindIndex", "FindLast", "FindLastIndex"] {
        let comma_uses = STANDARD_SOURCE
            .matches(&format!("FindViaPredicateKind::{variant},"))
            .count();
        let closing_uses = STANDARD_SOURCE
            .matches(&format!("FindViaPredicateKind::{variant})"))
            .count();
        assert_eq!(comma_uses + closing_uses, 2);
    }
    assert!(!STANDARD_SOURCE.contains("TypedArrayFindKind"));
}
