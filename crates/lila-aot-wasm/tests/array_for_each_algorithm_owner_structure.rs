use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CALLBACK_RECEIVER_GUARD: &str = include_str!("array_callback_receiver_kind_structure.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const FOR_EACH_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_foreach_resizable_typedarray.js");

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
fn removed_direct_for_each_owner_cannot_be_called() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("emit_array_for_each_method_call")
            .count(),
        0
    );
    assert_eq!(
        rust_source
            .matches("fn compile_array_like_for_each_builtin(")
            .count(),
        1
    );
}

#[test]
fn standard_dispatch_owns_array_like_and_typed_array_for_each() {
    for (start, end, receiver_kind) in [
        (
            "            StandardBuiltinId::ArrayPrototypeForEach => {",
            "            StandardBuiltinId::TypedArrayPrototypeForEach => {",
            "ArrayCallbackReceiverKind::ArrayLike",
        ),
        (
            "            StandardBuiltinId::TypedArrayPrototypeForEach => {",
            "            StandardBuiltinId::ArrayPrototypeFilter => {",
            "ArrayCallbackReceiverKind::TypedArray",
        ),
    ] {
        let standard_arm = bounded(STANDARD_SOURCE, start, end);
        assert_eq!(
            standard_arm
                .matches("self.compile_array_like_for_each_builtin(")
                .count(),
            1,
            "{start}"
        );
        assert_eq!(standard_arm.matches(receiver_kind).count(), 1, "{start}");
        assert!(!standard_arm.contains("emit_array_for_each_method_call"));
    }
}

#[test]
fn iterator_for_each_dispatch_remains_a_distinct_owner() {
    let iterator_arm = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == IteratorHelper::ForEach.property_name())",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if matches!(name.as_str(), \"trim\"",
    );

    for marker in [
        "receiver_shape_targets_iterator_helper(receiver, IteratorHelper::ForEach)",
        ".emit_iterator_prototype_helper_method_call(",
        "IteratorHelper::ForEach,",
        "                    args,",
        "MethodCallDestination::new(payload_local, tag_local)",
    ] {
        assert!(
            iterator_arm.contains(marker),
            "missing Iterator forEach marker `{marker}`"
        );
    }
    assert_eq!(
        iterator_arm
            .matches(".emit_iterator_prototype_helper_method_call(")
            .count(),
        1
    );
    assert!(!iterator_arm.contains("StandardBuiltinId::ArrayPrototypeForEach"));
    assert!(!iterator_arm.contains("emit_array_for_each_method_call"));
}

#[test]
fn canonical_for_each_order_and_focused_control_remain_owned() {
    let canonical = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_like_for_each_builtin(",
        "    pub(crate) fn emit_alloc_array_payload_with_length(",
    );
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
        1
    );
    for (earlier, later) in [
        (
            "self.emit_builtin_arg_to_locals(0,",
            "self.emit_builtin_arg_to_locals(1,",
        ),
        (
            "self.emit_builtin_arg_to_locals(1,",
            "self.emit_array_iteration_to_object(",
        ),
        (
            "self.emit_array_iteration_to_object(",
            "self.emit_is_callable_i32(",
        ),
        (
            "self.emit_is_callable_i32(",
            "self.emit_object_has_property_i32(",
        ),
        (
            "self.emit_object_has_property_i32(",
            "self.emit_array_index_get_with_prototype(",
        ),
        (
            "self.emit_array_index_get_with_prototype(",
            "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        ),
    ] {
        assert_before(canonical, earlier, later);
    }

    assert!(
        CALLBACK_RECEIVER_GUARD.contains("    pub(crate) fn emit_alloc_array_payload_with_length(")
    );
    assert!(!CALLBACK_RECEIVER_GUARD.contains("emit_array_for_each_method_call"));

    for marker in [
        "Array.prototype.forEach.call(array",
        "rab.resize(3);",
        "rab.resize(6);",
        "midBuffer.resize(2);",
        "same(seen, [10, 11]);",
    ] {
        assert!(
            FOR_EACH_FIXTURE.contains(marker),
            "missing forEach control marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS.contains(
        "fn run_wasm_backend_succeeds_for_supported_array_foreach_resizable_typedarray_fixture()"
    ));
}
