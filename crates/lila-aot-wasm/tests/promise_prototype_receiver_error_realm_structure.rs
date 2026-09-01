use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_prototype_receiver_type_error.rs");
const PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_prototype_then_invocation.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
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

fn count_in_rust_sources(root: &Path, fragment: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, fragment);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(fragment)
                .count()
        })
        .sum()
}

#[test]
fn promise_prototype_receiver_error_proof_is_private_and_one_shot() {
    let proof = between(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE,
        "#[must_use = \"Promise prototype receiver TypeError prototype must be consumed\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_prototype_receiver_type_error;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_prototype_receiver_type_error;"));
    assert!(!PROMISE_SOURCE.contains("promise_prototype_receiver_type_error::"));
    assert!(!PROMISE_SOURCE.contains("PromisePrototypeReceiverTypeErrorPrototypeLocal"));
    assert!(!PROMISE_SOURCE.contains("PromisePrototypeReceiverError"));
    assert!(
        proof.contains("pub(super) struct PromisePrototypeReceiverTypeErrorPrototypeLocal(u32);")
    );
    assert!(proof.contains("(u32);"));
    assert!(!proof.contains("derive(Clone"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE.contains(&format!(
                "impl {capability} for PromisePrototypeReceiverTypeErrorPrototypeLocal"
            )),
            "PromisePrototypeReceiverTypeErrorPrototypeLocal must not acquire manual {capability}",
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "PromisePrototypeReceiverTypeErrorPrototypeLocal",
        ),
        6,
        "the private child must own every Promise receiver TypeError proof use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "PromisePrototypeReceiverError"),
        5,
        "the private child must own every raw Promise receiver error policy use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_prototype_receiver_type_error::"),
        0,
        "the Promise receiver TypeError owner must have no import or re-export",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
            .matches("PromisePrototypeReceiverTypeErrorPrototypeLocal(prototype_local)")
            .count(),
        1,
        "only the private child may construct the raw prototype proof",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
            .matches("prototype.0")
            .count(),
        2,
        "only the consuming child method may project and release the raw prototype",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
            .matches("emit_load_promise_prototype_receiver_type_error_prototype(")
            .count(),
        1,
        "the private child must own the sole proof factory",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_load_promise_prototype_receiver_type_error_prototype(")
            .count(),
        2,
        "then and finally must remain the only proof factory callers",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
            .matches("emit_throw_promise_prototype_receiver_error(")
            .count(),
        3,
        "the private child must own the raw consumer and both semantic wrappers",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_throw_promise_prototype_receiver_error(")
            .count(),
        0,
        "the parent must not select the raw receiver error policy",
    );
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
        "#[must_use = \"Promise prototype receiver TypeError prototype must be consumed\"]",
    );
    for marker in [
        "ThenIncompatible,",
        "FinallyNonObject,",
        "Promise.prototype.then called on incompatible receiver",
        "Promise.prototype.finally called on non-object receiver",
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
        assert_eq!(PROMISE_SOURCE.matches(&format!(".{wrapper}(")).count(), 1);
    }
}

#[test]
fn promise_prototype_receiver_error_proof_uses_only_the_executing_function_snapshot() {
    let factory = between(
        PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE,
        "fn emit_load_promise_prototype_receiver_type_error_prototype(",
        "fn emit_throw_promise_prototype_receiver_error(",
    );
    for marker in [
        "self.current_env_local",
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            factory.contains(marker),
            "missing Realm authority: {marker}"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factory.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(!factory.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert_eq!(factory.matches("Instruction::Unreachable").count(), 1);
    let nonentry_branch = factory.find("Instruction::Else").unwrap();
    assert!(factory.find("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX").unwrap() < nonentry_branch);
    assert!(
        factory
            .find("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
            .unwrap()
            > nonentry_branch
    );
    assert_eq!(factory.matches("reserve_temp_local()").count(), 1);
}

#[test]
fn promise_prototype_receiver_error_consumer_releases_its_proof_on_emit_error() {
    let consumer = PROMISE_PROTOTYPE_RECEIVER_TYPE_ERROR_SOURCE
        .split_once("fn emit_throw_promise_prototype_receiver_error(")
        .expect("Promise receiver TypeError proof consumer")
        .1
        .split_once(
            "\n    }\n\n    pub(super) fn emit_throw_promise_then_incompatible_receiver_error(",
        )
        .expect("Promise receiver TypeError proof consumer end")
        .0;
    assert!(consumer.contains("prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal"));
    assert!(consumer.contains("error: PromisePrototypeReceiverError"));
    assert!(consumer.contains("let result = self.emit_throw_runtime_error_with_prototype_local("));
    assert!(consumer.contains("error.message()"));
    assert!(
        consumer
            .find("emit_throw_runtime_error_with_prototype_local(")
            .unwrap()
            < consumer.find("release_temp_local(prototype.0)").unwrap()
    );
    assert!(!consumer.contains(")?;"));
    assert!(consumer.trim_end().ends_with("result"));
}

#[test]
fn then_and_finally_consume_receiver_error_authority_before_species_selection() {
    let then_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_then(",
        "pub(crate) fn emit_promise_prototype_catch(",
    );
    let finally_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_finally(",
        "fn emit_run_async_continuation_job(",
    );

    for (builtin, receiver_check, throw_method) in [
        (
            then_builtin,
            "Instruction::LocalGet(valid_receiver_local)",
            "emit_throw_promise_then_incompatible_receiver_error(",
        ),
        (
            finally_builtin,
            "Instruction::I32Eqz",
            "emit_throw_promise_finally_non_object_receiver_error(",
        ),
    ] {
        let invalid_branch = builtin.find(receiver_check).unwrap();
        let proof = builtin
            .find("emit_load_promise_prototype_receiver_type_error_prototype(function)")
            .unwrap();
        let throw = builtin.find(throw_method).unwrap();
        let completion_return = builtin
            .find("emit_return_current_completion(function)")
            .unwrap();
        let species = builtin
            .find("emit_current_function_promise_species_realm_context(function)")
            .unwrap();
        assert!(invalid_branch < proof);
        assert!(proof < throw);
        assert!(throw < completion_return);
        assert!(completion_return < species);
        assert!(!builtin.contains("self.emit_throw_runtime_error(\n            TYPE_ERROR_NAME,"));
    }
}

#[test]
fn delegated_then_invocation_is_private_validated_and_one_shot() {
    let declaration = between(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE,
        "#[must_use = \"a validated Promise prototype then invocation must be called\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_prototype_then_invocation;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_prototype_then_invocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_prototype_then_invocation::"));
    assert!(!PROMISE_SOURCE.contains("ValidatedPromisePrototypeThenInvocationLocals"));
    assert!(declaration.contains("pub(super) struct ValidatedPromisePrototypeThenInvocationLocals"));
    assert!(declaration.contains("method: TaggedLocals,"));
    assert!(declaration.contains("receiver: TaggedLocals,"));
    assert!(!declaration.contains("derive(Clone"));
    assert!(!declaration.contains("pub(crate)"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE.contains(&format!(
                "impl {capability} for ValidatedPromisePrototypeThenInvocationLocals"
            )),
            "ValidatedPromisePrototypeThenInvocationLocals must not acquire manual {capability}",
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "ValidatedPromisePrototypeThenInvocationLocals",
        ),
        5,
        "the private child must own every validated delegated-then carrier use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_prototype_then_invocation::"),
        0,
        "the delegated-then carrier owner must have no import or re-export",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE
            .matches("Ok(ValidatedPromisePrototypeThenInvocationLocals { method, receiver })")
            .count(),
        1,
        "only the private child may construct a validated delegated-then pair",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE
            .matches("let ValidatedPromisePrototypeThenInvocationLocals { method, receiver }")
            .count(),
        1,
        "only the private child may project a validated delegated-then pair",
    );

    assert_eq!(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE
            .matches("emit_validate_promise_prototype_then_invocation(")
            .count(),
        1,
        "the private child must own the sole delegated-then validator",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_validate_promise_prototype_then_invocation(")
            .count(),
        2,
        "catch and finally must remain the only delegated-then validator callers",
    );
    assert_eq!(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE
            .matches("emit_call_validated_promise_prototype_then_invocation(")
            .count(),
        1,
        "the private child must own the sole delegated-then consumer",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_call_validated_promise_prototype_then_invocation(")
            .count(),
        2,
        "catch and finally must remain the only delegated-then consumer callers",
    );

    let validator = between(
        PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE,
        "fn emit_validate_promise_prototype_then_invocation(",
        "pub(super) fn emit_call_validated_promise_prototype_then_invocation(",
    );
    assert!(validator.contains("method: TaggedLocals,"));
    assert!(validator.contains("receiver: TaggedLocals,"));
    assert_eq!(validator.matches("emit_is_callable_i32(").count(), 1);
    assert_eq!(
        validator
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert!(validator.contains("\"value is not callable\""));
    assert!(validator.contains("emit_return_current_completion(function)"));
    assert!(!validator.contains("emit_function_or_proxy_call_leave_throw_completion("));

    let consumer = PROMISE_PROTOTYPE_THEN_INVOCATION_SOURCE
        .split_once("fn emit_call_validated_promise_prototype_then_invocation(")
        .expect("validated delegated-then consumer")
        .1
        .rsplit_once("\n    }\n}")
        .expect("validated delegated-then consumer end")
        .0;
    assert!(consumer.contains("invocation: ValidatedPromisePrototypeThenInvocationLocals,"));
    assert!(consumer.contains("first_argument: TaggedLocals,"));
    assert!(consumer.contains("second_argument: TaggedLocals,"));
    assert_eq!(
        consumer
            .matches("emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1
    );
    assert!(!consumer.contains("emit_is_callable_i32("));
    assert!(!consumer.contains("emit_throw_current_function_realm_type_error("));

    let shared_call = between(
        FUNCTIONS_SOURCE,
        "fn emit_function_or_proxy_call_with_argv_inner(",
        "pub(crate) fn emit_function_handle_call_with_argv_inner(",
    );
    let non_callable_errors = shared_call
        .match_indices("\"value is not callable\"")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(non_callable_errors.len(), 2);
    assert_eq!(
        shared_call
            .matches("self.emit_throw_runtime_error(")
            .count(),
        3
    );
    assert_eq!(
        shared_call
            .matches("self.emit_throw_runtime_error_with_prototype_local(")
            .count(),
        1
    );
    let revoked_proxy_error = shared_call.find("\"Proxy handler is null\"").unwrap();
    let apply_lookup = shared_call.find("self.emit_object_read(").unwrap();
    let non_callable_apply_error = shared_call
        .find("\"Proxy apply trap is not callable\"")
        .unwrap();
    assert!(non_callable_errors[0] < non_callable_errors[1]);
    assert!(non_callable_errors[1] < revoked_proxy_error);
    assert!(revoked_proxy_error < apply_lookup);
    assert!(apply_lookup < non_callable_apply_error);
}

#[test]
fn catch_and_finally_route_lookup_before_validated_delegated_then_call() {
    let catch_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_catch(",
        "pub(crate) fn emit_promise_prototype_finally(",
    );
    let finally_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_finally(",
        "fn emit_run_async_continuation_job(",
    );

    assert!(catch_builtin.contains("emit_value_to_current_function_realm_object_locals("));
    assert!(!catch_builtin.contains("self.emit_value_to_object_locals("));
    for builtin in [catch_builtin, finally_builtin] {
        assert_eq!(
            builtin
                .matches("emit_validate_promise_prototype_then_invocation(")
                .count(),
            1
        );
        assert_eq!(
            builtin
                .matches("emit_call_validated_promise_prototype_then_invocation(")
                .count(),
            1
        );
        assert!(!builtin.contains("self.emit_function_or_proxy_call_leave_throw_completion("));

        let read = builtin.find("self.emit_object_read(").unwrap();
        let completion = builtin[read..]
            .find("self.emit_return_current_completion_if_throw(function)")
            .unwrap()
            + read;
        let validation = builtin
            .find("self.emit_validate_promise_prototype_then_invocation(")
            .unwrap();
        let call = builtin
            .find("self.emit_call_validated_promise_prototype_then_invocation(")
            .unwrap();
        assert!(read < completion);
        assert!(completion < validation);
        assert!(validation < call);
    }

    let realm_aware_to_object = between(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_value_to_current_function_realm_object_locals(",
        "pub(crate) fn emit_to_integer_or_infinity_number_payload_from_number_payload(",
    );
    assert!(realm_aware_to_object.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert!(realm_aware_to_object.contains("HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET"));
    assert!(realm_aware_to_object.contains("emit_throw_current_function_realm_type_error("));
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
