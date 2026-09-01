const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_async_execution_realm.js");

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
fn async_function_activation_traces_its_defining_realm() {
    for marker in [
        "pub(crate) const HEAP_ASYNC_ACTIVATION_RECORD_SIZE: u64 = 144;",
        "pub(crate) const HEAP_ASYNC_FUNCTION_REALM_OFFSET: u64 = 136;",
        "pub(crate) const HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT",
        "name: \"realm\"",
        "offset: HEAP_ASYNC_FUNCTION_REALM_OFFSET",
        "pointer: true",
        "assert_layout(\n            HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT",
        ".chain(HEAP_ASYNC_FUNCTION_ACTIVATION_LAYOUT.iter())",
    ] {
        assert!(
            HEAP_SOURCE.contains(marker),
            "missing heap marker: {marker}"
        );
    }
    assert!(!HEAP_SOURCE.contains("HEAP_ASYNC_GENERATOR_REALM_OFFSET"));
}

#[test]
fn async_execution_context_is_opaque_and_has_static_realm_factories() {
    let declaration = between(
        PROMISE_SOURCE,
        "#[must_use = \"async execution Realm context must be explicitly released\"]",
        "enum PromiseResolveRealmAuthority",
    );
    assert!(declaration.contains("pub(crate) struct AsyncExecutionRealmContext"));
    assert!(declaration.contains("realm_local: u32"));
    assert!(!declaration.contains("derive("));

    let factories = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_async_execution_realm_context_from_function(",
        "pub(crate) fn emit_current_function_realm_intrinsic_promise_allocation_context(",
    );
    for marker in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_ASYNC_FUNCTION_REALM_OFFSET",
        "HEAP_ASYNC_GENERATOR_FUNCTION_OFFSET",
        "HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET",
        "emit_store_async_function_execution_realm(",
        "release_async_execution_realm_context(",
    ] {
        assert!(
            factories.contains(marker),
            "missing factory marker: {marker}"
        );
    }
    assert!(!factories.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!PROMISE_SOURCE.contains("emit_current_realm_promise_allocation_context"));

    let generator_factory = between(
        factories,
        "pub(crate) fn emit_async_generator_execution_realm_context_from_activation(",
        "pub(crate) fn emit_store_async_function_execution_realm(",
    );
    assert!(
        generator_factory.find("let realm_local").unwrap()
            < generator_factory.find("let function_object_local").unwrap()
    );
    assert!(
        generator_factory
            .find("release_temp_local(function_object_local)")
            .unwrap()
            < generator_factory
                .rfind("AsyncExecutionRealmContext { realm_local }")
                .unwrap()
    );

    let allocation_factory = between(
        factories,
        "pub(crate) fn emit_async_execution_promise_allocation_context(",
        "pub(crate) fn release_async_execution_realm_context(",
    );
    assert!(
        allocation_factory.find("let realm_local").unwrap()
            < allocation_factory.find("let prototype_local").unwrap()
    );
    assert!(
        allocation_factory.find("let prototype_local").unwrap()
            < allocation_factory.find("let intrinsics_local").unwrap()
    );
    assert!(allocation_factory.contains("release_temp_local(intrinsics_local)"));
}

#[test]
fn all_four_direct_async_promise_allocations_borrow_the_typed_context() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_async_execution_promise_allocation_context(")
            .count()
            + FUNCTIONS_SOURCE
                .matches("emit_async_execution_promise_allocation_context(")
                .count()
            + CONTROL_FLOW_SOURCE
                .matches("emit_async_execution_promise_allocation_context(")
                .count(),
        5,
        "one factory plus four direct async allocation sites"
    );
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("emit_async_execution_promise_allocation_context(")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("emit_async_execution_promise_allocation_context(")
            .count(),
        3
    );
    assert!(FUNCTIONS_SOURCE.contains("emit_async_execution_realm_context_from_function("));
    assert!(FUNCTIONS_SOURCE.contains("emit_store_async_function_execution_realm("));
    let for_await = between(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_async_for_of_iterator(",
        "pub(crate) fn compile_for_of_iterator(",
    );
    assert_eq!(
        for_await
            .matches("emit_async_execution_promise_allocation_context(")
            .count(),
        2,
        "iterator-close and iterator-next rejection wrappers"
    );
    assert_eq!(
        for_await
            .matches("emit_async_function_execution_realm_context_from_activation(")
            .count(),
        2
    );
    assert_eq!(
        for_await
            .matches("emit_async_generator_execution_realm_context_from_activation(")
            .count(),
        2
    );
}

#[test]
fn default_and_captured_reactions_have_separate_realm_apis() {
    let initialization = between(
        PROMISE_SOURCE,
        "fn emit_initialize_promise_reaction(",
        "fn emit_append_promise_reaction(",
    );
    for marker in [
        "fn emit_initialize_default_promise_reaction(",
        "PromiseReactionInitialization::Default",
        "fn emit_initialize_async_execution_promise_reaction(",
        "PromiseReactionInitialization::AsyncExecution",
        "realm.realm_local",
    ] {
        assert!(
            initialization.contains(marker),
            "missing reaction marker: {marker}"
        );
    }
    assert!(!initialization.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let await_reactions = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_async_await_reactions(",
        "pub(crate) fn emit_promise_prototype_then(",
    );
    assert!(await_reactions.contains("emit_default_intrinsic_await_reactions("));
    assert!(await_reactions.contains("emit_async_execution_intrinsic_await_reactions("));
    assert!(
        await_reactions.contains("emit_async_function_execution_realm_context_from_activation(")
    );
    assert!(
        await_reactions.contains("emit_async_generator_execution_realm_context_from_activation(")
    );
    assert!(!await_reactions.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let ordinary_await = between(
        await_reactions,
        "fn emit_await_reactions(",
        "fn emit_default_intrinsic_await_reactions(",
    );
    assert!(
        ordinary_await.find("let realm =").unwrap()
            < ordinary_await.find("let undefined_payload_local").unwrap()
    );
    assert!(
        ordinary_await
            .find("release_temp_local(undefined_payload_local)")
            .unwrap()
            < ordinary_await
                .find("release_async_execution_realm_context(realm)")
                .unwrap()
    );
}

#[test]
fn focused_fixture_crosses_job_realms_without_waiting() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_async_function_realms_for_promises_and_reactions()"));
    assert!(CLI_TESTS.contains("wasm_async_execution_realm.js"));
    for marker in [
        "async invocation Promise prototype",
        "async captured reaction Realm",
        "async-generator activation function Realm",
        "async-generator captured reaction Realm",
        "async-execution-realm:ok",
    ] {
        assert!(CLI_FIXTURE.contains(marker), "missing CLI marker: {marker}");
    }
    assert!(!CLI_FIXTURE.contains("Atomics"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
