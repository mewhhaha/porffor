const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

const PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_combinator_algorithm_error_realm.rs");

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
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_combinator_algorithm_error_realm.js");

#[test]
fn created_realm_fixture_covers_all_six_combinator_method_identities() {
    for marker in [
        "other.Promise.all.call(Promise, null)",
        "other.Promise.allSettled.call(Promise, null)",
        "other.Promise.allKeyed.call(Promise, 0)",
        "other.Promise.allSettledKeyed.call(Promise, 0)",
        "other.Promise.any.call(Promise, null)",
        "other.Promise.race.call(Promise, null)",
        "other.TypeError.prototype",
        "promise-combinator-algorithm-error-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_created_realm_promise_combinator_algorithm_errors()"));
    assert!(CLI_TESTS.contains("wasm_promise_combinator_algorithm_error_realm.js"));
}

#[test]
fn promise_combinator_algorithm_error_realm_context_is_private_paired_and_non_copyable() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_combinator_algorithm_error_realm;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_combinator_algorithm_error_realm;"));
    let declaration = between(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct PromiseCombinatorAlgorithmErrorRealmContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE.contains(&format!(
                "impl {capability} for PromiseCombinatorAlgorithmErrorRealmContext"
            ))
        );
    }
}
