use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_finalization_registry_record_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/finalization-registry-record-heap-slot-authority.md"
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
fn finalization_registry_record_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum FinalizationRegistryRecordHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "CleanupCallbackTag,",
            "CleanupCallbackPayload,",
            "CellsPointer,",
            "CellsLength,",
            "CellsCapacity,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ")
                && line.contains(" for FinalizationRegistryRecordHeapSlot")
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
                "impl {capability} for FinalizationRegistryRecordHeapSlot"
            )),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_five_exact_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl FinalizationRegistryRecordHeapSlot {",
        "pub(crate) const HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 2);
    assert_eq!(implementation.matches("pointer: false").count(), 3);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::CleanupCallbackTag=>FinalizationRegistryRecordHeapSlotMetadata{",
            "record:\"finalization-registry-record\",name:\"cleanup_callback_tag\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::CleanupCallbackPayload=>FinalizationRegistryRecordHeapSlotMetadata{",
            "record:\"finalization-registry-record\",name:\"cleanup_callback_payload\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::CellsPointer=>FinalizationRegistryRecordHeapSlotMetadata{",
            "record:\"finalization-registry-record\",name:\"cells_ptr\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::CellsLength=>FinalizationRegistryRecordHeapSlotMetadata{",
            "record:\"finalization-registry-record\",name:\"cells_len\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::CellsCapacity=>FinalizationRegistryRecordHeapSlotMetadata{",
            "record:\"finalization-registry-record\",name:\"cells_cap\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,width:8,pointer:false,},"
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
fn typed_registry_preserves_callback_tag_payload_pointer_length_capacity_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_FINALIZATION_REGISTRY_RECORD_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[FinalizationRegistryRecordHeapSlot]=&[",
            "FinalizationRegistryRecordHeapSlot::CleanupCallbackTag,",
            "FinalizationRegistryRecordHeapSlot::CleanupCallbackPayload,",
            "FinalizationRegistryRecordHeapSlot::CellsPointer,",
            "FinalizationRegistryRecordHeapSlot::CellsLength,",
            "FinalizationRegistryRecordHeapSlot::CellsCapacity,"
        )
    );
}

#[test]
fn finalization_registry_record_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_finalization_registry_record_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_finalization_registry_record_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"finalization-registry-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"finalization-registry-record\""),
        5
    );
    assert_eq!(
        recursive_rust_source_count(
            &source_root,
            "pub(crate) enum FinalizationRegistryRecordHeapSlot {"
        ),
        1
    );
    assert!(CONTRACT.contains("FinalizationRegistryRecordHeapSlot"));
    assert!(TASK.contains("finalization-registry-record-heap-slot-authority.md"));
}
