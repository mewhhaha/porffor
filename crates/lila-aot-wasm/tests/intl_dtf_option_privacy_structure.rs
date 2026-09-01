use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/intl-date-time-format-option-privacy.md");
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
fn option_record_and_tables_are_owner_private() {
    assert!(DTF_SOURCE.contains("\nstruct IntlDtfOption {"));
    assert!(!DTF_SOURCE.contains("pub(crate) struct IntlDtfOption"));
    let record = bounded(
        DTF_SOURCE,
        "struct IntlDtfOption {",
        "/// ECMA-402 11.5 Table 7",
    );
    assert!(!record.contains("pub("));
    for field in ["property:", "slot_offset:", "codes:"] {
        assert_eq!(record.matches(field).count(), 1, "field `{field}`");
    }

    for table in [
        "INTL_DTF_COMPONENT_OPTIONS",
        "INTL_DTF_HOUR_CYCLE_OPTION",
        "INTL_DTF_DATE_STYLE_OPTION",
        "INTL_DTF_TIME_STYLE_OPTION",
    ] {
        assert!(DTF_SOURCE.contains(&format!("const {table}:")));
        assert!(!DTF_SOURCE.contains(&format!("pub(crate) const {table}:")));
    }
}

#[test]
fn option_family_has_one_recursive_owner_and_frozen_evidence() {
    assert_eq!(DTF_SOURCE.matches("IntlDtfOption").count(), 22);
    assert_eq!(DTF_SOURCE.matches("    IntlDtfOption {").count(), 10);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "IntlDtfOption"), 22);

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `IntlDtfOption`"));
        assert!(
            evidence.contains("8a430fd40eae20d6975444489d14d9c6d4ef75deaabe9bacde5bd8fce382dc0f")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(CONTRACT.contains("does not close T23"));
}
