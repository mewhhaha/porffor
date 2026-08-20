const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn function_between(start: &str, end: &str) -> &'static str {
    OBJECT_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing `{end}` after `{start}`"))
        .0
}

fn struct_fields(name: &str) -> Vec<&'static str> {
    OBJECT_SOURCE
        .split_once(&format!("struct {name} {{"))
        .unwrap_or_else(|| panic!("missing `{name}`"))
        .1
        .split_once('}')
        .unwrap_or_else(|| panic!("missing end of `{name}`"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn invoke_receiver_roles_and_validated_call_are_private_non_copy_states() {
    let receiver_name = "ObjectToLocaleStringGetVLocals";
    let receiver_declaration = OBJECT_SOURCE
        .split_once(&format!("struct {receiver_name} {{"))
        .expect("receiver roles")
        .0
        .rsplit_once("\n\n")
        .expect("receiver attribute boundary")
        .1;
    assert!(receiver_declaration.contains("#[must_use"));
    assert!(!receiver_declaration.contains("derive"));
    assert!(!receiver_declaration.contains("pub"));
    assert!(!OBJECT_SOURCE.contains("impl Copy for ObjectToLocaleStringGetVLocals"));
    assert_eq!(
        struct_fields(receiver_name),
        [
            "original_receiver: TaggedLocals,",
            "boxed_lookup: TaggedLocals,",
            "method: TaggedLocals,"
        ]
    );

    let invocation_name = "ValidatedObjectToLocaleStringInvocationLocals";
    let invocation_declaration = OBJECT_SOURCE
        .split_once(&format!("struct {invocation_name} {{"))
        .expect("validated invocation")
        .0
        .rsplit_once("\n\n")
        .expect("invocation attribute boundary")
        .1;
    assert!(invocation_declaration.contains("#[must_use"));
    assert!(!invocation_declaration.contains("derive"));
    assert!(!invocation_declaration.contains("pub"));
    assert!(!OBJECT_SOURCE.contains("impl Copy for ValidatedObjectToLocaleStringInvocationLocals"));
    assert_eq!(
        struct_fields(invocation_name),
        ["method: TaggedLocals,", "receiver: TaggedLocals,"]
    );
}

#[test]
fn get_v_validation_and_call_have_one_typed_role_mapping() {
    let get_v = function_between(
        "fn emit_object_to_locale_string_get_v(",
        "fn emit_validate_object_to_locale_string_invocation(",
    );
    assert_eq!(get_v.matches("emit_object_read(").count(), 1);
    for mapping in [
        "get_v.boxed_lookup.payload,",
        "get_v.boxed_lookup.tag,",
        "get_v.original_receiver.payload,",
        "get_v.original_receiver.tag,",
        "get_v.method.payload,",
        "get_v.method.tag,",
    ] {
        assert_eq!(get_v.matches(mapping).count(), 1, "{mapping}");
    }
    assert_before(
        get_v,
        "get_v.boxed_lookup.tag,",
        "get_v.original_receiver.payload,",
    );

    let validator = function_between(
        "fn emit_validate_object_to_locale_string_invocation(",
        "fn emit_call_validated_object_to_locale_string_invocation(",
    );
    assert_eq!(validator.matches("emit_is_callable_i32(").count(), 1);
    assert_eq!(
        validator
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert!(validator.contains("let ObjectToLocaleStringGetVLocals"));
    assert!(validator.contains("original_receiver,"));
    assert!(validator.contains("method,"));
    assert!(validator.contains("receiver: original_receiver,"));
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
        "Ok(ValidatedObjectToLocaleStringInvocationLocals",
    );

    let consumer = function_between(
        "fn emit_call_validated_object_to_locale_string_invocation(",
        "pub(super) fn compile_object_prototype_to_locale_string_builtin(",
    );
    assert_eq!(
        consumer
            .matches("emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1
    );
    for mapping in [
        "method.payload,",
        "method.tag,",
        "receiver.payload,",
        "receiver.tag,",
        "&[],",
        "result.payload,",
        "result.tag,",
    ] {
        assert_eq!(consumer.matches(mapping).count(), 1, "{mapping}");
    }
    assert!(consumer.contains("let ValidatedObjectToLocaleStringInvocationLocals"));
    assert!(!consumer.contains("emit_is_callable_i32("));
    assert!(!consumer.contains("emit_function_handle_call"));
}

#[test]
fn builtin_uses_current_realm_errors_and_only_the_typed_invoke_path() {
    let builtin = function_between(
        "pub(super) fn compile_object_prototype_to_locale_string_builtin(",
        "pub(super) fn compile_object_prototype_value_of_builtin(",
    );

    assert_eq!(
        builtin
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(
        builtin
            .matches("emit_object_to_locale_string_get_v(")
            .count(),
        1
    );
    assert_eq!(
        builtin
            .matches("emit_validate_object_to_locale_string_invocation(")
            .count(),
        1
    );
    assert_eq!(
        builtin
            .matches("emit_call_validated_object_to_locale_string_invocation(")
            .count(),
        1
    );
    assert_eq!(
        builtin
            .matches("emit_return_current_completion_if_throw(function);")
            .count(),
        2
    );
    for forbidden in [
        "emit_throw_runtime_error(",
        "emit_object_read(",
        "emit_is_callable_i32(",
        "emit_function_handle_call",
        "emit_function_or_proxy_call_leave_throw_completion(",
        "ValueKind::Function",
        "argc_local",
        "argv_local",
    ] {
        assert!(!builtin.contains(forbidden), "raw operation `{forbidden}`");
    }

    assert!(builtin.contains(
        "original_receiver: TaggedLocals::new(receiver_payload_local, receiver_tag_local)"
    ));
    assert!(
        builtin.contains("boxed_lookup: TaggedLocals::new(lookup_payload_local, lookup_tag_local)")
    );
    assert!(builtin.contains("TaggedLocals::new(method_payload_local, method_tag_local)"));
    assert!(builtin.contains("TaggedLocals::new(self.result_local, self.result_tag_local)"));

    assert_before(
        builtin,
        "compile_nullish_tagged_i32(",
        "emit_throw_current_function_realm_type_error(",
    );
    assert_before(
        builtin,
        "emit_throw_current_function_realm_type_error(",
        "emit_value_to_current_function_realm_object_locals(",
    );
    assert_before(
        builtin,
        "emit_value_to_current_function_realm_object_locals(",
        "let get_v = ObjectToLocaleStringGetVLocals",
    );
    assert_before(
        builtin,
        "emit_object_to_locale_string_get_v(",
        "emit_validate_object_to_locale_string_invocation(",
    );
    let after_get_v = builtin
        .split_once("emit_object_to_locale_string_get_v(")
        .expect("GetV call")
        .1;
    assert_before(
        after_get_v,
        "emit_return_current_completion_if_throw(function);",
        "emit_validate_object_to_locale_string_invocation(",
    );
    assert_before(
        builtin,
        "emit_validate_object_to_locale_string_invocation(",
        "emit_call_validated_object_to_locale_string_invocation(",
    );
    let after_call = builtin
        .split_once("emit_call_validated_object_to_locale_string_invocation(")
        .expect("Call boundary")
        .1;
    assert!(after_call.contains("emit_return_current_completion_if_throw(function);"));
}
