use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARGUMENT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_last_index_of_argument_evaluation.js");

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
fn direct_last_index_of_branch_delegates_with_the_complete_argument_list() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"lastIndexOf\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Find.property_name())",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypeLastIndexOf,")
            .count(),
        1
    );
    assert_eq!(
        direct.matches("\"Array.prototype.lastIndexOf\",").count(),
        1
    );
    assert_eq!(direct.matches("                args,").count(), 1);

    for forbidden in [
        "emit_array_last_index_of_method_call(",
        "args.first()",
        "args.get(1)",
        "compile_expr_to_locals(",
        "emit_array_last_index_of_from_locals(",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct lastIndexOf branch must not retain `{forbidden}`"
        );
    }
}

#[test]
fn removed_direct_last_index_of_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("emit_array_last_index_of_method_call")
            .count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_last_index_of_builtin(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("self.emit_array_last_index_of_from_locals(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeLastIndexOf => {",
        "            StandardBuiltinId::TypedArrayPrototypeIncludes => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_array_prototype_last_index_of_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_entry_own_argument_and_reverse_search_order() {
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
        "    pub(crate) fn compile_array_prototype_last_index_of_builtin(",
        "    pub(crate) fn compile_typed_array_prototype_includes_builtin(",
    );
    assert_eq!(
        canonical
            .matches("self.emit_builtin_arg_to_locals(")
            .count(),
        2
    );
    assert_eq!(canonical.matches("self.argc_param_local()").count(), 1);
    assert_eq!(canonical.matches("ValueKind::Dynamic.tag()").count(), 1);
    assert_eq!(
        canonical
            .matches("self.emit_array_last_index_of_from_locals(")
            .count(),
        1
    );

    let algorithm = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn emit_array_last_index_of_from_locals(",
        "    fn emit_array_iteration_to_object(",
    );
    for (earlier, later) in [
        (
            "self.emit_array_iteration_to_object(",
            "self.emit_to_length_i64_from_value_locals(",
        ),
        (
            "self.emit_to_length_i64_from_value_locals(",
            "ValueKind::Dynamic.tag()",
        ),
        (
            "ValueKind::Dynamic.tag()",
            "self.emit_array_retreat_to_previous_present_index(",
        ),
        (
            "self.emit_array_retreat_to_previous_present_index(",
            "self.emit_object_has_property_i32(",
        ),
        (
            "self.emit_object_has_property_i32(",
            "self.emit_array_index_get_with_prototype(",
        ),
        (
            "self.emit_array_index_get_with_prototype(",
            "self.emit_tagged_payload_equality_i32(",
        ),
    ] {
        assert_before(algorithm, earlier, later);
    }
}

#[test]
fn focused_fixture_observes_all_arguments_before_reverse_index_search() {
    for marker in [
        "var index = receiver.lastIndexOf(",
        "ignoredThirdArgument(),",
        "...ignoredSpread",
        "order[0] === \"search\"",
        "order[2] === \"third\"",
        "order[3] === \"iterator\"",
        "order[5] === \"next2\"",
        "order[6] === \"get\"",
    ] {
        assert!(
            ARGUMENT_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_evaluates_all_array_last_index_of_arguments_before_search()"
    ));
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_succeeds_for_supported_array_lastindexof_fromindex_fixture()"
    ));
}
