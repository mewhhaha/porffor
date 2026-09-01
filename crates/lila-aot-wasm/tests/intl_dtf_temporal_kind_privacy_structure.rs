use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-temporal-kind-privacy.md"
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

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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
fn temporal_kind_table_is_owner_private_and_complete() {
    assert!(DTF_SOURCE.contains("\nstruct IntlDtfTemporalKind {"));
    assert!(DTF_SOURCE.contains("\nenum DtfTimeBasis {"));
    assert!(DTF_SOURCE.contains("\nconst INTL_DTF_TEMPORAL_KINDS: &[IntlDtfTemporalKind] = &["));
    for forbidden in [
        "pub(crate) struct IntlDtfTemporalKind",
        "pub(crate) enum DtfTimeBasis",
        "pub(crate) const INTL_DTF_TEMPORAL_KINDS",
    ] {
        assert!(!DTF_SOURCE.contains(forbidden), "found `{forbidden}`");
    }

    let record = bounded(
        DTF_SOURCE,
        "struct IntlDtfTemporalKind {",
        "/// Whether a time value is an exact point",
    );
    assert!(!record.contains("pub("));
    for field in [
        "code:",
        "brand:",
        "type_name:",
        "allowed:",
        "defaults:",
        "rejected_style:",
        "basis:",
    ] {
        assert_eq!(record.matches(field).count(), 1, "field `{field}`");
    }

    let basis = normalized(bounded(
        DTF_SOURCE,
        "enum DtfTimeBasis {",
        "/// The `allowed` set shared by the instant-like types",
    ));
    assert!(basis.contains("Exact,"));
    assert!(basis.contains("Plain,"));

    let table = bounded(
        DTF_SOURCE,
        "const INTL_DTF_TEMPORAL_KINDS: &[IntlDtfTemporalKind] = &[",
        "/// The `kind_local` code for `Temporal.ZonedDateTime`.",
    );
    assert_eq!(table.matches("    IntlDtfTemporalKind {").count(), 6);
    assert_eq!(table.matches("basis: DtfTimeBasis::Plain,").count(), 5);
    assert_eq!(table.matches("basis: DtfTimeBasis::Exact,").count(), 1);
}

#[test]
fn value_kind_domains_are_owner_private_and_exhaustive() {
    for domain in ["DtfBrandedKind", "DtfValueKind"] {
        assert!(DTF_SOURCE.contains(&format!("\nenum {domain} {{")));
        assert!(!DTF_SOURCE.contains(&format!("pub(crate) enum {domain}")));
    }

    let branded = normalized(bounded(
        DTF_SOURCE,
        "enum DtfBrandedKind {",
        "impl DtfBrandedKind {",
    ));
    assert!(branded.contains("Temporal(&'staticIntlDtfTemporalKind),"));
    assert!(branded.contains("ZonedDateTime,"));

    let value = normalized(bounded(
        DTF_SOURCE,
        "enum DtfValueKind {",
        "impl DtfValueKind {",
    ));
    assert!(value.contains("Legacy,"));
    assert!(value.contains("Branded(DtfBrandedKind),"));

    let branded_projection = bounded(
        DTF_SOURCE,
        "impl DtfBrandedKind {",
        "/// What one `format`/`formatRange` argument turned out to be",
    );
    let value_projection = bounded(
        DTF_SOURCE,
        "impl DtfValueKind {",
        "/// The two halves of a resolved time zone",
    );
    assert!(!branded_projection.contains("_ =>"));
    assert!(!value_projection.contains("_ =>"));
}

#[test]
fn temporal_kind_family_has_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, count) in [
        ("IntlDtfTemporalKind", 10),
        ("DtfTimeBasis", 10),
        ("INTL_DTF_TEMPORAL_KINDS", 18),
        ("DtfBrandedKind", 7),
        ("DtfValueKind", 12),
    ] {
        assert_eq!(DTF_SOURCE.matches(name).count(), count, "owner `{name}`");
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            count,
            "recursive `{name}`"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `IntlDtfTemporalKind`"));
        assert!(evidence.contains("owner-private `DtfTimeBasis`"));
        assert!(evidence.contains("owner-private `DtfBrandedKind`"));
        assert!(evidence.contains("owner-private `DtfValueKind`"));
        assert!(
            evidence.contains("d6a9458aa55ec9362cf9fb4481717b58c78f7d4a5bc856d3b377444c853a5d7e")
        );
        assert!(
            evidence.contains("72754a52a21a06e8db39307da0b08b43ae0d57ec28e5f7cc212e0ca1e032ee40")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(CONTRACT.contains("does not close T23"));
}
