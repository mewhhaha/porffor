use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const OBJECT_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CONVERSION_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_to_string_conversion_matrix.js");
const SUBCLASS_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_subclass_named_property_read.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
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
fn direct_array_to_string_routes_to_the_shared_standard_builtin() {
    let direct_branch = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"toString\")",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"reverse\")",
    );

    assert!(direct_branch.contains("&& args.is_empty()"));
    assert!(direct_branch.contains("KindSet::from_kind(ValueKind::Array)"));
    assert_eq!(
        direct_branch
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct_branch
            .matches("StandardBuiltinId::TypedArrayPrototypeToString,")
            .count(),
        1
    );
    assert_eq!(
        direct_branch
            .matches("\"Array.prototype.toString\",")
            .count(),
        1
    );
    for forbidden in [
        "emit_array_join_method_call(",
        "emit_array_join_array_from_locals(",
        "emit_array_join_with_length_from_locals(",
        "emit_array_index_get_with_prototype(",
        "HEAP_LEN_OFFSET",
    ] {
        assert!(
            !direct_branch.contains(forbidden),
            "direct Array toString must not own `{forbidden}`"
        );
    }
}

#[test]
fn removed_array_only_to_string_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    for removed_owner in [
        "emit_array_join_method_call",
        "emit_array_join_array_from_locals",
    ] {
        assert_eq!(
            rust_source.matches(removed_owner).count(),
            0,
            "removed Array toString owner `{removed_owner}` must stay absent"
        );
    }
    assert_eq!(
        rust_source
            .matches("fn compile_typed_array_prototype_to_string_builtin(")
            .count(),
        1
    );

    let standard_arm = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TypedArrayPrototypeToString => {",
        "            StandardBuiltinId::TypedArrayPrototypeJoin => {",
    );
    assert_eq!(
        standard_arm
            .matches("self.compile_typed_array_prototype_to_string_builtin(function)?;")
            .count(),
        1
    );
}

#[test]
fn canonical_to_string_owns_join_lookup_callability_and_fallback_order() {
    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_typed_array_prototype_to_string_builtin(",
        "    pub(crate) fn compile_array_prototype_to_locale_string_builtin(",
    );

    for operation in [
        "emit_value_to_current_function_realm_object_locals(",
        "self.strings.payload(\"join\")",
        "self.emit_object_read(",
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        "self.emit_object_prototype_to_string_result_from_locals(",
    ] {
        assert_eq!(
            canonical.matches(operation).count(),
            1,
            "canonical Array toString must own one `{operation}`"
        );
    }
    assert_before(
        canonical,
        "emit_value_to_current_function_realm_object_locals(",
        "self.strings.payload(\"join\")",
    );
    assert_before(
        canonical,
        "self.strings.payload(\"join\")",
        "self.emit_object_read(",
    );
    assert_before(
        canonical,
        "self.emit_object_read(",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        canonical,
        "self.emit_is_callable_i32(",
        "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
    );
    assert_before(
        canonical,
        "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        "self.emit_object_prototype_to_string_result_from_locals(",
    );

    let array_join = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_join_builtin(",
        "    pub(crate) fn compile_typed_array_prototype_join_builtin(",
    );
    assert_eq!(
        array_join
            .matches("self.emit_array_join_generic_from_locals(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("self.emit_array_join_with_length_from_locals(")
            .count(),
        2
    );
}

#[test]
fn focused_runtime_witnesses_cover_direct_dispatch_overrides_and_fallback() {
    for marker in [
        "var actual = array.toString();",
        "var joined = array.join();",
        "[throwingElement].join();",
    ] {
        assert!(
            CONVERSION_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    for marker in [
        "class DefaultArray extends Array",
        "join() { return \"overridden join\"; }",
        "direct.join() !== \"overridden join\"",
    ] {
        assert!(
            SUBCLASS_FIXTURE.contains(marker),
            "missing marker: {marker}"
        );
    }
    assert!(
        ARRAY_CLI_TESTS.contains("fn run_wasm_backend_matches_array_to_string_conversion_matrix()")
    );
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_succeeds_for_array_subclass_named_property_read_fixture()"));
    assert!(OBJECT_CLI_TESTS.contains(
        "fn object_prototype_tostring_classifies_proxy_arrays_and_rejects_revoked_proxies()"
    ));
}
