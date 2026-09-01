use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_weak_set_entry_layout.rs");
const WEAK_EDGE_SOURCE: &str = include_str!("../src/heap_weak_edges.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/weak-set-entry-heap-slot-authority.md");
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
fn weak_set_entry_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum WeakSetEntryHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Present,", "ValueTag,", "ValuePayload,"],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for WeakSetEntryHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for WeakSetEntryHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_three_non_pointer_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl WeakSetEntryHeapSlot {",
        "pub(crate) const HEAP_WEAK_SET_ENTRY_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: false").count(), 3);
    assert!(!implementation.contains("pointer: true"));
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::Present=>WeakSetEntryHeapSlotMetadata{record:\"weak-set-entry\",",
            "name:\"present\",offset:HEAP_WEAK_SET_ENTRY_PRESENT_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::ValueTag=>WeakSetEntryHeapSlotMetadata{record:\"weak-set-entry\",",
            "name:\"value_tag\",offset:HEAP_WEAK_SET_ENTRY_VALUE_TAG_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::ValuePayload=>WeakSetEntryHeapSlotMetadata{record:\"weak-set-entry\",",
            "name:\"value_payload\",offset:HEAP_WEAK_SET_ENTRY_VALUE_PAYLOAD_OFFSET,",
            "width:8,pointer:false,},"
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
fn typed_registry_and_weak_edge_authority_preserve_non_retention() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_WEAK_SET_ENTRY_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[WeakSetEntryHeapSlot]=&[",
            "WeakSetEntryHeapSlot::Present,WeakSetEntryHeapSlot::ValueTag,",
            "WeakSetEntryHeapSlot::ValuePayload,"
        )
    );

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
    let retention = normalized(bounded(
        WEAK_EDGE_SOURCE,
        "impl HeapWeakEdgeKind {",
        "pub(crate) enum HeapWeakEdge {",
    ));
    assert!(retention.contains(concat!(
        "Self::EphemeronKey|Self::WeakTarget|Self::FinalizerToken=>{",
        "HeapWeakEdgeRetention::DoesNotRetain}"
    )));
}

#[test]
fn weak_set_entry_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_weak_set_entry_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_weak_set_entry_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"weak-set-entry\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"weak-set-entry\""),
        4
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum WeakSetEntryHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("WeakSetEntryHeapSlot"));
    assert!(TASK.contains("weak-set-entry-heap-slot-authority.md"));
}
