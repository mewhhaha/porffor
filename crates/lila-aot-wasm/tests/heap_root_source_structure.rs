const ROOT_SOURCE: &str = include_str!("../src/heap_root_sources.rs");
const HOST_BOUNDARY_SOURCE: &str = include_str!("../src/heap_host_boundary.rs");
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
fn root_source_and_kind_are_exact_closed_domains() {
    assert_eq!(
        variants(ROOT_SOURCE, "pub(crate) enum HeapRootSource {"),
        [
            "RealmGlobals,",
            "ActiveFrameLocals,",
            "LexicalEnvironments,",
            "CompletionRecords,",
            "FunctionTable,",
            "HostBorrowedValues,",
            "PendingJobs,",
        ]
    );
    assert_eq!(
        variants(ROOT_SOURCE, "pub(crate) enum HeapRootKind {"),
        [
            "PersistentNonTagged,",
            "PersistentTaggedValues,",
            "TransientTaggedValues,",
        ]
    );
}

#[test]
fn one_exhaustive_metadata_projection_owns_every_root_meaning() {
    let implementation = bounded(
        ROOT_SOURCE,
        "impl HeapRootSource {",
        "pub(crate) const HEAP_ROOT_SOURCES",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert!(!implementation.contains("_ =>"));
    for (variant, name, owner, kind) in [
        (
            "RealmGlobals",
            "realm-globals",
            "module-globals",
            "PersistentNonTagged",
        ),
        (
            "ActiveFrameLocals",
            "active-frame-locals",
            "function-locals",
            "TransientTaggedValues",
        ),
        (
            "LexicalEnvironments",
            "lexical-environments",
            "environment-chain",
            "PersistentTaggedValues",
        ),
        (
            "CompletionRecords",
            "completion-records",
            "completion-abi",
            "TransientTaggedValues",
        ),
        (
            "FunctionTable",
            "function-table",
            "indirect-call-table",
            "PersistentNonTagged",
        ),
        (
            "HostBorrowedValues",
            "host-borrowed-values",
            "host-import-boundary",
            "TransientTaggedValues",
        ),
        (
            "PendingJobs",
            "pending-jobs",
            "job-queue",
            "PersistentTaggedValues",
        ),
    ] {
        let arm = implementation
            .split_once(&format!("Self::{variant} => HeapRootMetadata {{"))
            .unwrap_or_else(|| panic!("missing metadata arm for {variant}"))
            .1
            .split_once("},")
            .unwrap_or_else(|| panic!("unbounded metadata arm for {variant}"))
            .0;
        assert!(arm.contains(&format!("name: \"{name}\"")));
        assert!(arm.contains(&format!("owner: \"{owner}\"")));
        assert!(arm.contains(&format!("kind: HeapRootKind::{kind}")));
    }

    let normalized_implementation = normalized(implementation);
    for accessor in ["name", "owner", "kind"] {
        assert!(normalized_implementation.contains(&format!("pub(crate)constfn{accessor}(self)->")));
    }
    assert_eq!(implementation.matches("self.metadata().").count(), 3);
}

#[test]
fn root_registry_contains_each_typed_source_once() {
    let registry = bounded(
        ROOT_SOURCE,
        "pub(crate) const HEAP_ROOT_SOURCES: &[HeapRootSource] = &[",
        "];",
    );
    assert_eq!(registry.matches("HeapRootSource::").count(), 7);
    for variant in [
        "RealmGlobals",
        "ActiveFrameLocals",
        "LexicalEnvironments",
        "CompletionRecords",
        "FunctionTable",
        "HostBorrowedValues",
        "PendingJobs",
    ] {
        assert_eq!(
            registry
                .matches(&format!("HeapRootSource::{variant},"))
                .count(),
            1,
            "root source {variant} must occur exactly once"
        );
    }
    assert!(!ROOT_SOURCE.contains("tagged_values:"));
    assert!(!ROOT_SOURCE.contains("transient:"));
}

#[test]
fn host_boundary_owns_the_typed_borrowed_root_source() {
    let projection = bounded(
        HOST_BOUNDARY_SOURCE,
        "pub(crate) const fn borrowed_root_source",
        "pub(crate) const HEAP_HOST_BOUNDARY_POLICY",
    );
    assert!(projection.contains("super::heap_root_sources::HeapRootSource::HostBorrowedValues"));
    assert!(!projection.contains("\"host-borrowed-values\""));

    assert!(HEAP_SOURCE
        .contains("assert_eq!(borrowed_root_source, HeapRootSource::HostBorrowedValues);"));
    assert!(!HEAP_SOURCE.contains("borrowed_root_source: &'static str"));
}
