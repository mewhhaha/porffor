const ARRAY_FROM_ASYNC_SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_from_async_internal_callback_realm.js");

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
fn callback_pair_materializer_installs_one_complete_realm_owned_closure_shape() {
    let materializer = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_internal_callback_pair(",
        "fn release_array_from_async_execution_realm_context(",
    );

    assert!(materializer.contains("realm: &ArrayFromAsyncExecutionRealmContext"));
    assert_eq!(
        materializer
            .matches("StandardBuiltinId::ArrayFromAsyncFulfilled")
            .count(),
        1
    );
    assert_eq!(
        materializer
            .matches("StandardBuiltinId::ArrayFromAsyncRejected")
            .count(),
        1
    );
    assert_eq!(
        materializer.matches("emit_function_value_payload(").count(),
        1
    );
    for marker in [
        "emit_store_function_defining_realm(",
        "realm.realm_local",
        "HEAP_PROTOTYPE_OFFSET",
        "realm.function_prototype_local",
        "HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET",
        "ValueKind::Function.tag() as u64",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "realm.type_error_prototype_local",
        "HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET",
        "state_local",
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "callback_payload_local",
    ] {
        assert!(
            materializer.contains(marker),
            "missing materializer marker: {marker}"
        );
    }
}

#[test]
fn both_branches_use_only_the_shared_callback_pair_materializer() {
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("emit_array_from_async_internal_callback_pair(")
            .count(),
        3,
        "one definition plus the array-like and iterable producers"
    );

    let array_like = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_array_like_start(",
        "fn emit_array_from_async_iterable_start(",
    );
    let iterable = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_iterable_start(",
        "pub(crate) fn emit_array_from_async_fulfilled(",
    );
    for branch in [array_like, iterable] {
        assert_eq!(
            branch
                .matches("self.emit_array_from_async_internal_callback_pair(")
                .count(),
            1
        );
        assert!(!branch.contains("StandardBuiltinId::ArrayFromAsyncFulfilled"));
        assert!(!branch.contains("StandardBuiltinId::ArrayFromAsyncRejected"));
        assert!(!branch.contains("emit_function_value_payload("));
        assert!(!branch.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
        assert_eq!(
            branch
                .matches("ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET")
                .count(),
            1
        );
        assert_eq!(
            branch
                .matches("ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET")
                .count(),
            1
        );
    }
}

#[test]
fn callback_consumers_recover_state_only_from_the_gc_visible_closure_slot() {
    assert!(ARRAY_FROM_ASYNC_SOURCE.contains("const ARRAY_FROM_ASYNC_STATE_SIZE: u64 = 176;"));
    assert!(!ARRAY_FROM_ASYNC_SOURCE.contains("ARRAY_FROM_ASYNC_REALM_ENV_OFFSET"));
    assert!(!ARRAY_FROM_ASYNC_SOURCE.contains("Instruction::LocalGet(0)"));
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET")
            .count(),
        3,
        "one pair producer and two callback consumers"
    );
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
            .count(),
        1,
        "only the pair materializer may self-back the callback environment"
    );

    let fulfilled = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "pub(crate) fn emit_array_from_async_fulfilled(",
        "pub(crate) fn emit_array_from_async_rejected(",
    );
    let rejected = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "pub(crate) fn emit_array_from_async_rejected(",
        "fn emit_array_from_async_schedule_await(",
    );
    for callback in [fulfilled, rejected] {
        assert_eq!(
            callback
                .matches("HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET")
                .count(),
            1
        );
        assert!(callback.contains("self.current_env_local"));
        assert!(!callback.contains("Instruction::LocalSet(self.current_env_local)"));
    }
}

#[test]
fn all_nine_await_schedules_reuse_the_rooted_callback_pair() {
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("self.emit_array_from_async_schedule_await(")
            .count(),
        9
    );
    let scheduler = between(
        ARRAY_FROM_ASYNC_SOURCE,
        "fn emit_array_from_async_schedule_await(",
        "fn emit_array_from_async_schedule_iterator_step_callback(",
    );
    for marker in [
        "ARRAY_FROM_ASYNC_FULFILLED_CALLBACK_OFFSET",
        "ARRAY_FROM_ASYNC_REJECTED_CALLBACK_OFFSET",
        "emit_intrinsic_await_with_handlers(",
    ] {
        assert!(
            scheduler.contains(marker),
            "missing scheduler marker: {marker}"
        );
    }

    let planning = between(
        PLANNING_SOURCE,
        "if builtin == StandardBuiltinId::ArrayFromAsync {",
        "if builtin == StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose {",
    );
    assert!(planning.contains("StandardBuiltinId::ArrayFromAsyncFulfilled"));
    assert!(planning.contains("StandardBuiltinId::ArrayFromAsyncRejected"));
}

#[test]
fn finite_fixture_covers_both_callback_kinds_in_both_source_modes() {
    for marker in [
        "arrayLikeFulfilledValue",
        "iterableFulfilledTypeError",
        "arrayLikeRejectedReason",
        "iterableRejectedReason",
        "other.Array.fromAsync.call",
        "otherTypeErrorPrototype",
        "array-from-async-internal-callback-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_preserves_array_from_async_internal_callback_realms()"));
    assert!(CLI_TESTS.contains("wasm_array_from_async_internal_callback_realm.js"));
}
