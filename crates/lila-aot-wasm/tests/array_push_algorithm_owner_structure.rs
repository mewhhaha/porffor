use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const PUSH_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_push_argument_expansion.js");

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
fn static_array_push_delegates_with_complete_arguments() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"push\")",
        "        if matches!(key, PropertyKeyIr::StaticString(name) if name == \"toLocaleString\")",
    );

    for marker in [
        ".is_subset_of(KindSet::from_kind(ValueKind::Array))",
        "self.emit_array_direct_builtin_method_call(",
        "StandardBuiltinId::ArrayPrototypePush,",
        "\"Array.prototype.push\",",
        "                receiver,",
        "                args,",
    ] {
        assert!(direct.contains(marker), "missing Push marker `{marker}`");
    }
    assert!(!direct.contains("emit_array_push_method_call"));
}

#[test]
fn push_has_one_unbounded_canonical_owner() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );
    assert_eq!(
        rust_source.matches("emit_array_push_method_call").count(),
        0
    );

    let canonical = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypePush => {",
        "            StandardBuiltinId::ArrayPrototypeShift => {",
    );
    assert!(!canonical.contains("for arg_index in 0..8"));
    assert_eq!(
        canonical
            .matches("Instruction::Loop(BlockType::Empty)")
            .count(),
        2
    );
    assert_eq!(
        canonical
            .matches("self.emit_array_read(\n                    self.argv_param_local(),")
            .count(),
        2
    );
    assert_eq!(canonical.matches("Instruction::Br(0)").count(), 2);
    assert_eq!(
        canonical
            .matches("let arg_index_local = self.reserve_temp_local();")
            .count(),
        1
    );
    assert_eq!(
        canonical
            .matches("self.release_temp_local(arg_index_local);")
            .count(),
        1
    );
}

#[test]
fn shared_boundary_and_both_push_receiver_paths_preserve_order() {
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
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypePush => {",
        "            StandardBuiltinId::ArrayPrototypeShift => {",
    );
    let (dense, generic) = canonical
        .split_once(
            "                function.instruction(&Instruction::Else);\n\n                function.instruction(&Instruction::I64Const(self.strings.payload(\"length\")));",
        )
        .expect("dense and generic Push receiver paths");

    for (earlier, later) in [
        ("HEAP_LEN_OFFSET", "Instruction::LocalSet(arg_index_local)"),
        (
            "self.emit_array_read(",
            "self.emit_array_inherited_index_set_state(",
        ),
        (
            "self.emit_array_inherited_index_set_state(",
            "self.emit_array_length_writable_i64(",
        ),
        (
            "self.emit_array_length_writable_i64(",
            "self.store_i64_local_at_offset(",
        ),
    ] {
        assert_before(dense, earlier, later);
    }

    assert_eq!(generic.matches("self.emit_object_write(").count(), 2);
    for (earlier, later) in [
        (
            "self.emit_object_read(",
            "self.emit_to_length_i64_from_value_locals(",
        ),
        (
            "self.emit_to_length_i64_from_value_locals(",
            "MAX_SAFE_INTEGER",
        ),
        ("MAX_SAFE_INTEGER", "Instruction::LocalSet(arg_index_local)"),
        ("self.emit_array_read(", "self.emit_object_write("),
    ] {
        assert_before(generic, earlier, later);
    }
}

#[test]
fn focused_push_control_covers_more_than_eight_and_spread_arguments() {
    for marker in [
        "let target = [0];",
        "record(8),",
        "...spread,",
        "record(12),",
        "if (target.length !== 1) throw \"push started before spread expansion\";",
        "let correct = length === 13 && target.length === 13",
        "correct = correct && target[index] === index;",
    ] {
        assert!(
            PUSH_FIXTURE.contains(marker),
            "missing Push marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_expands_all_array_push_arguments_before_appending()"));
}
