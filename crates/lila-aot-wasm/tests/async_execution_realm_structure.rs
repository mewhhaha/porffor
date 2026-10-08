const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_async_execution_realm.js");

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

#[test]
fn async_execution_context_is_opaque_and_has_static_realm_factories() {
    let declaration = between(
        PROMISE_SOURCE,
        "pub(crate) struct AsyncExecutionRealmContext",
        "enum PromiseResolveRealmAuthority",
    );
    let preceding_attributes = PROMISE_SOURCE
        .split_once("pub(crate) struct AsyncExecutionRealmContext")
        .expect("async execution Realm context")
        .0
        .rsplit("\n\n")
        .next()
        .expect("context declaration prefix");
    assert!(!preceding_attributes.contains("derive("));
    assert!(!declaration.contains("derive("));
    assert!(!PROMISE_SOURCE.contains("pub struct AsyncExecutionRealmContext"));
    for capability in ["Clone", "Copy"] {
        assert!(
            !PROMISE_SOURCE.contains(&format!("impl {capability} for AsyncExecutionRealmContext"))
        );
    }
}
