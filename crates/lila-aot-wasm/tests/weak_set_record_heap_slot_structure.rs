use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_weak_set_record_layout.rs");
const ENTRY_LAYOUT_SOURCE: &str = include_str!("../src/heap_weak_set_entry_layout.rs");
const WEAK_EDGE_SOURCE: &str = include_str!("../src/heap_weak_edges.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/weak-set-record-heap-slot-authority.md");
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
fn weak_set_record_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum WeakSetRecordHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "EntriesPointer,",
            "EntriesLength,",
            "EntriesCapacity,",
            "LiveCount,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for WeakSetRecordHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for WeakSetRecordHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_four_exact_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl WeakSetRecordHeapSlot {",
        "pub(crate) const HEAP_WEAK_SET_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 1);
    assert_eq!(implementation.matches("pointer: false").count(), 3);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::EntriesPointer=>WeakSetRecordHeapSlotMetadata{",
            "record:\"weak-set-record\",name:\"entries_ptr\",",
            "offset:HEAP_WEAK_SET_ENTRIES_PTR_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::EntriesLength=>WeakSetRecordHeapSlotMetadata{",
            "record:\"weak-set-record\",name:\"entries_len\",",
            "offset:HEAP_WEAK_SET_ENTRIES_LEN_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::EntriesCapacity=>WeakSetRecordHeapSlotMetadata{",
            "record:\"weak-set-record\",name:\"entries_cap\",",
            "offset:HEAP_WEAK_SET_ENTRIES_CAP_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::LiveCount=>WeakSetRecordHeapSlotMetadata{",
            "record:\"weak-set-record\",name:\"live_count\",",
            "offset:HEAP_WEAK_SET_LIVE_COUNT_OFFSET,width:8,pointer:false,},"
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
fn typed_registry_separates_storage_reachability_from_weak_value_retention() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_WEAK_SET_RECORD_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[WeakSetRecordHeapSlot]=&[",
            "WeakSetRecordHeapSlot::EntriesPointer,WeakSetRecordHeapSlot::EntriesLength,",
            "WeakSetRecordHeapSlot::EntriesCapacity,WeakSetRecordHeapSlot::LiveCount,"
        )
    );

    let entry_implementation = bounded(
        ENTRY_LAYOUT_SOURCE,
        "impl WeakSetEntryHeapSlot {",
        "pub(crate) const HEAP_WEAK_SET_ENTRY_LAYOUT",
    );
    assert_eq!(entry_implementation.matches("pointer: false").count(), 3);
    assert!(!entry_implementation.contains("pointer: true"));

    let weak_edges = normalized(bounded(
        WEAK_EDGE_SOURCE,
        "impl HeapWeakEdge {",
        "pub(crate) const HEAP_WEAK_EDGES",
    ));
    assert!(weak_edges.contains(concat!(
        "Self::WeakSetValue=>HeapWeakEdgeMetadata{",
        "record:\"weak-set-entry\",name:\"value\",",
        "kind:HeapWeakEdgeKind::EphemeronKey,},"
    )));
}

#[test]
fn weak_set_record_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_weak_set_record_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_weak_set_record_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"weak-set-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"weak-set-record\""),
        4
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum WeakSetRecordHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("WeakSetRecordHeapSlot"));
    assert!(TASK.contains("weak-set-record-heap-slot-authority.md"));
}
