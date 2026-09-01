use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_symbol_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/symbol-heap-slot-authority.md");
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
fn symbol_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(LAYOUT_SOURCE, "pub(crate) enum SymbolHeapSlot {", "\n}");
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "DescriptionTag,",
            "DescriptionPayload,",
            "RegistryKeyPayload,",
            "SymbolId,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for SymbolHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for SymbolHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_four_exact_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl SymbolHeapSlot {",
        "pub(crate) const HEAP_SYMBOL_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 2);
    assert_eq!(implementation.matches("pointer: false").count(), 2);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::DescriptionTag=>SymbolHeapSlotMetadata{record:\"symbol-record\",",
            "name:\"description_tag\",offset:HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::DescriptionPayload=>SymbolHeapSlotMetadata{record:\"symbol-record\",",
            "name:\"description_payload\",",
            "offset:HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::RegistryKeyPayload=>SymbolHeapSlotMetadata{record:\"symbol-record\",",
            "name:\"registry_key_payload\",",
            "offset:HEAP_SYMBOL_REGISTRY_KEY_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::SymbolId=>SymbolHeapSlotMetadata{record:\"symbol-record\",",
            "name:\"symbol_id\",offset:HEAP_SYMBOL_ID_OFFSET,width:8,pointer:false,},"
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
fn typed_registry_preserves_symbol_storage_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_SYMBOL_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[SymbolHeapSlot]=&[SymbolHeapSlot::DescriptionTag,",
            "SymbolHeapSlot::DescriptionPayload,SymbolHeapSlot::RegistryKeyPayload,",
            "SymbolHeapSlot::SymbolId,"
        )
    );
}

#[test]
fn symbol_layout_has_one_private_recursive_owner() {
    assert_eq!(LIB_SOURCE.matches("\nmod heap_symbol_layout;\n").count(), 1);
    assert!(!LIB_SOURCE.contains("\npub mod heap_symbol_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"symbol-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"symbol-record\""),
        4
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum SymbolHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("SymbolHeapSlot"));
    assert!(TASK.contains("symbol-heap-slot-authority.md"));
}
