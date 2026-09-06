use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARGUMENT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_flat_map_argument_evaluation.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn collect_rust_source(path: &Path, source: &mut String) {
    for directory_entry in fs::read_dir(path).expect("Rust source directory") {
        let directory_entry = directory_entry.expect("Rust source directory entry");
        let child_path = directory_entry.path();
        if child_path.is_dir() {
            collect_rust_source(&child_path, source);
        } else if child_path
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("rs")
        {
            source.push_str(&fs::read_to_string(&child_path).expect("Rust source file"));
        }
    }
}

#[test]
fn array_flat_map_arm_delegates_with_complete_arguments_without_changing_iterator_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::FlatMap.property_name())",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"at\") {",
    );

    for marker in [
        "let receiver_is_array = receiver.kind == ValueKind::Array",
        "if receiver_is_array {",
        ".emit_iterator_prototype_helper_method_call(",
        "IteratorHelper::FlatMap,",
    ] {
        assert!(
            direct.contains(marker),
            "missing dispatch marker `{marker}`"
        );
    }
    let array_arm = bounded(
        direct,
        "            if receiver_is_array {",
        "            return self\n                .emit_iterator_prototype_helper_method_call(",
    );
    assert_eq!(
        array_arm
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        array_arm
            .matches("StandardBuiltinId::ArrayPrototypeFlatMap,")
            .count(),
        1
    );
    assert_eq!(array_arm.matches("\"Array.prototype.flatMap\",").count(), 1);
    assert_eq!(array_arm.matches("                    args,").count(), 1);

    for forbidden in [
        "emit_array_flat_map_method_call(",
        "for arg in args",
        "compile_expr_to_locals(",
        "emit_direct_js_call(",
    ] {
        assert!(
            !direct.contains(forbidden),
            "Array flatMap arm must not retain `{forbidden}`"
        );
    }
}

#[test]
fn removed_direct_flat_map_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("emit_array_flat_map_method_call")
            .count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_flat_map_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeFlatMap => {",
        "            StandardBuiltinId::ArrayPrototypeAt => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_flat_map_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_compiler_own_argument_and_mapping_order() {
    let direct_call = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn emit_array_direct_builtin_method_call(",
        "    pub(crate) fn compile_array_prototype_join_builtin(",
    );
    assert_before(
        direct_call,
        "self.compile_expr_to_locals(",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        direct_call,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.emit_call_args_vector(args, function)",
    );
    assert_before(
        direct_call,
        "self.emit_call_args_vector(args, function)",
        "self.emit_direct_js_call_with_argv(",
    );

    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_flat_map_builtin(",
        "    fn emit_flat_map_append(",
    );
    assert_eq!(canonical.matches("self.argc_param_local()").count(), 0);
    assert_eq!(
        canonical
            .matches("self.emit_builtin_arg_to_locals(0,")
            .count(),
        1
    );
    assert_eq!(
        canonical
            .matches("self.emit_builtin_arg_to_locals(1,")
            .count(),
        1
    );
    for (earlier, later) in [
        (
            "self.emit_array_like_length_snapshot(",
            "self.emit_builtin_arg_to_locals(0,",
        ),
        (
            "self.emit_builtin_arg_to_locals(0,",
            "self.emit_is_callable_i32(",
        ),
        (
            "self.emit_is_callable_i32(",
            "self.emit_builtin_arg_to_locals(1,",
        ),
        (
            "self.emit_builtin_arg_to_locals(1,",
            "self.emit_array_species_create(",
        ),
        (
            "self.emit_array_species_create(",
            "self.emit_object_has_property_i32(",
        ),
        (
            "self.emit_object_has_property_i32(",
            "self.emit_typed_array_or_object_index_read_from_locals(",
        ),
        (
            "self.emit_typed_array_or_object_index_read_from_locals(",
            "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        ),
        (
            "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
            "self.emit_is_array_i64(",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn focused_fixture_observes_all_arguments_before_mapping() {
    for marker in [
        "var flattened = receiver.flatMap(",
        "ignoredThirdArgument(),",
        "...ignoredSpread",
        "order[0] === \"callback\"",
        "order[2] === \"third\"",
        "order[3] === \"iterator\"",
        "order[5] === \"next2\"",
        "order[6] === \"get\"",
        "order[7] === \"map\"",
    ] {
        assert!(
            ARGUMENT_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_evaluates_all_array_flat_map_arguments_before_mapping()"));
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_flat_map_core_fixture()"));
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_succeeds_for_supported_array_flat_map_proxy_access_count_fixture()"
    ));
}

#[test]
fn one_append_owner_bounds_the_index_before_defining_and_incrementing() {
    let append = bounded(
        ARRAY_SOURCE,
        "    fn emit_flat_map_append(",
        "    pub(crate) fn emit_array_like_length_snapshot(",
    );
    assert_eq!(
        append
            .matches("emit_array_target_create_data_property_or_throw(")
            .count(),
        1
    );
    assert_before(
        append,
        "Instruction::I64Const(MAX_SAFE_INTEGER as i64)",
        "emit_array_target_create_data_property_or_throw(",
    );
    assert_before(
        append,
        "emit_array_target_create_data_property_or_throw(",
        "emit_return_current_completion_if_throw(",
    );
    assert_before(
        append,
        "emit_return_current_completion_if_throw(",
        "Instruction::I64Add",
    );
    assert!(!append.contains("emit_object_write("));
}

#[test]
fn flat_map_roots_the_shared_target_definition_builtin() {
    let planning = include_str!("../src/planning.rs");
    let start = planning
        .find("            StandardBuiltinId::ArrayPrototypeFlatMap\n")
        .expect("flatMap dependency arm");
    let arm = &planning[start..];
    let end = arm.find("\n        }").expect("dependency arm end");
    assert!(arm[..end]
        .contains("self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);"));
}
