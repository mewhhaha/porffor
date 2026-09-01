use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARGUMENT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_flat_argument_evaluation.js");
const FLAT_OVERRIDE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_flat_own_method_dispatch.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-flat-algorithm-owner.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

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
fn direct_flat_branch_has_one_closed_single_target_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"flat\") {",
        "        // `flatMap` is handled once, here.",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypeFlat,")
            .count(),
        1
    );
    assert_eq!(direct.matches("\"Array.prototype.flat\",").count(), 1);
    assert_eq!(direct.matches("                        args,").count(), 1);

    for marker in [
        "enum FlatMethodDispatch {",
        "ArrayCanonical,",
        "GenericGetCall,",
        "let flat_dispatch = if receiver",
        "read_static_heap_shape_property(shape, \"flat\")",
        "info.function_targets.exact_single_target()",
        "StandardBuiltinId::ArrayPrototypeFlat.function_id()",
        "match flat_dispatch {",
        "FlatMethodDispatch::ArrayCanonical => {",
        "FlatMethodDispatch::GenericGetCall => {}",
    ] {
        assert!(
            direct.contains(marker),
            "missing Flat dispatch marker `{marker}`"
        );
    }
    assert_eq!(direct.matches("enum FlatMethodDispatch {").count(), 1);
    assert_eq!(direct.matches("FlatMethodDispatch::").count(), 4);
    for variant in ["ArrayCanonical", "GenericGetCall"] {
        assert_eq!(
            direct
                .matches(&format!("FlatMethodDispatch::{variant}"))
                .count(),
            2,
            "{variant} must have one producer and one exhaustive consumer"
        );
    }

    for forbidden in [
        "#[derive",
        "_ =>",
        "unreachable!",
        "emit_array_flat_method_call(",
        "for arg in args",
        "compile_expr_to_locals(",
        "emit_direct_js_call(",
        "emit_object_read(",
        "possible_kinds",
        "HeapShape::Array",
        "ValueKind::Array",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct flat branch must not retain `{forbidden}`"
        );
    }
}

#[test]
fn removed_direct_flat_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source.matches("emit_array_flat_method_call").count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_flat_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeFlat => {",
        "            StandardBuiltinId::ArrayPrototypeFlatMap => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_flat_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_compiler_own_argument_and_flatten_order() {
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
        "    pub(crate) fn compile_array_prototype_flat_builtin(",
        "    pub(crate) fn compile_array_prototype_concat_builtin(",
    );
    assert_eq!(canonical.matches("self.argc_param_local()").count(), 1);
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
    for (earlier, later) in [
        (
            "self.argc_param_local()",
            "self.emit_builtin_arg_to_locals(0,",
        ),
        (
            "self.emit_builtin_arg_to_locals(0,",
            "self.emit_value_to_number_payload(",
        ),
        (
            "self.emit_value_to_number_payload(",
            "self.emit_to_length_i64_from_value_locals(",
        ),
        (
            "self.emit_to_length_i64_from_value_locals(",
            "self.emit_object_has_property_i32(",
        ),
        (
            "self.emit_object_has_property_i32(",
            "self.emit_array_index_get_with_prototype(",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn focused_fixture_observes_all_arguments_before_flattening() {
    for marker in [
        "var flattened = receiver.flat(",
        "ignoredSecondArgument(),",
        "...ignoredSpread",
        "order[0] === \"depth\"",
        "order[1] === \"second\"",
        "order[2] === \"iterator\"",
        "order[4] === \"next2\"",
        "order[5] === \"get\"",
    ] {
        assert!(
            ARGUMENT_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_evaluates_all_array_flat_arguments_before_flattening()"));
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_flat_core_fixture()"));
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_succeeds_for_supported_array_flat_proxy_access_count_fixture()"
    ));
}

#[test]
fn flat_override_fixture_and_evidence_remain_in_inventory() {
    for marker in [
        "target.flat = function (first, second, third, fourth)",
        "target.flat(record(1), ...[record(2), record(3)], record(4))",
        "receiver === target",
        "callCount === 1",
        "target.length === 2",
        "target[0] === 1",
        "target[1][0] === 2",
        "target[1][1] === 3",
    ] {
        assert!(
            FLAT_OVERRIDE_FIXTURE.contains(marker),
            "missing Flat override marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains("fn run_wasm_backend_calls_an_arrays_own_flat_method()"));
    assert!(ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_array_flat_own_method_dispatch.js\")"));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("`FlatMethodDispatch::{ArrayCanonical, GenericGetCall}`"));
    }
}
