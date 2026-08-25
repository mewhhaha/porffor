const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/object.rs");
const REFLECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const HANDLER_PROTOCOL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_define_property_handler_protocol.js");

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn mask_line_and_block_comments(source: &str) -> String {
    let mut characters = source.chars().peekable();
    let mut masked = String::with_capacity(source.len());
    let mut quote = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;

    while let Some(character) = characters.next() {
        if line_comment {
            if character == '\n' {
                masked.push(character);
                line_comment = false;
            } else {
                masked.push(' ');
            }
            continue;
        }

        if block_comment_depth > 0 {
            if character == '/' && characters.peek() == Some(&'*') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth += 1;
                continue;
            }
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth -= 1;
                continue;
            }
            masked.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }

        if let Some(delimiter) = quote {
            masked.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }

        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            masked.push(character);
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            characters.next();
            masked.push_str("  ");
            line_comment = true;
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            masked.push_str("  ");
            block_comment_depth = 1;
            continue;
        }
        masked.push(character);
    }

    assert_eq!(block_comment_depth, 0, "unterminated block comment");
    masked
}

fn anchored_offsets(source: &str, declaration: &str) -> Vec<usize> {
    source
        .match_indices(declaration)
        .filter_map(|(offset, _)| {
            let line_start = source[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            source[line_start..offset]
                .chars()
                .all(char::is_whitespace)
                .then_some(offset)
        })
        .collect()
}

fn braced_rust_function<'a>(source: &'a str, declaration: &str) -> &'a str {
    let offsets = anchored_offsets(source, declaration);
    assert_eq!(offsets.len(), 1, "exact Rust owner `{declaration}`");
    let start = offsets[0];
    let mut depth = 0;
    let mut body_started = false;
    for (relative_offset, character) in source[start..].char_indices() {
        match character {
            '{' => {
                depth += 1;
                body_started = true;
            }
            '}' => {
                depth -= 1;
                if body_started && depth == 0 {
                    return &source[start..start + relative_offset + character.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated Rust owner `{declaration}`");
}

fn assert_live_wasm_cli_test(source: &str, name: &str, fixture: &str) {
    let declaration = format!("fn {name}() {{");
    let offsets = anchored_offsets(source, &declaration);
    assert_eq!(offsets.len(), 1, "exact CLI test owner `{name}`");

    let attached_attributes = source[..offsets[0]]
        .rsplit_once("\n}\n")
        .unwrap_or_else(|| panic!("preceding top-level CLI owner for `{name}`"))
        .1;
    assert_eq!(
        attached_attributes.matches("#[test]").count(),
        1,
        "`{name}` must remain a live Rust test"
    );
    for disabling_attribute in ["#[cfg", "#[cfg_attr", "#[ignore"] {
        assert!(
            !attached_attributes.contains(disabling_attribute),
            "`{name}` must not carry `{disabling_attribute}`"
        );
    }

    let body = braced_rust_function(source, &declaration);
    for marker in [
        ".arg(\"run\")",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "output.status.success()",
        "stdout.contains(\"backend_used: WasmAot\")",
        "stdout.contains(\"boolean(true)\")",
    ] {
        assert_eq!(
            body.matches(marker).count(),
            1,
            "`{name}` must retain CLI marker `{marker}`"
        );
    }
    assert_eq!(
        body.matches(&format!("fixture_path(\n            \"{fixture}\","))
            .count(),
        1,
        "`{name}` must run its exact fixture"
    );
}

fn define_property_acquisition() -> &'static str {
    braced_rust_function(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_proxy_define_property_trap_result(",
    )
}

fn assert_typed_consumer(
    consumer: &str,
    handled_local: &str,
    object: &str,
    target: &str,
    handler: &str,
    trap: &str,
    trap_result: &str,
    fabricated_handler_tag: &str,
) {
    assert_eq!(
        consumer
            .matches("self.emit_proxy_define_property_trap_result(")
            .count(),
        1
    );
    assert_eq!(consumer.matches("ProxySlotLocals::new(").count(), 1);
    assert_eq!(consumer.matches("ProxyTargetLocals::new(").count(), 1);
    assert_eq!(consumer.matches("ProxyHandlerLocals::new(").count(), 1);
    assert_eq!(consumer.matches("PropertyKeyLocals::new(").count(), 1);
    for marker in [
        handled_local,
        object,
        target,
        handler,
        "PropertyKeyLocals::new(key_string_local, proxy_key_tag_local)",
        "TaggedLocals::new(descriptor_payload_local, descriptor_tag_local)",
        trap,
        trap_result,
    ] {
        assert!(
            consumer.contains(marker),
            "typed consumer marker `{marker}`"
        );
    }
    assert_eq!(
        consumer
            .matches("self.emit_proxy_define_property_trap_invariants(")
            .count(),
        1,
        "the typed acquisition must retain its post-trap invariant consumer"
    );
    assert_before(
        consumer,
        "self.emit_proxy_define_property_trap_result(",
        "self.emit_proxy_define_property_trap_invariants(",
    );

    for retired_inline_acquisition in [
        "self.strings.payload(\"defineProperty\")",
        "self.emit_object_read_ordinary(",
        "self.emit_function_or_proxy_call_leave_throw_completion(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
        "Proxy handler is null",
        "Proxy defineProperty trap is not callable",
        fabricated_handler_tag,
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,\n            proxy_target_payload_local",
        "HEAP_OBJECT_BOXED_TAG_OFFSET,\n            proxy_target_tag_local",
    ] {
        assert!(
            !consumer.contains(retired_inline_acquisition),
            "consumer must not retain raw acquisition `{retired_inline_acquisition}`"
        );
    }
}

#[test]
fn acquisition_has_one_typed_live_slot_authority() {
    assert_eq!(
        OBJECTS_SOURCE
            .matches("pub(crate) fn emit_proxy_define_property_trap_result(")
            .count(),
        1
    );
    let acquisition = define_property_acquisition();

    for role in [
        "object: TaggedLocals,",
        "handled_local: u32,",
        "slots: ProxySlotLocals,",
        "key: PropertyKeyLocals,",
        "descriptor: TaggedLocals,",
        "trap: TaggedLocals,",
        "trap_result: TaggedLocals,",
    ] {
        assert_eq!(acquisition.matches(role).count(), 1, "typed role `{role}`");
    }
    for mapping in [
        "let target_payload_local = slots.target.0.payload;",
        "let target_tag_local = slots.target.0.tag;",
        "let handler_payload_local = slots.handler.0.payload;",
        "let handler_tag_local = slots.handler.0.tag;",
    ] {
        assert_eq!(
            acquisition.matches(mapping).count(),
            1,
            "typed mapping `{mapping}`"
        );
    }
    for typed_field_use in [
        "object.payload",
        "object.tag",
        "key.0.payload",
        "key.0.tag",
        "descriptor.payload",
        "descriptor.tag",
        "trap.payload",
        "trap.tag",
        "trap_result.payload",
        "trap_result.tag",
    ] {
        assert!(
            acquisition.contains(typed_field_use),
            "typed field `{typed_field_use}` must be consumed"
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
            .matches("self.strings.payload(\"defineProperty\")")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("HEAP_OBJECT_BOXED_KIND_OFFSET,")
            .count(),
        1,
        "the direct heap read must only classify the current object as a Proxy"
    );
    for forbidden in [
        "HEAP_PROXY_HANDLER_TAG_OFFSET",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
        "HEAP_OBJECT_BOXED_TAG_OFFSET",
        "Instruction::LocalSet(handler_tag_local)",
    ] {
        assert!(
            !acquisition.contains(forbidden),
            "live Proxy role `{forbidden}` must not be reconstructed"
        );
    }
    assert!(acquisition.contains(
        "Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(object.payload));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(object.tag));"
    ));
}

#[test]
fn acquisition_routes_full_get_method_before_exact_handler_call() {
    let acquisition = define_property_acquisition();

    for operation in [
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(",
        "self.emit_property_key_payload_to_value_payload(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
        "self.emit_throw_current_function_realm_type_error(",
    ] {
        assert_eq!(
            acquisition.matches(operation).count(),
            1,
            "one acquisition operation `{operation}`"
        );
    }
    assert!(acquisition.contains(
        "self.emit_object_read_without_throw_propagation(\n            handler_payload_local,\n            handler_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            trap_key_local,\n            trap.payload,\n            trap.tag,"
    ));
    assert!(acquisition.contains(
        "self.emit_function_or_proxy_call_with_throw_propagation(\n            trap.payload,\n            trap.tag,\n            handler_payload_local,\n            handler_tag_local,"
    ));
    for argument in [
        "(target_payload_local, target_tag_local)",
        "(key_value_payload_local, key.0.tag)",
        "(descriptor.payload, descriptor.tag)",
    ] {
        assert_eq!(
            acquisition.matches(argument).count(),
            1,
            "exact trap argument `{argument}`"
        );
    }
    assert_before(
        acquisition,
        "(target_payload_local, target_tag_local)",
        "(key_value_payload_local, key.0.tag)",
    );
    assert_before(
        acquisition,
        "(key_value_payload_local, key.0.tag)",
        "(descriptor.payload, descriptor.tag)",
    );

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
        "self.emit_property_key_payload_to_value_payload(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
    );

    for forbidden in [
        "self.emit_object_read_ordinary(",
        "self.emit_object_read(",
        "self.emit_function_handle_call",
        "self.emit_function_or_proxy_call_leave_throw_completion(",
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
fn object_and_reflect_each_have_one_typed_consumer() {
    assert_eq!(
        OBJECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_define_property_trap_result(")
            .count(),
        1
    );
    assert_eq!(
        REFLECT_BUILTINS_SOURCE
            .matches("self.emit_proxy_define_property_trap_result(")
            .count(),
        1
    );

    let object = braced_rust_function(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_define_property_builtin(",
    );
    assert_typed_consumer(
        object,
        "proxy_handled_local,",
        "TaggedLocals::new(proxy_traversal_payload_local, proxy_traversal_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local)",
        "TaggedLocals::new(proxy_trap_payload_local, proxy_trap_tag_local)",
        "TaggedLocals::new(proxy_trap_result_payload_local, proxy_trap_result_tag_local)",
        "Instruction::LocalSet(proxy_handler_tag_local)",
    );
    assert_before(
        object,
        "Instruction::LocalSet(original_target_payload_local)",
        "self.emit_proxy_define_property_trap_result(",
    );
    assert_before(
        object,
        "self.emit_proxy_define_property_trap_result(",
        "Instruction::LocalGet(original_target_payload_local)",
    );

    let reflect = braced_rust_function(
        REFLECT_BUILTINS_SOURCE,
        "pub(crate) fn compile_reflect_define_property_builtin(",
    );
    assert_typed_consumer(
        reflect,
        "handled_local,",
        "TaggedLocals::new(target_payload_local, target_tag_local)",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(handler_payload_local, handler_tag_local)",
        "TaggedLocals::new(trap_payload_local, trap_tag_local)",
        "TaggedLocals::new(trap_result_payload_local, trap_result_tag_local)",
        "Instruction::LocalSet(handler_tag_local)",
    );
}

#[test]
fn cli_regression_is_live_and_covers_handler_protocol_boundaries() {
    const TEST_NAME: &str = "run_wasm_backend_succeeds_for_proxy_define_property_handler_protocol";
    let declaration = format!("fn {TEST_NAME}() {{");
    for commented_owner in [
        format!("// #[test]\n// {declaration}\n// }}"),
        format!("/*\n#[test]\n{declaration}\n}}\n*/"),
    ] {
        let active_source = mask_line_and_block_comments(&commented_owner);
        assert!(
            anchored_offsets(&active_source, &declaration).is_empty(),
            "commented CLI owner must not count as active"
        );
    }

    const EXECUTABLE_MARKER: &str =
        "assert(this === expectedHandler, currentScenario + \" trap this\");";
    for commented_marker in [
        format!("// {EXECUTABLE_MARKER}"),
        format!("/* {EXECUTABLE_MARKER} */"),
    ] {
        assert!(
            !mask_line_and_block_comments(&commented_marker).contains(EXECUTABLE_MARKER),
            "commented fixture assertion must not count as executable"
        );
    }

    let cli_object_source = mask_line_and_block_comments(CLI_OBJECT_SOURCE);
    let fixture = mask_line_and_block_comments(HANDLER_PROTOCOL_FIXTURE);
    assert_live_wasm_cli_test(
        &cli_object_source,
        TEST_NAME,
        "wasm_proxy_define_property_handler_protocol.js",
    );

    for marker in [
        "function functionHandler() {}",
        "var arrayHandler = [];",
        "var argumentsHandler = (function () { return arguments; })(1, 2, 3);",
        "proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);",
        "var callableProxyTrap = new Proxy(callableTrapTarget, callableTrapHandler);",
        EXECUTABLE_MARKER,
        "assert(target === expectedTarget, currentScenario + \" target\");",
        "assert(key === expectedKey, currentScenario + \" key\");",
        "assert(arguments.length === 3, currentScenario + \" trap arity\");",
        "throw lookupSentinel;",
        "new Proxy(nestedTarget, { defineProperty: null })",
        "new Proxy(nestedTarget, { defineProperty: undefined })",
        "Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype",
        "Object.getPrototypeOf(revokedError) === other.TypeError.prototype",
    ] {
        assert!(
            fixture.contains(marker),
            "fixture protocol marker `{marker}`"
        );
    }
    assert_eq!(fixture.matches("exerciseHandlerBrand(").count(), 5);
    assert!(fixture.trim_end().ends_with("true;"));
    assert_before(
        &fixture,
        "return ordinaryDefineTrap;",
        "exerciseHandlerBrand(functionHandler, {}, \"Function handler\");",
    );
    assert_before(
        &fixture,
        "throw lookupSentinel;",
        "assert(lookupError === lookupSentinel, \"abrupt lookup sentinel\");",
    );
    assert_before(
        &fixture,
        "new Proxy(nestedTarget, { defineProperty: null })",
        "assert(nestedCalls === 2, \"nested fallback call count\");",
    );
}
