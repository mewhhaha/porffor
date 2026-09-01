use std::fs;
use std::path::Path;

const LAYOUT: &str = include_str!("../src/heap_temporal_plain_date_time_layout.rs");
const HEAP: &str = include_str!("../src/heap.rs");
const LIB: &str = include_str!("../src/lib.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/temporal-plain-date-time-heap-slot-authority.md"
);
const TASK: &str = include_str!("../../../tasks/05-values-heap-gc.md");

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
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn recursive_rust_source_count(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return recursive_rust_source_count(&path, needle);
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
fn temporal_plain_date_time_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT,
        "pub(crate) enum TemporalPlainDateTimeHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "IsoYear,",
            "IsoMonth,",
            "IsoDay,",
            "Hour,",
            "Minute,",
            "Second,",
            "Millisecond,",
            "Microsecond,",
            "Nanosecond,",
            "CalendarPayload,",
        ]
    );
    assert!(!LAYOUT.contains("#[derive"));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!LAYOUT.contains(&format!(
            "impl {capability} for TemporalPlainDateTimeHeapSlot"
        )));
    }
}

#[test]
fn one_exhaustive_projection_owns_ten_exact_rows() {
    let implementation = bounded(
        LAYOUT,
        "impl TemporalPlainDateTimeHeapSlot {",
        "pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: false").count(), 9);
    assert_eq!(implementation.matches("pointer: true").count(), 1);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));

    let implementation = normalized(implementation);
    for (variant, name, offset, pointer) in [
        ("IsoYear", "iso_year", "ISO_YEAR", "false"),
        ("IsoMonth", "iso_month", "ISO_MONTH", "false"),
        ("IsoDay", "iso_day", "ISO_DAY", "false"),
        ("Hour", "hour", "HOUR", "false"),
        ("Minute", "minute", "MINUTE", "false"),
        ("Second", "second", "SECOND", "false"),
        ("Millisecond", "millisecond", "MILLISECOND", "false"),
        ("Microsecond", "microsecond", "MICROSECOND", "false"),
        ("Nanosecond", "nanosecond", "NANOSECOND", "false"),
        (
            "CalendarPayload",
            "calendar_payload",
            "CALENDAR_PAYLOAD",
            "true",
        ),
    ] {
        let row = format!(
            "Self::{variant}=>TemporalPlainDateTimeHeapSlotMetadata{{record:\"temporal-plain-date-time-record\",name:\"{name}\",offset:HEAP_TEMPORAL_PLAIN_DATE_TIME_{offset}_OFFSET,width:8,pointer:{pointer},}}"
        );
        assert!(implementation.contains(&row), "missing exact row `{row}`");
    }
}

#[test]
fn typed_registry_preserves_plain_date_time_component_order() {
    let registry = normalized(bounded(
        LAYOUT,
        "pub(crate) const HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT",
        "];",
    ));
    let expected = [
        "IsoYear",
        "IsoMonth",
        "IsoDay",
        "Hour",
        "Minute",
        "Second",
        "Millisecond",
        "Microsecond",
        "Nanosecond",
        "CalendarPayload",
    ]
    .into_iter()
    .map(|variant| format!("TemporalPlainDateTimeHeapSlot::{variant},"))
    .collect::<String>();
    assert_eq!(
        registry,
        format!(":&[TemporalPlainDateTimeHeapSlot]=&[{expected}")
    );
}

#[test]
fn temporal_plain_date_time_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB.matches("\nmod heap_temporal_plain_date_time_layout;\n")
            .count(),
        1
    );
    assert!(!LIB.contains("\npub mod heap_temporal_plain_date_time_layout;\n"));
    assert!(!HEAP.contains("record: \"temporal-plain-date-time-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"temporal-plain-date-time-record\""),
        10
    );
    assert_eq!(
        recursive_rust_source_count(
            &source_root,
            "pub(crate) enum TemporalPlainDateTimeHeapSlot {"
        ),
        1
    );
    assert!(CONTRACT.contains("TemporalPlainDateTimeHeapSlot"));
    assert!(TASK.contains("temporal-plain-date-time-heap-slot-authority.md"));
}
