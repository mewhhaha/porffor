use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object/define_property.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const FILTER_OWNER_GUARD: &str = include_str!("array_filter_algorithm_owner_structure.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CONCAT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_concat_core.js");
const CONCAT_SPREADABLE_DESCRIPTOR_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_array_concat_spreadable_descriptor_assignment.js"
);
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-concat-spreadable-tagged-slot.md");
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
fn concat_spreadable_uses_only_the_ordinary_array_named_property_owner() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );
    for removed_owner in [
        "ArrayConcatSpreadableSlotValue",
        "HEAP_ARRAY_IS_CONCAT_SPREADABLE",
        "emit_array_is_concat_spreadable_read",
        "emit_array_is_concat_spreadable_write",
        "emit_array_is_concat_spreadable_slot_write",
    ] {
        assert_eq!(
            rust_source.matches(removed_owner).count(),
            0,
            "{removed_owner}"
        );
    }

    let own_read = bounded(
        OBJECTS_SOURCE,
        "    fn emit_array_own_named_property_read(",
        "    pub(crate) fn emit_object_own_property_present(",
    );
    assert!(own_read.contains("HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET"));
    assert!(own_read.contains("HEAP_OBJECT_GETTER_PAYLOAD_OFFSET"));
    assert!(!own_read.contains("Symbol.isConcatSpreadable"));

    let ordinary_set = bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(",
        "    pub(crate) fn emit_is_standard_builtin_constructor_payload(",
    );
    assert!(ordinary_set.contains("HEAP_OBJECT_SETTER_PAYLOAD_OFFSET"));
    assert!(ordinary_set.contains("OBJECT_DESCRIPTOR_WRITABLE"));

    let concat = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_concat_builtin(",
        "    pub(crate) fn compile_array_prototype_flat_map_builtin(",
    );
    let spreadable_read = bounded(
        concat,
        "property_key_symbol_payload(\"Symbol.isConcatSpreadable\")",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_eq!(spreadable_read.matches("self.emit_object_read(").count(), 1);
    assert_eq!(
        spreadable_read
            .matches("self.emit_arguments_is_concat_spreadable_read(")
            .count(),
        1
    );

    for owner in [
        "emit_array_define_named_accessor_descriptor(",
        "emit_array_define_named_data_descriptor(",
    ] {
        assert!(
            OBJECT_SOURCE.matches(owner).count() >= 2,
            "named owner `{owner}`"
        );
    }
    for marker in [
        "Object.defineProperty(getterOnly, Symbol.isConcatSpreadable",
        "getterOnly[Symbol.isConcatSpreadable] = true;",
        "Object.defineProperty(setterArray, Symbol.isConcatSpreadable",
        "setterArray[Symbol.isConcatSpreadable] = true;",
        "Object.defineProperty(nonWritable, Symbol.isConcatSpreadable",
        "\"use strict\";",
    ] {
        assert!(
            CONCAT_SPREADABLE_DESCRIPTOR_FIXTURE.contains(marker),
            "missing descriptor witness `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_preserves_array_concat_spreadable_descriptor_assignment_semantics()"
    ));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("ordinary Array named-property owner"));
    }
}

#[test]
fn concat_fallback_delegates_without_changing_string_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"concat\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"flat\") {",
    );

    for marker in [
        "let receiver_is_string =",
        "let receiver_has_string_concat =",
        "StandardBuiltinId::StringPrototypeConcat,",
        "\"String.prototype.concat\",",
        "StandardBuiltinId::ArrayPrototypeConcat,",
        "\"Array.prototype.concat\",",
    ] {
        assert!(
            direct.contains(marker),
            "missing dispatch marker `{marker}`"
        );
    }
    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        2
    );
    assert_eq!(direct.matches("                receiver,").count(), 2);
    assert_eq!(direct.matches("                args,").count(), 2);
    assert!(!direct.contains("emit_array_concat_method_call"));
}

#[test]
fn removed_array_concat_wrapper_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source.matches("emit_array_concat_method_call").count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_concat_builtin(")
            .count(),
        1
    );
    assert!(!FILTER_OWNER_GUARD.contains("emit_array_concat_method_call"));

    let standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeConcat => {",
        "            StandardBuiltinId::ArrayPrototypeJoin => {",
    );
    assert_eq!(
        standard
            .matches("self.compile_array_prototype_concat_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_concat_compiler_own_order() {
    let direct_call = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn emit_array_direct_builtin_method_call(",
        "    pub(crate) fn compile_array_prototype_join_builtin(",
    );
    for (earlier, later) in [
        (
            "self.compile_expr_to_locals(",
            "self.emit_propagate_throw_from_locals_if_needed(",
        ),
        (
            "self.emit_propagate_throw_from_locals_if_needed(",
            "self.emit_call_args_vector(args, function)",
        ),
        (
            "self.emit_call_args_vector(args, function)",
            "self.emit_direct_js_call_with_argv(",
        ),
    ] {
        assert_before(direct_call, earlier, later);
    }

    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_concat_builtin(",
        "    pub(crate) fn compile_array_prototype_flat_map_builtin(",
    );
    for (earlier, later) in [
        (
            "self.emit_array_iteration_to_object(",
            "self.emit_array_constructor_read(",
        ),
        (
            "self.emit_array_constructor_read(",
            "property_key_symbol_payload(\"Symbol.species\")",
        ),
        (
            "property_key_symbol_payload(\"Symbol.species\")",
            "self.emit_alloc_array_payload_with_length(",
        ),
        (
            "self.emit_alloc_array_payload_with_length(",
            "self.emit_function_handle_construct_with_argv(",
        ),
        (
            "property_key_symbol_payload(\"Symbol.isConcatSpreadable\")",
            "self.emit_concat_length_of_array_like(",
        ),
        (
            "self.emit_concat_length_of_array_like(",
            "self.emit_concat_typed_array_has_index_i32(",
        ),
        (
            "self.emit_concat_typed_array_has_index_i32(",
            "self.emit_array_index_get_with_prototype(",
        ),
        (
            "self.emit_array_index_get_with_prototype(",
            "self.emit_object_write(",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn focused_concat_control_covers_generic_sparse_and_multi_argument_results() {
    for marker in [
        "let sparseResult = sparse.concat([3]);",
        "let zeroArg = zeroSource.concat();",
        "let arrayArg = [1].concat([2, 3]);",
        "let nonArrayArg = [1].concat(objectValue);",
        "let multipleArgs = [1].concat([2], 3, [4, 5]);",
        "Object.prototype.hasOwnProperty.call(sparseResult, \"1\") === false",
    ] {
        assert!(
            CONCAT_FIXTURE.contains(marker),
            "missing Concat marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_concat_core_fixture()"));
}
