const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOUND_FUNCTION_SOURCE: &str = include_str!("../src/functions/bound_function_allocation.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_internal_function_materialization.rs");
const PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_combinator_element_materialization.rs");
const PROMISE_FINALLY_COMPLETION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_finally_completion.rs");
const PROMISE_KEYED_COMBINATOR_MODE_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_keyed_combinator_mode.rs");
const PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_keyed_element_projection.rs");
const PROMISE_RESOLVE_REALM_CONTEXT_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_resolve_realm_context.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

fn promise_internal_context_source() -> &'static str {
    PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
}

#[test]
fn function_layout_has_a_gc_visible_builtin_closure_context() {
    for marker in [
        "pub(crate) const HEAP_FUNCTION_OBJECT_SIZE: u64 = 312;",
        "pub(crate) const HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET: u64 = 304;",
        "name: \"builtin_closure_context\"",
        "offset: HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET",
        "pointer: true",
    ] {
        assert!(
            HEAP_SOURCE.contains(marker),
            "missing heap marker: {marker}"
        );
    }
    assert!(FUNCTIONS_SOURCE.contains("HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,\n        0,"));
    assert!(BOUND_FUNCTION_SOURCE
        .contains("HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,\n            0,"));
}

#[test]
fn promise_internal_function_context_couples_one_realm_and_its_prototypes() {
    let context = promise_internal_context_source();
    let declaration = PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
        .split_once(
            "#[must_use = \"Promise internal function Realm context must be explicitly released\"]",
        )
        .expect("Promise internal function must-use marker")
        .1
        .split_once("}\n\n")
        .expect("Promise internal function context declaration")
        .0;
    assert!(PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE.contains(
        "#[must_use = \"Promise internal function Realm context must be explicitly released\"]"
    ));
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_internal_function_materialization;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_internal_function_materialization;"));
    assert!(!PROMISE_SOURCE.contains("pub(crate) mod promise_internal_function_materialization;"));
    assert_eq!(
        PROMISE_SOURCE
            .matches(concat!(
                "use self::promise_internal_function_materialization::",
                "PromiseInternalFunctionMaterializationContext;",
            ))
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub use self::promise_internal_function_materialization"));
    assert!(PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
        .contains("pub(super) struct PromiseInternalFunctionMaterializationContext {"));
    for field in [
        "realm_local: u32",
        "function_prototype_local: u32",
        "type_error_prototype_local: u32",
        "range_error_prototype_local: u32",
    ] {
        assert!(context.contains(field), "missing context field: {field}");
    }
    assert!(!declaration.contains("derive("));
    assert!(!declaration.contains("pub realm_local"));
    assert!(!declaration.contains("pub function_prototype_local"));
    assert!(!declaration.contains("pub type_error_prototype_local"));
    assert!(!declaration.contains("pub range_error_prototype_local"));
    for intrinsic in [
        "HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET",
        "HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            context.contains(intrinsic),
            "missing intrinsic: {intrinsic}"
        );
    }
    assert!(context.contains("fn release_promise_internal_function_materialization_context("));
    assert_eq!(
        PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
            .matches("PromiseInternalFunctionMaterializationContext")
            .count(),
        8,
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("PromiseInternalFunctionMaterializationContext")
            .count(),
        1,
        "the parent retains only the private import",
    );
    assert!(!PROMISE_RESOLVE_REALM_CONTEXT_SOURCE.contains("materialization_context.realm_local"));

    let from_realm = context
        .split_once("fn emit_promise_internal_function_materialization_context_from_realm(")
        .expect("Realm context factory")
        .1
        .split_once("fn emit_current_function_promise_internal_function_materialization_context(")
        .expect("Realm context factory end")
        .0;
    assert!(
        from_realm.find("let function_prototype_local").unwrap()
            < from_realm.find("let intrinsics_local").unwrap()
    );
    assert!(
        from_realm
            .find("release_temp_local(intrinsics_local)")
            .unwrap()
            < from_realm
                .rfind("PromiseInternalFunctionMaterializationContext {")
                .unwrap()
    );
}

#[test]
fn materialization_factories_have_static_realm_authority() {
    let context = promise_internal_context_source();
    let current_function_factory = context
        .split_once("fn emit_current_function_promise_internal_function_materialization_context(")
        .expect("current-function factory")
        .1
        .split_once("fn emit_promise_record_internal_function_materialization_context(")
        .expect("current-function factory end")
        .0;
    assert!(current_function_factory.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(current_function_factory.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert!(!current_function_factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(
        current_function_factory.find("let realm_local").unwrap()
            < current_function_factory
                .find("let active_function_local")
                .unwrap()
    );
    assert!(
        current_function_factory
            .find("release_temp_local(active_function_local)")
            .unwrap()
            < current_function_factory
                .find("emit_promise_internal_function_materialization_context_from_realm(")
                .unwrap()
    );

    let promise_record_factory = context
        .split_once("fn emit_promise_record_internal_function_materialization_context(")
        .expect("Promise-record factory")
        .1
        .split_once("fn emit_promise_internal_function_value(")
        .expect("Promise-record factory end")
        .0;
    assert!(promise_record_factory.contains("HEAP_PROMISE_REALM_OFFSET"));
    assert!(!promise_record_factory.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let realm_intrinsics = context
        .split_once("fn emit_load_promise_internal_function_realm_intrinsics(")
        .expect("Realm-intrinsics capability")
        .1
        .split_once("fn release_promise_internal_function_materialization_context(")
        .expect("Realm-intrinsics capability end")
        .0;
    assert!(realm_intrinsics.contains("context: &PromiseInternalFunctionMaterializationContext"));
    assert!(realm_intrinsics.contains("context.realm_local"));
    assert!(realm_intrinsics.contains("HEAP_REALM_INTRINSICS_OFFSET"));
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("emit_load_promise_internal_function_realm_intrinsics(")
            .count(),
        1,
    );
}

#[test]
fn all_escaping_promise_closures_use_the_typed_materializer() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        4,
        "the parent retains four materialization sites"
    );
    assert_eq!(
        PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        1,
        "the private child owns the materializer definition",
    );
    assert_eq!(
        PROMISE_RESOLVE_REALM_CONTEXT_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        2,
        "the PromiseResolve Realm-context owner materializes both local resolve functions"
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        1,
        "the standard combinator element materializer remains the eighth escaping closure site"
    );
    assert_eq!(
        PROMISE_FINALLY_COMPLETION_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        1,
        "the finally-completion owner materializes its escaping closure"
    );
    assert_eq!(
        PROMISE_KEYED_COMBINATOR_MODE_SOURCE
            .matches("emit_promise_internal_function_value(")
            .count(),
        2,
        "the keyed-combinator owner materializes both escaping closures"
    );
    assert_eq!(
        PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
            .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
            .count(),
        1,
        "only the typed materializer may write the environment handle"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_load_promise_internal_function_context(")
            .count(),
        5,
        "the parent retains five callback body loads"
    );
    assert_eq!(
        PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
            .matches("emit_load_promise_internal_function_context(")
            .count(),
        1,
        "the private child owns the closure-context loader definition",
    );
    assert_eq!(
        PROMISE_FINALLY_COMPLETION_SOURCE
            .matches("emit_load_promise_internal_function_context(")
            .count(),
        2,
        "the finally-completion owner loads both callback contexts"
    );
    assert_eq!(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE
            .matches("emit_load_promise_internal_function_context(")
            .count(),
        1,
        "the keyed-element owner loads its callback context"
    );
    for source in [
        PROMISE_SOURCE,
        PROMISE_FINALLY_COMPLETION_SOURCE,
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
    ] {
        assert!(!source.contains("Instruction::LocalGet(0)"));
    }
    assert_eq!(
        PROMISE_INTERNAL_FUNCTION_MATERIALIZATION_SOURCE
            .matches("emit_function_value_payload(")
            .count(),
        1,
        "every Promise function, including nonescaping PromiseResolve operations, uses the typed materializer"
    );
    for builtin in [
        "PromiseResolveFunction",
        "PromiseRejectFunction",
        "PromiseCapabilityExecutor",
        "PromiseThenFinally",
        "PromiseCatchFinally",
        "PromiseValueThunk",
        "PromiseThrower",
        "PromiseAllKeyedResolveElement",
        "PromiseAllSettledKeyedResolveElement",
        "PromiseAllSettledKeyedRejectElement",
        "PromiseAllResolveElement",
        "PromiseAllSettledResolveElement",
        "PromiseAllSettledRejectElement",
        "PromiseAnyRejectElement",
    ] {
        assert!(
            [
                PROMISE_SOURCE,
                PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE,
                PROMISE_FINALLY_COMPLETION_SOURCE,
                PROMISE_KEYED_COMBINATOR_MODE_SOURCE,
                PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE,
            ]
            .iter()
            .any(|source| source.contains(builtin)),
            "missing closure: {builtin}"
        );
    }
}

#[test]
fn promise_internal_errors_and_job_realm_use_function_owned_authority() {
    let capability = PROMISE_SOURCE
        .split_once("pub(crate) fn emit_promise_capability_executor(")
        .expect("capability executor")
        .1
        .split_once("pub(crate) fn emit_promise_static_settle(")
        .expect("capability executor end")
        .0;
    assert!(capability.contains("emit_throw_current_function_realm_type_error("));
    assert!(!capability.contains("emit_throw_runtime_error("));

    let resolution = PROMISE_SOURCE
        .split_once("pub(crate) fn emit_resolve_promise_record(")
        .expect("Promise resolution")
        .1
        .split_once("pub(crate) fn emit_settle_promise_record(")
        .expect("Promise resolution end")
        .0;
    assert!(resolution.contains("HEAP_PROMISE_REALM_OFFSET"));
    assert!(resolution.contains("HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET"));
    assert!(resolution.contains("emit_throw_runtime_error_with_prototype_local("));

    assert!(PROMISE_SOURCE.contains("self.emit_get_function_realm("));
    assert!(PROMISE_SOURCE.contains("FunctionRealmRevokedRoute::UseCurrentRealm"));
}

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
