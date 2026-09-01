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

fn normalized(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn host_boundary_policy_is_the_exact_closed_domain() {
    let declaration = bounded(
        HOST_BOUNDARY_SOURCE,
        "pub(crate) enum HeapHostBoundaryPolicy {",
        "}\n\nimpl HeapHostBoundaryPolicy",
    );
    assert_eq!(
        declaration.trim(),
        "ImportCallOnlyWithTransientTaggedRoots,"
    );
}

#[test]
fn host_boundary_policy_owns_name_and_typed_root_projections() {
    let implementation = normalized(bounded(
        HOST_BOUNDARY_SOURCE,
        "impl HeapHostBoundaryPolicy {",
        "pub(crate) const HEAP_HOST_BOUNDARY_POLICY",
    ));
    assert_eq!(implementation.matches("matchself{").count(), 2);
    assert!(!implementation.contains("_=>"));
    assert!(implementation
        .contains("Self::ImportCallOnlyWithTransientTaggedRoots=>\"host-import-memory-borrow\","));
    assert!(implementation.contains(
        "Self::ImportCallOnlyWithTransientTaggedRoots=>{super::heap_root_sources::HeapRootSource::HostBorrowedValues}"
    ));
    assert!(!implementation.contains("\"host-borrowed-values\""));
}

#[test]
fn host_boundary_policy_has_one_exact_producer() {
    let producer = normalized(bounded(
        HOST_BOUNDARY_SOURCE,
        "pub(crate) const HEAP_HOST_BOUNDARY_POLICY",
        ";",
    ));
    assert_eq!(
        producer,
        ":HeapHostBoundaryPolicy=HeapHostBoundaryPolicy::ImportCallOnlyWithTransientTaggedRoots"
    );
    assert!(!HOST_BOUNDARY_SOURCE.contains("bool"));
    assert!(!HOST_BOUNDARY_SOURCE.contains("durable_host_pointers"));
    assert!(!HOST_BOUNDARY_SOURCE.contains("reentrant_imports_require_transient_roots"));
    assert!(!HEAP_SOURCE.contains("HeapHostBoundaryContract"));
    assert!(!HEAP_SOURCE.contains("HostMemoryBorrowDuration"));
}

#[test]
fn heap_owner_consumes_only_the_closed_host_policy() {
    let owner = bounded(
        HEAP_SOURCE,
        "fn assert_host_boundary_policy(",
        "fn assert_value_encodings(",
    );
    assert!(owner.contains("policy: &HeapHostBoundaryPolicy"));
    assert!(owner.contains("let borrowed_root_source = policy.borrowed_root_source();"));
    assert!(owner.contains("HeapRootSource::HostBorrowedValues"));
    assert!(owner.contains("HeapRootKind::TransientTaggedValues"));
    assert!(!owner.contains("durable_host_pointers"));
    assert!(!owner.contains("reentrant_imports_require_transient_roots"));

    let test = bounded(
        HEAP_SOURCE,
        "fn heap_host_boundary_is_call_scoped_and_transiently_rooted()",
        "fn heap_value_encoding_registry_covers_ecmascript_language_types()",
    );
    assert!(test.contains("assert_host_boundary_policy(&HEAP_HOST_BOUNDARY_POLICY);"));
}
