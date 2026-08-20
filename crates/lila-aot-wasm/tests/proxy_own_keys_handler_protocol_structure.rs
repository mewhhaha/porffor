const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/object.rs");
const REFLECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/reflect.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn after<'a>(source: &'a str, start: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn own_keys_acquisition() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_proxy_own_keys_trap_result(",
        "fn emit_proxy_own_keys_validated_snapshot(",
    )
}

fn assert_typed_caller(caller: &str, object: &str, target: &str, handler: &str, validator: &str) {
    assert_eq!(
        caller
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        1
    );
    assert_eq!(caller.matches("ProxySlotLocals::new(").count(), 1);
    assert_eq!(caller.matches("ProxyTargetLocals::new(").count(), 1);
    assert_eq!(caller.matches("ProxyHandlerLocals::new(").count(), 1);
    assert_eq!(caller.matches("TaggedLocals::new(").count(), 3);
    assert!(caller.contains(object));
    assert!(caller.contains(target));
    assert!(caller.contains(handler));
    assert_eq!(caller.matches(validator).count(), 1);
    assert_before(
        caller,
        "ProxyTargetLocals::new(",
        "ProxyHandlerLocals::new(",
    );
    assert_before(caller, "self.emit_proxy_own_keys_trap_result(", validator);
    for retired_inline_acquisition in [
        "self.strings.payload(\"ownKeys\")",
        "Proxy ownKeys trap is not callable",
    ] {
        assert!(
            !caller.contains(retired_inline_acquisition),
            "caller must not retain the raw ownKeys acquisition `{retired_inline_acquisition}`",
        );
    }
}

#[test]
fn acquisition_has_one_typed_live_slot_read() {
    let acquisition = own_keys_acquisition();

    for role in [
        "object: TaggedLocals,",
        "slots: ProxySlotLocals,",
        "trap: TaggedLocals,",
        "trap_result: TaggedLocals,",
    ] {
        assert_eq!(acquisition.matches(role).count(), 1, "typed role `{role}`");
    }
    for mapping in [
        "let object_payload_local = object.payload;",
        "let object_tag_local = object.tag;",
        "let target_payload_local = slots.target.0.payload;",
        "let target_tag_local = slots.target.0.tag;",
        "let handler_payload_local = slots.handler.0.payload;",
        "let handler_tag_local = slots.handler.0.tag;",
        "let trap_payload_local = trap.payload;",
        "let trap_tag_local = trap.tag;",
        "let trap_result_payload_local = trap_result.payload;",
        "let trap_result_tag_local = trap_result.tag;",
    ] {
        assert_eq!(
            acquisition.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }

    assert_eq!(
        acquisition
            .matches("self.emit_load_live_proxy_slots(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("ProxyRevocationRoute::CurrentFunctionRealm,")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("HEAP_OBJECT_BOXED_KIND_OFFSET,")
            .count(),
        1,
        "the one direct heap read is classification only"
    );
    for forbidden in [
        "HEAP_PROXY_HANDLER_TAG_OFFSET",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
        "HEAP_OBJECT_BOXED_TAG_OFFSET",
        "Instruction::LocalSet(handler_tag_local)",
        "ValueKind::Object.tag() as i64));\n        function.instruction(&Instruction::LocalSet(handler_tag_local)",
    ] {
        assert!(
            !acquisition.contains(forbidden),
            "live Proxy slot `{forbidden}` must not be reconstructed here"
        );
    }
}

#[test]
fn get_method_completion_and_proxy_aware_call_keep_exact_handler_tags() {
    let acquisition = own_keys_acquisition();

    assert_eq!(
        acquisition
            .matches("self.emit_object_read_without_throw_propagation(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("self.emit_return_current_completion_if_throw(function);")
            .count(),
        1
    );
    assert_eq!(acquisition.matches("self.emit_is_callable_i32(").count(), 1);
    assert_eq!(
        acquisition
            .matches("self.emit_function_or_proxy_call_with_throw_propagation(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("self.emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );

    assert!(acquisition.contains(
        "self.emit_object_read_without_throw_propagation(\n            handler_payload_local,\n            handler_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            key_payload_local,\n            trap_payload_local,\n            trap_tag_local,"
    ));
    assert!(acquisition.contains(
        "self.emit_function_or_proxy_call_with_throw_propagation(\n            trap_payload_local,\n            trap_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            &[(target_payload_local, target_tag_local)],"
    ));

    assert_before(
        acquisition,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        acquisition,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
    );
    assert_before(
        acquisition,
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        acquisition,
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
    );

    for forbidden in [
        "self.emit_object_read(",
        "self.emit_function_handle_call",
        "self.emit_throw_runtime_error(",
        "ValueKind::Function.tag()",
    ] {
        assert!(
            !acquisition.contains(forbidden),
            "raw operation `{forbidden}` bypasses the handler protocol"
        );
    }
}

#[test]
fn nullish_fallback_retains_the_tagged_target() {
    let acquisition = own_keys_acquisition();

    assert!(acquisition.contains(
        "Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(object_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(object_tag_local));\n        function.instruction(&Instruction::Br(2));"
    ));
    assert_before(
        acquisition,
        "ValueKind::Undefined.tag()",
        "Instruction::LocalGet(target_payload_local)",
    );
    assert_before(
        acquisition,
        "Instruction::LocalGet(target_payload_local)",
        "self.emit_throw_current_function_realm_type_error(",
    );
}

#[test]
fn all_four_consumers_use_the_typed_acquisition_and_keep_validation() {
    assert_eq!(
        OBJECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        3
    );
    assert_eq!(
        REFLECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_own_keys_trap_result(")
            .count(),
        1
    );

    let names = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_get_own_property_names_builtin(",
        "pub(super) fn compile_object_get_own_property_symbols_builtin(",
    );
    assert_typed_caller(
        names,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "self.emit_proxy_own_keys_filtered_result(",
    );

    let symbols = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_get_own_property_symbols_builtin(",
        "pub(super) fn compile_object_keys_builtin(",
    );
    assert_typed_caller(
        symbols,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "self.emit_proxy_own_keys_filtered_result(",
    );

    let keys = bounded(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_keys_builtin(",
        "fn compile_object_own_descriptor_predicate_builtin(",
    );
    assert_typed_caller(
        keys,
        "TaggedLocals::new(arg_payload_local, arg_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "self.emit_proxy_object_keys_from_own_keys_result(",
    );

    let reflect = after(
        REFLECT_BUILTINS_SOURCE,
        "pub(crate) fn compile_reflect_own_keys_builtin(",
    );
    assert_typed_caller(
        reflect,
        "TaggedLocals::new(target_payload_local, target_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(handler_payload_local, handler_tag_local)",
        "self.emit_proxy_own_keys_array_result(",
    );
}
