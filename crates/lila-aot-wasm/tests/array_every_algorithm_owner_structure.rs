use std::fs;
use std::path::Path;

const CALLBACK_ITERATION_SOURCE: &str = include_str!("../src/builtins/array/callback_iteration.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARGUMENT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_every_argument_evaluation.js");

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
fn array_every_arm_delegates_with_complete_arguments_without_changing_iterator_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Every.property_name())",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Some.property_name())",
    );
    for marker in [
        "let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)",
        "let receiver_is_iterator =",
        "if receiver_is_iterator {",
        "if receiver_is_array || receiver_has_array_every {",
        "if receiver_needs_dynamic_helper_dispatch(receiver) {",
        "IteratorHelper::Every,",
    ] {
        assert!(
            direct.contains(marker),
            "missing dispatch marker `{marker}`"
        );
    }

    let array_arm = bounded(
        direct,
        "            if receiver_is_array || receiver_has_array_every {",
        "            if receiver_needs_dynamic_helper_dispatch(receiver) {",
    );
    assert_eq!(
        array_arm
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        array_arm
            .matches("StandardBuiltinId::ArrayPrototypeEvery,")
            .count(),
        1
    );
    assert_eq!(array_arm.matches("\"Array.prototype.every\",").count(), 1);
    assert_eq!(array_arm.matches("                    args,").count(), 1);

    for forbidden in [
        "emit_array_every_method_call(",
        "for arg in args",
        "compile_expr_to_locals(",
        "emit_direct_js_call(",
    ] {
        assert!(
            !array_arm.contains(forbidden),
            "Array every arm must not retain `{forbidden}`"
        );
    }
}

#[test]
fn removed_direct_every_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source.matches("emit_array_every_method_call").count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_every_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeEvery => {",
        "            StandardBuiltinId::TypedArrayPrototypeEvery => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_every_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_compiler_own_argument_and_iteration_order() {
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
        "    pub(crate) fn compile_array_prototype_every_builtin(",
        "    pub(crate) fn compile_array_prototype_some_builtin(",
    );
    assert_eq!(
        canonical
            .matches("self.compile_array_callback_iteration(")
            .count(),
        1
    );
    assert!(canonical.contains("ArrayCallbackIterationKind::Every"));
    assert!(!canonical.contains("emit_builtin_arg_to_locals("));
    let shared = CALLBACK_ITERATION_SOURCE;
    assert_eq!(
        shared.matches("self.emit_builtin_arg_to_locals(0,").count(),
        1
    );
    assert_eq!(
        shared.matches("self.emit_builtin_arg_to_locals(1,").count(),
        1
    );
    for (earlier, later) in [
        (
            "self.emit_array_iteration_length_before_callback_validation(",
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
    ] {
        assert_before(shared, earlier, later);
    }
    for forbidden in [
        "emit_function_handle_call_with_argv(",
        "emit_array_index_get_with_prototype(",
        "emit_load_typed_array_private_state(",
        "ARRAY_LENGTH_OFFSET",
        "property_key_symbol_payload(\"Symbol.species\")",
    ] {
        assert!(
            !shared.contains(forbidden),
            "shared every loop must not duplicate {forbidden}"
        );
    }
}

#[test]
fn focused_fixture_observes_all_arguments_before_iteration() {
    for marker in [
        "var matched = receiver.every(",
        "ignoredThirdArgument(),",
        "...ignoredSpread",
        "order[0] === \"callback\"",
        "order[2] === \"third\"",
        "order[3] === \"iterator\"",
        "order[5] === \"next2\"",
        "order[6] === \"get\"",
        "order[7] === \"every\"",
    ] {
        assert!(
            ARGUMENT_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_evaluates_all_array_every_arguments_before_iteration()"));
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_every_core_fixture()"));
}
