use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");
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

#[test]
fn sync_iterator_protocol_view_is_an_exact_capability_free_local_set() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) struct SyncIteratorLocals {",
        "#[must_use = \"reserved synchronous iterator locals must be released\"]",
    );
    assert_eq!(declaration.matches("pub(crate) ").count(), 11);
    assert_eq!(declaration.matches(": u32,").count(), 11);
    assert!(!declaration.contains("#[derive"));
    for type_name in ["SyncIteratorLocals", "ReservedSyncIteratorLocals"] {
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
            assert!(!CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for {type_name}")));
        }
    }
}

#[test]
fn reserved_owner_is_the_only_value_accepted_by_reverse_release() {
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) struct ReservedSyncIteratorLocals {",
        "pub(crate) enum SyncIteratorConsumer {",
    );
    assert!(CONTROL_FLOW_SOURCE.contains(concat!(
        "#[must_use = \"reserved synchronous iterator locals must be released\"]\n",
        "pub(crate) struct ReservedSyncIteratorLocals {"
    )));
    assert_eq!(owner.matches("locals: SyncIteratorLocals,").count(), 1);
    assert_eq!(owner.matches("impl std::ops::Deref").count(), 1);
    assert_eq!(owner.matches("&self.locals").count(), 1);
    assert!(!owner.contains("DerefMut"));

    let lifecycle = bounded(
        CONTROL_FLOW_SOURCE,
        "    /// Reserves the common GetIterator/IteratorStep/IteratorValue working set.",
        "    fn emit_arguments_iterator_method_to_locals(",
    );
    assert!(lifecycle.contains(
        "pub(crate) fn reserve_sync_iterator_locals(&mut self) -> ReservedSyncIteratorLocals"
    ));
    assert_eq!(lifecycle.matches("ReservedSyncIteratorLocals {").count(), 3);
    assert!(lifecycle.contains(
        "pub(crate) fn release_sync_iterator_locals(&mut self, reserved: ReservedSyncIteratorLocals)"
    ));
    assert!(lifecycle.contains("let ReservedSyncIteratorLocals { locals } = reserved;"));
    assert!(
        !lifecycle.contains("release_sync_iterator_locals(&mut self, locals: SyncIteratorLocals)")
    );
}

#[test]
fn all_protocol_operations_borrow_before_the_reserved_owner_is_consumed() {
    let acquisition = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_iterator_from_value_locals(",
        "    fn emit_sync_iterator_protocol_type_error(",
    );
    assert_eq!(
        acquisition.matches("locals: &SyncIteratorLocals,").count(),
        2
    );

    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert_eq!(step.matches("locals: &SyncIteratorLocals,").count(), 1);
    assert_eq!(CONTROL_FLOW_SOURCE.matches("&locals.protocol()").count(), 1);

    for consumer in [ARRAY_SOURCE, MATH_SOURCE] {
        assert_eq!(consumer.matches("&iterator_locals").count(), 2);
        assert_eq!(
            consumer
                .matches("self.release_sync_iterator_locals(iterator_locals);")
                .count(),
            1
        );
        assert!(!consumer.contains("iterator_locals.clone()"));
    }
}

#[test]
fn release_ownership_contract_and_recursive_census_remain_closed() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "ReservedSyncIteratorLocals"),
        6
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "SyncIteratorLocals"),
        15
    );
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("ReservedSyncIteratorLocals"));
        assert!(evidence.contains("SyncIteratorLocals"));
        assert!(evidence.contains("release"));
        assert!(evidence.contains("Batch AG"));
    }
}
