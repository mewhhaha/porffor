use std::fs;
use std::path::Path;

const PLAIN_DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-calendar-carrier-privacy.md");
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
fn calendar_carrier_and_raw_fast_path_are_owner_private() {
    assert!(PLAIN_DATE_SOURCE.contains("\nenum TemporalCalendarCarrier {"));
    assert!(!PLAIN_DATE_SOURCE.contains("pub(crate) enum TemporalCalendarCarrier"));

    let implementation = bounded(
        PLAIN_DATE_SOURCE,
        "impl TemporalCalendarCarrier {",
        "/// `ISODateToEpochDays` bounds",
    );
    assert!(!implementation.contains("pub("));
    for member in [
        "const ALL:",
        "const fn brand(",
        "const fn calendar_payload_offset(",
    ] {
        assert_eq!(
            implementation.matches(member).count(),
            1,
            "member `{member}`"
        );
    }

    assert_eq!(
        PLAIN_DATE_SOURCE
            .matches("    fn emit_temporal_calendar_slot_fast_path(")
            .count(),
        1
    );
    assert!(!PLAIN_DATE_SOURCE.contains("pub(crate) fn emit_temporal_calendar_slot_fast_path("));
}

#[test]
fn carrier_exhaustively_pairs_each_brand_with_its_calendar_slot() {
    let declaration = bounded(
        PLAIN_DATE_SOURCE,
        "enum TemporalCalendarCarrier {",
        "impl TemporalCalendarCarrier {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "PlainDate,",
            "PlainDateTime,",
            "PlainMonthDay,",
            "PlainYearMonth,",
            "ZonedDateTime,",
            "}",
        ]
    );

    let implementation = normalized(bounded(
        PLAIN_DATE_SOURCE,
        "impl TemporalCalendarCarrier {",
        "/// `ISODateToEpochDays` bounds",
    ));
    assert!(implementation.contains(
        "constALL:[Self;5]=[Self::PlainDate,Self::PlainDateTime,Self::PlainMonthDay,Self::PlainYearMonth,Self::ZonedDateTime,];"
    ));
    for mapping in [
        "Self::PlainDate=>OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE",
        "Self::PlainDateTime=>OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME",
        "Self::PlainMonthDay=>OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY",
        "Self::PlainYearMonth=>OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH",
        "Self::ZonedDateTime=>OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME",
        "Self::PlainDate|Self::PlainMonthDay|Self::PlainYearMonth=>{HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET}",
        "Self::PlainDateTime=>HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET",
        "Self::ZonedDateTime=>HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET",
    ] {
        assert_eq!(
            implementation.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }
    assert_eq!(implementation.matches("matchself{").count(), 2);
    assert!(!implementation.contains("_=>"));

    let fast_path = normalized(bounded(
        PLAIN_DATE_SOURCE,
        "fn emit_temporal_calendar_slot_fast_path(",
        "/// `CanonicalizeCalendar`, shared by every constructor",
    ));
    assert_eq!(
        fast_path
            .matches("forcarrierinTemporalCalendarCarrier::ALL{")
            .count(),
        1
    );
    assert_eq!(fast_path.matches("carrier.brand()").count(), 1);
    assert_eq!(
        fast_path
            .matches("carrier.calendar_payload_offset()")
            .count(),
        1
    );
}

#[test]
fn calendar_carrier_has_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, count) in [
        ("TemporalCalendarCarrier", 3),
        ("emit_temporal_calendar_slot_fast_path", 4),
    ] {
        assert_eq!(
            PLAIN_DATE_SOURCE.matches(name).count(),
            count,
            "owner `{name}`"
        );
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            count,
            "recursive `{name}`"
        );
    }
    assert_eq!(
        PLAIN_DATE_SOURCE
            .matches("self.emit_temporal_calendar_slot_fast_path(")
            .count(),
        2
    );

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TemporalCalendarCarrier`"));
        assert!(
            evidence.contains("1726881c45223f008814169edef8a3066c23b8733d86714d63570535ba3dd831")
        );
        assert!(
            evidence.contains("a74006922ea5018cd1d001421de4f83b70c23db9b73924ab24627415c642765c")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
