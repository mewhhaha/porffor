use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_bigint_layout.rs");
const SIDE_STORAGE_SOURCE: &str = include_str!("../src/heap_side_storage.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/bigint-heap-slot-authority.md");
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
fn bigint_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(LAYOUT_SOURCE, "pub(crate) enum BigIntHeapSlot {", "\n}");
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Sign,", "LimbsPointer,", "LimbsLength,", "LimbsCapacity,"],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for BigIntHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for BigIntHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_four_exact_rows() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl BigIntHeapSlot {",
        "pub(crate) const HEAP_BIGINT_LAYOUT",
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
            "Self::Sign=>BigIntHeapSlotMetadata{record:\"bigint-record\",",
            "name:\"sign\",offset:HEAP_BIGINT_SIGN_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::LimbsPointer=>BigIntHeapSlotMetadata{record:\"bigint-record\",",
            "name:\"limbs_ptr\",offset:HEAP_BIGINT_LIMBS_PTR_OFFSET,",
            "width:8,pointer:true,},"
        ),
        concat!(
            "Self::LimbsLength=>BigIntHeapSlotMetadata{record:\"bigint-record\",",
            "name:\"limbs_len\",offset:HEAP_BIGINT_LIMBS_LEN_OFFSET,",
            "width:8,pointer:false,},"
        ),
        concat!(
            "Self::LimbsCapacity=>BigIntHeapSlotMetadata{record:\"bigint-record\",",
            "name:\"limbs_cap\",offset:HEAP_BIGINT_LIMBS_CAP_OFFSET,",
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
fn typed_registry_preserves_bigint_storage_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_BIGINT_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[BigIntHeapSlot]=&[BigIntHeapSlot::Sign,BigIntHeapSlot::LimbsPointer,",
            "BigIntHeapSlot::LimbsLength,BigIntHeapSlot::LimbsCapacity,"
        )
    );
    assert!(SIDE_STORAGE_SOURCE.contains("Self::BigIntLimbs => LinearSideStorageMetadata"));
}

#[test]
fn bigint_layout_has_one_private_recursive_owner() {
    assert_eq!(LIB_SOURCE.matches("\nmod heap_bigint_layout;\n").count(), 1);
    assert!(!LIB_SOURCE.contains("\npub mod heap_bigint_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"bigint-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"bigint-record\""),
        4
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum BigIntHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("BigIntHeapSlot"));
    assert!(TASK.contains("bigint-heap-slot-authority.md"));
}
