use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_realm_record_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/realm-record-heap-slot-authority.md");
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
fn realm_record_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum RealmRecordHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "RealmId,",
            "AgentId,",
            "GlobalObject,",
            "GlobalThis,",
            "GlobalEnvironment,",
            "Intrinsics,",
            "HostHooks,",
            "ModuleRegistry,",
            "PrivateElements,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ") && line.contains(" for RealmRecordHeapSlot")
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
            !LAYOUT_SOURCE.contains(&format!("impl {capability} for RealmRecordHeapSlot")),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_nine_exact_rows_and_retention_classes() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl RealmRecordHeapSlot {",
        "pub(crate) const HEAP_REALM_RECORD_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 7);
    assert_eq!(implementation.matches("pointer: false").count(), 2);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::RealmId=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"realm_id\",",
            "offset:HEAP_REALM_ID_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::AgentId=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"agent_id\",",
            "offset:HEAP_REALM_AGENT_ID_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::GlobalObject=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"global_object\",",
            "offset:HEAP_REALM_GLOBAL_OBJECT_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::GlobalThis=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"global_this\",",
            "offset:HEAP_REALM_GLOBAL_THIS_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::GlobalEnvironment=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"global_environment\",",
            "offset:HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::Intrinsics=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"intrinsics\",",
            "offset:HEAP_REALM_INTRINSICS_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::HostHooks=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"host_hooks\",",
            "offset:HEAP_REALM_HOST_HOOKS_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::ModuleRegistry=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"module_registry\",",
            "offset:HEAP_REALM_MODULE_REGISTRY_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::PrivateElements=>RealmRecordHeapSlotMetadata{",
            "record:\"realm-record\",name:\"private_elements\",",
            "offset:HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,width:8,pointer:true,},"
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
fn typed_registry_preserves_ids_and_seven_realm_edge_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_REALM_RECORD_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[RealmRecordHeapSlot]=&[",
            "RealmRecordHeapSlot::RealmId,",
            "RealmRecordHeapSlot::AgentId,",
            "RealmRecordHeapSlot::GlobalObject,",
            "RealmRecordHeapSlot::GlobalThis,",
            "RealmRecordHeapSlot::GlobalEnvironment,",
            "RealmRecordHeapSlot::Intrinsics,",
            "RealmRecordHeapSlot::HostHooks,",
            "RealmRecordHeapSlot::ModuleRegistry,",
            "RealmRecordHeapSlot::PrivateElements,"
        )
    );
}

#[test]
fn realm_record_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_realm_record_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_realm_record_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"realm-record\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"realm-record\""),
        9
    );
    assert_eq!(
        recursive_rust_source_count(&source_root, "pub(crate) enum RealmRecordHeapSlot {"),
        1
    );
    assert!(CONTRACT.contains("RealmRecordHeapSlot"));
    assert!(TASK.contains("realm-record-heap-slot-authority.md"));
}
