use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FIND_SOURCE: &str = include_str!("../src/builtins/array/find_via_predicate.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const FIND_STRUCTURE_GUARD: &str = include_str!("find_via_predicate_structure.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const FIND_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_array_find_core.js");

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
fn array_find_arm_delegates_without_changing_typed_array_or_iterator_dispatch() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::Find.property_name())",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"findIndex\") {",
    );
    for marker in [
        "let receiver_is_array =",
        "let receiver_is_iterator =",
        "let receiver_has_typed_array_find =",
        "let receiver_has_array_find =",
        "if receiver_is_iterator {",
        "if receiver_has_typed_array_find {",
        "StandardBuiltinId::TypedArrayPrototypeFind,",
        "if receiver_is_array || receiver_has_array_find {",
        "StandardBuiltinId::ArrayPrototypeFind,",
        "if receiver_needs_dynamic_helper_dispatch(receiver) {",
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
    assert_eq!(
        direct
            .matches(".emit_iterator_prototype_helper_method_call(")
            .count(),
        2
    );

    let array_arm = bounded(
        direct,
        "            if receiver_is_array || receiver_has_array_find {",
        "            // Every receiver-specific alternative has been ruled out above",
    );
    for marker in [
        "StandardBuiltinId::ArrayPrototypeFind,",
        "\"Array.prototype.find\",",
        "                    receiver,",
        "                    args,",
        "                    payload_local,",
        "                    tag_local,",
    ] {
        assert!(
            array_arm.contains(marker),
            "missing Array arm marker `{marker}`"
        );
    }
    assert!(!array_arm.contains("emit_array_find_method_call"));
}

#[test]
fn removed_array_find_wrapper_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source.matches("emit_array_find_method_call").count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_find_builtin(")
            .count(),
        1
    );
    assert!(!FIND_STRUCTURE_GUARD.contains("emit_array_find_method_call"));

    let standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeFind => {",
        "            StandardBuiltinId::ArrayPrototypeFindIndex => {",
    );
    assert_eq!(
        standard
            .matches(
                "self.compile_array_prototype_find_builtin(function, FindViaPredicateKind::Find)?;"
            )
            .count(),
        1
    );
}

#[test]
fn shared_call_boundary_and_canonical_find_compiler_own_order() {
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
        FIND_SOURCE,
        "    pub(crate) fn compile_array_prototype_find_builtin(",
        "\n}\n",
    );
    for (earlier, later) in [
        (
            "self.emit_array_iteration_to_object(",
            "self.emit_validate_find_predicate(",
        ),
        (
            "self.emit_validate_find_predicate(",
            "self.emit_builtin_arg_to_locals(1,",
        ),
        (
            "self.emit_builtin_arg_to_locals(1,",
            "self.emit_array_index_get_with_prototype(",
        ),
        (
            "self.emit_array_index_get_with_prototype(",
            "self.emit_call_validated_find_predicate(",
        ),
        (
            "self.emit_call_validated_find_predicate(",
            "self.compile_truthy_tagged_i32(",
        ),
        (
            "self.compile_truthy_tagged_i32(",
            "self.emit_project_find_match(",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn focused_find_control_covers_sparse_typed_array_and_proxy_callbacks() {
    for marker in [
        "sparse.find(function ()",
        "Array.prototype.find.call(fixedBytes",
        "rab.resize(3);",
        "let callableProxy = new Proxy(proxyTarget",
        "let proxyFindResult = proxySource.find(callableProxy, proxyThis);",
        "proxyFindResult === 5",
        "proxyArgsOk === true",
    ] {
        assert!(
            FIND_FIXTURE.contains(marker),
            "missing Find marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_find_core_fixture()"));
}
