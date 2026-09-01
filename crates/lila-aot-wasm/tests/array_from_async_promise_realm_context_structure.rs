const ARRAY_FROM_ASYNC_SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_from_async_promise_realm.js");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

#[test]
fn execution_realm_context_is_private_noncopyable_and_must_use() {
    let declaration = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "#[must_use = \"Array.fromAsync execution Realm context must be explicitly released\"]",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(declaration.contains("struct ArrayFromAsyncExecutionRealmContext"));
    for field in [
        "constructor_payload_local: u32",
        "realm_local: u32",
        "function_prototype_local: u32",
        "type_error_prototype_local: u32",
    ] {
        assert!(
            declaration.contains(field),
            "missing context field: {field}"
        );
    }
    assert!(!declaration.contains("pub "));
    assert!(!declaration.contains("derive("));
}

#[test]
fn factory_selects_entry_explicitly_and_traps_missing_nonentry_catalog_state() {
    let factory = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_execution_realm_context(",
        "fn emit_array_from_async_intrinsic_promise_capability(",
    );
    for marker in [
        "self.current_env_local",
        "Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX)",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET",
        "HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET",
        "HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(factory.contains(marker), "missing factory marker: {marker}");
    }
    assert_eq!(factory.matches("Instruction::Unreachable").count(), 4);
    assert!(
        factory.find("let constructor_payload_local").unwrap()
            < factory.find("let realm_local").unwrap()
    );
    assert!(
        factory.find("let realm_local").unwrap()
            < factory.find("let function_prototype_local").unwrap()
    );
    assert!(
        factory.find("let function_prototype_local").unwrap()
            < factory.find("let type_error_prototype_local").unwrap()
    );
    assert!(
        factory.find("let type_error_prototype_local").unwrap()
            < factory.find("let intrinsics_local").unwrap()
    );
    assert!(factory.contains("release_temp_local(intrinsics_local)"));
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
}

#[test]
fn all_three_capabilities_borrow_one_context_before_one_consuming_release() {
    let consumer = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_intrinsic_promise_capability(",
        "fn emit_array_from_async_internal_callback_pair(",
    );
    assert!(consumer.contains("realm: &ArrayFromAsyncExecutionRealmContext"));
    assert!(consumer.contains("realm.constructor_payload_local"));
    assert!(consumer.contains("let result = self.emit_new_promise_capability("));
    assert!(
        consumer
            .find("release_temp_local(constructor_tag_local)")
            .unwrap()
            < consumer.rfind("result").unwrap()
    );

    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("emit_array_from_async_intrinsic_promise_capability(")
            .count(),
        4,
        "one consumer definition plus outer, array-like and iterable calls"
    );
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("self.emit_new_promise_capability(")
            .count(),
        1,
        "raw capability allocation must remain inside the typed consumer"
    );
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX)")
            .count(),
        1,
        "the entry Promise global is only the factory's explicit zero-environment route"
    );

    let main = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "pub(crate) fn emit_array_from_async(",
        "fn emit_array_from_async_array_like_start(",
    );
    assert_eq!(
        main.matches("emit_array_from_async_execution_realm_context(function)")
            .count(),
        1
    );
    assert_eq!(
        main.matches("release_array_from_async_execution_realm_context(execution_realm)")
            .count(),
        1
    );
    assert!(
        main.find("emit_array_from_async_execution_realm_context(function)")
            .unwrap()
            < main
                .find("emit_array_from_async_intrinsic_promise_capability(")
                .unwrap()
    );
    assert!(
        main.rfind("emit_array_from_async_iterable_start(").unwrap()
            < main
                .find("release_array_from_async_execution_realm_context(execution_realm)")
                .unwrap()
    );
}

#[test]
fn branch_helpers_accept_only_the_typed_execution_realm() {
    let array_like = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_array_like_start(",
        "fn emit_array_from_async_iterable_start(",
    );
    let iterable = ARRAY_FROM_ASYNC_SOURCE
        .split_once("fn emit_array_from_async_iterable_start(")
        .expect("iterable helper")
        .1;
    for branch in [array_like, iterable] {
        assert!(branch.contains("execution_realm: &ArrayFromAsyncExecutionRealmContext"));
        assert!(branch.contains("emit_array_from_async_intrinsic_promise_capability("));
        assert!(!branch.contains("promise_constructor_payload_local"));
        assert!(!branch.contains("promise_constructor_tag_local"));
    }
}

#[test]
fn created_realm_publication_and_finite_fixture_cover_independent_authorities() {
    let publication = between(
        HOST_SOURCE,
        "        for (name, meta) in &array_static_method_metas {",
        "        let species_key_local",
    );
    for marker in [
        "emit_function_value_payload_in_realm(",
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            publication.contains(marker),
            "missing publication marker: {marker}"
        );
    }

    for marker in [
        "other.Array.fromAsync.call(Array, [1])",
        "Array.fromAsync.call(other.Array, [2])",
        "other.Array.fromAsync.call(Array, [], 0)",
        "otherPromisePrototype",
        "otherArrayPrototype",
        "otherTypeErrorPrototype",
        "array-from-async-promise-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_the_array_from_async_method_realm_for_its_promise()"));
    assert!(CLI_TESTS.contains("wasm_array_from_async_promise_realm.js"));
}
