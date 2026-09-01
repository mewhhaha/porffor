use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-extension-key-privacy.md"
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
fn relevant_extension_domains_are_owner_private_and_closed() {
    for domain in ["IntlDtfExtensionResolution", "IntlDtfRelevantExtensionKey"] {
        assert!(DTF_SOURCE.contains(&format!("\nenum {domain} {{")));
        assert!(!DTF_SOURCE.contains(&format!("pub(crate) enum {domain}")));
    }
    assert!(!DTF_SOURCE.contains("pub(crate) const ALL: [Self; 3]"));

    let resolution = normalized(bounded(
        DTF_SOURCE,
        "enum IntlDtfExtensionResolution {",
        "/// `Intl.DateTimeFormat`'s `[[RelevantExtensionKeys]]`",
    ));
    assert!(resolution.contains("CanonicalString{default:&'staticstr},"));
    assert!(resolution.contains("HourCycleCode,"));

    let key = normalized(bounded(
        DTF_SOURCE,
        "enum IntlDtfRelevantExtensionKey {",
        "impl IntlDtfRelevantExtensionKey {",
    ));
    assert!(key.contains("Ca,Hc,Nu,"));
}

#[test]
fn each_extension_key_selects_one_resolution_shape() {
    let implementation = normalized(bounded(
        DTF_SOURCE,
        "impl IntlDtfRelevantExtensionKey {",
        "/// Every byte is `0-9`, `a-z` or `-`; in particular no `A-Z`.",
    ));
    assert!(implementation.contains("constALL:[Self;3]=[Self::Ca,Self::Hc,Self::Nu];"));
    for mapping in [
        "Self::Ca=>IntlDtfExtensionResolution::CanonicalString{default:INTL_DTF_RESOLVED_CALENDAR,}",
        "Self::Hc=>IntlDtfExtensionResolution::HourCycleCode",
        "Self::Nu=>IntlDtfExtensionResolution::CanonicalString{default:INTL_DTF_RESOLVED_NUMBERING_SYSTEM,}",
    ] {
        assert_eq!(implementation.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert!(!implementation.contains("_=>"));

    assert!(DTF_SOURCE.contains("\nstruct IntlDtfKeywordNeedle {"));
    assert!(!DTF_SOURCE.contains("pub(crate) struct IntlDtfKeywordNeedle"));
    assert_eq!(DTF_SOURCE.matches("IntlDtfKeywordNeedle").count(), 5);
    let needle = bounded(
        DTF_SOURCE,
        "struct IntlDtfKeywordNeedle {",
        "const INTL_DTF_RESOLVED_CALENDAR",
    );
    assert_eq!(needle.matches("fn keyword(").count(), 1);
    assert_eq!(needle.matches("fn bytes(").count(), 1);
    assert_eq!(needle.matches("fn len(").count(), 1);
}

#[test]
fn extension_key_family_has_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, count) in [
        ("IntlDtfExtensionResolution", 12),
        ("IntlDtfRelevantExtensionKey", 26),
    ] {
        assert_eq!(DTF_SOURCE.matches(name).count(), count, "owner `{name}`");
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            count,
            "recursive `{name}`"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `IntlDtfExtensionResolution`"));
        assert!(evidence.contains("owner-private `IntlDtfRelevantExtensionKey`"));
        assert!(
            evidence.contains("81c65d3e0cba0940b53421102caa0536bb0820d0acbe531d75abbeb8f555274e")
        );
        assert!(
            evidence.contains("ab8dfe30ef006fe674cf1ba6663072d923941c92b568bb8825a15c29ccb4b8a2")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(CONTRACT.contains("does not close T23"));
}
