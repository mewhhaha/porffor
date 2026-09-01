const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FIND_VIA_PREDICATE_SOURCE: &str = include_str!("../src/builtins/array/find_via_predicate.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-find-via-predicate.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T16: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

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
            &projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_payload_local, index_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_advance_find_index(&direction, index_local, function);
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
            &projection,
            TaggedLocals::new(element_payload_local, element_tag_local),
            TaggedLocals::new(index_number_payload_local, number_tag_local),
            function,
        );
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_advance_find_index(&direction, index_local, function);
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
fn find_via_predicate_has_one_private_child_owner() {
    assert_eq!(
        ARRAY_SOURCE.matches("\nmod find_via_predicate;\n").count(),
        1
    );
    assert!(!ARRAY_SOURCE.contains("\npub mod find_via_predicate;\n"));
    assert!(!ARRAY_SOURCE.contains("\npub(crate) mod find_via_predicate;\n"));
    assert!(!ARRAY_SOURCE.contains("FindViaPredicateKind"));
    assert!(FIND_VIA_PREDICATE_SOURCE.starts_with("use super::*;\n\n"));

    for declaration in [
        "enum FindViaPredicateKind {",
        "enum FindDirection {",
        "enum FindProjection {",
        "struct ValidatedFindPredicateLocals(TaggedLocals);",
        "mod find_via_predicate_tests {",
    ] {
        assert_eq!(
            FIND_VIA_PREDICATE_SOURCE.matches(declaration).count(),
            1,
            "child must own exactly one `{declaration}`"
        );
        assert!(
            !ARRAY_SOURCE.contains(declaration),
            "parent must not retain `{declaration}`"
        );
    }

    let kind_impl = FIND_VIA_PREDICATE_SOURCE
        .split_once("impl FindViaPredicateKind {")
        .expect("find kind projections")
        .1
        .split_once("\n}\n\n/// Predicate locals")
        .expect("find kind projections end")
        .0;
    assert_eq!(
        kind_impl
            .lines()
            .filter(|line| line.starts_with("    const fn "))
            .count(),
        7
    );
    for projection in [
        "    const fn direction(&self)",
        "    const fn projection(&self)",
        "    const fn array_method_name(&self)",
        "    const fn typed_array_method_name(&self)",
        "    const fn array_nullish_message(&self)",
        "    const fn array_predicate_not_callable_message(&self)",
        "    const fn typed_array_predicate_not_callable_message(&self)",
    ] {
        assert_eq!(
            kind_impl.matches(projection).count(),
            1,
            "find kind must borrow through `{projection}`"
        );
    }
    for forbidden in [
        "const fn direction(self)",
        "const fn projection(self)",
        "find_kind.clone()",
        "find_kind ==",
        "find_kind !=",
        "matches!(find_kind",
        "if find_kind",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE.contains(forbidden),
            "find kind authority must not escape through `{forbidden}`"
        );
    }

    let direction_declaration = FIND_VIA_PREDICATE_SOURCE
        .split_once("enum FindDirection {")
        .expect("find direction")
        .1
        .split_once('}')
        .expect("find direction end")
        .0;
    let direction_variants = direction_declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(direction_variants, ["Ascending,", "Descending,"]);
    let direction_offset = FIND_VIA_PREDICATE_SOURCE
        .find("enum FindDirection {")
        .expect("find direction declaration");
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE[..direction_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE.contains(&format!("impl {capability} for FindDirection")),
            "find direction must not implement {capability}"
        );
    }

    let normalized_kind_impl = without_whitespace(kind_impl);
    for mapping in [
        "Self::Find|Self::FindIndex=>FindDirection::Ascending",
        "Self::FindLast|Self::FindLastIndex=>FindDirection::Descending",
    ] {
        assert_eq!(
            normalized_kind_impl.matches(mapping).count(),
            1,
            "find direction producer must retain `{mapping}`"
        );
    }
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches("direction: &FindDirection,")
            .count(),
        2
    );
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches("match direction {")
            .count(),
        2
    );
    assert_eq!(FIND_VIA_PREDICATE_SOURCE.matches("&direction").count(), 4);
    for forbidden in [
        "direction: FindDirection,",
        "direction.clone()",
        "direction ==",
        "direction !=",
        "matches!(direction",
        "if direction",
        "assert_eq!(kind.direction(), direction)",
        "_ =>",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE.contains(forbidden),
            "find direction authority must not escape through `{forbidden}`"
        );
    }
    for evidence in [CONTRACT, TASK_T02, TASK_T16] {
        assert!(evidence.contains("capability-free `FindDirection`"));
    }

    let projection_declaration = FIND_VIA_PREDICATE_SOURCE
        .split_once("enum FindProjection {")
        .expect("find projection")
        .1
        .split_once('}')
        .expect("find projection end")
        .0;
    let projection_variants = projection_declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(projection_variants, ["Value,", "Index,"]);
    let projection_offset = FIND_VIA_PREDICATE_SOURCE
        .find("enum FindProjection {")
        .expect("find projection declaration");
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE[..projection_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE.contains(&format!("impl {capability} for FindProjection")),
            "find projection must not implement {capability}"
        );
    }

    for mapping in [
        "Self::Find|Self::FindLast=>FindProjection::Value",
        "Self::FindIndex|Self::FindLastIndex=>FindProjection::Index",
    ] {
        assert_eq!(
            normalized_kind_impl.matches(mapping).count(),
            1,
            "find projection producer must retain `{mapping}`"
        );
    }
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches("projection: &FindProjection,")
            .count(),
        2
    );
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches("match projection {")
            .count(),
        2
    );
    assert_eq!(FIND_VIA_PREDICATE_SOURCE.matches("&projection").count(), 4);
    for forbidden in [
        "projection: FindProjection,",
        "projection.clone()",
        "projection ==",
        "projection !=",
        "matches!(projection",
        "if projection",
        "assert_eq!(kind.projection(), projection)",
        "_ =>",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE.contains(forbidden),
            "find projection authority must not escape through `{forbidden}`"
        );
    }
    for evidence in [CONTRACT, TASK_T02, TASK_T16] {
        assert!(evidence.contains("capability-free `FindProjection`"));
    }

    let builder_impl = FIND_VIA_PREDICATE_SOURCE
        .split_once("impl<'a> FunctionBuilder<'a> {")
        .expect("find builder owner")
        .1
        .rsplit_once("\n}")
        .expect("find builder owner end")
        .0;
    assert_eq!(
        builder_impl
            .lines()
            .filter(|line| line.starts_with("    pub(in crate::builtins) fn "))
            .count(),
        8
    );
    assert_eq!(
        builder_impl
            .lines()
            .filter(|line| line.starts_with("    fn "))
            .count(),
        8
    );
    for definition in [
        "    fn emit_validate_find_predicate(",
        "    fn emit_call_validated_find_predicate(",
        "    fn emit_initialize_find_result(",
        "    fn emit_initialize_find_index(",
        "    fn emit_project_find_match(",
        "    fn emit_advance_find_index(",
        "    fn compile_typed_array_find_with_kind(",
        "    fn compile_array_find_with_kind(",
    ] {
        assert_eq!(
            FIND_VIA_PREDICATE_SOURCE.matches(definition).count(),
            1,
            "child must own exactly one `{definition}`"
        );
        assert!(
            !ARRAY_SOURCE.contains(definition),
            "parent must not retain `{definition}`"
        );
    }
    for fixed_entry in [
        "compile_array_prototype_find_builtin",
        "compile_array_prototype_find_index_builtin",
        "compile_array_prototype_find_last_builtin",
        "compile_array_prototype_find_last_index_builtin",
        "compile_typed_array_prototype_find_builtin",
        "compile_typed_array_prototype_find_index_builtin",
        "compile_typed_array_prototype_find_last_builtin",
        "compile_typed_array_prototype_find_last_index_builtin",
    ] {
        let definition = format!("    pub(in crate::builtins) fn {fixed_entry}(");
        assert_eq!(FIND_VIA_PREDICATE_SOURCE.matches(&definition).count(), 1);
        assert!(!ARRAY_SOURCE.contains(&definition));
    }

    for retained_parent_method in [
        "    pub(crate) fn emit_array_direct_builtin_method_call(",
        "    fn emit_array_iteration_to_object(",
    ] {
        assert_eq!(ARRAY_SOURCE.matches(retained_parent_method).count(), 1);
        assert!(!FIND_VIA_PREDICATE_SOURCE.contains(retained_parent_method));
    }
}

#[test]
fn find_via_predicate_kind_has_exactly_four_inhabitants() {
    let declaration_header = FIND_VIA_PREDICATE_SOURCE
        .split_once("enum FindViaPredicateKind {")
        .expect("find kind declaration")
        .0;
    assert_eq!(declaration_header.trim(), "use super::*;");
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !FIND_VIA_PREDICATE_SOURCE
                .contains(&format!("impl {capability} for FindViaPredicateKind")),
            "find kind must not implement {capability}"
        );
    }

    let body = FIND_VIA_PREDICATE_SOURCE
        .split_once("enum FindViaPredicateKind {")
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

    for evidence in [CONTRACT, TASK_T02, TASK_T16] {
        assert!(evidence.contains("capability-free `FindViaPredicateKind`"));
        assert!(
            evidence.contains("3989f2ebe1ce925d23b20d4e06eb35f00e1e840f7509b8226b9b425a639c4e5c")
        );
        assert!(
            evidence.contains("40be1db2dd3ccb1f35a9e022061f4fb23a8adc8fac8e446f06fdb93879b3e92d")
        );
        assert!(
            evidence.contains("b71e9cfcea61c77cdbef9aeb68917c65e1e54ab1bbe735e49a4175d82f00673e")
        );
        assert!(evidence.contains("no new Array behavior"));
    }
    assert!(CONTRACT.contains("eight fixed entries"));
    assert!(CONTRACT.contains("does not close T16"));
}

#[test]
fn typed_array_find_brand_error_has_one_source_owner() {
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches(r#""TypedArray find method requires a TypedArray""#)
            .count(),
        1
    );
}

#[test]
fn predicate_witness_has_one_validator_and_one_proxy_aware_consumer() {
    let declaration = FIND_VIA_PREDICATE_SOURCE
        .split_once("struct ValidatedFindPredicateLocals")
        .expect("validated predicate declaration")
        .0
        .rsplit_once("\n\n")
        .expect("attribute boundary")
        .1;
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!FIND_VIA_PREDICATE_SOURCE.contains("impl Copy for ValidatedFindPredicateLocals"));
    assert_eq!(
        FIND_VIA_PREDICATE_SOURCE
            .matches("ValidatedFindPredicateLocals(")
            .count(),
        3
    );

    let validator = FIND_VIA_PREDICATE_SOURCE
        .split_once("fn emit_validate_find_predicate(")
        .expect("validator")
        .1
        .split_once("fn emit_call_validated_find_predicate(")
        .expect("validator end")
        .0;
    assert_eq!(validator.matches("emit_is_callable_i32").count(), 1);
    assert!(!validator.contains("ValueKind::Function"));

    let consumer = FIND_VIA_PREDICATE_SOURCE
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
    let typed_entry = FIND_VIA_PREDICATE_SOURCE
        .split_once("fn compile_typed_array_find_with_kind(")
        .expect("typed entry")
        .1
        .split_once("pub(in crate::builtins) fn compile_array_prototype_find_builtin(")
        .expect("typed entry end")
        .0;
    let array_entry = FIND_VIA_PREDICATE_SOURCE
        .split_once("fn compile_array_find_with_kind(")
        .expect("array entry")
        .1
        .rsplit_once("\n}")
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
    for (builtin, compiler) in [
        ("ArrayPrototypeFind", "compile_array_prototype_find_builtin"),
        (
            "ArrayPrototypeFindIndex",
            "compile_array_prototype_find_index_builtin",
        ),
        (
            "ArrayPrototypeFindLast",
            "compile_array_prototype_find_last_builtin",
        ),
        (
            "ArrayPrototypeFindLastIndex",
            "compile_array_prototype_find_last_index_builtin",
        ),
        (
            "TypedArrayPrototypeFind",
            "compile_typed_array_prototype_find_builtin",
        ),
        (
            "TypedArrayPrototypeFindIndex",
            "compile_typed_array_prototype_find_index_builtin",
        ),
        (
            "TypedArrayPrototypeFindLast",
            "compile_typed_array_prototype_find_last_builtin",
        ),
        (
            "TypedArrayPrototypeFindLastIndex",
            "compile_typed_array_prototype_find_last_index_builtin",
        ),
    ] {
        let mapping = format!("StandardBuiltinId::{builtin}=>{{self.{compiler}(function)?;}}");
        unique_normalized_position(
            &normalized_standard,
            &mapping,
            &format!("StandardBuiltinId::{builtin} -> {compiler}"),
        );
        assert_eq!(STANDARD_SOURCE.matches(&format!("{compiler}(")).count(), 1);
    }
    assert!(!STANDARD_SOURCE.contains("FindViaPredicateKind"));
    assert!(!STANDARD_SOURCE.contains("compile_array_find_with_kind("));
    assert!(!STANDARD_SOURCE.contains("compile_typed_array_find_with_kind("));
    assert!(!STANDARD_SOURCE.contains("TypedArrayFindKind"));
}
