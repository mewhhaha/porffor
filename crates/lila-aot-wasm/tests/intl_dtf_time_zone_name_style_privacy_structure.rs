use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-time-zone-name-style-privacy.md"
);
const TASK: &str = include_str!("../../../tasks/23-intl402.md");

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
fn time_zone_name_style_is_owner_private_and_complete() {
    assert!(DTF_SOURCE.contains("\nenum TimeZoneNameStyle {"));
    assert!(!DTF_SOURCE.contains("pub(crate) enum TimeZoneNameStyle"));
    let domain = bounded(
        DTF_SOURCE,
        "enum TimeZoneNameStyle {",
        "impl TimeZoneNameStyle {",
    );
    for style in [
        "Short",
        "Long",
        "ShortOffset",
        "LongOffset",
        "ShortGeneric",
        "LongGeneric",
    ] {
        assert_eq!(domain.matches(&format!("    {style},")).count(), 1);
    }
}

#[test]
fn style_projections_are_exhaustive_and_single_owner() {
    let implementation = bounded(
        DTF_SOURCE,
        "impl TimeZoneNameStyle {",
        "/// The `(spelling, code)` list",
    );
    assert_eq!(implementation.matches("Self::").count(), 18);
    assert!(!implementation.contains("_ =>"));
    assert_eq!(DTF_SOURCE.matches("TimeZoneNameStyle").count(), 13);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "TimeZoneNameStyle"), 13);
}

#[test]
fn style_privacy_has_frozen_evidence() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TimeZoneNameStyle`"));
        assert!(
            evidence.contains("ee5ac6a3396cdf58e102796ca82dbc6c75bf2799fe8c93bc3c42f17d091ea117")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(CONTRACT.contains("does not close T23"));
}
