use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_temporal_plain_time_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-plain-time-heap-slot-authority.md");
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
fn temporal_plain_time_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum TemporalPlainTimeHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "Hour,",
            "Minute,",
            "Second,",
            "Millisecond,",
            "Microsecond,",
            "Nanosecond,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ")
                && line.contains(" for TemporalPlainTimeHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for TemporalPlainTimeHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_six_exact_scalar_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl TemporalPlainTimeHeapSlot {",
        "pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 0);
    assert_eq!(implementation.matches("pointer: false").count(), 6);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::Hour=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"hour\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::Minute=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"minute\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::Second=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"second\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::Millisecond=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"millisecond\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::Microsecond=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"microsecond\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::Nanosecond=>TemporalPlainTimeHeapSlotMetadata{",
            "record:\"temporal-plain-time-record\",name:\"nanosecond\",",
            "offset:HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,width:8,pointer:false,},"
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
fn typed_registry_preserves_time_component_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_TEMPORAL_PLAIN_TIME_RECORD_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[TemporalPlainTimeHeapSlot]=&[",
            "TemporalPlainTimeHeapSlot::Hour,TemporalPlainTimeHeapSlot::Minute,",
            "TemporalPlainTimeHeapSlot::Second,TemporalPlainTimeHeapSlot::Millisecond,",
            "TemporalPlainTimeHeapSlot::Microsecond,TemporalPlainTimeHeapSlot::Nanosecond,"
        )
    );
}

#[test]
fn temporal_plain_time_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_temporal_plain_time_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_temporal_plain_time_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"temporal-plain-time-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"temporal-plain-time-record\""),
        6
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum TemporalPlainTimeHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("TemporalPlainTimeHeapSlot"));
    assert!(TASK.contains("temporal-plain-time-heap-slot-authority.md"));
}
