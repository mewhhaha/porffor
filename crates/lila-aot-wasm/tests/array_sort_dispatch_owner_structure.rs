use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARRAY_SORT_OVERRIDE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_sort_own_method_dispatch.js");
const TYPED_ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const TYPED_ARRAY_SORT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_prototype_sort.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-sort-dispatch-owner.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
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
fn direct_sort_branch_has_one_closed_single_target_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"sort\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"spliceFromArray\") {",
    );

    for marker in [
        "enum SortMethodDispatch {",
        "TypedArrayCanonical,",
        "ArrayCanonical,",
        "GenericGetCall,",
        "let sort_dispatch = if receiver",
        "match sort_dispatch {",
        "SortMethodDispatch::TypedArrayCanonical => {",
        "SortMethodDispatch::ArrayCanonical => {",
        "SortMethodDispatch::GenericGetCall => {}",
    ] {
        assert!(
            direct.contains(marker),
            "missing dispatch marker `{marker}`"
        );
    }
    assert_eq!(direct.matches("enum SortMethodDispatch {").count(), 1);
    assert_eq!(direct.matches("SortMethodDispatch::").count(), 6);
    for variant in ["TypedArrayCanonical", "ArrayCanonical", "GenericGetCall"] {
        assert_eq!(
            direct
                .matches(&format!("SortMethodDispatch::{variant}"))
                .count(),
            2,
            "{variant} must have one producer and one exhaustive consumer"
        );
    }
    assert_eq!(
        direct
            .matches("read_static_heap_shape_property(shape, \"sort\")")
            .count(),
        2
    );
    assert_eq!(
        direct
            .matches("info.function_targets.exact_single_target()")
            .count(),
        2
    );
    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        2
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::TypedArrayPrototypeSort,")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypeSort,")
            .count(),
        1
    );
    assert_eq!(direct.matches("                        args,").count(), 2);
    assert!(
        direct
            .find("StandardBuiltinId::TypedArrayPrototypeSort.function_id()")
            .unwrap()
            < direct
                .find("StandardBuiltinId::ArrayPrototypeSort.function_id()")
                .unwrap(),
        "strict TypedArray dispatch must have precedence"
    );
    for forbidden in [
        "#[derive",
        "_ =>",
        "unreachable!",
        "possible_kinds",
        "HeapShape::Array",
        "ValueKind::Array",
        "compile_expr_to_locals(",
        "emit_array_read(",
        "emit_array_write(",
    ] {
        assert!(
            !direct.contains(forbidden),
            "sort dispatch authority must not contain `{forbidden}`"
        );
    }
}

#[test]
fn array_and_typed_array_sort_keep_one_distinct_canonical_owner_each() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("fn compile_array_sort_with_output(")
            .count(),
        1
    );
    assert_eq!(
        rust_source
            .matches("fn compile_typed_array_prototype_sort_builtin(")
            .count(),
        1
    );

    let array_standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeSort => {",
        "            StandardBuiltinId::ArrayPrototypeToSorted => {",
    );
    assert_eq!(
        array_standard
            .matches("self.compile_array_prototype_sort_builtin(function)?;")
            .count(),
        1
    );
    let typed_standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TypedArrayPrototypeSort => {",
        "            StandardBuiltinId::TypedArrayPrototypeToReversed => {",
    );
    assert_eq!(
        typed_standard
            .matches("self.compile_typed_array_prototype_sort_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn canonical_sort_bodies_retain_their_distinct_length_and_order_policies() {
    let array_sort = bounded(
        ARRAY_SOURCE,
        "    fn compile_array_sort_with_output(",
        "    #[allow(clippy::too_many_arguments)]\n    fn emit_array_target_create_data_property_or_throw(",
    );
    for marker in [
        "output: ArraySortOutput,",
        "emit_value_to_current_function_realm_object_locals(",
        "self.strings.payload(\"length\")",
        "emit_to_length_i64_from_value_locals(",
        "match &output {",
        "ArraySortOutput::Receiver => {",
        "emit_string_payload_utf16_compare_i32(",
    ] {
        assert!(
            array_sort.contains(marker),
            "missing Array sort marker `{marker}`"
        );
    }

    let typed_sort = bounded(
        STANDARD_SOURCE,
        "    fn compile_typed_array_prototype_sort_builtin(",
        "    fn compile_typed_array_prototype_to_sorted_builtin(",
    );
    for marker in [
        "OBJECT_INTERNAL_BRAND_TYPED_ARRAY",
        "TypedArrayWitnessUse::ValidatedMethodEntry {",
        "length_local: receiver_length_local,",
        "self.emit_typed_array_stable_sort(",
        "receiver_element_kind_local,",
        "Instruction::LocalSet(self.result_local)",
    ] {
        assert!(
            typed_sort.contains(marker),
            "missing TypedArray sort marker `{marker}`"
        );
    }
    assert!(!typed_sort.contains("self.strings.payload(\"length\")"));
}

#[test]
fn sort_dispatch_runtime_controls_cover_override_and_strict_typed_array_paths() {
    for marker in [
        "target.sort = function (first, second, third, fourth) {",
        "receiver = this;",
        "let result = target.sort(record(1), ...[record(2), record(3)], record(4));",
        "result === 10",
        "target[0] === 3",
        "target[2] === 2",
    ] {
        assert!(
            ARRAY_SORT_OVERRIDE_FIXTURE.contains(marker),
            "missing Array sort override witness `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains("fn run_wasm_backend_calls_an_arrays_own_sort_method()"));
    assert!(ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_array_sort_own_method_dispatch.js\")"));

    for marker in [
        "var numeric = new Uint16Array([111, 3, 22, 2, 11, 1]);",
        "Object.defineProperty(numeric, \"length\", { value: 50 });",
        "assertSame(numeric.sort(), numeric, \"returns receiver\");",
        "assertSame(numeric.length, 50, \"own length is preserved\");",
        "assertIndexedSequence(numeric, [1, 2, 3, 11, 22, 111], \"default numeric order\");",
    ] {
        assert!(
            TYPED_ARRAY_SORT_FIXTURE.contains(marker),
            "missing TypedArray sort dispatch witness `{marker}`"
        );
    }
    assert!(TYPED_ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_stably_sorts_typedarray_in_place_across_buffer_states()"));
    assert!(TYPED_ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_typedarray_prototype_sort.js\")"));
}

#[test]
fn task_and_contract_record_the_closed_sort_dispatch() {
    for evidence in [TASK, CONTRACT] {
        assert!(evidence.contains("SortMethodDispatch"));
        assert!(evidence.contains("TypedArrayCanonical"));
        assert!(evidence.contains("GenericGetCall"));
        assert!(evidence.contains("sole"));
    }
}
