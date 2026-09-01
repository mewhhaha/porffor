use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_temporal_zoned_date_time_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/temporal-zoned-date-time-heap-slot-authority.md"
);
const TASK: &str = include_str!("../../../tasks/05-values-heap-gc.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
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
fn temporal_zoned_date_time_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum TemporalZonedDateTimeHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "EpochNanosecondsTag,",
            "EpochNanosecondsPayload,",
            "TimeZoneTag,",
            "TimeZonePayload,",
            "CalendarTag,",
            "CalendarPayload,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ")
                && line.contains(" for TemporalZonedDateTimeHeapSlot")
        }),
        "identity must not gain a manual trait capability"
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
        "Default",
    ] {
        assert!(
            !LAYOUT_SOURCE.contains(&format!(
                "impl {capability} for TemporalZonedDateTimeHeapSlot"
            )),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_three_exact_tag_payload_pairs() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl TemporalZonedDateTimeHeapSlot {",
        "pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 3);
    assert_eq!(implementation.matches("pointer: false").count(), 3);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::EpochNanosecondsTag=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"epoch_nanoseconds_tag\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::EpochNanosecondsPayload=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"epoch_nanoseconds_payload\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,",
            "width:8,pointer:true,},"
        ),
        concat!(
            "Self::TimeZoneTag=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"time_zone_tag\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::TimeZonePayload=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"time_zone_payload\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,",
            "width:8,pointer:true,},"
        ),
        concat!(
            "Self::CalendarTag=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"calendar_tag\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::CalendarPayload=>TemporalZonedDateTimeHeapSlotMetadata{",
            "record:\"temporal-zoned-date-time-record\",name:\"calendar_payload\",",
            "offset:HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,",
            "width:8,pointer:true,},"
        ),
    ] {
        assert!(implementation.contains(row), "missing exact row: {row}");
    }
    assert!(implementation.contains("letmetadata=self.metadata();"));
    for field in ["record", "name", "offset", "width", "pointer"] {
        assert!(
            implementation.contains(&format!("{field}:metadata.{field}")),
            "layout must project {field} through metadata"
        );
    }
}

#[test]
fn typed_registry_preserves_epoch_time_zone_calendar_pair_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[TemporalZonedDateTimeHeapSlot]=&[",
            "TemporalZonedDateTimeHeapSlot::EpochNanosecondsTag,",
            "TemporalZonedDateTimeHeapSlot::EpochNanosecondsPayload,",
            "TemporalZonedDateTimeHeapSlot::TimeZoneTag,",
            "TemporalZonedDateTimeHeapSlot::TimeZonePayload,",
            "TemporalZonedDateTimeHeapSlot::CalendarTag,",
            "TemporalZonedDateTimeHeapSlot::CalendarPayload,"
        )
    );
}

#[test]
fn temporal_zoned_date_time_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_temporal_zoned_date_time_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_temporal_zoned_date_time_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"temporal-zoned-date-time-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"temporal-zoned-date-time-record\""),
        6
    );
    assert_eq!(
        recursive_rust_source_count(
            &source_root,
            "pub(crate) enum TemporalZonedDateTimeHeapSlot {"
        ),
        1
    );
    assert!(CONTRACT.contains("TemporalZonedDateTimeHeapSlot"));
    assert!(TASK.contains("temporal-zoned-date-time-heap-slot-authority.md"));
}
