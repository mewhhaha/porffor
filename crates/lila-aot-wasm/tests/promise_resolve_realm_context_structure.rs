use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_RESOLVE_REALM_CONTEXT_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_resolve_realm_context.rs");
const PROMISE_FINALLY_COMPLETION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_finally_completion.rs");
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
fn promise_resolve_contexts_are_private_noncopyable_and_must_use() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_resolve_realm_context;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_resolve_realm_context;"));
    assert!(!PROMISE_SOURCE.contains("promise_resolve_realm_context::"));
    assert!(!PROMISE_SOURCE.contains("PromiseResolveOperationRealmContext"));
    assert!(!PROMISE_SOURCE.contains("IntrinsicPromiseResolveRealmContext"));
    let declarations = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "#[must_use = \"PromiseResolve operation Realm context must be explicitly released\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    for marker in [
        "pub(super) struct PromiseResolveOperationRealmContext",
        "resolve_function_payload_local: u32",
        "#[must_use = \"intrinsic PromiseResolve Realm context must be explicitly released\"]",
        "pub(super) struct IntrinsicPromiseResolveRealmContext",
        "operation: PromiseResolveOperationRealmContext",
        "constructor_payload_local: u32",
    ] {
        assert!(
            declarations.contains(marker),
            "missing declaration: {marker}"
        );
    }
    assert!(!declarations.contains("pub(crate) struct PromiseResolve"));
    assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains(&format!(
            "impl {capability} for PromiseResolveOperationRealmContext"
        )));
        assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains(&format!(
            "impl {capability} for IntrinsicPromiseResolveRealmContext"
        )));
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_resolve_realm_context::"),
        0,
        "the PromiseResolve Realm-context owner must have no import or re-export",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseResolveOperationRealmContext"),
        7,
        "the private child must own every operation-context type use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "IntrinsicPromiseResolveRealmContext"),
        6,
        "the private child must own every intrinsic-context type use",
    );
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("Ok(PromiseResolveOperationRealmContext {")
            .count(),
        1,
    );
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("Ok(IntrinsicPromiseResolveRealmContext {")
            .count(),
        1,
    );
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("context.resolve_function_payload_local")
            .count(),
        2,
    );
    let intrinsic_context_projections = PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
        .matches("context.constructor_payload_local")
        .count()
        - PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("resolve_context.constructor_payload_local")
            .count();
    assert_eq!(intrinsic_context_projections, 2);
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("resolve_context.constructor_payload_local")
            .count(),
        1,
    );
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("context.operation")
            .count(),
        2,
    );
}

#[test]
fn intrinsic_context_pairs_the_resolve_function_and_constructor_from_one_catalog() {
    let factories = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "fn emit_promise_resolve_internal_function_materialization_context(",
        "pub(super) fn emit_call_promise_resolve_operation(",
    );
    for marker in [
        "PromiseResolveRealmAuthority::CurrentFunction",
        "PromiseResolveRealmAuthority::AsyncExecution(realm)",
        "emit_current_function_promise_internal_function_materialization_context(function)",
        "emit_promise_internal_function_materialization_context_from_realm(",
        "emit_load_promise_internal_function_realm_intrinsics(",
        "HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET",
        "emit_promise_internal_function_value(",
        "IntrinsicPromiseResolveRealmContext",
    ] {
        assert!(
            factories.contains(marker),
            "missing factory route: {marker}"
        );
    }
    assert!(!factories.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factories.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(!factories.contains("materialization_context.realm_local"));
    assert_eq!(factories.matches("Instruction::Unreachable").count(), 2);

    let intrinsic_factory = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "fn emit_intrinsic_promise_resolve_realm_context(",
        "pub(super) fn emit_call_promise_resolve_operation(",
    );
    let resolve_reservation = intrinsic_factory
        .find("let resolve_function_payload_local")
        .unwrap();
    let constructor_reservation = intrinsic_factory
        .find("let constructor_payload_local")
        .unwrap();
    let intrinsics_reservation = intrinsic_factory.find("let intrinsics_local").unwrap();
    let materialization = intrinsic_factory
        .find("emit_promise_resolve_internal_function_materialization_context(")
        .unwrap();
    assert!(resolve_reservation < constructor_reservation);
    assert!(constructor_reservation < intrinsics_reservation);
    assert!(intrinsics_reservation < materialization);
    assert!(
        intrinsic_factory
            .find("release_promise_internal_function_materialization_context(")
            .unwrap()
            < intrinsic_factory
                .find("release_temp_local(intrinsics_local)")
                .unwrap()
    );

    let releases = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "fn release_promise_resolve_operation_realm_context(",
        "pub(super) fn emit_intrinsic_promise_resolve_to_locals(",
    );
    assert!(
        releases
            .find("release_temp_local(context.constructor_payload_local)")
            .unwrap()
            < releases
                .find("release_promise_resolve_operation_realm_context(context.operation)")
                .unwrap()
    );
}

#[test]
fn await_and_finally_consume_explicit_promise_resolve_realm_authority() {
    let intrinsic_call = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "fn emit_intrinsic_promise_resolve_to_locals(",
        "pub(super) fn emit_new_intrinsic_promise_resolve_rejection_capability(",
    );
    assert!(intrinsic_call.contains("context: &IntrinsicPromiseResolveRealmContext"));
    assert!(intrinsic_call.contains("&context.operation"));
    assert!(intrinsic_call.contains("context.constructor_payload_local"));
    assert!(!intrinsic_call.contains("emit_function_value_payload("));
    assert!(!intrinsic_call.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));

    let rejection_capability = between(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE,
        "fn emit_new_intrinsic_promise_resolve_rejection_capability(",
        "\n    }\n}",
    );
    assert!(rejection_capability.contains("resolve_context.constructor_payload_local"));
    assert_eq!(
        rejection_capability
            .matches("emit_new_promise_capability(")
            .count(),
        1,
    );

    let await_reactions = between(
        PROMISE_SOURCE,
        "fn emit_intrinsic_await_reactions(",
        "pub(crate) fn emit_async_generator_await_return_reactions(",
    );
    let context_factory = await_reactions
        .find("let resolve_realm_authority = match &initialization {")
        .unwrap();
    let resolve_call = await_reactions
        .find("emit_intrinsic_promise_resolve_to_locals(")
        .unwrap();
    let abrupt_constructor = await_reactions
        .find("emit_new_intrinsic_promise_resolve_rejection_capability(")
        .unwrap();
    let context_release = await_reactions
        .find("release_intrinsic_promise_resolve_realm_context(resolve_context)")
        .unwrap();
    assert!(context_factory < resolve_call);
    assert!(resolve_call < abrupt_constructor);
    assert!(abrupt_constructor < context_release);
    assert!(!await_reactions.contains("resolve_context.constructor_payload_local"));
    assert!(!await_reactions.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));

    let generator_return = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_async_generator_await_return_reactions(",
        "pub(crate) fn emit_promise_prototype_then(",
    );
    let realm_factory = generator_return
        .find("emit_async_generator_execution_realm_context_from_activation(")
        .unwrap();
    let resolve_authority = generator_return
        .find("PromiseResolveRealmAuthority::AsyncExecution(&realm)")
        .unwrap();
    let fulfill_reaction = generator_return
        .find("emit_initialize_async_execution_promise_reaction(")
        .unwrap();
    assert!(realm_factory < resolve_authority);
    assert!(resolve_authority < fulfill_reaction);
    assert_eq!(
        generator_return
            .matches("release_async_execution_realm_context(realm)")
            .count(),
        1
    );

    let finally_continuation = between(
        PROMISE_FINALLY_COMPLETION_SOURCE,
        "fn emit_promise_finally_continuation(",
        "pub(crate) fn emit_promise_value_thunk(",
    );
    let cleanup_call = finally_continuation
        .find("emit_function_or_proxy_call_leave_throw_completion(")
        .unwrap();
    let resolve_factory = finally_continuation
        .find("emit_promise_resolve_operation_realm_context(")
        .unwrap();
    let resolve_release = finally_continuation
        .find("release_promise_resolve_operation_realm_context(resolve_context)")
        .unwrap();
    assert!(cleanup_call < resolve_factory);
    assert!(resolve_factory < resolve_release);
    assert!(finally_continuation.contains("PromiseResolveRealmAuthority::CurrentFunction"));
    assert!(!finally_continuation.contains("emit_function_value_payload("));
}

#[test]
fn promise_resolve_capability_errors_use_the_operation_function_realm() {
    let capability = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_new_promise_capability(",
        "fn emit_initialize_promise_reaction(",
    );
    assert_eq!(
        capability
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2
    );
    assert!(!capability.contains("self.emit_throw_runtime_error("));

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
