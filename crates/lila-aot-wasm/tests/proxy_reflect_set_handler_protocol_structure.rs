const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const HANDLER_PROTOCOL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_reflect_set_handler_protocol.js");
const ERROR_REALM_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_set_error_realm.js");
const MODULE_BOUNDARY_SOURCE: &str = include_str!("../../../scripts/check-module-boundaries.sh");
const CONTRACT_SOURCE: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-reflect-set-handler-protocol.md");
const TASK_SOURCE: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");

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

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn reflect_set_owner() -> &'static str {
    braced_rust_function(REFLECT_SOURCE, "pub(crate) fn compile_reflect_set_builtin(")
}

#[test]
fn reflect_set_acquires_the_exact_live_proxy_handler_before_get_method() {
    let owner = reflect_set_owner();
    let acquisition_start = owner
        .find("self.emit_load_live_proxy_slots(")
        .expect("live Proxy slot acquisition");
    let acquisition_end = owner[acquisition_start..]
        .find("self.compile_truthy_tagged_i32(")
        .map(|offset| acquisition_start + offset)
        .expect("trap-result Boolean conversion");
    let acquisition = &owner[acquisition_start..acquisition_end];

    for operation in [
        "self.emit_load_live_proxy_slots(",
        "ProxySlotLocals::new(",
        "ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local)",
        "ProxyHandlerLocals::new(handler_payload_local, handler_tag_local)",
        "ProxyRevocationRoute::CurrentFunctionRealm,",
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
        "self.strings.payload(\"set\")",
    ] {
        assert_eq!(
            acquisition.matches(operation).count(),
            1,
            "one acquisition operation `{operation}`"
        );
    }
    assert_eq!(
        owner.matches("HEAP_OBJECT_BOXED_KIND_OFFSET,").count(),
        2,
        "only the outer and nullish-fallback Proxy classifications remain raw"
    );
    for forbidden in [
        "HEAP_PROXY_HANDLER_TAG_OFFSET",
        "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
        "HEAP_OBJECT_BOXED_TAG_OFFSET",
        "Instruction::LocalSet(handler_tag_local)",
        "self.emit_object_read_ordinary(",
        "\"Proxy handler is null\"",
    ] {
        assert!(
            !owner.contains(forbidden),
            "Reflect.set must not reconstruct or bypass `{forbidden}`"
        );
    }

    assert!(owner.contains(
        "self.emit_object_read_without_throw_propagation(\n            handler_payload_local,\n            handler_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            trap_key_local,\n            trap_payload_local,\n            trap_tag_local,"
    ));
    assert!(owner.contains(
        ")?;\n        self.emit_return_current_completion_if_throw(function);\n\n        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;"
    ));
    assert!(owner.contains(
        "self.emit_function_or_proxy_call_with_throw_propagation(\n            trap_payload_local,\n            trap_tag_local,\n            handler_payload_local,\n            handler_tag_local,\n            &[\n                (proxy_target_payload_local, proxy_target_tag_local),\n                (key_value_payload_local, key_property_tag_local),\n                (value_payload_local, value_tag_local),\n                (receiver_payload_local, receiver_tag_local),"
    ));

    assert_before(
        owner,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        owner,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_return_current_completion_if_throw(function);",
    );
    assert_before(
        owner,
        "self.emit_return_current_completion_if_throw(function);",
        "self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;",
    );
    assert_before(
        owner,
        "self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;",
        "self.emit_function_or_proxy_call_with_throw_propagation(",
    );
}

#[test]
fn reflect_set_retains_boolean_invariants_fallbacks_and_release_order() {
    let owner = reflect_set_owner();

    assert_eq!(
        owner
            .matches("self.emit_proxy_set_invariant_check(")
            .count(),
        1
    );
    assert_eq!(owner.matches("self.compile_truthy_tagged_i32(").count(), 1);
    assert_eq!(
        owner
            .matches("self.emit_ordinary_set_result_via_helper(")
            .count(),
        2
    );
    assert_eq!(owner.matches("self.emit_function_handle_call(").count(), 1);
    assert_eq!(
        owner
            .matches("self.emit_function_value_payload(&reflect_set_meta")
            .count(),
        1
    );
    assert_eq!(owner.matches("ValueKind::Undefined.tag()").count(), 1);
    assert_eq!(owner.matches("ValueKind::Null.tag()").count(), 1);
    assert_eq!(
        owner.matches("\"Proxy set trap is not callable\"").count(),
        1
    );
    assert!(owner.contains(
        "(proxy_target_payload_local, proxy_target_tag_local),\n                (key_value_payload_local, key_property_tag_local),\n                (value_payload_local, value_tag_local),\n                (receiver_payload_local, receiver_tag_local),"
    ));
    assert!(owner.contains(
        "self.emit_function_handle_call(\n            reflect_set_payload_local,\n            reflect_set_tag_local,\n            None,\n            &[\n                (proxy_target_payload_local, proxy_target_tag_local),\n                (key_value_payload_local, key_property_tag_local),\n                (value_payload_local, value_tag_local),\n                (receiver_payload_local, receiver_tag_local),"
    ));
    assert!(owner.contains(
        "self.release_temp_local(trap_key_local);\n        self.release_temp_local(proxy_target_tag_local);\n        self.release_temp_local(proxy_target_payload_local);\n        self.release_temp_local(handler_tag_local);\n        self.release_temp_local(handler_payload_local);\n        self.release_temp_local(receiver_tag_local);\n        self.release_temp_local(receiver_payload_local);\n        self.release_temp_local(value_tag_local);\n        self.release_temp_local(value_payload_local);\n        self.release_temp_local(key_property_tag_local);\n        self.release_temp_local(key_value_payload_local);"
    ));
}

#[test]
fn cli_regressions_cover_handler_brands_proxy_dispatch_and_error_realms() {
    for marker in [
        "fn run_wasm_backend_succeeds_for_proxy_reflect_set_handler_protocol()",
        "\"wasm_proxy_reflect_set_handler_protocol.js\"",
        "fn proxy_set_errors_use_the_borrowed_builtin_realm()",
        "\"wasm_proxy_set_error_realm.js\"",
    ] {
        assert!(CLI_OBJECT_SOURCE.contains(marker), "CLI marker `{marker}`");
    }

    for marker in [
        "function functionHandler() {}",
        "function tagSensitiveTarget() {}",
        "var tagSensitiveReceiver = [];",
        "var arrayHandler = [];",
        "var argumentsHandler = (function () { return arguments; })(1, 2, 3);",
        "proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);",
        "var callableProxyTrap = new Proxy(callableTrapTarget, callableTrapHandler);",
        "assert(this === expectedHandler, currentScenario + \" trap this\");",
        "assert(arguments.length === 4, currentScenario + \" trap arity\");",
        "assert(target === expectedTarget, currentScenario + \" target\");",
        "assert(typeof target === \"function\", currentScenario + \" target tag\");",
        "assert(Array.isArray(receiver), currentScenario + \" receiver tag\");",
        "assert(argumentsList[3] === expectedReceiver, currentScenario + \" receiver\");",
        "var symbolKey = Symbol(\"proxy-reflect-set-key\");",
        "throw trapCallSentinel;",
        "assert(trapCallError === trapCallSentinel, \"abrupt trap call sentinel\");",
        "throw lookupSentinel;",
        "new Proxy(nestedTarget, { set: null })",
        "new Proxy(nestedTarget, { set: undefined })",
    ] {
        assert!(
            HANDLER_PROTOCOL_FIXTURE.contains(marker),
            "fixture marker `{marker}`"
        );
    }
    assert_eq!(
        HANDLER_PROTOCOL_FIXTURE
            .matches("exerciseHandlerBrand(")
            .count(),
        5
    );
    assert!(HANDLER_PROTOCOL_FIXTURE.trim_end().ends_with("true;"));

    for realm_marker in [
        "other.Reflect.set(directReflectRevocable.proxy, \"value\", 1)",
        "other.Reflect.set(directReflectNonCallable, \"value\", 1)",
        "Object.getPrototypeOf(error) === other.TypeError.prototype",
    ] {
        assert!(
            ERROR_REALM_FIXTURE.contains(realm_marker),
            "Realm fixture marker `{realm_marker}`"
        );
    }
}

#[test]
fn boundary_contract_and_task_pin_the_bounded_reflect_owner() {
    for marker in [
        "Reflect Set must retain $required_proxy_reflect_set_seam",
        "Reflect Set must not reconstruct or bypass $forbidden_proxy_reflect_set_seam",
        "'live-Proxy-slot reader call in Reflect builtins'",
        "run_wasm_backend_succeeds_for_proxy_reflect_set_handler_protocol",
        "'direct Reflect Set handler-protocol fixture wiring'",
    ] {
        assert!(
            MODULE_BOUNDARY_SOURCE.contains(marker),
            "module-boundary marker `{marker}`"
        );
    }
    for marker in [
        "Proxy `[[Set]]` handler protocol in direct `Reflect.set`",
        "typed live-slot reader",
        "`GetMethod(handler, \"set\")`",
        "wasm_proxy_reflect_set_handler_protocol.js",
        "Verification pending",
    ] {
        assert!(
            CONTRACT_SOURCE.contains(marker),
            "contract marker `{marker}`"
        );
    }
    for marker in [
        "Direct `Reflect.set` handler acquisition",
        "proxy_reflect_set_handler_protocol_structure",
        "wasm_proxy_reflect_set_handler_protocol.js",
    ] {
        assert!(TASK_SOURCE.contains(marker), "T11 marker `{marker}`");
    }
}
