const IR_SOURCE: &str = include_str!("../../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering/for_of.rs");
const PROTOCOL_SOURCE: &str = include_str!("../../../lila-ir/src/lowering/for_of/protocol.rs");
const OBLIGATIONS_SOURCE: &str = include_str!("../../../lila-ir/src/iterator_obligations.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../../src/planning.rs");

#[test]
fn immediate_string_for_of_has_no_code_point_walk_ir_backend_or_witness() {
    assert!(!IR_SOURCE.contains("    ForOfString {"));
    assert!(!LOWERING_SOURCE.contains("StatementIr::ForOfString"));
    assert!(!CONTROL_FLOW_SOURCE.contains("StatementIr::ForOfString"));
    assert!(!CONTROL_FLOW_SOURCE.contains("fn compile_for_of_string("));
    assert!(!PLANNING_SOURCE.contains("StatementIr::ForOfString"));
    assert!(!OBLIGATIONS_SOURCE.contains("STRING_CODE_POINT_WALK"));
    assert!(!OBLIGATIONS_SOURCE.contains("StringIteratorIntact"));
    assert!(!OBLIGATIONS_SOURCE.contains("StringWalkIsCodePoint"));
}

#[test]
fn generic_string_values_are_dynamic_and_iterator_lookup_boxes_in_the_current_realm() {
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
fn directly_awaiting_string_loop_bodies_use_the_resumable_sync_protocol() {
    assert!(!LOWERING_SOURCE.contains("NonArrayIterable"));
    assert!(!LOWERING_SOURCE.contains("lower_async_for_of_array_with_body_await"));
    assert!(!LOWERING_SOURCE.contains("AsyncForOfArrayWalkForm"));
    assert!(IR_SOURCE.contains("    AsyncFunctionForOfIterator {"));
    assert!(PROTOCOL_SOURCE.contains("StatementIr::AsyncFunctionForOfIterator"));
    assert!(OBLIGATIONS_SOURCE.contains("RESUMABLE_SYNC_ITERATOR_PROTOCOL"));
}
