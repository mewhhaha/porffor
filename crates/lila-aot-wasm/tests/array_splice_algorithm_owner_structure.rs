use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const FIND_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_array_find_core.js");
const SPLICE_OVERRIDE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_splice_own_method_dispatch.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-splice-algorithm-owner.md");
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
fn removed_specialized_splice_subgraph_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    for removed in [
        "emit_array_splice_insert_method_call",
        "emit_array_splice_delete_one_method_call",
    ] {
        assert_eq!(rust_source.matches(removed).count(), 0, "{removed}");
    }
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_splice_builtin(")
            .count(),
        1
    );
    assert_eq!(
        rust_source
            .matches("fn emit_array_splice_from_array_method_call(")
            .count(),
        1
    );
}

#[test]
fn direct_splice_branch_has_one_closed_single_target_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"splice\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"sort\") {",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    for marker in [
        "enum SpliceMethodDispatch {",
        "ArrayCanonical,",
        "GenericGetCall,",
        "let splice_dispatch = if receiver",
        "read_static_heap_shape_property(shape, \"splice\")",
        "info.function_targets.exact_single_target()",
        "StandardBuiltinId::ArrayPrototypeSplice.function_id()",
        "match splice_dispatch {",
        "SpliceMethodDispatch::ArrayCanonical => {",
        "SpliceMethodDispatch::GenericGetCall => {}",
        "StandardBuiltinId::ArrayPrototypeSplice,",
        "\"Array.prototype.splice\",",
        "                        receiver,",
        "                        args,",
        "                        payload_local,",
        "                        tag_local,",
    ] {
        assert!(direct.contains(marker), "missing direct marker `{marker}`");
    }
    assert_eq!(direct.matches("enum SpliceMethodDispatch {").count(), 1);
    assert_eq!(direct.matches("SpliceMethodDispatch::").count(), 4);
    for variant in ["ArrayCanonical", "GenericGetCall"] {
        assert_eq!(
            direct
                .matches(&format!("SpliceMethodDispatch::{variant}"))
                .count(),
            2,
            "{variant} must have one producer and one exhaustive consumer"
        );
    }
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
        "emit_object_read(",
        "emit_object_delete(",
        "emit_object_write",
    ] {
        assert!(
            !direct.contains(forbidden),
            "Splice dispatch authority must not contain `{forbidden}`"
        );
    }

    let standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeSplice => {",
        "            StandardBuiltinId::ArrayPrototypeFill => {",
    );
    assert_eq!(
        standard
            .matches("self.compile_array_prototype_splice_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn canonical_splice_owner_fixes_observable_operation_order() {
    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_splice_builtin(",
        "    pub(crate) fn emit_array_splice_from_array_method_call(",
    );

    for (earlier, later) in [
        (
            "self.emit_value_to_current_function_realm_object_locals(",
            "self.strings.payload(\"length\")",
        ),
        (
            "self.strings.payload(\"length\")",
            "self.emit_to_length_i64_from_value_locals(",
        ),
        (
            "self.emit_to_length_i64_from_value_locals(",
            "self.emit_builtin_arg_to_locals(0,",
        ),
        (
            "self.emit_builtin_arg_to_locals(0,",
            "self.emit_builtin_arg_to_locals(\n            1,",
        ),
        (
            "self.emit_builtin_arg_to_locals(\n            1,",
            "self.emit_array_species_create(",
        ),
        (
            "self.emit_array_species_create(",
            "self.emit_object_has_property_i32(",
        ),
        (
            "self.emit_object_has_property_i32(",
            "self.emit_array_target_create_data_property_or_throw(",
        ),
        (
            "self.emit_array_target_create_data_property_or_throw(",
            "self.emit_delete_property_or_throw(",
        ),
        (
            "self.emit_delete_property_or_throw(",
            "self.argv_param_local()",
        ),
        (
            "self.argv_param_local()",
            "self.set_completion_kind(CompletionKind::Normal, function)",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn splice_from_array_and_focused_mutation_control_remain_live() {
    let custom = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"spliceFromArray\") {",
        "        let static_array_iterator_method = match key {",
    );
    assert_eq!(
        custom
            .matches("self.emit_array_splice_from_array_method_call(")
            .count(),
        1
    );
    for marker in ["                receiver,", "                args,"] {
        assert!(custom.contains(marker), "missing custom marker `{marker}`");
    }

    for marker in [
        "spliceArray.splice(1, 1);",
        "spliceCount === 3",
        "spliceFirst === \"Shoes\"",
        "spliceSecond === \"Bike\"",
        "spliceThird === undefined",
    ] {
        assert!(
            FIND_FIXTURE.contains(marker),
            "missing Splice mutation control `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_find_core_fixture()"));
}

#[test]
fn splice_override_fixture_and_evidence_remain_in_inventory() {
    for marker in [
        "target.splice = function (first, second, third, fourth)",
        "target.splice(record(1), ...[record(2), record(3)], record(4))",
        "receiver === target",
        "callCount === 1",
        "target.length === 3",
        "target[0] === 1",
        "target[1] === 2",
        "target[2] === 3",
    ] {
        assert!(
            SPLICE_OVERRIDE_FIXTURE.contains(marker),
            "missing Splice override marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains("fn run_wasm_backend_calls_an_arrays_own_splice_method()"));
    assert!(ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_array_splice_own_method_dispatch.js\")"));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("`SpliceMethodDispatch::{ArrayCanonical, GenericGetCall}`"));
    }
}
