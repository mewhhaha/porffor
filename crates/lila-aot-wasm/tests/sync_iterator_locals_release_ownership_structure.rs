use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const MATH_SOURCE: &str = include_str!("../src/builtins/math/sum_precise.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/sync-iterator-locals-release-ownership.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

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

fn assert_before(source: &str, earlier: &str, later: &str) {
    assert!(source.find(earlier).expect(earlier) < source.find(later).expect(later));
}

#[test]
fn sync_iterator_owner_exposes_only_a_borrowed_typed_record() {
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) struct OwnedSyncIterator {",
        "#[derive(Clone, Copy)]\npub(crate) enum SyncIteratorConsumer",
    );
    let fields = owner.split_once("}").unwrap().0;
    assert_eq!(
        fields.trim(),
        "record: GcLocal<IteratorRecord>,\n    consumer: SyncIteratorConsumer,"
    );
    assert!(CONTROL_FLOW_SOURCE.contains("#[must_use = \"a completed iterator record must be cleared or captured by its consumer\"]\npub(crate) struct OwnedSyncIterator"));
    assert!(owner.contains("pub(crate) fn record(&self) -> &GcLocal<IteratorRecord>"));
    assert!(owner.contains("pub(crate) fn clear(self, function: &mut Function)"));
    assert_eq!(owner.matches("self.record.clear(function)").count(), 1);
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
        "Deref",
        "DerefMut",
    ] {
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for OwnedSyncIterator")));
    }
    assert!(!owner.contains("#[derive"));
    assert!(!fields.contains("pub"));
}

#[test]
fn acquisition_publishes_a_complete_record_before_consuming_cleanup() {
    let acquisition = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_sync_iterator(",
        "    /// GetIteratorDirect",
    );
    assert!(acquisition.contains(") -> Result<OwnedSyncIterator, EmitError>"));
    assert_before(
        acquisition,
        "self.emit_object_read(boxed.value(), source, &key, &method, function)",
        "self.emit_is_callable_i32(method.value(), function)",
    );
    assert_before(
        acquisition,
        "self.emit_function_or_proxy_call_with_argv(",
        "self.emit_get_sync_iterator_direct(iterator.value(), consumer, function)",
    );
    assert_before(
        acquisition,
        "self.emit_get_sync_iterator_direct(iterator.value(), consumer, function)",
        "iterator.clear(function)",
    );
    let direct = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_sync_iterator_direct(",
        "    fn emit_sync_iterator_protocol_type_error(",
    );
    assert!(direct.contains(") -> Result<OwnedSyncIterator, EmitError>"));
    assert_eq!(direct.matches("self.emit_object_read(").count(), 1);
    assert!(
        direct.contains("self.emit_object_read(iterator, iterator, &next_key, &next, function)")
    );
    assert!(
        !direct.contains("emit_is_callable_i32"),
        "GetIteratorDirect caches next without eagerly testing callability"
    );
    assert_before(
        direct,
        "self.emit_object_read(",
        ".from_value(iterator, function)",
    );
    assert_before(
        direct,
        ".from_value(iterator, function)",
        ".from_value(next.value(), function)",
    );
    assert_before(
        direct,
        "GcOperand::reference(&iterator_value, schema)",
        "GcOperand::reference(&next_value, schema)",
    );
    assert_before(
        direct,
        "GcOperand::reference(&next_value, schema)",
        "GcOperand::boolean(false)",
    );
    assert_before(
        direct,
        "invalid.clear(function)",
        "Ok(OwnedSyncIterator { record, consumer })",
    );
}

#[test]
fn all_protocol_operations_borrow_before_the_owner_is_consumed() {
    let close = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_close(",
        "    pub(crate) fn emit_get_sync_iterator(",
    );
    assert!(close.contains("iterator: &OwnedSyncIterator"));
    let steps = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert_eq!(steps.matches("iterator: &OwnedSyncIterator,").count(), 6);
    assert!(!steps.contains("iterator.clear("));
    assert!(!close.contains("iterator.clear("));
    for (consumer, acquisitions, releases) in [(ARRAY_SOURCE, 2, 2), (MATH_SOURCE, 1, 1)] {
        assert_eq!(consumer.matches("let iterator =").count(), acquisitions);
        assert_eq!(
            consumer
                .matches("self.emit_sync_iterator_step_value(&iterator,")
                .count(),
            acquisitions
        );
        assert_eq!(consumer.matches("iterator.clear(").count(), releases);
        assert!(!consumer.contains("iterator.clone()"));
    }
    assert_before(
        MATH_SOURCE,
        "self.emit_sync_iterator_step_value(&iterator,",
        "iterator.clear(function)",
    );
    assert_before(
        MATH_SOURCE,
        "accumulator.clear(function)",
        "iterator.clear(function)",
    );
}

#[test]
fn release_ownership_contract_and_recursive_census_remain_closed() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "OwnedSyncIterator"), 29);
    // Definition + impl + seven in-family constructors; outside helpers can
    // only admit a typed record through the fixed helper-consumer constructor.
    assert_eq!(
        count_in_rust_sources(&source_root, "OwnedSyncIterator {"),
        9
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "OwnedSyncIterator::from_helper_record("),
        2
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "ReservedSyncIteratorLocals"),
        0
    );
    assert_eq!(count_in_rust_sources(&source_root, "SyncIteratorLocals"), 0);
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("ReservedSyncIteratorLocals"));
        assert!(evidence.contains("SyncIteratorLocals"));
        assert!(evidence.contains("release"));
        assert!(evidence.contains("Batch AG"));
    }
}
