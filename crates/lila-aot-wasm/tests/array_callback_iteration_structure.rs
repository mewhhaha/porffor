const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const SHARED: &str = include_str!("../src/builtins/array/callback_iteration.rs");
const PLANNING: &str = include_str!("../src/planning.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect("start marker")
        .1
        .split_once(end)
        .expect("end marker")
        .0
}

#[test]
fn closed_result_policy_has_four_real_producers_and_no_receiver_mode() {
    assert!(ARRAY_SOURCE.contains("mod callback_iteration;"));
    assert!(!ARRAY_SOURCE.contains("emit_string_hex_length_to_i64_local"));
    assert!(SHARED.contains("pub(super) enum ArrayCallbackIterationKind"));
    assert!(!SHARED.contains("_ =>"));
    assert!(!SHARED.contains("typed_array_only"));
    assert_eq!(
        ARRAY_SOURCE
            .matches("self.compile_array_callback_iteration(")
            .count(),
        4
    );
    for (method, kind, next) in [
        ("map", "Map", "compile_typed_array_prototype_slice_builtin"),
        ("filter", "Filter", "emit_array_direct_builtin_method_call"),
        ("every", "Every", "compile_array_prototype_some_builtin"),
        ("some", "Some", "compile_array_prototype_filter_builtin"),
    ] {
        let wrapper = bounded(
            ARRAY_SOURCE,
            &format!("pub(crate) fn compile_array_prototype_{method}_builtin("),
            &format!("pub(crate) fn {next}("),
        );
        assert_eq!(
            wrapper
                .matches("self.compile_array_callback_iteration(")
                .count(),
            1
        );
        assert!(wrapper.contains(&format!("ArrayCallbackIterationKind::{kind}")));
        assert!(!wrapper.contains("function.instruction("));
    }
}

#[test]
fn shared_loop_has_one_observable_owner_for_each_input_operation() {
    for operation in [
        "self.emit_array_iteration_length_before_callback_validation(",
        "self.emit_is_callable_i32(",
        "self.emit_object_has_property_i32(",
        "self.emit_typed_array_or_object_index_read_from_locals(",
        "self.emit_pre_evaluated_arg_vector(",
        "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
    ] {
        assert_eq!(SHARED.matches(operation).count(), 1, "{operation}");
    }
    for forbidden in [
        "emit_load_typed_array_private_state(",
        "TypedArrayViewLocals::new(",
        "emit_function_handle_call_with_argv(",
        "emit_function_handle_construct_with_argv(",
        "emit_array_length_of_array_like(",
        "emit_array_index_get_with_prototype(",
        "ARRAY_LENGTH_OFFSET",
        "Symbol.species",
        "emit_array_write_index(",
    ] {
        assert!(
            !SHARED.contains(forbidden),
            "duplicated policy: {forbidden}"
        );
    }
    let after_has = SHARED
        .split_once("self.emit_object_has_property_i32(")
        .unwrap()
        .1;
    assert!(
        after_has
            .find("self.emit_return_current_completion_if_throw(function)")
            .unwrap()
            < after_has
                .find("self.emit_typed_array_or_object_index_read_from_locals(")
                .unwrap()
    );
    let after_call = SHARED
        .split_once("self.emit_function_or_proxy_call_with_argv_leave_throw_completion(")
        .unwrap()
        .1;
    assert!(
        after_call
            .find("self.emit_propagate_throw_from_locals_if_needed(")
            .unwrap()
            < after_call.find("match kind {").unwrap()
    );
}

#[test]
fn result_policy_uses_species_only_for_producers_and_data_definition_for_writes() {
    let creation = bounded(SHARED, "match kind {", "ValueKind::Number.tag()");
    assert_eq!(
        creation.matches("self.emit_array_species_create(").count(),
        2
    );
    assert!(creation
        .contains("ArrayCallbackIterationKind::Every | ArrayCallbackIterationKind::Some => {}"));
    assert_eq!(
        SHARED
            .matches("self.emit_array_target_create_data_property_or_throw(")
            .count(),
        2
    );
    let filter = bounded(
        SHARED,
        "ArrayCallbackIterationKind::Filter => {",
        "ArrayCallbackIterationKind::Every => {",
    );
    assert!(filter.contains("self.compile_truthy_tagged_i32("));
    let write = filter
        .split_once("self.emit_array_target_create_data_property_or_throw(")
        .unwrap()
        .1;
    assert!(write.contains("element_payload_local,"));
    assert!(write.contains("element_tag_local,"));
    assert!(!filter.contains("self.emit_typed_array_or_object_index_read_from_locals("));
}

#[test]
fn shared_temporary_lifecycle_is_complete_unique_and_lifo() {
    let reservations: Vec<_> = SHARED
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("let ")?
                .strip_suffix(" = self.reserve_temp_local();")
        })
        .collect();
    let releases: Vec<_> = SHARED
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("self.release_temp_local(")?
                .strip_suffix(");")
        })
        .collect();
    assert_eq!(reservations.len(), 20);
    assert_eq!(
        SHARED.matches("reserve_temp_local()").count(),
        reservations.len()
    );
    assert_eq!(
        SHARED.matches("release_temp_local(").count(),
        releases.len()
    );
    let mut unique = reservations.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), reservations.len());
    assert_eq!(releases, reservations.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn map_and_filter_root_descriptor_definition_even_in_minimal_programs() {
    let dependency = PLANNING
        .split_once("self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);")
        .expect("definition dependency")
        .0;
    let dependency = dependency
        .rsplit_once("if matches!(")
        .expect("dependency condition")
        .1;
    for builtin in [
        "ArrayPrototypeMap",
        "ArrayPrototypeFilter",
        "ArrayPrototypeFlatMap",
        "ArrayPrototypeSlice",
        "ArrayPrototypeSplice",
    ] {
        assert!(
            dependency.contains(&format!("StandardBuiltinId::{builtin}")),
            "{builtin}"
        );
    }
}

#[test]
fn internal_array_result_descriptor_has_no_observable_prototype() {
    let body = bounded(
        ARRAY_SOURCE,
        "fn emit_array_target_create_data_property_or_throw(",
        "pub(crate) fn compile_array_prototype_fill_builtin(",
    );
    assert!(body.contains("self.emit_alloc_plain_object_with_prototype(None, None, function)"));
    assert!(!body.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));
}
