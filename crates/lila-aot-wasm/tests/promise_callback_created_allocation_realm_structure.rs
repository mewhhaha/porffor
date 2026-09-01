use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_combinator_element_materialization.rs");
const PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_keyed_element_projection.rs");
const PROMISE_INTERNAL_FUNCTION_REALM_CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/promise-internal-function-realm-context.md");
const MODULARITY_TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_settlement_record_allocation.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const PROTOTYPE_OWNER_SOURCE: &str =
    include_str!("../src/functions/current_function_realm_array_prototype.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const ERROR_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const PROMISE_ANY_ERROR_SOURCE: &str = include_str!("../src/builtins/errors/promise_any.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_callback_created_allocation_realm.js");

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
fn settlement_record_context_requires_the_self_backed_callback_object_intrinsic() {
    let context_type = between(
        PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE,
        "#[must_use = \"Promise settlement record allocation context must be consumed\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    let context = PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE;
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_settlement_record_allocation;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_settlement_record_allocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_settlement_record_allocation::"));
    assert!(!PROMISE_SOURCE.contains("PromiseSettlementRecordAllocationContext"));
    assert!(context_type.contains("pub(super) struct PromiseSettlementRecordAllocationContext"));
    assert!(context_type.contains("prototype_local: u32,"));
    for marker in [
        "#[must_use = \"Promise settlement record allocation context must be consumed\"]",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET",
        "emit_alloc_promise_settlement_record(",
    ] {
        assert!(context.contains(marker), "missing marker: {marker}");
    }
    assert!(!context_type.contains("derive(Clone"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !context.contains(&format!(
                "impl {capability} for PromiseSettlementRecordAllocationContext"
            )),
            "PromiseSettlementRecordAllocationContext must not acquire manual {capability}",
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseSettlementRecordAllocationContext"),
        4,
        "the private child must own every settlement-record allocation context use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_settlement_record_allocation::"),
        0,
        "the settlement-record allocation owner must have no import or re-export",
    );
    assert_eq!(
        context
            .matches("PromiseSettlementRecordAllocationContext { prototype_local }")
            .count(),
        1,
        "only the private child may construct a settlement-record allocation context",
    );
    assert_eq!(
        context.matches("context.prototype_local").count(),
        2,
        "only the consuming child allocator may project and release the prototype local",
    );
    assert!(!context.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!context.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));

    assert!(
        context.find("let prototype_local").unwrap() < context.find("let realm_local").unwrap()
    );
    assert!(
        context.find("let realm_local").unwrap() < context.find("let intrinsics_local").unwrap()
    );
    assert!(
        context
            .find("release_temp_local(intrinsics_local)")
            .unwrap()
            < context.find("release_temp_local(realm_local)").unwrap()
    );
    assert!(
        context
            .find("emit_alloc_plain_object_with_prototype")
            .unwrap()
            < context
                .rfind("release_temp_local(context.prototype_local)")
                .unwrap()
    );
}

#[test]
fn both_all_settled_record_sites_consume_the_typed_context() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_self_backed_promise_settlement_record_allocation_context(")
            .count(),
        1,
        "the parent must retain the standard factory caller",
    );
    assert_eq!(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE
            .matches("emit_self_backed_promise_settlement_record_allocation_context(")
            .count(),
        1,
        "the keyed projection owner must retain the keyed factory caller",
    );
    assert_eq!(
        PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE
            .matches("emit_self_backed_promise_settlement_record_allocation_context(")
            .count(),
        1,
        "the private child must own the sole context factory",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_alloc_promise_settlement_record(")
            .count(),
        1,
        "the parent must retain the standard allocator caller",
    );
    assert_eq!(
        PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE
            .matches("emit_alloc_promise_settlement_record(")
            .count(),
        1,
        "the keyed projection owner must retain the keyed allocator caller",
    );
    assert_eq!(
        PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE
            .matches("emit_alloc_promise_settlement_record(")
            .count(),
        1,
        "the private child must own the sole consuming allocator",
    );

    let keyed = PROMISE_KEYED_ELEMENT_PROJECTION_SOURCE;
    let standard = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_all_settled_element(",
        "pub(crate) fn emit_promise_any_reject_element(",
    );
    for (name, callback) in [("keyed", keyed), ("standard", standard)] {
        assert!(callback.contains("emit_alloc_promise_settlement_record("));
        assert!(
            !callback.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"),
            "{name}"
        );
        assert!(callback.find("\"status\"").unwrap() < callback.rfind("result_property").unwrap());
        assert!(callback.matches("true,").count() >= 6, "{name}");
    }
}

#[test]
fn promise_any_context_closes_callback_and_constructor_fallback_authority() {
    let context_type = between(
        ERROR_SOURCE,
        "struct PromiseAnyAggregateErrorAllocationContext",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(ERROR_SOURCE.contains(
        "#[must_use = \"Promise.any AggregateError allocation context must be consumed\"]"
    ));
    assert!(!context_type.contains("derive(Clone"));
    assert!(!PROMISE_ANY_ERROR_SOURCE.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!ERROR_SOURCE.contains("emit_promise_any_aggregate_error_from_locals("));
    assert!(!PROMISE_ANY_ERROR_SOURCE.contains("emit_promise_any_aggregate_error_from_locals("));

    let strict = between(
        PROMISE_ANY_ERROR_SOURCE,
        "emit_self_backed_promise_any_aggregate_error_allocation_context(",
        "emit_promise_combinator_aggregate_error_allocation_context(",
    );
    assert!(strict.contains("self.current_env_local"));
    assert!(strict.contains("Instruction::Unreachable"));
    assert!(strict.contains("HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET"));
    assert!(!strict.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));

    let fallback = between(
        PROMISE_ANY_ERROR_SOURCE,
        "emit_promise_combinator_aggregate_error_allocation_context(",
        "emit_promise_any_aggregate_error_from_context(",
    );
    assert!(fallback.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(
        fallback.find("let prototype_local").unwrap()
            < fallback.find("let active_function_local").unwrap()
    );
    assert!(fallback.contains("release_temp_local(active_function_local)"));

    let consumer = PROMISE_ANY_ERROR_SOURCE
        .split_once("emit_promise_any_aggregate_error_from_context(")
        .expect("AggregateError consumer")
        .1;
    assert!(consumer.contains("context: PromiseAnyAggregateErrorAllocationContext"));
    assert!(
        consumer
            .find("emit_finish_aggregate_error_instance(")
            .unwrap()
            < consumer
                .find("release_temp_local(context.prototype_local)")
                .unwrap()
    );
}

#[test]
fn combinator_materialization_propagates_the_aggregate_error_snapshot() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_combinator_element_materialization;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_combinator_element_materialization;"));
    assert!(!PROMISE_SOURCE.contains("promise_combinator_element_materialization::"));
    assert!(!PROMISE_SOURCE.contains("PromiseCombinatorElementFunctionMaterializationContext"));
    assert!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .lines()
            .count()
            <= 90
    );
    assert!(PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
        .contains("pub(super) struct PromiseCombinatorElementFunctionMaterializationContext {"));
    assert!(!PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
        .contains("pub(crate) struct PromiseCombinatorElementFunctionMaterializationContext"));
    assert_eq!(
        between(
            PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE,
            "pub(super) struct PromiseCombinatorElementFunctionMaterializationContext {",
            "\n}\n\nimpl<'a> FunctionBuilder<'a>",
        ),
        concat!(
            "\n    internal: PromiseInternalFunctionMaterializationContext,",
            "\n    aggregate_error_prototype_local: u32,",
        ),
    );
    assert!(!PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE.contains(&format!(
                "impl {capability} for PromiseCombinatorElementFunctionMaterializationContext"
            ))
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "PromiseCombinatorElementFunctionMaterializationContext",
        ),
        5,
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_combinator_element_materialization::"),
        0,
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches(
                "PromiseCombinatorElementFunctionMaterializationContext {\n            internal,",
            )
            .count(),
        1,
    );
    for method in [
        "emit_current_function_promise_combinator_element_materialization_context",
        "emit_promise_combinator_element_function_value",
        "release_promise_combinator_element_function_materialization_context",
    ] {
        assert_eq!(
            PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
                .matches(&format!("pub(super) fn {method}("))
                .count(),
            1,
        );
    }
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("emit_current_function_promise_combinator_element_materialization_context(")
            .count(),
        1,
        "one child-owned factory"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_current_function_promise_combinator_element_materialization_context(")
            .count(),
        1,
        "one standard combinator factory call"
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("emit_promise_combinator_element_function_value(")
            .count(),
        1,
        "one child-owned materializer"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_promise_combinator_element_function_value(")
            .count(),
        2,
        "the resolve/reject element materializer calls"
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("release_promise_combinator_element_function_materialization_context(")
            .count(),
        1,
        "one child-owned consuming release"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("release_promise_combinator_element_function_materialization_context(")
            .count(),
        1,
        "one standard combinator release call"
    );
    assert!(!PROMISE_SOURCE.contains("AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX"));

    let wrapper = between(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE,
        "emit_current_function_promise_combinator_element_materialization_context(",
        "pub(super) fn emit_promise_combinator_element_function_value(",
    );
    assert!(wrapper.contains("HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET"));
    assert!(!wrapper.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(
        wrapper.find("let aggregate_error_prototype_local").unwrap()
            < wrapper.find("let internal").unwrap()
    );

    let release = between(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE,
        "release_promise_combinator_element_function_materialization_context(",
        "\n}",
    );
    assert!(
        release
            .find("release_promise_internal_function_materialization_context")
            .unwrap()
            < release
                .find("release_temp_local(context.aggregate_error_prototype_local)")
                .unwrap()
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("context.internal")
            .count(),
        2,
    );
    assert_eq!(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE
            .matches("context.aggregate_error_prototype_local")
            .count(),
        2,
    );
    assert!(PROMISE_INTERNAL_FUNCTION_REALM_CONTRACT
        .contains("PromiseCombinatorElementFunctionMaterializationContext"));
    for text in [PROMISE_INTERNAL_FUNCTION_REALM_CONTRACT, MODULARITY_TASK] {
        assert!(text.contains("promise_callback_created_allocation_realm_structure"));
    }
}

#[test]
fn created_realm_promise_statics_capture_the_aggregate_error_prototype() {
    let publication = between(
        HOST_SOURCE,
        "for builtin in PROMISE_STATIC_METHOD_PUBLICATIONS",
        "let promise_species_key_local",
    );
    for marker in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET",
        "aggregate_error_prototype_local",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            publication.contains(marker),
            "missing publication marker: {marker}"
        );
    }
}

#[test]
fn standard_combinator_outer_array_uses_the_executing_method_realm() {
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod current_function_realm_array_prototype;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("current_function_realm_array_prototype::"));
    assert!(!FUNCTIONS_SOURCE.contains("struct CurrentFunctionRealmArrayPrototypeLocal"));
    assert!(PROTOTYPE_OWNER_SOURCE.contains("struct CurrentFunctionRealmArrayPrototypeLocal(u32)"));
    assert!(!PROTOTYPE_OWNER_SOURCE.contains(
        "#[derive(Clone, Copy)]\npub(crate) struct CurrentFunctionRealmArrayPrototypeLocal"
    ));
    assert_eq!(
        PROTOTYPE_OWNER_SOURCE
            .matches("CurrentFunctionRealmArrayPrototypeLocal(prototype_local)")
            .count(),
        1
    );
    assert_eq!(PROTOTYPE_OWNER_SOURCE.matches("prototype.0").count(), 2);
    for owner_method in [
        "emit_load_current_function_realm_array_prototype",
        "emit_install_current_function_realm_array_prototype",
    ] {
        let definition = format!("pub(crate) fn {owner_method}(");
        assert_eq!(PROTOTYPE_OWNER_SOURCE.matches(&definition).count(), 1);
        assert!(!FUNCTIONS_SOURCE.contains(&definition));
    }

    let allocator = between(
        ARRAY_SOURCE,
        "pub(crate) fn emit_alloc_array_payload_with_length_in_current_function_realm(",
        "pub(crate) fn emit_array_like_snapshot_payload(",
    );
    assert!(allocator.contains("emit_load_current_function_realm_array_prototype(function)"));
    assert!(allocator.contains("emit_install_current_function_realm_array_prototype("));
    assert_eq!(
        allocator
            .matches("emit_load_current_function_realm_array_prototype(function)")
            .count(),
        1
    );
    assert_eq!(
        allocator
            .matches("emit_install_current_function_realm_array_prototype(")
            .count(),
        1
    );
    assert!(
        allocator
            .find("emit_alloc_array_payload_with_length(len_local")
            .unwrap()
            < allocator
                .find("emit_load_current_function_realm_array_prototype(function)")
                .unwrap()
    );

    let combinator = between(
        PROMISE_SOURCE,
        "fn emit_promise_combinator(",
        "pub(crate) fn emit_promise_resolving_function(",
    );
    assert_eq!(
        combinator
            .matches("emit_alloc_array_payload_with_length_in_current_function_realm(")
            .count(),
        1
    );
    assert!(!combinator.contains("emit_alloc_array_payload_with_length(index_local"));
    for mode in [
        "PromiseCombinatorMode::Values",
        "PromiseCombinatorMode::SettledRecords",
        "PromiseCombinatorMode::FirstFulfillment",
    ] {
        assert!(combinator.contains(mode), "missing combinator mode: {mode}");
    }
}

#[test]
fn focused_fixture_covers_all_five_nonblocking_allocation_branches() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_callback_realms_for_promise_created_allocations()"));
    assert!(CLI_TESTS.contains("wasm_promise_callback_created_allocation_realm.js"));
    for marker in [
        "Promise.all result array prototype",
        "allSettled result array prototype",
        "standard fulfilled",
        "standard rejected",
        "keyed fulfilled",
        "keyed rejected",
        "nonempty any error prototype",
        "nonempty any errors array prototype",
        "empty any error prototype",
        "empty any errors array prototype",
        "status,",
        "errors",
        "promise-callback-created-allocation-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(!CLI_FIXTURE.contains("Atomics.wait"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
