use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const ARRAY_DESTRUCTURING_SOURCE: &str =
    include_str!("../../src/control_flow/array_destructuring.rs");
const RESUMABLE_ARRAY_SOURCE: &str =
    include_str!("../../src/control_flow/resumable_array_destructuring.rs");
const RETAINED_ITERATOR_SOURCE: &str =
    include_str!("../../src/environments/retained_array_iterator.rs");
const GC_LAYOUT_SOURCE: &str = include_str!("../../src/gc_types/layouts.rs");
const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/destructuring-iterator-locals-ownership.md"
);
const TASK: &str = include_str!("../../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
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
fn destructuring_iterator_locals_is_one_capability_free_reservation_bundle() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) struct OwnedSyncIterator {",
        "impl OwnedSyncIterator {",
    );
    assert!(declaration.contains("record: GcLocal<IteratorRecord>,"));
    assert!(declaration.contains("consumer: SyncIteratorConsumer,"));
    assert_eq!(declaration.matches(":").count(), 2);
    assert!(!declaration.contains(": u32"));
    let offset = CONTROL_FLOW_SOURCE
        .find("pub(crate) struct OwnedSyncIterator {")
        .unwrap();
    assert!(CONTROL_FLOW_SOURCE[..offset].trim_end().ends_with(
        "#[must_use = \"a completed iterator record must be cleared or captured by its consumer\"]"
    ));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for OwnedSyncIterator")));
    }
    let schema = bounded(
        GC_LAYOUT_SOURCE,
        "struct IteratorRecord => IteratorRecordSchema {",
        "struct CompletionRecord => CompletionRecordSchema {",
    );
    assert!(schema.contains("ITERATOR: GcRef<StoredValue>, Immutable, NonNullable;"));
    assert!(schema.contains("NEXT_METHOD: GcRef<StoredValue>, Immutable, NonNullable;"));
    assert!(schema.contains("DONE: bool, Mutable, NonNullable;"));
}

#[test]
fn protocol_projection_borrows_the_bundle_without_transferring_its_release_owner() {
    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "impl OwnedSyncIterator {",
        "#[derive(Clone, Copy)]",
    );
    assert!(projection.contains("fn record(&self) -> &GcLocal<IteratorRecord> {"));
    assert!(projection.contains("&self.record"));
    assert!(projection.contains("fn clear(self, function: &mut Function) {"));
    assert!(projection.contains("self.record.clear(function);"));
    assert!(!projection.contains("self.clone()"));
    for method in [
        "emit_sync_iterator_step_value",
        "emit_sync_iterator_step_without_value",
        "emit_sync_iterator_close",
    ] {
        let signature = bounded(
            CONTROL_FLOW_SOURCE,
            &format!("    pub(crate) fn {method}("),
            ") -> Result<(), EmitError> {",
        );
        assert!(
            signature.contains("iterator: &OwnedSyncIterator,"),
            "{method} must borrow the same record"
        );
    }
}

#[test]
fn array_destructuring_borrows_one_bundle_then_releases_all_locals_in_reverse() {
    let compiler = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "    pub(crate) fn compile_array_destructure_from_value_locals(",
        "    fn compile_array_destructuring_element(",
    );
    assert_eq!(compiler.matches("self.emit_get_sync_iterator(").count(), 1);
    assert_eq!(
        compiler
            .matches("SyncIteratorConsumer::ArrayDestructuring,")
            .count(),
        1
    );
    assert!(compiler.contains(
        "self.compile_array_destructuring_element(element, &iterator, done, &value, function)?;"
    ));
    assert_eq!(
        compiler
            .matches("self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;")
            .count(),
        2
    );
    let cleanup = bounded(compiler, "self.pop_control(ControlFrameKind::Block);\n        function.instruction(&Instruction::End);\n        closed.clear(function);", "        self.emit_propagate_current_throw_if_needed(function);");
    let mut cursor = 0;
    for marker in [
        "pending.clear(function);",
        "value.clear(function);",
        "schema.release_i32_local(done, function);",
        "iterator.clear(function);",
    ] {
        cursor += cleanup[cursor..]
            .find(marker)
            .expect("reverse release owner")
            + marker.len();
    }
    let element = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "    fn compile_array_destructuring_element(",
        "    fn emit_array_destructuring_rest_array(",
    );
    assert!(element.contains("iterator: &OwnedSyncIterator,"));
    assert!(!element.contains("iterator.clear("));

    // A suspension stores the actual native record in the original invocation
    // cell; nested lexical and With environments cannot choose a different cell.
    assert!(RETAINED_ITERATOR_SOURCE.contains("FunctionExecutionKind::Generator"));
    assert!(RETAINED_ITERATOR_SOURCE.contains("owned == binding"));
    assert!(RETAINED_ITERATOR_SOURCE
        .contains("self.owned_env_slot(&binding.name) != Some(binding.slot)"));
    assert!(RETAINED_ITERATOR_SOURCE.contains("InvocationFrameSchema::INVOCATION_ENVIRONMENT"));
    assert_eq!(
        RETAINED_ITERATOR_SOURCE
            .matches(".field(BindingCellSchema::DESTRUCTURING_ITERATOR_RECORD)")
            .count(),
        3
    );
    let publication = bounded(
        RETAINED_ITERATOR_SOURCE,
        "    pub(crate) fn emit_publish_retained_array_iterator(",
        "    pub(crate) fn emit_load_retained_array_iterator(",
    );
    assert!(publication.contains("record: &GcLocal<IteratorRecord>,"));
    assert!(publication.contains("GcOperand::nullable_reference(record, schema)"));
    let retirement = bounded(
        RETAINED_ITERATOR_SOURCE,
        "    pub(crate) fn emit_retire_retained_array_iterator(",
        "\n    }\n}",
    );
    assert!(retirement.contains("GcOperand::null(schema)"));
    assert_eq!(
        RESUMABLE_ARRAY_SOURCE
            .matches("self.emit_retire_retained_array_iterator(&storage, function);")
            .count(),
        1
    );
    let close = RESUMABLE_ARRAY_SOURCE
        .find("self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;")
        .unwrap();
    let retire = RESUMABLE_ARRAY_SOURCE
        .find("self.emit_retire_retained_array_iterator(&storage, function);")
        .unwrap();
    let dispatch = RESUMABLE_ARRAY_SOURCE
        .find("self.emit_dispatch_current_completion(function)?;")
        .unwrap();
    assert!(close < retire && retire < dispatch);
}

#[test]
fn ownership_contract_and_recursive_source_census_remain_closed() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "struct OwnedSyncIterator {"),
        1
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "DestructuringIteratorLocals"),
        0
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "fn emit_publish_retained_array_iterator("),
        1
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "fn emit_load_retained_array_iterator("),
        1
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "fn emit_retire_retained_array_iterator("),
        1
    );
    assert_eq!(
        GC_LAYOUT_SOURCE
            .matches("struct IteratorRecord => IteratorRecordSchema {")
            .count(),
        1
    );
    assert_eq!(
        GC_LAYOUT_SOURCE
            .matches("DESTRUCTURING_ITERATOR_RECORD: GcRef<IteratorRecord>, Mutable, Nullable;")
            .count(),
        1
    );
    // These retained documents record the predecessor's borrowing obligation.
    // The current native ownership checks above follow its single GC successor.
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("DestructuringIteratorLocals"));
        assert!(evidence.contains("capability-free"));
        assert!(evidence.contains("borrow"));
        assert!(evidence.contains("Batch AF"));
    }
}
