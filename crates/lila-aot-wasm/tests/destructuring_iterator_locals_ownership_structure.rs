use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/destructuring-iterator-locals-ownership.md");
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
fn destructuring_iterator_locals_is_one_capability_free_reservation_bundle() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "struct DestructuringIteratorLocals {",
        "/// The activation layout shared by the two execution kinds",
    );
    assert_eq!(declaration.matches(": u32,").count(), 18);
    assert!(!declaration.contains("#[derive"));
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
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!(
            "impl {capability} for DestructuringIteratorLocals"
        )));
    }
}

#[test]
fn protocol_projection_borrows_the_bundle_without_transferring_its_release_owner() {
    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "impl DestructuringIteratorLocals {",
        "enum DestructuringIteratorStepKind {",
    );
    assert!(projection.contains("fn protocol(&self) -> SyncIteratorLocals {"));
    assert_eq!(projection.matches("self.").count(), 11);
    assert!(!projection.contains("self.clone()"));
}

#[test]
fn array_destructuring_borrows_one_bundle_then_releases_all_locals_in_reverse() {
    let compiler = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_array_destructure_from_value_locals(",
        "    /// Reserves the common GetIterator/IteratorStep/IteratorValue working set.",
    );
    assert_eq!(
        compiler
            .matches("let locals = DestructuringIteratorLocals {")
            .count(),
        1
    );
    assert_eq!(
        compiler
            .matches(
                "self.compile_array_destructuring_element(element, &locals, &consumer, function)?;"
            )
            .count(),
        1
    );
    assert_eq!(
        compiler
            .matches("let consumer = SyncIteratorConsumer::ArrayDestructuring;")
            .count(),
        1
    );
    assert_eq!(compiler.matches("&consumer").count(), 2);
    assert_eq!(
        compiler.matches("self.release_temp_local(local);").count(),
        1
    );
    assert!(!compiler.contains("locals.clone()"));
    assert!(!compiler.contains("let copied_locals"));

    let element = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_array_destructuring_element(",
        "    fn emit_destructuring_iterator_step(",
    );
    assert!(element.contains("locals: &DestructuringIteratorLocals,"));
    assert!(element.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(
        element
            .matches("self.emit_destructuring_iterator_step(")
            .count(),
        3
    );

    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_destructuring_iterator_step(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert!(step.contains("locals: &DestructuringIteratorLocals,"));
    assert!(step.contains("consumer: &SyncIteratorConsumer,"));
}

#[test]
fn ownership_contract_and_recursive_source_census_remain_closed() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "DestructuringIteratorLocals"),
        5
    );
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("DestructuringIteratorLocals"));
        assert!(evidence.contains("capability-free"));
        assert!(evidence.contains("borrow"));
        assert!(evidence.contains("Batch AF"));
    }
}
