const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

#[test]
fn focused_cli_fixture_covers_callbacks_without_blocking() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_preserves_created_realm_promise_internal_callbacks()"));
    assert!(CLI_TESTS.contains("wasm_promise_internal_callback_realm.js"));
    for marker in [
        "resolving function prototypes",
        "capability executor prototype",
        "capability executor TypeError realm",
        "finally outer function prototypes",
        "finally continuation prototypes",
        "standard combinator function prototypes",
        "keyed combinator function prototypes",
        "Promise self-resolution TypeError realm",
        "promise-internal-callback-realm:ok",
    ] {
        assert!(CLI_FIXTURE.contains(marker), "missing CLI marker: {marker}");
    }
    assert!(!CLI_FIXTURE.contains("Atomics.wait"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
