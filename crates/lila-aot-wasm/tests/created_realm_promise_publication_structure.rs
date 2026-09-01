const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const ORDINARY_PROTOTYPES_SOURCE: &str =
    include_str!("../src/functions/required_resolved_realm_ordinary_prototype.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const MAIN_REALM_SOURCE: &str = include_str!("../src/intrinsics/promise.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_created_realm.js");

fn promise_constructor() -> &'static str {
    PROMISE_SOURCE
        .split_once("pub(crate) fn emit_promise_constructor(")
        .expect("Promise constructor")
        .1
        .split_once("pub(crate) fn emit_promise_prototype_then(")
        .expect("Promise constructor end")
        .0
}

fn created_realm_publication() -> &'static str {
    HOST_SOURCE
        .split_once(
            "        self.emit_function_value_payload_in_realm(\n            &promise_meta,",
        )
        .expect("created-Realm Promise publication")
        .1
        .split_once(
            "        self.emit_function_value_payload_in_realm(\n            &iterator_meta,",
        )
        .expect("created-Realm Promise publication end")
        .0
}

#[test]
fn promise_prototype_is_a_required_typed_realm_intrinsic() {
    for marker in [
        "pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 424;",
        "pub(crate) const HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET: u64 = 400;",
        "pub(crate) const HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET: u64 = 408;",
        "pub(crate) const HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET: u64 = 416;",
        "name: \"%Promise.prototype%\"",
        "name: \"%Function.prototype%\"",
        "offset: HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET",
    ] {
        assert!(
            HEAP_SOURCE.contains(marker),
            "missing heap invariant: {marker}"
        );
    }
    assert!(FUNCTIONS_SOURCE.contains("PromisePrototype,"));
    assert!(FUNCTIONS_SOURCE.contains("PromiseConstructor,"));
    assert!(FUNCTIONS_SOURCE
        .contains("Self::PromisePrototype => HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET,"));
    assert!(ORDINARY_PROTOTYPES_SOURCE
        .contains("Self::Promise => HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET,"));
    assert!(BOOTSTRAP_SOURCE.contains(
        "PROMISE_PROTOTYPE_GLOBAL_INDEX,\n            NonArrayRealmIntrinsicSlot::PromisePrototype,"
    ));
    assert!(BOOTSTRAP_SOURCE.contains(
        "FUNCTION_PROTOTYPE_GLOBAL_INDEX,\n            NonArrayRealmIntrinsicSlot::FunctionPrototype,"
    ));
    assert!(HOST_SOURCE.contains(
        "NonArrayRealmIntrinsicSlot::PromisePrototype,\n            promise_prototype_local,"
    ));
    assert!(BOOTSTRAP_SOURCE.contains(
        "PROMISE_CONSTRUCTOR_GLOBAL_INDEX,\n                NonArrayRealmIntrinsicSlot::PromiseConstructor,"
    ));
    assert!(HOST_SOURCE.contains(
        "NonArrayRealmIntrinsicSlot::PromiseConstructor,\n            promise_constructor_local,"
    ));
    assert!(HOST_SOURCE
        .contains("self.emit_store_realm_function_prototype(&realm_functions, function);"));

    let constructor = promise_constructor();
    assert!(constructor.contains(
        "NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(\n                OrdinaryDefaultPrototype::Promise,"
    ));
    assert!(!constructor.contains("NewTargetPrototypeFallback::CurrentGlobal"));
    assert!(!constructor.contains("NewTargetPrototypeFallback::RealmIntrinsic"));
}

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
        .split_once("impl<'a> FunctionBuilder<'a>")
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
    for source in [MAIN_REALM_SOURCE, HOST_SOURCE] {
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
fn created_realm_promise_callables_capture_identity_and_error_realms() {
    let publication = created_realm_publication();

    assert_eq!(
        publication
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        3
    );
    for binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(publication.matches(binding).count(), 4, "{binding}");
    }
    assert!(publication.contains(
        "promise_constructor_local,\n            promise_prototype_local,\n            false,\n            false,\n            false,\n            true,"
    ));
    assert!(publication.contains("promise_prototype_local,\n            \"Symbol.toStringTag\","));
    assert!(publication.contains("self.emit_object_define_accessor("));
    assert!(HOST_SOURCE.contains(
        "global_local,\n            PROMISE_NAME,\n            promise_constructor_local,"
    ));
}

#[test]
fn promise_allocation_context_owns_realm_and_prototype_selection() {
    let context = PROMISE_SOURCE
        .split_once("pub(crate) struct PromiseAllocationContext")
        .expect("Promise allocation context")
        .1
        .split_once("pub(crate) fn emit_alloc_promise_with_prototype(")
        .expect("Promise allocation context end")
        .0;
    assert!(context.contains("prototype_local: u32"));
    assert!(context.contains("realm_local: u32"));
    assert!(PROMISE_SOURCE.contains(
        "#[must_use = \"Promise allocation context must be consumed by Promise allocation\"]"
    ));
    assert!(context.contains("HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET"));
    assert!(context.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert_eq!(
        PROMISE_SOURCE
            .matches("\n        PromiseAllocationContext {\n")
            .count(),
        3
    );
    assert!(context.contains(
        "pub(crate) fn emit_current_function_realm_intrinsic_promise_allocation_context("
    ));
    let constructor_context = context
        .split_once("fn emit_current_function_realm_promise_allocation_context(")
        .expect("Promise constructor allocation context")
        .1;
    assert!(constructor_context.contains("GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX)"));
    assert!(!constructor_context.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let allocator = PROMISE_SOURCE
        .split_once("pub(crate) fn emit_alloc_promise_with_prototype(")
        .expect("Promise allocator")
        .1
        .split_once("fn emit_create_promise_resolving_functions(")
        .expect("Promise allocator end")
        .0;
    assert!(allocator.contains("context: PromiseAllocationContext"));
    assert!(allocator.contains("context.prototype_local"));
    assert!(allocator.contains("context.realm_local"));
    assert!(!allocator.contains("CURRENT_REALM_GLOBAL_INDEX"));
    let allocation_sources = [
        PROMISE_SOURCE,
        FUNCTIONS_SOURCE,
        include_str!("../src/control_flow.rs"),
        include_str!("../src/builtins/atomics.rs"),
        include_str!("../src/builtins/atomics/wait_async_result.rs"),
    ];
    assert_eq!(
        allocation_sources
            .iter()
            .map(|source| source.matches("emit_alloc_promise_with_prototype(").count())
            .sum::<usize>(),
        7
    );
    assert_eq!(
        allocation_sources
            .iter()
            .map(|source| source.matches("promise_allocation_context,").count())
            .sum::<usize>(),
        6
    );

    let resolving_functions = PROMISE_SOURCE
        .split_once("fn emit_create_promise_resolving_functions(")
        .expect("Promise resolving functions")
        .1
        .split_once("fn emit_enqueue_promise_reaction_list(")
        .expect("Promise resolving functions end")
        .0;
    assert!(resolving_functions
        .contains("emit_promise_record_internal_function_materialization_context("));
    assert!(resolving_functions.contains("emit_promise_internal_function_value("));
    assert!(!resolving_functions.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
    assert!(!resolving_functions.contains("emit_function_value_payload("));
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
