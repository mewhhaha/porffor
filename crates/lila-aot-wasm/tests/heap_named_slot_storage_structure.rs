const HEAP_SOURCE: &str = include_str!("../src/heap.rs");

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

#[test]
fn named_slot_storage_is_one_closed_exhaustive_domain() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum HeapNamedSlotStorage {",
        "}\n\n#[allow(dead_code)]\nimpl HeapNamedSlotStorage",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["StrongReference,", "Scalar,"],
    );

    let projections = normalized(bounded(
        HEAP_SOURCE,
        "impl HeapNamedSlotStorage {",
        "#[allow(dead_code)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub(crate) struct HeapNamedSlot",
    ));
    for projection in ["is_strong_reference", "scans_target"] {
        assert!(
            projections.contains(&format!(
                "constfn{projection}(self)->bool{{matchself{{Self::StrongReference=>true,Self::Scalar=>false,}}}}"
            )),
            "{projection} must exhaustively derive its meaning from HeapNamedSlotStorage"
        );
    }
}

#[test]
fn named_slots_cannot_store_independent_strength_or_tracing_flags() {
    let named_slot = normalized(bounded(
        HEAP_SOURCE,
        "pub(crate) struct HeapNamedSlot {",
        "#[allow(dead_code)]\nimpl HeapLayoutSlot",
    ));
    assert_eq!(
        named_slot,
        "pubrecord:&'staticstr,pubkey:&'staticstr,pubstorage:HeapNamedSlotStorage,}"
    );
    assert!(!named_slot.contains("bool"));
}

#[test]
fn all_named_slot_rows_select_one_storage_class() {
    let registry = bounded(
        HEAP_SOURCE,
        "pub(crate) const HEAP_ARRAY_ITERATOR_NAMED_SLOTS",
        "pub(crate) enum HeapNamedSlotFamily",
    );
    assert_eq!(registry.matches("HeapNamedSlot {").count(), 50);
    assert_eq!(
        registry
            .matches("storage: HeapNamedSlotStorage::StrongReference")
            .count(),
        30
    );
    assert_eq!(
        registry
            .matches("storage: HeapNamedSlotStorage::Scalar")
            .count(),
        20
    );
    assert!(!registry.contains("strong_reference:"));
    assert!(!registry.contains("scans_target:"));
}

#[test]
fn named_slot_family_owns_the_typed_registry() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum HeapNamedSlotFamily {",
        "}\n\n#[allow(dead_code)]\nimpl HeapNamedSlotFamily",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "ArrayIterator,",
            "StringIterator,",
            "RegExpStringIterator,",
            "IteratorHelper,",
            "IteratorConcatState,",
            "IteratorZipState,",
        ],
    );
    assert!(HEAP_SOURCE.contains("#[allow(dead_code)]\npub(crate) enum HeapNamedSlotFamily {"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!HEAP_SOURCE.contains(&format!("impl {capability} for HeapNamedSlotFamily")));
    }

    let projection = normalized(bounded(
        HEAP_SOURCE,
        "impl HeapNamedSlotFamily {",
        "pub(crate) const HEAP_NAMED_SLOT_FAMILIES",
    ));
    assert_eq!(projection.matches("matchself{").count(), 1);
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("unreachable!"));
    assert!(!projection.contains("todo!"));
    for (variant, slots) in [
        ("ArrayIterator", "HEAP_ARRAY_ITERATOR_NAMED_SLOTS"),
        ("StringIterator", "HEAP_STRING_ITERATOR_NAMED_SLOTS"),
        (
            "RegExpStringIterator",
            "HEAP_REGEXP_STRING_ITERATOR_NAMED_SLOTS",
        ),
        ("IteratorHelper", "HEAP_ITERATOR_HELPER_NAMED_SLOTS"),
        (
            "IteratorConcatState",
            "HEAP_ITERATOR_CONCAT_STATE_NAMED_SLOTS",
        ),
        ("IteratorZipState", "HEAP_ITERATOR_ZIP_STATE_NAMED_SLOTS"),
    ] {
        assert!(projection.contains(&format!("Self::{variant}=>{slots},")));
    }

    let registry = normalized(bounded(
        HEAP_SOURCE,
        "pub(crate) const HEAP_NAMED_SLOT_FAMILIES",
        "];",
    ));
    assert_eq!(
        registry,
        ":&[HeapNamedSlotFamily]=&[HeapNamedSlotFamily::ArrayIterator,HeapNamedSlotFamily::StringIterator,HeapNamedSlotFamily::RegExpStringIterator,HeapNamedSlotFamily::IteratorHelper,HeapNamedSlotFamily::IteratorConcatState,HeapNamedSlotFamily::IteratorZipState,"
    );
    assert!(!HEAP_SOURCE.contains("HEAP_NAMED_SLOT_LAYOUTS"));
}
