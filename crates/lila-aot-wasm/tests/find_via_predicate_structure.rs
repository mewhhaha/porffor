use std::collections::BTreeSet;

const ARRAY: &str = include_str!("../src/builtins/array.rs");
const FIND: &str = include_str!("../src/builtins/array/find_via_predicate.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const REGRESSIONS: &str = include_str!("../../lila-engine/tests/aot_array_find.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end: {end}"))
        .0
}

fn normalized(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn in_order(source: &str, operations: &[&str]) {
    let mut remaining = source;
    for operation in operations {
        remaining = remaining
            .split_once(operation)
            .unwrap_or_else(|| panic!("missing or out-of-order operation: {operation}"))
            .1;
    }
}

fn array_entry() -> &'static str {
    FIND.split_once("    fn compile_array_find_with_kind(")
        .expect("generic Array entry")
        .1
}

fn typed_entry() -> &'static str {
    bounded(
        FIND,
        "    fn compile_typed_array_find_with_kind(",
        "    pub(in crate::builtins) fn compile_array_prototype_find_builtin(",
    )
}

#[test]
fn closed_find_policy_has_one_private_owner_and_eight_fixed_entries() {
    assert_eq!(ARRAY.matches("\nmod find_via_predicate;\n").count(), 1);
    assert!(!ARRAY.contains("FindViaPredicateKind"));
    for (name, variants) in [
        ("FindViaPredicateKind", "Find,FindIndex,FindLast,FindLastIndex,"),
        ("FindDirection", "Ascending,Descending,"),
        ("FindProjection", "Value,Index,"),
    ] {
        let declaration = format!("enum {name} {{");
        assert_eq!(FIND.matches(&declaration).count(), 1);
        assert_eq!(normalized(bounded(FIND, &declaration, "}")), variants);
        assert!(!FIND.contains(&format!("pub enum {name}")));
        assert!(!FIND.contains(&format!("impl Copy for {name}")));
        assert!(!FIND.contains(&format!("impl Clone for {name}")));
    }
    let policy = normalized(bounded(FIND, "impl FindViaPredicateKind {", "\n}\n"));
    for mapping in [
        "Self::Find|Self::FindIndex=>FindDirection::Ascending",
        "Self::FindLast|Self::FindLastIndex=>FindDirection::Descending",
        "Self::Find|Self::FindLast=>FindProjection::Value",
        "Self::FindIndex|Self::FindLastIndex=>FindProjection::Index",
    ] {
        assert!(
            policy.contains(mapping),
            "missing exhaustive mapping: {mapping}"
        );
    }
    assert!(!FIND.contains("_ =>"));
    assert!(!FIND.contains("#[derive("));

    let standard = normalized(STANDARD).replace(",)", ")");
    for family in ["array", "typed_array"] {
        for (suffix, kind) in [
            ("find", "Find"),
            ("find_index", "FindIndex"),
            ("find_last", "FindLast"),
            ("find_last_index", "FindLastIndex"),
        ] {
            let compiler = format!("compile_{family}_prototype_{suffix}_builtin");
            let definition = format!("pub(in crate::builtins) fn {compiler}(");
            assert_eq!(FIND.matches(&definition).count(), 1);
            let entry = bounded(FIND, &definition, "\n    }");
            assert!(entry.contains(&format!(
                "self.compile_{family}_find_with_kind(function, FindViaPredicateKind::{kind})"
            )));
            let builtin_family = if family == "array" {
                "Array"
            } else {
                "TypedArray"
            };
            assert!(standard.contains(&format!(
                "StandardBuiltinId::{builtin_family}Prototype{kind}=>{{self.{compiler}(function)?;}}"
            )));
        }
    }
}

#[test]
fn generic_find_uses_one_observable_length_and_no_private_receiver_shortcuts() {
    let entry = array_entry();
    assert_eq!(entry.matches("emit_array_like_length_snapshot(").count(), 1);
    in_order(
        entry,
        &[
            "self.emit_array_like_length_snapshot(",
            "self.emit_validate_find_predicate(",
            "self.emit_builtin_arg_to_locals(1,",
            "self.emit_initialize_find_index(&direction,",
        ],
    );
    for forbidden in [
        "HEAP_LEN_OFFSET",
        "SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX",
        "emit_load_typed_array_private_state(",
        "TypedArrayViewLocals",
        "TypedArrayWitnessUse::",
        "emit_array_iteration_to_object(",
        "emit_to_length_i64_from_value_locals(",
        "emit_arguments_read(",
        "emit_array_index_get_with_prototype(",
        "emit_object_read(",
        "emit_object_has_property_i32(",
        "Instruction::LocalSet(len_local)",
    ] {
        assert!(
            !entry.contains(forbidden),
            "generic find bypasses shared owner: {forbidden}"
        );
    }
}

#[test]
fn strict_typed_array_entry_retains_brand_and_validated_buffer_witness() {
    let entry = typed_entry();
    in_order(
        entry,
        &[
            "HEAP_OBJECT_INTERNAL_BRAND_OFFSET",
            "OBJECT_INTERNAL_BRAND_TYPED_ARRAY",
            "self.emit_throw_current_function_realm_type_error(",
            "self.emit_return_current_completion(function)",
            "self.emit_load_typed_array_private_state(",
            "let receiver_view = TypedArrayViewLocals::new(",
            "self.emit_typed_array_witness(",
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            "self.emit_validate_find_predicate(",
        ],
    );
    assert_eq!(entry.matches("emit_typed_array_witness(").count(), 1);
    assert_eq!(entry.matches("length_local: len_local").count(), 1);
    assert!(!entry.contains("emit_array_like_length_snapshot("));
    assert!(!entry.contains("ArrayLikeLengthSnapshot"));
    assert!(!entry.contains("Instruction::LocalSet(len_local)"));
    assert!(!entry.contains("Instruction::I64DivU"));
}

#[test]
fn both_loops_get_then_call_without_skipping_holes_or_leaking_scratch_results() {
    for entry in [array_entry(), typed_entry()] {
        in_order(
            entry,
            &[
                "Instruction::Loop(BlockType::Empty)",
                "Instruction::LocalGet(len_local)",
                "Instruction::I64GeU",
                "Instruction::BrIf(1)",
                "self.emit_typed_array_or_object_index_read_from_locals(",
                "self.emit_propagate_throw_from_locals_if_needed(",
                "Instruction::F64ConvertI64U",
                "self.emit_call_validated_find_predicate(",
                "&predicate,",
                "self.emit_propagate_throw_from_locals_if_needed(",
                "self.compile_truthy_tagged_i32(",
                "self.emit_project_find_match(",
                "self.emit_return_current_completion(function)",
                "self.emit_advance_find_index(&direction,",
                "Instruction::Br(0)",
                "Instruction::End",
                "Instruction::End",
                "self.emit_initialize_find_result(&projection, function)",
                "self.release_find_predicate(predicate)",
            ],
        );
        assert_eq!(entry.matches("emit_initialize_find_result(").count(), 1);
        assert_eq!(entry.matches("emit_call_validated_find_predicate(").count(), 1);
        assert_eq!(
            entry
                .matches("emit_typed_array_or_object_index_read_from_locals(")
                .count(),
            1
        );
        assert!(!entry.contains("emit_object_has_property_i32("));
        assert!(!entry.contains("emit_function_handle_call_with_argv"));
    }
}

#[test]
fn predicate_witness_is_borrowed_by_call_and_released_only_by_loop_owner() {
    let validator = bounded(
        FIND,
        "    fn emit_validate_find_predicate(",
        "    fn emit_call_validated_find_predicate(",
    );
    let call = bounded(
        FIND,
        "    fn emit_call_validated_find_predicate(",
        "    fn release_find_predicate(",
    );
    assert_eq!(validator.matches("emit_is_callable_i32(").count(), 1);
    assert!(!validator.contains("ValueKind::Function"));
    assert!(call.contains("predicate: &ValidatedFindPredicateLocals"));
    assert!(!call.contains("release_temp_local"));
    assert_eq!(
        call.matches("emit_function_or_proxy_call_with_argv_leave_throw_completion(")
            .count(),
        1
    );
    in_order(
        call,
        &[
            "(element.payload, element.tag)",
            "(index.payload, index.tag)",
            "(receiver.payload, receiver.tag)",
            "predicate.payload,",
            "predicate.tag,",
            "this_argument.payload,",
            "this_argument.tag,",
        ],
    );
    assert_eq!(
        FIND.matches("self.release_find_predicate(predicate);").count(),
        2
    );
    let release = bounded(
        FIND,
        "    fn release_find_predicate(",
        "    fn emit_initialize_find_result(",
    );
    assert!(release.contains("predicate: ValidatedFindPredicateLocals"));
    assert!(release.contains("self.release_temp_local(predicate.tag)"));
    assert!(release.contains("self.release_temp_local(predicate.payload)"));
}

#[test]
fn every_loop_temporary_is_released_exactly_once() {
    for entry in [array_entry(), typed_entry()] {
        let reserved: BTreeSet<_> = entry
            .lines()
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("let ")?
                    .strip_suffix(" = self.reserve_temp_local();")
            })
            .collect();
        assert!(!reserved.is_empty());
        for local in &reserved {
            assert_eq!(
                entry
                    .matches(&format!("self.release_temp_local({local});"))
                    .count(),
                1
            );
        }
        assert_eq!(
            entry.matches("self.release_temp_local(").count(),
            reserved.len()
        );
    }
}

#[test]
fn descending_iteration_stops_at_zero_before_subtraction() {
    let advance = bounded(
        FIND,
        "    fn emit_advance_find_index(",
        "    pub(in crate::builtins) fn compile_typed_array_prototype_find_builtin(",
    );
    let descending = advance
        .split_once("FindDirection::Descending => {")
        .unwrap()
        .1;
    in_order(
        descending,
        &[
            "Instruction::LocalGet(index_local)",
            "Instruction::I64Eqz",
            "Instruction::BrIf(1)",
            "Instruction::I64Sub",
        ],
    );
}

#[test]
fn complete_regression_inventory_targets_the_product_backend() {
    assert_eq!(REGRESSIONS.matches("#[test]").count(), 24);
    assert!(!REGRESSIONS.contains("#[ignore"));
    assert!(REGRESSIONS.contains("backend: ExecutionBackend::WasmAot"));
    for scenario in [
        "borrowed_typed_array_observes_own_and_inherited_length",
        "arguments_length_getter_and_coercion_are_observable",
        "global_object_is_a_valid_generic_receiver",
        "proxy_receivers_get_every_index_without_has_property",
        "all_eight_methods_keep_predicates_alive_and_return_no_match_sentinels",
        "huge_lengths_do_not_trap_or_truncate_reverse_indices",
        "strict_typed_array_validation_remains_separate_from_generic_length",
    ] {
        assert!(REGRESSIONS.contains(scenario));
    }
}
