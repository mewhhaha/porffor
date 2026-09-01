use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_object_entry_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-entry-heap-slot-authority.md");
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
fn object_entry_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum ObjectEntryHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "Key,",
            "DescriptorKind,",
            "DataTag,",
            "DataPayload,",
            "GetterTag,",
            "GetterPayload,",
            "SetterTag,",
            "SetterPayload,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for ObjectEntryHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for ObjectEntryHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_eight_exact_rows_and_retention_classes() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl ObjectEntryHeapSlot {",
        "pub(crate) const HEAP_OBJECT_ENTRY_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 4);
    assert_eq!(implementation.matches("pointer: false").count(), 4);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::Key=>ObjectEntryHeapSlotMetadata{record:\"object-entry\",name:\"key\",",
            "offset:HEAP_OBJECT_KEY_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::DescriptorKind=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"descriptor_kind\",",
            "offset:HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::DataTag=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"data_tag\",",
            "offset:HEAP_OBJECT_DATA_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::DataPayload=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"data_payload\",",
            "offset:HEAP_OBJECT_DATA_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::GetterTag=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"getter_tag\",",
            "offset:HEAP_OBJECT_GETTER_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::GetterPayload=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"getter_payload\",",
            "offset:HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::SetterTag=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"setter_tag\",",
            "offset:HEAP_OBJECT_SETTER_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::SetterPayload=>ObjectEntryHeapSlotMetadata{",
            "record:\"object-entry\",name:\"setter_payload\",",
            "offset:HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,width:8,pointer:true,},"
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
fn typed_registry_preserves_key_descriptor_and_tag_payload_pair_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_OBJECT_ENTRY_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[ObjectEntryHeapSlot]=&[",
            "ObjectEntryHeapSlot::Key,",
            "ObjectEntryHeapSlot::DescriptorKind,",
            "ObjectEntryHeapSlot::DataTag,",
            "ObjectEntryHeapSlot::DataPayload,",
            "ObjectEntryHeapSlot::GetterTag,",
            "ObjectEntryHeapSlot::GetterPayload,",
            "ObjectEntryHeapSlot::SetterTag,",
            "ObjectEntryHeapSlot::SetterPayload,"
        )
    );
}

#[test]
fn object_entry_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_object_entry_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_object_entry_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"object-entry\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"object-entry\""),
        8
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum ObjectEntryHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("ObjectEntryHeapSlot"));
    assert!(TASK.contains("object-entry-heap-slot-authority.md"));
}
