const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/for_of.rs");
const PROTOCOL_SOURCE: &str = include_str!("../../lila-ir/src/lowering/for_of/protocol.rs");
const OBLIGATIONS_SOURCE: &str = include_str!("../../lila-ir/src/iterator_obligations.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");

#[test]
fn immediate_array_for_of_has_no_index_walk_ir_or_backend() {
    assert!(!IR_SOURCE.contains("    ForOfArray {"));
    assert!(!LOWERING_SOURCE.contains("StatementIr::ForOfArray"));
    assert!(!CONTROL_FLOW_SOURCE.contains("StatementIr::ForOfArray"));
    assert!(!CONTROL_FLOW_SOURCE.contains("fn compile_for_of_array("));
    assert!(!PLANNING_SOURCE.contains("StatementIr::ForOfArray"));
    assert!(!OBLIGATIONS_SOURCE.contains("ARRAY_INDEX_WALK =>"));
}

#[test]
fn every_array_iterator_path_uses_dynamic_protocol_values_without_a_kind_gate() {
    assert!(!LOWERING_SOURCE.contains("let iterable_is_array ="));
    assert!(!LOWERING_SOURCE.contains("is_subset_of(KindSet::from_kind(ValueKind::Array))"));
    assert!(!LOWERING_SOURCE.contains("is_subset_of(KindSet::from_kind(ValueKind::String))"));

    let generic_value = LOWERING_SOURCE
        .split_once("// A generic iterator can yield values unrelated to the iterable's")
        .expect("generic iterator value boundary")
        .1
        .split_once("        };")
        .expect("generic iterator value boundary end")
        .0;
    assert!(generic_value.contains("kind: ValueKind::Dynamic"));
    assert!(generic_value.contains("possible_kinds: KindSet::all_runtime_tags()"));
    assert!(generic_value.contains("heap_shape: None"));
    assert!(generic_value.contains("function_targets: FunctionTargetKnowledge::unknown()"));
}

#[test]
fn resumable_array_iteration_has_a_dedicated_sync_protocol_statement() {
    assert!(!OBLIGATIONS_SOURCE.contains("ARRAY_INDEX_WALK_RESUMABLE"));
    assert!(!LOWERING_SOURCE.contains("lower_async_for_of_array_with_body_await"));
    assert!(!LOWERING_SOURCE.contains("AsyncForOfArrayWalkForm"));
    assert!(IR_SOURCE.contains("    AsyncFunctionForOfIterator {"));
    assert!(PROTOCOL_SOURCE.contains("StatementIr::AsyncFunctionForOfIterator"));
    assert!(OBLIGATIONS_SOURCE.contains("RESUMABLE_SYNC_ITERATOR_PROTOCOL"));
}
