use std::fs;
use std::path::Path;

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/ordinary-to-primitive-receiver-kind.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
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
fn ordinary_to_primitive_receiver_kind_is_closed_and_capability_free() {
    let declaration = bounded(
        OPERATIONS_SOURCE,
        "enum OrdinaryToPrimitiveReceiverKind {",
        "\n}",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Object,", "Function,"],
    );

    let declaration_prefix = bounded(
        OPERATIONS_SOURCE,
        "/// The ordinary heap-record families admitted by OrdinaryToPrimitive.",
        "impl OrdinaryToPrimitiveReceiverKind",
    );
    assert!(!declaration_prefix.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!OPERATIONS_SOURCE.contains(&format!(
            "impl {capability} for OrdinaryToPrimitiveReceiverKind"
        )));
    }
}

#[test]
fn exhaustive_projections_own_tag_and_boxed_slot_policy() {
    let implementation = normalized(bounded(
        OPERATIONS_SOURCE,
        "impl OrdinaryToPrimitiveReceiverKind {",
        "/// The realm that owns TypeErrors created inside a conversion composite.",
    ));
    assert_eq!(implementation.matches("matchself{").count(), 2);
    assert!(implementation.contains(concat!(
        "constfnvalue_kind(&self)->ValueKind{matchself{",
        "Self::Object=>ValueKind::Object,Self::Function=>ValueKind::Function,}}"
    )));
    assert!(implementation.contains(concat!(
        "constfnhas_boxed_primitive_slot(&self)->bool{matchself{",
        "Self::Object=>true,Self::Function=>false,}}"
    )));
    assert!(!implementation.contains("_=>"));
}

#[test]
fn only_live_object_and_tagged_function_paths_reach_the_inner_emitter() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for removed in [
        "emit_function_to_primitive_locals",
        "emit_function_to_primitive_locals_pending",
    ] {
        assert_eq!(
            recursive_rust_source_count(&source_root, removed),
            0,
            "{removed}"
        );
    }

    assert_eq!(
        OPERATIONS_SOURCE
            .matches("OrdinaryToPrimitiveReceiverKind::Object")
            .count(),
        2,
    );
    assert_eq!(
        OPERATIONS_SOURCE
            .matches("OrdinaryToPrimitiveReceiverKind::Function")
            .count(),
        1,
    );

    let inner = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn emit_object_to_primitive_locals_inner(",
        "pub(crate) fn emit_ordinary_object_default_to_string_applies_i32(",
    ));
    assert!(inner.contains("receiver_kind:OrdinaryToPrimitiveReceiverKind"));
    assert!(!inner.contains("receiver_kind:ValueKind"));
    assert!(inner.contains("ifreceiver_kind.has_boxed_primitive_slot(){"));
    assert!(inner.contains("receiver_kind.value_kind().tag()asi64"));
}

#[test]
fn contract_and_task_record_the_closed_receiver_boundary() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("OrdinaryToPrimitiveReceiverKind"));
        assert!(evidence.contains("Object"));
        assert!(evidence.contains("Function"));
        assert!(evidence.contains("unrepresentable"));
    }
}
