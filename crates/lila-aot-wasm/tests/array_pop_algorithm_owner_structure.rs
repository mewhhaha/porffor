use std::fs;
use std::path::Path;

const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARRAY_POP_OVERRIDE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_pop_own_method_dispatch.js");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-pop-algorithm-owner.md");
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

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn direct_pop_branch_has_one_closed_single_target_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "if matches!(key, PropertyKeyIr::StaticString(name) if name == \"pop\") {",
        "if matches!(key, PropertyKeyIr::StaticString(name) if name == \"splice\") {",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypePop,")
            .count(),
        1
    );
    assert_eq!(direct.matches("\"Array.prototype.pop\",").count(), 1);
    assert_eq!(direct.matches("                        args,").count(), 1);

    for marker in [
        "enum PopMethodDispatch {",
        "ArrayCanonical,",
        "GenericGetCall,",
        "let pop_dispatch = if receiver",
        "read_static_heap_shape_property(shape, \"pop\")",
        "info.function_targets.exact_single_target()",
        "StandardBuiltinId::ArrayPrototypePop.function_id()",
        "match pop_dispatch {",
        "PopMethodDispatch::ArrayCanonical => {",
        "PopMethodDispatch::GenericGetCall => {}",
    ] {
        assert!(
            direct.contains(marker),
            "missing Pop dispatch marker `{marker}`"
        );
    }
    assert_eq!(direct.matches("enum PopMethodDispatch {").count(), 1);
    assert_eq!(direct.matches("PopMethodDispatch::").count(), 4);
    for variant in ["ArrayCanonical", "GenericGetCall"] {
        assert_eq!(
            direct
                .matches(&format!("PopMethodDispatch::{variant}"))
                .count(),
            2,
            "{variant} must have one producer and one exhaustive consumer"
        );
    }

    for forbidden in [
        "#[derive",
        "_ =>",
        "unreachable!",
        "HEAP_LEN_OFFSET",
        "emit_array_read(",
        "load_i64_to_local_from_offset(",
        "store_i64_local_at_offset(",
        "reserve_temp_local(",
        "compile_expr_to_locals(",
        "emit_object_read(",
        "emit_object_delete(",
        "emit_object_write",
        "emit_throw_runtime_error(",
        "possible_kinds",
        "HeapShape::Array",
        "ValueKind::Array",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct pop branch must not retain parallel operation `{forbidden}`"
        );
    }
}

#[test]
fn recursive_pop_owner_census_has_one_canonical_algorithm() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("StandardBuiltinId::ArrayPrototypePop => {")
            .count(),
        1
    );
    assert_eq!(rust_source.matches("emit_array_pop_method_call").count(), 0);
}

#[test]
fn standard_pop_body_owns_the_complete_ordered_algorithm() {
    let canonical = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::ArrayPrototypePop => {",
        "StandardBuiltinId::ArrayPrototypePush => {",
    );

    assert_eq!(
        canonical
            .matches("emit_value_to_current_function_realm_object_locals(")
            .count(),
        1
    );
    assert_eq!(
        canonical
            .matches("self.strings.payload(\"length\")")
            .count(),
        2
    );
    assert_eq!(canonical.matches("self.emit_object_read(").count(), 2);
    assert_eq!(
        canonical
            .matches("emit_to_length_i64_from_value_locals(")
            .count(),
        1
    );
    assert_eq!(canonical.matches("self.emit_object_delete(").count(), 1);
    assert_eq!(
        canonical
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2
    );
    assert_eq!(
        canonical.matches("self.emit_object_write_strict(").count(),
        1
    );

    assert_before(
        canonical,
        "emit_value_to_current_function_realm_object_locals(",
        "self.strings.payload(\"length\")",
    );
    assert_before(
        canonical,
        "self.strings.payload(\"length\")",
        "emit_to_length_i64_from_value_locals(",
    );

    let after_length = canonical
        .split_once("emit_to_length_i64_from_value_locals(")
        .expect("LengthOfArrayLike conversion")
        .1;
    assert_before(
        after_length,
        "self.emit_object_read(",
        "self.emit_object_delete(",
    );

    let after_delete = canonical
        .split_once("self.emit_object_delete(")
        .expect("DeletePropertyOrThrow boundary")
        .1;
    assert_before(
        after_delete,
        "emit_throw_current_function_realm_type_error(",
        "self.emit_object_write_strict(",
    );
}

#[test]
fn pop_override_runtime_control_requires_generic_get_and_call_fallthrough() {
    for marker in [
        "target.pop = function (first, second, third, fourth) {",
        "receiver = this;",
        "let result = target.pop(record(1), ...[record(2), record(3)], record(4));",
        "result === 10",
        "callCount === 1",
        "target.length === 3",
        "target[0] === 1",
        "target[2] === 3",
    ] {
        assert!(
            ARRAY_POP_OVERRIDE_FIXTURE.contains(marker),
            "missing Array Pop override witness `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains("fn run_wasm_backend_calls_an_arrays_own_pop_method()"));
    assert!(ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_array_pop_own_method_dispatch.js\")"));
}

#[test]
fn task_and_contract_record_the_closed_pop_dispatch() {
    for evidence in [TASK, CONTRACT] {
        assert!(evidence.contains("PopMethodDispatch"));
        assert!(evidence.contains("ArrayCanonical"));
        assert!(evidence.contains("GenericGetCall"));
        assert!(evidence.contains("sole"));
    }
}
