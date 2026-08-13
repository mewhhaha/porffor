const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn to_locale_string_receiver_kind_is_closed_and_owns_surface_text() {
    let body = ARRAY_SOURCE
        .split_once("enum ToLocaleStringReceiverKind {")
        .expect("receiver kind")
        .1
        .split_once('}')
        .expect("receiver kind end")
        .0;
    let variants = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["ArrayLike,", "TypedArray,"]);

    let mappings = ARRAY_SOURCE
        .split_once("impl ToLocaleStringReceiverKind {")
        .expect("receiver kind mappings")
        .1
        .split_once("struct ValidatedToLocaleStringInvocationLocals {")
        .expect("receiver kind mappings end")
        .0;
    assert_eq!(mappings.matches("match self {").count(), 2);
    assert!(!mappings.contains("_ =>"));
    for text in [
        "\"Array.prototype.toLocaleString\",",
        "\"TypedArray.prototype.toLocaleString\",",
        "\"Array.prototype.toLocaleString element method is not callable\"",
        "\"TypedArray.prototype.toLocaleString element method is not callable\"",
    ] {
        assert_eq!(mappings.matches(text).count(), 1, "{text}");
    }
}

#[test]
fn invocation_token_has_one_validator_and_one_proxy_aware_consumer() {
    let declaration = ARRAY_SOURCE
        .split_once("struct ValidatedToLocaleStringInvocationLocals {")
        .expect("invocation token")
        .0
        .rsplit_once("\n\n")
        .expect("invocation token attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!ARRAY_SOURCE.contains("impl Copy for ValidatedToLocaleStringInvocationLocals"));

    let fields = ARRAY_SOURCE
        .split_once("struct ValidatedToLocaleStringInvocationLocals {")
        .expect("invocation token fields")
        .1
        .split_once('}')
        .expect("invocation token fields end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(fields, ["method: TaggedLocals,", "receiver: TaggedLocals,"]);
    assert_eq!(
        ARRAY_SOURCE
            .matches("ValidatedToLocaleStringInvocationLocals {")
            .count(),
        3
    );

    let validator = ARRAY_SOURCE
        .split_once("fn emit_validate_to_locale_string_invocation(")
        .expect("invocation validator")
        .1
        .split_once("fn emit_call_validated_to_locale_string_invocation(")
        .expect("invocation validator end")
        .0;
    assert_eq!(validator.matches("emit_is_callable_i32(").count(), 1);
    assert_eq!(
        validator
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert!(validator.contains("receiver_kind.element_method_not_callable_message()"));
    assert!(!validator.contains("emit_throw_runtime_error("));
    assert!(!validator.contains("ValueKind::Function"));
    assert_before(
        validator,
        "emit_is_callable_i32(",
        "emit_throw_current_function_realm_type_error(",
    );
    assert_before(
        validator,
        "emit_throw_current_function_realm_type_error(",
        "Ok(ValidatedToLocaleStringInvocationLocals {",
    );

    let consumer = ARRAY_SOURCE
        .split_once("fn emit_call_validated_to_locale_string_invocation(")
        .expect("invocation consumer")
        .1
        .split_once("fn compile_to_locale_string_builtin(")
        .expect("invocation consumer end")
        .0;
    assert_eq!(
        consumer
            .matches("emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1
    );
    assert!(consumer.contains("let ValidatedToLocaleStringInvocationLocals"));
    assert!(consumer.contains("method.payload,"));
    assert!(consumer.contains("method.tag,"));
    assert!(consumer.contains("receiver.payload,"));
    assert!(consumer.contains("receiver.tag,"));
    assert!(consumer.contains("&[],"));
    assert!(!consumer.contains("emit_is_callable_i32("));
    assert!(!consumer.contains("emit_function_handle_call"));
}

#[test]
fn array_and_typed_array_entries_share_the_validated_invocation_boundary() {
    let dispatch = ARRAY_SOURCE
        .split_once("pub(crate) fn compile_array_prototype_to_locale_string_builtin(")
        .expect("Array toLocaleString entry")
        .1
        .split_once("fn emit_validate_to_locale_string_invocation(")
        .expect("toLocaleString entries end")
        .0;
    assert_eq!(
        dispatch
            .matches("compile_to_locale_string_builtin(ToLocaleStringReceiverKind::ArrayLike")
            .count(),
        1
    );
    assert_eq!(
        dispatch
            .matches("compile_to_locale_string_builtin(ToLocaleStringReceiverKind::TypedArray")
            .count(),
        1
    );

    let shared = ARRAY_SOURCE
        .split_once("fn compile_to_locale_string_builtin(")
        .expect("shared toLocaleString emitter")
        .1
        .split_once("pub(crate) fn emit_object_has_array_index_key_in_range_i32(")
        .expect("shared toLocaleString emitter end")
        .0;
    assert_eq!(
        shared
            .matches("emit_validate_to_locale_string_invocation(")
            .count(),
        1
    );
    assert_eq!(
        shared
            .matches("emit_call_validated_to_locale_string_invocation(")
            .count(),
        1
    );
    assert!(!shared.contains("emit_is_callable_i32("));
    assert!(!shared.contains("emit_function_or_proxy_call_leave_throw_completion("));
    assert!(!shared.contains("emit_throw_runtime_error("));
    assert_before(
        shared,
        "emit_validate_to_locale_string_invocation(",
        "emit_call_validated_to_locale_string_invocation(",
    );

    let element_invocation = shared
        .split_once("self.compile_nullish_tagged_i32(element_tag_local, function)?;")
        .expect("non-nullish element invocation")
        .1;
    assert_before(
        element_invocation,
        "LocalSet(original_element_payload_local)",
        "emit_array_iteration_to_object(",
    );
    assert_before(
        element_invocation,
        "LocalSet(original_element_tag_local)",
        "emit_array_iteration_to_object(",
    );
    assert_before(
        element_invocation,
        "self.emit_object_read(",
        "emit_validate_to_locale_string_invocation(",
    );
    assert!(
        element_invocation.contains("TaggedLocals::new(method_payload_local, method_tag_local)")
    );
    assert!(element_invocation
        .contains("TaggedLocals::new(original_element_payload_local, original_element_tag_local)"));
    assert!(
        element_invocation.contains("TaggedLocals::new(element_payload_local, element_tag_local)")
    );
}
