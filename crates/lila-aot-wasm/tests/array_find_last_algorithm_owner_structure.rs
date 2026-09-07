use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FIND_SOURCE: &str = include_str!("../src/builtins/array/find_via_predicate.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const FIND_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_find_last_core.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "{earlier} must precede {later}"
    );
}

fn collect_rust_source(path: &Path, source: &mut String) {
    for directory_entry in fs::read_dir(path).expect("Rust source directory") {
        let child_path = directory_entry.expect("Rust source directory entry").path();
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
fn direct_dispatch_retains_receiver_specific_alternatives() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"findLast\") {",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"findLastIndex\") {",
    );
    for marker in [
        "let receiver_has_typed_array_find_last =",
        "StandardBuiltinId::TypedArrayPrototypeFindLast,",
        "\"TypedArray.prototype.findLast\",",
        "StandardBuiltinId::ArrayPrototypeFindLast,",
        "\"Array.prototype.findLast\",",
    ] {
        assert!(direct.contains(marker), "missing dispatch marker: {marker}");
    }
    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        2
    );
    assert_eq!(direct.matches("                receiver,").count(), 2);
    assert_eq!(direct.matches("                args,").count(), 2);
    assert!(!direct.contains("emit_array_find_last_method_call"));
}

#[test]
fn fixed_entry_is_the_only_owner_and_removed_wrapper_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );
    assert_eq!(
        rust_source
            .matches("emit_array_find_last_method_call")
            .count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_prototype_find_last_builtin(")
            .count(),
        1
    );
    let standard = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeFindLast => {",
        "            StandardBuiltinId::ArrayPrototypeFindLastIndex => {",
    );
    assert_eq!(
        standard
            .matches("self.compile_array_prototype_find_last_builtin(function)?;")
            .count(),
        1
    );
    assert!(!standard.contains("FindViaPredicateKind"));
    let fixed = bounded(
        FIND_SOURCE,
        "    pub(in crate::builtins) fn compile_array_prototype_find_last_builtin(",
        "\n    }",
    );
    assert!(
        fixed.contains("self.compile_array_find_with_kind(function, FindViaPredicateKind::FindLast)")
    );
}

#[test]
fn shared_call_boundary_and_canonical_compiler_own_observable_order() {
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
    let canonical = FIND_SOURCE
        .split_once("    fn compile_array_find_with_kind(")
        .expect("generic compiler")
        .1;
    for (earlier, later) in [
        (
            "self.emit_array_like_length_snapshot(",
            "self.emit_validate_find_predicate(",
        ),
        (
            "self.emit_validate_find_predicate(",
            "self.emit_builtin_arg_to_locals(1,",
        ),
        (
            "self.emit_builtin_arg_to_locals(1,",
            "self.emit_initialize_find_index(&direction,",
        ),
        (
            "self.emit_initialize_find_index(&direction,",
            "self.emit_typed_array_or_object_index_read_from_locals(",
        ),
        (
            "self.emit_typed_array_or_object_index_read_from_locals(",
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
        (
            "self.emit_project_find_match(",
            "self.emit_advance_find_index(&direction,",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }
}

#[test]
fn retained_cli_fixture_covers_generic_and_proxy_callbacks() {
    for marker in [
        "let findLast = Array.prototype.findLast;",
        "let orderResult = [1, 2, 3].findLast(function (value, index, source)",
        "Array.prototype.findLast.call(fixedBytes",
        "rab.resize(3);",
        "let proxyFindLastResult = proxySource.findLast(callableProxy, proxyThis);",
        "proxyFindLastResult === 5",
        "proxySource.findLast(nonCallableProxy)",
        "proxySource.findLast(revokedCallableProxy.proxy)",
    ] {
        assert!(
            FIND_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_supported_array_find_last_core_fixture()"));
}
