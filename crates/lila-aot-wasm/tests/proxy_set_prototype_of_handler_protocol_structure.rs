const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const OBJECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/object.rs");
const REFLECT_BUILTINS_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const MODULE_BOUNDARY_SOURCE: &str = include_str!("../../../scripts/check-module-boundaries.sh");
const CONTRACT_SOURCE: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-set-prototype-of-handler-protocol.md");
const TASK_SOURCE: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const HANDLER_PROTOCOL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_set_prototype_of_handler_protocol.js");

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

fn set_prototype_of_dispatch() -> &'static str {
    braced_rust_function(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_set_prototype_of_i32(",
    )
}

#[test]
fn acquisition_uses_typed_live_slots_and_full_get_method() {
    let dispatch = set_prototype_of_dispatch();

    for operation in [
        "self.emit_load_live_proxy_slots(",
        "ProxySlotLocals::new(",
        "ProxyTargetLocals::new(target_payload_local, target_tag_local)",
        "ProxyHandlerLocals::new(handler_payload_local, handler_tag_local)",
        "ProxyRevocationRoute::ObjectMutationRealmToActiveHandler,",
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
        "self.strings.payload(\"setPrototypeOf\")",
    ] {
        assert_eq!(
            dispatch.matches(operation).count(),
            1,
            "one acquisition operation `{operation}`"
        );
    }
    assert_eq!(
        dispatch.matches("HEAP_OBJECT_BOXED_KIND_OFFSET,").count(),
        1,
        "the direct heap read may only classify the current object as a Proxy"
    );
    for forbidden in [
        "HEAP_PROXY_HANDLER_TAG_OFFSET",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
        "HEAP_OBJECT_BOXED_TAG_OFFSET",
        "Instruction::LocalSet(handler_tag_local)",
        "self.emit_object_read_ordinary(",
        "self.emit_throw_runtime_error_to_active_handler(",
    ] {
        assert!(
            !dispatch.contains(forbidden),
            "SetPrototypeOf must not reconstruct or bypass `{forbidden}`"
        );
    }
    assert_eq!(
        dispatch
            .matches("self.emit_object_mutation_type_error_to_active_handler(")
            .count(),
        2,
        "non-callable and local invariant errors must share the Realm-aware active route"
    );

    assert!(dispatch.contains(
        "self.emit_object_read_without_throw_propagation(\n            handler_payload_local,\n            handler_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            key_local,\n            trap_payload_local,\n            trap_tag_local,"
    ));
    assert!(dispatch.contains(
        ")?;\n        self.emit_return_current_completion_if_throw(function);\n        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;"
    ));
    assert!(dispatch.contains(
        "self.emit_function_or_proxy_call_with_throw_propagation(\n            trap_payload_local,\n            trap_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            &[\n                (target_payload_local, target_tag_local),\n                (proto_payload_local, proto_tag_local),"
    ));

    assert_before(
        dispatch,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        dispatch,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
    );
    assert_before(
        dispatch,
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        dispatch,
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
    );
}

#[test]
fn fallback_invariants_results_and_release_order_remain_owned_by_existing_paths() {
    let dispatch = set_prototype_of_dispatch();

    for invariant in [
        "self.emit_call_object_is_extensible_helper(",
        "self.emit_call_object_get_prototype_of_helper(",
        "self.emit_tagged_payload_same_value_i32(",
    ] {
        assert_eq!(
            dispatch.matches(invariant).count(),
            1,
            "invariant `{invariant}`"
        );
    }
    assert!(dispatch.contains(
        "Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(object_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(object_tag_local));\n        function.instruction(&Instruction::I64Const(2));\n        function.instruction(&Instruction::LocalSet(handled_local));"
    ));
    assert!(dispatch.contains(
        "self.release_temp_local(key_local);\n        self.release_temp_local(target_proto_tag_local);\n        self.release_temp_local(target_proto_payload_local);\n        self.release_temp_local(target_extensible_local);\n        self.release_temp_local(trap_truthy_local);\n        self.release_temp_local(trap_result_tag_local);\n        self.release_temp_local(trap_result_payload_local);\n        self.release_temp_local(trap_tag_local);\n        self.release_temp_local(trap_payload_local);\n        self.release_temp_local(target_tag_local);\n        self.release_temp_local(target_payload_local);\n        self.release_temp_local(handler_tag_local);\n        self.release_temp_local(handler_payload_local);\n        self.release_temp_local(handled_local);"
    ));

    assert_eq!(
        OBJECT_BUILTINS_SOURCE
            .matches("self.emit_object_set_prototype_of_i32(")
            .count(),
        2,
        "Object.setPrototypeOf and Object.prototype.__proto__ must share the internal method"
    );
    assert_eq!(
        REFLECT_BUILTINS_SOURCE
            .matches("self.emit_object_set_prototype_of_i32(")
            .count(),
        1,
        "Reflect.setPrototypeOf must share the internal method"
    );

    let object = braced_rust_function(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_set_prototype_of_builtin(",
    );
    assert!(object.contains("Instruction::LocalSet(self.result_local)"));
    assert!(object.contains("Instruction::LocalSet(self.result_tag_local)"));
    assert!(object.contains("Object.setPrototypeOf returned false"));
    assert_before(
        object,
        "self.emit_object_set_prototype_of_i32(",
        "Object.setPrototypeOf returned false",
    );

    let reflect = braced_rust_function(
        REFLECT_BUILTINS_SOURCE,
        "pub(crate) fn compile_reflect_set_prototype_of_builtin(",
    );
    assert!(reflect.contains("Instruction::LocalSet(self.result_local)"));
    assert!(reflect.contains("ValueKind::Boolean.tag()"));
    assert_before(
        reflect,
        "self.emit_object_set_prototype_of_i32(",
        "ValueKind::Boolean.tag()",
    );

    let proto_setter = braced_rust_function(
        OBJECT_BUILTINS_SOURCE,
        "pub(super) fn compile_object_prototype_proto_setter_builtin(",
    );
    assert!(proto_setter.contains("ValueKind::Undefined.tag()"));
    assert!(proto_setter.contains("Object.prototype.__proto__ setter could not set prototype"));
    assert_before(
        proto_setter,
        "ValueKind::Undefined.tag()",
        "self.emit_object_set_prototype_of_i32(",
    );
}

#[test]
fn module_boundary_and_written_contract_pin_the_typed_acquisition() {
    for marker in [
        "GetPrototypeOf, SetPrototypeOf,",
        "\"$proxy_slot_reader\" 9 'live-Proxy-slot reader definition/internal call'",
        "proxy_set_prototype_of_dispatch=\"$(sed -n",
        "Proxy SetPrototypeOf must retain $required_proxy_set_prototype_of_seam",
        "Proxy SetPrototypeOf must not reconstruct or bypass $forbidden_proxy_set_prototype_of_seam",
    ] {
        assert!(
            MODULE_BOUNDARY_SOURCE.contains(marker),
            "module-boundary marker `{marker}`"
        );
    }
    for marker in [
        "Proxy `[[SetPrototypeOf]]` handler protocol",
        "typed live-slot reader",
        "`GetMethod(handler, \"setPrototypeOf\")`",
        "wasm_proxy_set_prototype_of_handler_protocol.js",
        "Verification pending",
    ] {
        assert!(
            CONTRACT_SOURCE.contains(marker),
            "contract marker `{marker}`"
        );
    }
    for marker in [
        "Proxy `[[SetPrototypeOf]]` handler acquisition",
        "proxy_set_prototype_of_handler_protocol_structure",
        "wasm_proxy_set_prototype_of_handler_protocol.js",
    ] {
        assert!(TASK_SOURCE.contains(marker), "T11 marker `{marker}`");
    }
}

#[test]
fn cli_regression_is_live_and_covers_handler_protocol_boundaries() {
    const TEST_NAME: &str = "run_wasm_backend_succeeds_for_proxy_set_prototype_of_handler_protocol";
    let cli_object_source = mask_line_and_block_comments(CLI_OBJECT_SOURCE);
    let fixture = mask_line_and_block_comments(HANDLER_PROTOCOL_FIXTURE);
    assert_live_wasm_cli_test(
        &cli_object_source,
        TEST_NAME,
        "wasm_proxy_set_prototype_of_handler_protocol.js",
    );

    for marker in [
        "function functionHandler() {}",
        "var arrayHandler = [];",
        "var argumentsHandler = (function () { return arguments; })(1, 2, 3);",
        "proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);",
        "var callableProxyTrap = new Proxy(callableTrapTarget, callableTrapHandler);",
        "assert(this === expectedHandler, currentScenario + \" trap this\");",
        "assert(target === expectedTarget, currentScenario + \" target\");",
        "assert(prototype === expectedPrototype, currentScenario + \" prototype\");",
        "assert(arguments.length === 2, currentScenario + \" trap arity\");",
        "assert(key === \"setPrototypeOf\", currentScenario + \" lookup key\");",
        "throw lookupSentinel;",
        "new Proxy(nestedTarget, { setPrototypeOf: null })",
        "new Proxy(nestedTarget, { setPrototypeOf: undefined })",
        "Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype",
        "Object.getPrototypeOf(invariantError) === other.TypeError.prototype",
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
        "return ordinarySetPrototypeOfTrap;",
        "exerciseHandlerBrand(functionHandler, {}, \"Function handler\");",
    );
    assert_before(
        &fixture,
        "throw lookupSentinel;",
        "assert(lookupError === lookupSentinel, \"abrupt lookup sentinel\");",
    );
    assert_before(
        &fixture,
        "new Proxy(nestedTarget, { setPrototypeOf: null })",
        "assert(nestedCalls === 2, \"nested fallback call count\");",
    );
}
