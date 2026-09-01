use std::fs;
use std::path::Path;

const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/obsolete-builtin-emitter-removal.md");
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
fn obsolete_builtin_emitters_are_absent_from_backend_sources() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for name in [
        "emit_date_time_within_day",
        "emit_throw_if_shared_array_buffer",
        "emit_string_match_all_global_ascii_word_iterator_from_string_locals",
    ] {
        assert_eq!(
            count_identifier_in_rust_sources(&source_root, name),
            0,
            "`{name}`"
        );
    }
}

#[test]
fn live_neighboring_emitters_remain_owned_and_reachable() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, expected) in [
        ("emit_date_positive_mod", 13),
        ("emit_date_make_time", 6),
        ("emit_throw_if_array_buffer_immutable", 6),
        (
            "emit_string_match_all_global_ascii_word_iterator_from_string_locals_from_start",
            2,
        ),
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
            "e69fe8ffc2517b72e18a85800ae0556736ede49cf01cd12c29a563008d7d3767",
            "df9bc99017d1ab0080f962469ea29e263e3d59c15ba720e2eacfe099dacca563",
            "934f091b5e4b1e04057b0a56b51a7897dc1c2537057b748d4e4f01f411198471",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
