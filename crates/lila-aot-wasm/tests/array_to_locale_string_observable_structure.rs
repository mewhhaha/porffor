const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start boundary: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end boundary after {start}: {end}"))
        .0
}

fn locale_body() -> &'static str {
    between(
        ARRAY_SOURCE,
        "    fn compile_to_locale_string_builtin(",
        "    pub(crate) fn emit_object_has_array_index_key_in_range_i32(",
    )
}

fn assert_ordered(source: &str, needles: &[&str]) {
    let mut remaining = source;
    for needle in needles {
        remaining = remaining
            .split_once(needle)
            .unwrap_or_else(|| panic!("missing ordered operation: {needle}"))
            .1;
    }
}

#[test]
fn array_like_entry_observes_one_public_length_and_coercion() {
    let generic = between(
        locale_body(),
        "        } else {",
        "        function.instruction(&Instruction::Loop(BlockType::Empty));",
    );
    assert_eq!(
        generic.matches("self.strings.payload(\"length\")").count(),
        1
    );
    assert_eq!(generic.matches("self.emit_object_read(").count(), 1);
    assert_ordered(
        generic,
        &[
            "emit_array_iteration_to_object(",
            "self.strings.payload(\"length\")",
            "emit_object_read(",
            "emit_propagate_throw_from_locals_if_needed(",
            "emit_to_length_i64_from_value_locals(",
            "emit_return_current_completion_if_throw(function);",
        ],
    );
    for forbidden in [
        "HEAP_LEN_OFFSET",
        "TypedArrayWitnessUse",
        "emit_load_typed_array_private_state(",
        "ValueKind::Array",
        "ValueKind::Arguments",
    ] {
        assert!(
            !generic.contains(forbidden),
            "private length bypass: {forbidden}"
        );
    }
}

#[test]
fn only_direct_typed_array_entry_uses_a_private_length_witness() {
    let body = locale_body();
    let direct = between(body, "        if typed_array_entry {", "        } else {");
    assert_eq!(body.matches("self.emit_typed_array_witness(").count(), 1);
    assert_eq!(
        direct
            .matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert!(!body.contains("TypedArrayWitnessUse::ArrayLikeLengthSnapshot"));
    assert_ordered(
        direct,
        &[
            "OBJECT_INTERNAL_BRAND_TYPED_ARRAY",
            "emit_throw_current_function_realm_type_error(",
            "emit_load_typed_array_private_state(",
            "TypedArrayWitnessUse::ValidatedMethodEntry",
        ],
    );
}

#[test]
fn indexed_get_uses_shared_dispatch_before_the_nullish_check() {
    let body = locale_body();
    assert_eq!(
        body.matches("self.emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        1
    );
    assert_ordered(
        body,
        &[
            "Instruction::Loop(BlockType::Empty)",
            "emit_typed_array_or_object_index_read_from_locals(",
            "emit_propagate_throw_from_locals_if_needed(",
            "compile_nullish_tagged_i32(element_tag_local, function)",
        ],
    );
    for forbidden in [
        "emit_arguments_read(",
        "emit_array_index_get_with_prototype(",
    ] {
        assert!(
            !body.contains(forbidden),
            "private indexed Get bypass: {forbidden}"
        );
    }
}

#[test]
fn every_non_nullish_element_uses_the_validated_invocation_protocol() {
    let invocation = between(
        locale_body(),
        "self.emit_array_iteration_to_object(element_payload_local, element_tag_local, function)?;",
        "self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;",
    );
    assert_ordered(
        invocation,
        &[
            "self.strings.payload(\"toLocaleString\")",
            "emit_object_read(",
            "emit_propagate_throw_from_locals_if_needed(",
            "emit_validate_to_locale_string_invocation(",
            "TaggedLocals::new(original_element_payload_local, original_element_tag_local)",
            "emit_call_validated_to_locale_string_invocation(",
            "emit_return_current_completion_if_throw(function);",
            "emit_value_to_string_payload(",
            "emit_return_current_completion_if_throw(function);",
        ],
    );
    for forbidden in [
        "ValueKind::Object",
        "ValueKind::Function",
        "Instruction::Else",
    ] {
        assert!(
            !invocation.contains(forbidden),
            "element Invoke bypass: {forbidden}"
        );
    }
}
