const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");

const PROMISE_RESOLVE_REALM_CONTEXT_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_resolve_realm_context.rs");

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
    include_str!("../../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

#[test]
fn promise_resolve_capability_errors_use_the_operation_function_realm() {
    for marker in [
        "MissingCapabilitySpecies",
        "other.Promise.prototype.finally.call(missingCapabilityReceiver",
        "borrowed Promise.finally PromiseResolve TypeError realm",
        "promise-internal-callback-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
}

#[test]
fn promise_resolve_contexts_are_private_noncopyable_and_must_use() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_resolve_realm_context;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_resolve_realm_context;"));
    assert!(!PROMISE_SOURCE.contains("promise_resolve_realm_context::"));
    assert!(!PROMISE_SOURCE.contains("PromiseResolveOperationRealmContext"));
    assert!(!PROMISE_SOURCE.contains("IntrinsicPromiseResolveRealmContext"));
    let declaration = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct PromiseResolveOperationRealmContext"));
    assert!(declaration.contains("pub(super) struct IntrinsicPromiseResolveRealmContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains(&format!(
            "impl {capability} for PromiseResolveOperationRealmContext"
        )));
        assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains(&format!(
            "impl {capability} for IntrinsicPromiseResolveRealmContext"
        )));
    }
}
