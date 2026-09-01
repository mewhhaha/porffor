use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-time-zone-authority-privacy.md"
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
fn time_zone_value_and_table_domains_are_owner_private() {
    for declaration in [
        "struct TzOffsetMinutes(i16);",
        "struct IntlDtfNamedZone {",
        "const INTL_DTF_NAMED_ZONES: &[IntlDtfNamedZone] = &[",
        "struct DtfCanonicalTimeZone {",
        "struct DtfResolvedTimeZone(DtfCanonicalTimeZone);",
    ] {
        assert_eq!(
            DTF_SOURCE.matches(declaration).count(),
            1,
            "`{declaration}`"
        );
    }
    for name in [
        "TzOffsetMinutes",
        "IntlDtfNamedZone",
        "INTL_DTF_NAMED_ZONES",
        "DtfCanonicalTimeZone",
        "DtfResolvedTimeZone",
    ] {
        assert!(!DTF_SOURCE.contains(&format!("pub(crate) struct {name}")));
        assert!(!DTF_SOURCE.contains(&format!("pub(crate) const {name}")));
    }

    let row = bounded(
        DTF_SOURCE,
        "struct IntlDtfNamedZone {",
        "impl IntlDtfNamedZone {",
    );
    assert!(!row.contains("pub("));
    assert_eq!(row.matches("identifier:").count(), 1);
    assert_eq!(row.matches("offset:").count(), 1);
}

#[test]
fn offset_range_and_named_zone_catalogue_remain_single_authorities() {
    let offset = normalized(bounded(
        DTF_SOURCE,
        "struct TzOffsetMinutes(i16);",
        "/// One row of `AvailableNamedTimeZoneIdentifiers()`.",
    ));
    for invariant in [
        "constMAX_HOUR:i64=23;",
        "constMAX_MINUTE:i64=59;",
        "constMAX:i16=(Self::MAX_HOUR*60+Self::MAX_MINUTE)asi16;",
        "constMIN:i16=-Self::MAX;",
        "Self::MIN..=Self::MAX=>Some(Self(minutes))",
        "constUTC:Self=Self::from_hours(0);",
    ] {
        assert!(offset.contains(invariant), "missing `{invariant}`");
    }

    let catalogue = bounded(
        DTF_SOURCE,
        "const INTL_DTF_NAMED_ZONES: &[IntlDtfNamedZone] = &[",
        "const fn intl_dtf_ascii_lower_byte",
    );
    assert_eq!(
        catalogue.matches("IntlDtfNamedZone::utc_alias(").count(),
        18
    );
    assert_eq!(catalogue.matches("IntlDtfNamedZone::etc_gmt(").count(), 26);
    let uniqueness = normalized(bounded(
        DTF_SOURCE,
        "/// The lookup is ASCII-case-insensitive",
        "/// The six `timeZoneName` widths",
    ));
    assert!(uniqueness.contains("whilei<INTL_DTF_NAMED_ZONES.len(){"));
    assert!(uniqueness.contains("whilej<INTL_DTF_NAMED_ZONES.len(){"));
    assert!(uniqueness.contains("!intl_dtf_ascii_eq_ignore_case("));
}

#[test]
fn reserved_and_resolved_zone_lifecycle_is_private_and_move_only() {
    let lifecycle = normalized(bounded(
        DTF_SOURCE,
        "struct DtfCanonicalTimeZone {",
        "/// The broken-down components of one side of a format.",
    ));
    assert!(!lifecycle.contains("derive(Clone"));
    assert!(!lifecycle.contains("derive(Copy"));
    assert_eq!(lifecycle.matches("fnstore(").count(), 1);
    assert_eq!(lifecycle.matches("fnrelease(").count(), 1);
    assert!(!normalized(bounded(
        DTF_SOURCE,
        "impl DtfCanonicalTimeZone {",
        "/// A [`DtfCanonicalTimeZone`] whose three locals",
    ))
    .contains("fnstore("));

    let resolver = normalized(bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_time_zone_option(",
        "/// `UTCOffset[~SubMinutePrecision]`",
    ));
    assert!(resolver.contains("zone:DtfCanonicalTimeZone,"));
    assert!(resolver.contains(")->Result<DtfResolvedTimeZone,EmitError>{"));
    assert_eq!(resolver.matches("Ok(DtfResolvedTimeZone(zone))").count(), 1);
    assert_eq!(
        DTF_SOURCE
            .matches("time_zone.store(self, record_local, function);")
            .count(),
        1
    );
    assert_eq!(DTF_SOURCE.matches("time_zone.release(self);").count(), 1);
}

#[test]
fn time_zone_authority_has_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, owner_count, recursive_count) in [
        ("TzOffsetMinutes", 8, 9),
        ("IntlDtfNamedZone", 47, 47),
        ("INTL_DTF_NAMED_ZONES", 12, 12),
        ("DtfCanonicalTimeZone", 8, 8),
        ("DtfResolvedTimeZone", 6, 6),
    ] {
        assert_eq!(
            DTF_SOURCE.matches(name).count(),
            owner_count,
            "owner `{name}`"
        );
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            recursive_count,
            "recursive `{name}`"
        );
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TzOffsetMinutes`"));
        assert!(evidence.contains("owner-private `IntlDtfNamedZone`"));
        assert!(evidence.contains("owner-private `DtfCanonicalTimeZone`"));
        assert!(evidence.contains("owner-private `DtfResolvedTimeZone`"));
        assert!(
            evidence.contains("4f284353c06da9e135d8ff5e863a7310b4abc94a7e1fcc57076be898e54cf641")
        );
        assert!(
            evidence.contains("6a1a4427fae20803f35d9b3a35c62f12a1ee3224259e2a620365326b358c7513")
        );
        assert!(evidence.contains("no new Intl behavior"));
    }
    assert!(CONTRACT.contains("does not close T23"));
}
