use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARRAY_REVERSE_OVERRIDE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_reverse_own_method_dispatch.js");
const TYPED_ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const TYPED_ARRAY_REVERSE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_prototype_reverse.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn offsets(source: &str, needle: &str) -> Vec<usize> {
    source
        .match_indices(needle)
        .map(|(offset, _)| offset)
        .collect()
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
fn direct_reverse_branch_selects_only_proven_array_and_typed_array_owners() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"reverse\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"split\") {",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        2
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypeReverse,")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::TypedArrayPrototypeReverse,")
            .count(),
        1
    );
    assert_eq!(direct.matches("\"Array.prototype.reverse\",").count(), 1);
    assert_eq!(
        direct.matches("\"TypedArray.prototype.reverse\",").count(),
        1
    );
    assert_eq!(direct.matches("                        args,").count(), 2);

    for marker in [
        "enum ReverseMethodDispatch {",
        "TypedArrayCanonical,",
        "ArrayCanonical,",
        "GenericGetCall,",
        "let reverse_dispatch = if receiver",
        "read_static_heap_shape_property(shape, \"reverse\")",
        "match reverse_dispatch {",
        "ReverseMethodDispatch::TypedArrayCanonical => {",
        "ReverseMethodDispatch::ArrayCanonical => {",
        "ReverseMethodDispatch::GenericGetCall => {}",
    ] {
        assert!(direct.contains(marker), "missing dispatch proof `{marker}`");
    }
    assert_eq!(direct.matches("enum ReverseMethodDispatch {").count(), 1);
    assert_eq!(direct.matches("ReverseMethodDispatch::").count(), 6);
    assert_eq!(
        direct
            .matches("read_static_heap_shape_property(shape, \"reverse\")")
            .count(),
        2
    );
    assert_eq!(
        direct
            .matches("info.function_targets.exact_single_target()")
            .count(),
        2
    );
    for variant in ["TypedArrayCanonical", "ArrayCanonical", "GenericGetCall"] {
        assert_eq!(
            direct
                .matches(&format!("ReverseMethodDispatch::{variant}"))
                .count(),
            2,
            "{variant} must have one producer and one exhaustive consumer"
        );
    }
    for forbidden in [
        "#[derive",
        "_ =>",
        "unreachable!",
        "Default for ReverseMethodDispatch",
    ] {
        assert!(
            !direct.contains(forbidden),
            "reverse dispatch authority must not acquire `{forbidden}`"
        );
    }
    assert!(
        direct
            .find("StandardBuiltinId::TypedArrayPrototypeReverse.function_id()")
            .unwrap()
            < direct
                .find("StandardBuiltinId::ArrayPrototypeReverse.function_id()")
                .unwrap(),
        "strict TypedArray dispatch must precede generic Array dispatch"
    );

    for forbidden in [
        "emit_array_reverse_method_call(",
        "HEAP_LEN_OFFSET",
        "emit_array_read(",
        "emit_array_write(",
        "load_i64_to_local_from_offset(",
        "reserve_temp_local(",
        "compile_expr_to_locals(",
        "emit_throw_runtime_error(",
        "possible_kinds",
        "HeapShape::Array",
        "ValueKind::Array",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct reverse branch must not retain parallel operation `{forbidden}`"
        );
    }
}

#[test]
fn array_reverse_override_control_requires_generic_get_and_call_fallthrough() {
    for marker in [
        "target.reverse = function (first, second, third, fourth) {",
        "receiver = this;",
        "let result = target.reverse(record(1), ...[record(2), record(3)], record(4));",
        "result === 10",
        "receiver === target",
        "callCount === 1",
        "target[0] === 1",
        "target[2] === 3",
    ] {
        assert!(
            ARRAY_REVERSE_OVERRIDE_FIXTURE.contains(marker),
            "missing Array reverse override witness `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains("fn run_wasm_backend_calls_an_arrays_own_reverse_method()"));
    assert!(ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_array_reverse_own_method_dispatch.js\")"));
}

#[test]
fn removed_dense_reverse_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("emit_array_reverse_method_call")
            .count(),
        0
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("pub(crate) fn compile_array_prototype_reverse_builtin(")
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("fn compile_typed_array_prototype_reverse_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeReverse => {",
        "            StandardBuiltinId::ArrayPrototypeCopyWithin => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_reverse_builtin(function)?;")
            .count(),
        1
    );
    let typed_standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TypedArrayPrototypeReverse => {",
        "            StandardBuiltinId::TypedArrayPrototypeCopyWithin => {",
    );
    assert_eq!(
        typed_standard_arm
            .matches("self.compile_typed_array_prototype_reverse_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn canonical_reverse_owns_presence_observation_and_mutation_order() {
    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_reverse_builtin(",
        "    pub(crate) fn compile_array_prototype_copy_within_builtin(",
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
        1
    );
    assert_eq!(
        canonical
            .matches("emit_to_length_i64_from_value_locals(")
            .count(),
        1
    );

    let property_reads = offsets(canonical, "self.emit_object_read(");
    let presence_reads = offsets(canonical, "self.emit_object_has_property_i32(");
    assert_eq!(property_reads.len(), 3);
    assert_eq!(presence_reads.len(), 2);
    assert!(property_reads[0] < presence_reads[0]);
    assert!(presence_reads[0] < property_reads[1]);
    assert!(property_reads[1] < presence_reads[1]);
    assert!(presence_reads[1] < property_reads[2]);

    assert_eq!(
        canonical.matches("self.emit_object_write_strict(").count(),
        4
    );
    assert_eq!(
        canonical
            .matches("self.emit_delete_property_or_throw(")
            .count(),
        2
    );
    assert!(canonical.contains("Instruction::LocalSet(self.result_local)"));
    assert!(canonical.contains("Instruction::LocalSet(self.result_tag_local)"));
}

#[test]
fn typed_array_reverse_runtime_control_requires_internal_length_dispatch() {
    for marker in [
        "var ignoresLength = new Uint8Array([6, 7]);",
        "Object.defineProperty(ignoresLength, \"length\", {",
        "throw \"length must not be read\";",
        "ignoresLength.reverse();",
        "assertSame(ignoresLength[0], 7, \"internal length first value\");",
        "assertSame(ignoresLength[1], 6, \"internal length second value\");",
    ] {
        assert!(
            TYPED_ARRAY_REVERSE_FIXTURE.contains(marker),
            "missing TypedArray reverse dispatch witness `{marker}`"
        );
    }
    assert!(TYPED_ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_reverses_typedarray_in_place_and_returns_receiver()"));
    assert!(
        TYPED_ARRAY_CLI_TESTS.contains("fixture_path(\"wasm_typedarray_prototype_reverse.js\")")
    );
}
