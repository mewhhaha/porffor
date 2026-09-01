use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_finalization_registry_cell_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const WEAK_EDGE_SOURCE: &str = include_str!("../src/heap_weak_edges.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/finalization-registry-cell-heap-slot-authority.md"
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
fn finalization_registry_cell_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum FinalizationRegistryCellHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "State,",
            "TargetTag,",
            "TargetPayload,",
            "HoldingsTag,",
            "HoldingsPayload,",
            "UnregisterTokenTag,",
            "UnregisterTokenPayload,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ")
                && line.contains(" for FinalizationRegistryCellHeapSlot")
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
                "impl {capability} for FinalizationRegistryCellHeapSlot"
            )),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_seven_exact_rows_and_retention_classes() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl FinalizationRegistryCellHeapSlot {",
        "pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 1);
    assert_eq!(implementation.matches("pointer: false").count(), 6);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::State=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"state\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::TargetTag=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"target_tag\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::TargetPayload=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"target_payload\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::HoldingsTag=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"holdings_tag\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::HoldingsPayload=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"holdings_payload\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::UnregisterTokenTag=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"unregister_token_tag\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::UnregisterTokenPayload=>FinalizationRegistryCellHeapSlotMetadata{",
            "record:\"finalization-registry-cell\",name:\"unregister_token_payload\",",
            "offset:HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,width:8,pointer:false,},"
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
    for weak_edge in [
        "Self::FinalizationRegistryTarget",
        "Self::FinalizationRegistryHoldings",
        "Self::FinalizationRegistryUnregisterToken",
    ] {
        assert!(WEAK_EDGE_SOURCE.contains(weak_edge));
    }
}

#[test]
fn typed_registry_preserves_state_and_three_tag_payload_pairs() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_FINALIZATION_REGISTRY_CELL_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[FinalizationRegistryCellHeapSlot]=&[",
            "FinalizationRegistryCellHeapSlot::State,",
            "FinalizationRegistryCellHeapSlot::TargetTag,",
            "FinalizationRegistryCellHeapSlot::TargetPayload,",
            "FinalizationRegistryCellHeapSlot::HoldingsTag,",
            "FinalizationRegistryCellHeapSlot::HoldingsPayload,",
            "FinalizationRegistryCellHeapSlot::UnregisterTokenTag,",
            "FinalizationRegistryCellHeapSlot::UnregisterTokenPayload,"
        )
    );
}

#[test]
fn finalization_registry_cell_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_finalization_registry_cell_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_finalization_registry_cell_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"finalization-registry-cell\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"finalization-registry-cell\""),
        10
    );
    assert_eq!(
        recursive_rust_source_count(
            &source_root,
            "pub(crate) enum FinalizationRegistryCellHeapSlot {"
        ),
        1
    );
    assert!(CONTRACT.contains("FinalizationRegistryCellHeapSlot"));
    assert!(TASK.contains("finalization-registry-cell-heap-slot-authority.md"));
}
