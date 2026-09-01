use std::fs;
use std::path::Path;

const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/obsolete-core-backend-api-removal.md");
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .filter(|token| *token == identifier)
                .count()
        })
        .sum()
}

#[test]
fn obsolete_core_backend_apis_are_absent() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for name in [
        "static_number_expr_value",
        "buffer_memarg32",
        "buffer_memarg16",
        "emit_store_realm_type_error_prototype",
        "standard_builtin_prototype_global_index",
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, name),
            0,
            "`{name}`"
        );
    }
}

#[test]
fn live_neighboring_apis_remain_reachable() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, expected) in [
        ("emit_string_substring_method_call", 2),
        ("emit_string_char_code_at_from_locals", 9),
        ("buffer_memarg64", 11),
        ("buffer_memarg8", 31),
        ("emit_store_realm_message_error_prototype", 4),
        ("emit_store_current_realm_message_error_prototype", 10),
        ("standard_builtin_function_global_index", 3),
        ("standard_builtin_constructor_global_index", 17),
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, name),
            expected,
            "`{name}`"
        );
    }
}

#[test]
fn removal_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "f3bc9cf6043c6d927bf0d51a9f600cf28f1c2e86291f623c47ba9406b35bc0c7",
            "6af38235bb977a2b2673f8424ea1bfa1b4fb4b958df5f4a06b9490bb8e270b48",
            "7860a2a85f440682f332a7be0a6bee8d1a7f92eaa2de78025329ae026dd699fb",
            "ceac7d89945f7aeaeff7721ff47901b0c9980405f35d05aa33396ec25aab608b",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
