const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");

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

    let validator = ARRAY_SOURCE
        .split_once("fn emit_validate_to_locale_string_invocation(")
        .expect("invocation validator")
        .1
        .split_once("fn emit_call_validated_to_locale_string_invocation(")
        .expect("invocation validator end")
        .0;
    assert_eq!(validator.matches("match receiver_kind {").count(), 1);
    assert!(!validator.contains("_ =>"));
    for text in [
        "\"Array.prototype.toLocaleString element method is not callable\"",
        "\"TypedArray.prototype.toLocaleString element method is not callable\"",
    ] {
        assert_eq!(validator.matches(text).count(), 1, "{text}");
    }

    let shared = ARRAY_SOURCE
        .split_once("fn compile_to_locale_string_builtin(")
        .expect("shared toLocaleString emitter")
        .1
        .split_once("pub(crate) fn emit_object_has_array_index_key_in_range_i32(")
        .expect("shared toLocaleString emitter end")
        .0;
    assert_eq!(shared.matches("match &receiver_kind {").count(), 2);
    for text in [
        "\"Array.prototype.toLocaleString\"",
        "\"TypedArray.prototype.toLocaleString\"",
    ] {
        assert_eq!(shared.matches(text).count(), 1, "{text}");
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
    assert!(validator.contains("error_message,"));
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

#[test]
fn created_realm_installs_the_self_backed_typed_array_entry() {
    let method_metas = HOST_SOURCE
        .split_once("        let typed_array_method_metas = [")
        .expect("created-realm TypedArray method table")
        .1
        .split_once("        let number_meta = self")
        .expect("created-realm TypedArray method table end")
        .0;
    let entry_start = "            (\n                \"toLocaleString\",";
    assert_eq!(method_metas.matches(entry_start).count(), 1);
    let entry = method_metas
        .split_once(entry_start)
        .expect("created-realm TypedArray toLocaleString entry")
        .1
        .split_once("            ),")
        .expect("created-realm TypedArray toLocaleString entry end")
        .0;
    assert_eq!(
        entry
            .matches("StandardBuiltinId::TypedArrayPrototypeToLocaleString.function_id()")
            .count(),
        1
    );

    let installer = HOST_SOURCE
        .split_once("        for (name, meta) in &typed_array_method_metas {")
        .expect("created-realm TypedArray method installer")
        .1
        .split_once("        let typed_array_buffer_key_local")
        .expect("created-realm TypedArray method installer end")
        .0;
    assert_eq!(
        installer
            .matches("emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches(
                "method_payload_local,\n                HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n                method_payload_local,",
            )
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches(
                "method_payload_local,\n                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,\n                type_error_prototype_local,",
            )
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("typed_array_prototype_local,\n                name,")
            .count(),
        1
    );
}
