const SIDE_STORAGE_SOURCE: &str = include_str!("../src/heap_side_storage.rs");
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

fn variants<'a>(source: &'a str, declaration: &str) -> Vec<&'a str> {
    source
        .split_once(declaration)
        .unwrap_or_else(|| panic!("missing enum declaration: {declaration}"))
        .1
        .split_once("\n}")
        .unwrap_or_else(|| panic!("unterminated enum declaration: {declaration}"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn normalized(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn side_storage_element_is_the_exact_closed_domain() {
    assert_eq!(
        variants(
            SIDE_STORAGE_SOURCE,
            "pub(crate) enum LinearSideStorageElement {"
        ),
        ["Byte,", "Utf16CodeUnit,", "BigIntLimb,"],
    );
}

#[test]
fn side_storage_element_owns_width_and_reference_classification() {
    let projections = normalized(bounded(
        SIDE_STORAGE_SOURCE,
        "impl LinearSideStorageElement {",
        "pub(crate) enum LinearSideStorage {",
    ));
    assert!(projections.contains(
        "constfnbyte_width(self)->u64{matchself{Self::Byte=>1,Self::Utf16CodeUnit=>2,Self::BigIntLimb=>8,}}"
    ));
    assert!(projections.contains(
        "constfnis_reference_storage(self)->bool{matchself{Self::Byte=>false,Self::Utf16CodeUnit=>false,Self::BigIntLimb=>false,}}"
    ));
    assert_eq!(projections.matches("matchself{").count(), 2);
    assert!(!projections.contains("_=>"));
}

#[test]
fn side_storage_identity_is_the_exact_capability_free_domain() {
    assert_eq!(
        variants(SIDE_STORAGE_SOURCE, "pub(crate) enum LinearSideStorage {"),
        [
            "ArrayBufferBackingStore,",
            "StringCodeUnits,",
            "BigIntLimbs,"
        ],
    );
    assert!(SIDE_STORAGE_SOURCE.contains("}\n\npub(crate) enum LinearSideStorage {"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!SIDE_STORAGE_SOURCE.contains(&format!("impl {capability} for LinearSideStorage")));
    }
    assert!(!SIDE_STORAGE_SOURCE.contains("struct LinearSideStorageLayout"));
}

#[test]
fn side_storage_identity_owns_exact_metadata_and_registry_order() {
    let implementation = bounded(
        SIDE_STORAGE_SOURCE,
        "impl LinearSideStorage {",
        "pub(crate) const LINEAR_SIDE_STORAGES",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for (variant, record, length_source, element) in [
        (
            "ArrayBufferBackingStore",
            "array-buffer-backing-store",
            "array-buffer-object-header.max_byte_length",
            "Byte",
        ),
        (
            "StringCodeUnits",
            "string-code-units",
            "string-record.code_unit_len",
            "Utf16CodeUnit",
        ),
        (
            "BigIntLimbs",
            "bigint-limbs",
            "bigint-record.limbs_len",
            "BigIntLimb",
        ),
    ] {
        let arm = format!(
            "Self::{variant}=>LinearSideStorageMetadata{{record:\"{record}\",length_source:\"{length_source}\",element:LinearSideStorageElement::{element},}},"
        );
        assert!(
            implementation.contains(&arm),
            "missing exact side-storage metadata arm for {variant}"
        );
    }
    for accessor in ["record", "length_source", "element"] {
        assert!(
            implementation.contains(&format!("self.metadata().{accessor}")),
            "{accessor} must project through the sole metadata authority"
        );
    }

    let registry = normalized(bounded(
        SIDE_STORAGE_SOURCE,
        "pub(crate) const LINEAR_SIDE_STORAGES",
        "];",
    ));
    assert_eq!(
        registry,
        ":&[LinearSideStorage]=&[LinearSideStorage::ArrayBufferBackingStore,LinearSideStorage::StringCodeUnits,LinearSideStorage::BigIntLimbs,"
    );
    assert!(
        HEAP_SOURCE.contains("LinearSideStorage, LinearSideStorageElement, LINEAR_SIDE_STORAGES")
    );
    assert!(!HEAP_SOURCE.contains("LINEAR_SIDE_STORAGE_LAYOUTS"));
}
