const FUNCTIONS_SOURCE: &str = include_str!("../../src/functions.rs");

const PROTOTYPE_OWNER_SOURCE: &str =
    include_str!("../../src/functions/current_function_realm_array_prototype.rs");

const PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_combinator_element_materialization.rs");

const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");

const PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_settlement_record_allocation.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str = include_str!(
    "../../../lila-cli/tests/fixtures/wasm_promise_callback_created_allocation_realm.js"
);

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

#[test]
fn settlement_record_context_requires_the_self_backed_callback_object_intrinsic() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_settlement_record_allocation;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_settlement_record_allocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_settlement_record_allocation::"));
    assert!(!PROMISE_SOURCE.contains("PromiseSettlementRecordAllocationContext"));
    let declaration = between(
        PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct PromiseSettlementRecordAllocationContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_SETTLEMENT_RECORD_ALLOCATION_SOURCE.contains(&format!(
                "impl {capability} for PromiseSettlementRecordAllocationContext"
            ))
        );
    }
}

#[test]
fn combinator_materialization_propagates_the_aggregate_error_snapshot() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_combinator_element_materialization;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_combinator_element_materialization;"));
    assert!(!PROMISE_SOURCE.contains("promise_combinator_element_materialization::"));
    assert!(!PROMISE_SOURCE.contains("PromiseCombinatorElementFunctionMaterializationContext"));
    let declaration = between(
        PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration
        .contains("pub(super) struct PromiseCombinatorElementFunctionMaterializationContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_COMBINATOR_ELEMENT_MATERIALIZATION_SOURCE.contains(&format!(
                "impl {capability} for PromiseCombinatorElementFunctionMaterializationContext"
            ))
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
    assert!(!PROTOTYPE_OWNER_SOURCE.contains(
        "#[derive(Clone, Copy)]\npub(crate) struct CurrentFunctionRealmArrayPrototypeLocal"
    ));
}
