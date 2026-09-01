const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/for_of.rs");
const PROTOCOL_SOURCE: &str = include_str!("../../lila-ir/src/lowering/for_of/protocol.rs");
const OBLIGATIONS_SOURCE: &str = include_str!("../../lila-ir/src/iterator_obligations.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

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

    let generic_for_of = between(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for_of_iterator(",
        "    pub(crate) fn compile_object_destructure_to_locals(",
    );
    assert!(generic_for_of.contains("self.emit_value_to_current_function_realm_object_locals("));
    assert!(!generic_for_of.contains("self.emit_value_to_object_locals("));
    let iterator_method_lookup = between(
        generic_for_of,
        "        self.emit_object_read(",
        "        )?;",
    );
    let wrapper_payload = iterator_method_lookup
        .find("iterable_object_payload_local,")
        .expect("String wrapper payload");
    let wrapper_tag = iterator_method_lookup
        .find("iterable_object_tag_local,")
        .expect("String wrapper tag");
    let primitive_payload = iterator_method_lookup
        .find("iterable_payload_local,")
        .expect("primitive String receiver payload");
    let primitive_tag = iterator_method_lookup
        .find("iterable_tag_local,")
        .expect("primitive String receiver tag");
    assert!(wrapper_payload < wrapper_tag);
    assert!(wrapper_tag < primitive_payload);
    assert!(primitive_payload < primitive_tag);
    let current_realm_boxing = between(
        OPERATIONS_SOURCE,
        "    pub(crate) fn emit_value_to_current_function_realm_object_locals(",
        "    pub(crate) fn emit_to_integer_or_infinity_number_payload_from_number_payload(",
    );
    assert!(current_realm_boxing.contains("HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET"));
    assert!(current_realm_boxing.contains("self.emit_load_realm_intrinsic_prototype_or_global("));
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
