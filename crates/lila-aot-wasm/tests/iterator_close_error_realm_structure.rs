use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_iterator_close_generated_error_realm.js");
const ITERATOR_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/iterator-close-error-realm.md");
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker: {marker}"));
        cursor += offset + marker.len();
    }
}

fn count_in_rust_sources(directory: &Path, needle: &str) -> usize {
    fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn iterator_close_protocol_errors_use_the_current_function_realm_in_order() {
    let close = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_iterator_close(",
        "    pub(crate) fn emit_iterator_close_preserving_current_throw(",
    );

    assert_eq!(
        close
            .matches("self.emit_throw_current_function_realm_type_error(")
            .count(),
        2
    );
    assert_eq!(close.matches("emit_throw_runtime_error(").count(), 0);
    for message in [
        "IteratorClose return method must be callable",
        "IteratorClose return result must be object",
    ] {
        assert_eq!(close.matches(message).count(), 1, "message: {message}");
        let emission = format!(
            "self.emit_throw_current_function_realm_type_error(\n            \"{message}\""
        );
        assert_eq!(
            close.matches(emission.as_str()).count(),
            1,
            "emission: {message}"
        );
    }
    positions_in_order(
        close,
        &[
            "self.emit_object_read(",
            "self.emit_is_callable_i32(return_tag_local, return_payload_local, function)?;",
            "IteratorClose return method must be callable",
            "self.emit_propagate_current_throw(function);",
            "self.emit_function_handle_call(",
            "self.emit_propagate_current_completion_if_throw(function);",
            "self.emit_function_or_proxy_call_leave_throw_completion(",
            "self.emit_propagate_current_completion_if_throw(function);",
            "self.emit_is_heap_object_like_tag_i32(result_tag_local, function);",
            "IteratorClose return result must be object",
            "self.emit_propagate_current_throw(function);",
        ],
    );
}

#[test]
fn every_external_iterator_close_route_reaches_the_shared_realm_owner() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let direct_routes = count_in_rust_sources(&source_root, ".emit_iterator_close(");
    let preserving_current_routes = count_in_rust_sources(
        &source_root,
        ".emit_iterator_close_preserving_current_throw(",
    );
    let preserving_saved_routes =
        count_in_rust_sources(&source_root, ".emit_iterator_close_preserving_saved_throw(");

    let preserving_current_owner = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_iterator_close_preserving_current_throw(",
        "    pub(crate) fn emit_iterator_close_preserving_saved_throw(",
    );
    let preserving_saved_owner = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_iterator_close_preserving_saved_throw(",
        "    pub(crate) fn emit_iterator_flat_map_close_outer_after_throw(",
    );
    assert_eq!(
        preserving_current_owner
            .matches("self.emit_iterator_close_preserving_saved_throw(")
            .count(),
        1
    );
    assert_eq!(
        preserving_saved_owner
            .matches("self.emit_iterator_close(")
            .count(),
        1
    );

    let external_direct_routes = direct_routes - 1;
    let external_preserving_saved_routes = preserving_saved_routes - 1;
    assert_eq!(external_direct_routes, 16);
    assert_eq!(preserving_current_routes, 48);
    assert_eq!(external_preserving_saved_routes, 3);
    assert_eq!(
        external_direct_routes + preserving_current_routes + external_preserving_saved_routes,
        67
    );
}

#[test]
fn runtime_witness_uses_a_borrowed_iterator_helper_realm_for_both_errors() {
    for marker in [
        "var other = __lilaCreateRealm().global;",
        "caught instanceof other.TypeError",
        "!(caught instanceof TypeError)",
        "var nonCallableReturn =",
        "return: 0",
        "non-callable return",
        "var primitiveReturn =",
        "return 0;",
        "primitive return result",
        "var validReturn =",
    ] {
        assert!(FIXTURE.contains(marker), "fixture marker: {marker}");
    }
    assert_eq!(
        FIXTURE
            .matches("other.Iterator.prototype.some.call(")
            .count(),
        3
    );

    let cli_test = bounded(
        ITERATOR_CLI_TESTS,
        "fn run_wasm_backend_uses_borrowed_iterator_helper_realm_for_iterator_close_errors() {",
        "\n#[test]",
    );
    assert!(cli_test.contains("wasm_iterator_close_generated_error_realm.js"));
    assert!(cli_test.contains(".arg(\"wasm\")"));
    assert!(cli_test.contains("stdout.contains(\"backend_used: WasmAot\")"));
    assert!(cli_test.contains("stdout.contains(\"boolean(true)\")"));
    assert_eq!(
        ITERATOR_CLI_TESTS
            .matches("wasm_iterator_close_generated_error_realm.js")
            .count(),
        1
    );
    assert_eq!(
        ITERATOR_CLI_TESTS
            .matches(
                "fn run_wasm_backend_uses_borrowed_iterator_helper_realm_for_iterator_close_errors()",
            )
            .count(),
        1
    );
}

#[test]
fn published_boundary_names_the_owner_routes_witness_and_nonclaims() {
    for marker in [
        "emit_iterator_close",
        "IteratorClose return method must be callable",
        "IteratorClose return result must be object",
        "67",
        "16 routes call `emit_iterator_close` directly",
        "48 routes call `emit_iterator_close_preserving_current_throw`",
        "3 routes call `emit_iterator_close_preserving_saved_throw` directly",
        "wasm_iterator_close_generated_error_realm.js",
        "iterator_close_error_realm_structure",
        "run_wasm_backend_uses_borrowed_iterator_helper_realm_for_iterator_close_errors",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker: {marker}");
    }
    for source in [README, TASK] {
        assert!(source.contains("iterator-close-error-realm.md"));
        assert!(source.contains("67"));
    }
    for retired in ["LegacyMainRealm", "legacy main-Realm policy"] {
        assert!(
            !CONTRACT.contains(retired),
            "retired contract claim `{retired}`"
        );
    }
    let nonclaim = bounded(CONTRACT, "## Nonclaim", "## Focused verification");
    for marker in [
        "only the two errors created by IteratorClose",
        "direct `for-of`",
        "`GetIterator`",
        "`IteratorStep`",
        "direct-synchronous-for-of-protocol-error-realm.md",
        "sync-iterator-consumer-capability.md",
        "expands this contract's ownership beyond IteratorClose",
    ] {
        assert!(nonclaim.contains(marker), "nonclaim marker `{marker}`");
    }
}
