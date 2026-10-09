const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");

const PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_with_resolvers_result_allocation.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_promise_created_realm.js");

#[test]
fn created_realm_fixture_separates_method_and_constructor_ownership() {
    for marker in [
        "borrowed Promise.withResolvers result object realm",
        "borrowed Promise.withResolvers constructor promise realm",
        "borrowed Promise.withResolvers resolve function realm",
        "borrowed Promise.withResolvers reject function realm",
        "entry Promise.withResolvers result object realm",
        "entry Promise.withResolvers constructor promise realm",
        "entry Promise.withResolvers resolve function realm",
        "entry Promise.withResolvers reject function realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("otherPromise.withResolvers.call(Promise)"));
    assert!(CLI_FIXTURE.contains("Promise.withResolvers.call(otherPromise)"));
}

#[test]
fn with_resolvers_result_context_is_opaque_and_consumed_once() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_with_resolvers_result_allocation;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_with_resolvers_result_allocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_with_resolvers_result_allocation::"));
    assert!(!PROMISE_SOURCE.contains("PromiseWithResolversResultAllocationContext"));
    let declaration = between(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct PromiseWithResolversResultAllocationContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE.contains(&format!(
                "impl {capability} for PromiseWithResolversResultAllocationContext"
            ))
        );
    }
}
