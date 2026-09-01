use std::fs;
use std::path::Path;

const LAYOUT_SOURCE: &str = include_str!("../src/heap_class_function_context_layout.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/class-function-context-heap-slot-authority.md"
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
fn class_function_context_heap_slot_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        LAYOUT_SOURCE,
        "pub(crate) enum ClassFunctionContextHeapSlot {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "LexicalEnvironment,",
            "ActiveFunction,",
            "HomeObjectPayload,",
            "HomeObjectTag,",
            "FieldKeys,",
            "PrivateEnvironment,",
        ],
    );
    assert!(!LAYOUT_SOURCE.contains("#[derive"));
    assert!(
        !LAYOUT_SOURCE.lines().any(|line| {
            line.trim_start().starts_with("impl ")
                && line.contains(" for ClassFunctionContextHeapSlot")
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
                "impl {capability} for ClassFunctionContextHeapSlot"
            )),
            "found manual {capability} capability"
        );
    }
}

#[test]
fn one_exhaustive_projection_owns_six_exact_rows_and_retention_classes() {
    let implementation = bounded(
        LAYOUT_SOURCE,
        "impl ClassFunctionContextHeapSlot {",
        "pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert_eq!(implementation.matches("pointer: true").count(), 5);
    assert_eq!(implementation.matches("pointer: false").count(), 1);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for row in [
        concat!(
            "Self::LexicalEnvironment=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"lexical_env\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::ActiveFunction=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"active_function\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::HomeObjectPayload=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"home_object_payload\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::HomeObjectTag=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"home_object_tag\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,width:8,pointer:false,},"
        ),
        concat!(
            "Self::FieldKeys=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"field_keys\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,width:8,pointer:true,},"
        ),
        concat!(
            "Self::PrivateEnvironment=>ClassFunctionContextHeapSlotMetadata{",
            "record:\"class-function-context\",name:\"private_environment\",",
            "offset:HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,width:8,pointer:true,},"
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
fn typed_registry_preserves_context_edge_and_home_object_order() {
    let registry = normalized(bounded(
        LAYOUT_SOURCE,
        "pub(crate) const HEAP_CLASS_FUNCTION_CONTEXT_LAYOUT",
        "];",
    ));
    assert_eq!(
        registry,
        concat!(
            ":&[ClassFunctionContextHeapSlot]=&[",
            "ClassFunctionContextHeapSlot::LexicalEnvironment,",
            "ClassFunctionContextHeapSlot::ActiveFunction,",
            "ClassFunctionContextHeapSlot::HomeObjectPayload,",
            "ClassFunctionContextHeapSlot::HomeObjectTag,",
            "ClassFunctionContextHeapSlot::FieldKeys,",
            "ClassFunctionContextHeapSlot::PrivateEnvironment,"
        )
    );
}

#[test]
fn class_function_context_layout_has_one_private_recursive_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("\nmod heap_class_function_context_layout;\n")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("\npub mod heap_class_function_context_layout;\n"));
    assert!(!HEAP_SOURCE.contains("record: \"class-function-context\""));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "record: \"class-function-context\""),
        6
    );
    assert_eq!(
        recursive_rust_source_count(
            &source_root,
            "pub(crate) enum ClassFunctionContextHeapSlot {"
        ),
        1
    );
    assert!(CONTRACT.contains("ClassFunctionContextHeapSlot"));
    assert!(TASK.contains("class-function-context-heap-slot-authority.md"));
}
