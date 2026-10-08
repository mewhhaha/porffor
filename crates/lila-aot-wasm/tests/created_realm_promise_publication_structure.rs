const MAIN_REALM_SOURCE: &str = include_str!("../src/intrinsics/promise.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_created_realm.js");

#[test]
fn promise_publication_catalogs_are_complete_and_shared_by_both_realms() {
    let prototype_catalog = MAIN_REALM_SOURCE
        .split_once("pub(crate) const PROMISE_PROTOTYPE_METHOD_PUBLICATIONS")
        .expect("Promise prototype catalog")
        .1
        .split_once("pub(crate) const PROMISE_STATIC_METHOD_PUBLICATIONS")
        .expect("Promise prototype catalog end")
        .0;
    let static_catalog = MAIN_REALM_SOURCE
        .split_once("pub(crate) const PROMISE_STATIC_METHOD_PUBLICATIONS")
        .expect("Promise static catalog")
        .1
        .split_once("impl FunctionBuilder<'_>")
        .expect("Promise static catalog end")
        .0;
    for builtin in [
        "PromisePrototypeThen",
        "PromisePrototypeCatch",
        "PromisePrototypeFinally",
    ] {
        assert_eq!(prototype_catalog.matches(builtin).count(), 1, "{builtin}");
    }
    for builtin in [
        "PromiseResolve",
        "PromiseReject",
        "PromiseAll",
        "PromiseAllSettled",
        "PromiseAllKeyed",
        "PromiseAllSettledKeyed",
        "PromiseAny",
        "PromiseRace",
        "PromiseWithResolvers",
        "PromiseTry",
    ] {
        assert_eq!(
            static_catalog
                .matches(&format!("StandardBuiltinId::{builtin},"))
                .count(),
            1,
            "{builtin}"
        );
    }
    assert!(MAIN_REALM_SOURCE.contains("[StandardBuiltinId; 3]"));
    assert!(MAIN_REALM_SOURCE.contains("[StandardBuiltinId; 10]"));
    for source in [MAIN_REALM_SOURCE] {
        assert_eq!(
            source
                .matches("for builtin in PROMISE_PROTOTYPE_METHOD_PUBLICATIONS")
                .count(),
            1
        );
        assert_eq!(
            source
                .matches("for builtin in PROMISE_STATIC_METHOD_PUBLICATIONS")
                .count(),
            1
        );
        assert!(source.contains("builtin.native_function_name()"));
    }
}

#[test]
fn focused_cli_fixture_exercises_created_promise_without_queuing_jobs() {
    assert!(CLI_TESTS.contains("fn run_wasm_backend_publishes_created_realm_promise_foundation()"));
    assert!(CLI_TESTS.contains("wasm_promise_created_realm.js"));
    for marker in [
        "created realm Promise identity",
        "created realm Promise global descriptor",
        "created Promise allocation prototype",
        "created Promise resolve function realm",
        "created Promise reject function realm",
        "created Promise.resolve allocation prototype",
        "created Promise constructor TypeError realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
    assert!(!CLI_FIXTURE.contains(".then("));
    assert!(!CLI_FIXTURE.contains("Atomics.wait"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
