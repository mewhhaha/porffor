const PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_prototype_then_invocation.rs");

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

const PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_prototype_receiver_type_error.rs");

const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

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
fn promise_prototype_receiver_errors_form_a_closed_two_variant_domain() {
    assert_eq!(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
            .matches("\nenum PromisePrototypeReceiverError {")
            .count(),
        1,
    );
    assert!(!PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
        .contains("pub(super) enum PromisePrototypeReceiverError"));
    assert!(!PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
        .contains("pub(crate) enum PromisePrototypeReceiverError"));
    let error_domain = between(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE,
        "enum PromisePrototypeReceiverError {",
        "pub(super) struct PromisePrototypeReceiverTypeErrorPrototypeLocal",
    );
    for marker in [
        "ThenIncompatible,",
        "FinallyNonObject,",
        "RuntimeErrorMessage::PROMISE_PROTOTYPE_THEN_CALLED_ON_INCOMPATIBLE_RECEIVER",
        "RuntimeErrorMessage::PROMISE_PROTOTYPE_FINALLY_CALLED_ON_NON_OBJECT_RECEIVER",
    ] {
        assert!(
            error_domain.contains(marker),
            "missing error domain member: {marker}"
        );
    }
    assert!(!error_domain.contains("_ =>"));
    assert!(!error_domain.contains("pub enum PromisePrototypeReceiverError"));
    assert!(!error_domain.contains("PartialEq"));
    assert!(!error_domain.contains("Eq"));
    for visibility in ["pub fn", "pub(super) fn", "pub(crate) fn"] {
        assert!(
            !PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE.contains(&format!(
                "{visibility} emit_throw_promise_prototype_receiver_error("
            ))
        );
    }

    for (wrapper, variant) in [
        (
            "emit_throw_promise_then_incompatible_receiver_error",
            "PromisePrototypeReceiverError::ThenIncompatible",
        ),
        (
            "emit_throw_promise_finally_non_object_receiver_error",
            "PromisePrototypeReceiverError::FinallyNonObject",
        ),
    ] {
        let semantic_wrapper = between(
            PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE,
            &format!("pub(super) fn {wrapper}("),
            "\n    }",
        );
        assert!(semantic_wrapper.contains(variant));
        assert_eq!(
            semantic_wrapper
                .matches("emit_throw_promise_prototype_receiver_error(")
                .count(),
            1,
        );
    }
}

#[test]
fn promise_internal_callback_fixture_observes_delegated_then_boundary() {
    for marker in [
        "other.Promise.prototype.then.call({})",
        "borrowed Promise.then receiver TypeError realm",
        "other.Promise.prototype.finally.call(null, noop)",
        "borrowed Promise.finally receiver TypeError realm",
        "other.Promise.prototype.catch.call(null, noop)",
        "borrowed Promise.catch ToObject TypeError realm",
        "other.Promise.prototype.catch.call({ then: 0 }, noop)",
        "borrowed Promise.catch then TypeError realm",
        "other.Promise.prototype.finally.call(nonCallableFinallyReceiver, noop)",
        "borrowed Promise.finally then TypeError realm",
        "other.Number.prototype.then = function(onFulfilled, onRejected)",
        "borrowed Promise.catch created-Realm primitive wrapper",
        "Object.defineProperty(poisonedCatchReceiver, \"then\"",
        "borrowed Promise.catch then getter abrupt completion",
        "var delegatedThenProxy = new Proxy(function() {}",
        "borrowed Promise.catch callable Proxy result",
        "borrowed Promise.catch callable Proxy receiver",
        "borrowed Promise.catch callable Proxy argument count",
        "promise-internal-callback-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
}

#[test]
fn promise_prototype_receiver_error_proof_is_private_and_one_shot() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_prototype_receiver_type_error;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_prototype_receiver_type_error;"));
    assert!(!PROMISE_SOURCE.contains("promise_prototype_receiver_type_error::"));
    assert!(!PROMISE_SOURCE.contains("PromisePrototypeReceiverTypeErrorPrototypeLocal"));
    let declaration = between(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(
        declaration.contains("pub(super) struct PromisePrototypeReceiverTypeErrorPrototypeLocal")
    );
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE.contains(&format!(
                "impl {capability} for PromisePrototypeReceiverTypeErrorPrototypeLocal"
            ))
        );
    }
}

#[test]
fn delegated_then_invocation_is_private_validated_and_one_shot() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_prototype_then_invocation;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_prototype_then_invocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_prototype_then_invocation::"));
    assert!(!PROMISE_SOURCE.contains("ValidatedPromisePrototypeThenInvocationLocals"));
    let declaration = between(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct ValidatedPromisePrototypeThenInvocationLocals"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE.contains(&format!(
            "impl {capability} for ValidatedPromisePrototypeThenInvocationLocals"
        )));
    }
}
