//! Retained semantic fixture witness; old raw-representation mirrors retired.
const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_async_execution_realm.js");

#[test]
fn finite_fixture_distinguishes_job_realm_from_request_method_realm() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_async_function_realms_for_promises_and_reactions()"));
    for marker in [
        "other.Array.prototype.map.bind",
        "generator.next",
        "async-generator request Promise defining Realm",
        "invalid async-generator request Promise defining Realm",
        "invalid async-generator request TypeError defining Realm",
        "async-execution-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(!CLI_FIXTURE.contains("request ownership deferred"));
    assert!(!CLI_FIXTURE.contains("Atomics"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
