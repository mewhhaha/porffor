use std::fs;
use std::path::Path;

const INSTANT_SOURCE: &str = include_str!("../src/builtins/temporal_instant.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-instant-diagnostic-privacy.md");
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
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
fn instant_diagnostics_are_owner_private() {
    for declaration in [
        "const TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE: &str =",
        "const TEMPORAL_INSTANT_VALUE_OF_MESSAGE: &str =",
    ] {
        assert_eq!(
            INSTANT_SOURCE.matches(declaration).count(),
            1,
            "`{declaration}`"
        );
        assert!(!INSTANT_SOURCE.contains(&format!("pub(crate) {declaration}")));
    }
}

#[test]
fn each_diagnostic_has_one_matching_throw_path() {
    let from_epoch_milliseconds = bounded(
        INSTANT_SOURCE,
        "pub(crate) fn emit_temporal_instant_from_epoch_milliseconds(",
        "/// Temporal proposal 8.3.12 `Temporal.Instant.prototype.valueOf`.",
    );
    assert_eq!(
        from_epoch_milliseconds
            .matches("TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE")
            .count(),
        1
    );
    assert!(from_epoch_milliseconds.contains("emit_throw_current_function_realm_range_error("));

    let value_of = bounded(
        INSTANT_SOURCE,
        "pub(crate) fn emit_temporal_instant_value_of(",
        "}\n}",
    );
    assert_eq!(
        value_of
            .matches("TEMPORAL_INSTANT_VALUE_OF_MESSAGE")
            .count(),
        1
    );
    assert!(value_of.contains("emit_throw_current_function_realm_type_error("));
}

#[test]
fn instant_diagnostics_have_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for name in [
        "TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE",
        "TEMPORAL_INSTANT_VALUE_OF_MESSAGE",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            2,
            "recursive `{name}`"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence
            .contains("owner-private `TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE`"));
        assert!(evidence.contains("owner-private `TEMPORAL_INSTANT_VALUE_OF_MESSAGE`"));
        assert!(
            evidence.contains("783be630ab0b186ca6e47d703313d37314540e71454c1d1ec5f994b93f4a249d")
        );
        assert!(
            evidence.contains("e50fae0dab7f68f5d12df521f40cea34c2d47ddf0e71078e02871ef30b754b11")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
