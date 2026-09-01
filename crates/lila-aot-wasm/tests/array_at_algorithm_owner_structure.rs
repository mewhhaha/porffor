use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARGUMENT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typed_array_at_argument_evaluation.js");

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
fn direct_at_branch_forwards_complete_arguments_to_the_strict_typed_array_entry() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"at\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"includes\") {",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::TypedArrayPrototypeAt,")
            .count(),
        1
    );
    assert_eq!(direct.matches("\"TypedArray.prototype.at\",").count(), 1);
    assert_eq!(direct.matches("                args,").count(), 1);
    assert!(!direct.contains("StandardBuiltinId::ArrayPrototypeAt"));

    for forbidden in [
        "emit_array_at_method_call(",
        "args.first()",
        "compile_expr_to_locals(",
        "emit_array_at_from_locals(",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct at branch must not retain `{forbidden}`"
        );
    }
}

#[test]
fn removed_direct_at_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(rust_source.matches("emit_array_at_method_call").count(), 0);
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_at_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TypedArrayPrototypeAt => {",
        "            StandardBuiltinId::ArrayPrototypeToReversed => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_at_builtin(")
            .count(),
        1
    );
    assert_eq!(
        standard_arm
            .matches("ArrayAtReceiverPolicy::TypedArray,")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_compiler_own_argument_and_index_order() {
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
        "    pub(crate) fn compile_array_prototype_at_builtin(",
        "    pub(crate) fn compile_array_prototype_to_reversed_builtin(",
    );
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
        0
    );
    assert_before(
        canonical,
        "self.emit_builtin_arg_to_locals(0,",
        "self.emit_array_at_from_locals(",
    );

    let algorithm = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn emit_array_at_from_locals(",
        "    pub(crate) fn emit_array_includes_from_locals(",
    );
    assert_before(
        algorithm,
        "self.emit_typed_array_witness(",
        "self.emit_value_to_number_payload(",
    );
    assert_before(
        algorithm,
        "self.emit_value_to_number_payload(",
        "self.emit_typed_array_or_object_index_read_from_locals(",
    );
}

#[test]
fn focused_fixture_observes_all_arguments_before_index_coercion() {
    for marker in [
        "var value = receiver.at(",
        "ignoredSecondArgument(),",
        "...ignoredSpread",
        "order[0] === \"index\"",
        "order[1] === \"second\"",
        "order[2] === \"iterator\"",
        "order[4] === \"next2\"",
        "order[5] === \"coerce\"",
    ] {
        assert!(
            ARGUMENT_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_evaluates_all_typed_array_at_arguments_before_index_coercion()"
    ));
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_at_runtime_kinds_fixture()"));
}
