const POLICY_SOURCE: &str = include_str!("../src/heap_collector_policy.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");

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
fn collector_policy_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        POLICY_SOURCE,
        "pub(crate) enum HeapCollectorPolicy {",
        "\n}",
    );
    assert_eq!(declaration.trim(), "NonMovingMetadataChecked,");
    assert!(POLICY_SOURCE.contains(
        "use super::heap_weak_edges::{HeapWeakEdge, HEAP_WEAK_EDGES};\n\npub(crate) enum HeapCollectorPolicy {"
    ));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!POLICY_SOURCE.contains(&format!("impl {capability} for HeapCollectorPolicy")));
    }
    assert!(!HEAP_SOURCE.contains("HeapCollectorContract"));
    assert!(!HEAP_SOURCE.contains("HeapCollectorCapability"));
    assert!(!HEAP_SOURCE.contains("HEAP_COLLECTOR_CONTRACT"));
}

#[test]
fn collector_policy_owns_six_exhaustive_projections() {
    let implementation = bounded(
        POLICY_SOURCE,
        "impl HeapCollectorPolicy {",
        "pub(crate) const HEAP_COLLECTOR_POLICY",
    );
    assert_eq!(implementation.matches("match self {").count(), 6);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let implementation = normalized(implementation);
    for projection in [
        "constfnname(&self)->&'staticstr{matchself{Self::NonMovingMetadataChecked=>\"non-moving-tracing-collector\",}}",
        "constfnmoves_objects(&self)->bool{matchself{Self::NonMovingMetadataChecked=>false,}}",
        "constfnroot_sources(&self)->&'static[HeapRootSource]{matchself{Self::NonMovingMetadataChecked=>HEAP_ROOT_SOURCES,}}",
        "constfnweak_edges(&self)->&'static[HeapWeakEdge]{matchself{Self::NonMovingMetadataChecked=>HEAP_WEAK_EDGES,}}",
        "constfnrequired_phases(&self)->&'static[RequiredHeapCollectorPhase]{matchself{Self::NonMovingMetadataChecked=>REQUIRED_HEAP_COLLECTOR_PHASES,}}",
        "constfnis_executable(&self)->bool{matchself{Self::NonMovingMetadataChecked=>false,}}",
    ] {
        assert!(
            implementation.contains(projection),
            "missing exact policy projection: {projection}"
        );
    }
}

#[test]
fn heap_owner_delegates_to_the_closed_policy() {
    let producer = normalized(bounded(
        POLICY_SOURCE,
        "pub(crate) const HEAP_COLLECTOR_POLICY",
        ";",
    ));
    assert_eq!(
        producer,
        ":HeapCollectorPolicy=HeapCollectorPolicy::NonMovingMetadataChecked"
    );
    assert!(HEAP_SOURCE.contains("use super::heap_collector_policy::HEAP_COLLECTOR_POLICY;"));
    let executable = normalized(bounded(
        HEAP_SOURCE,
        "pub(crate) const fn heap_collector_is_executable() -> bool {",
        "\n}",
    ));
    assert_eq!(executable, "HEAP_COLLECTOR_POLICY.is_executable()");
}

#[test]
fn host_gc_remains_explicitly_unsupported() {
    let owner = bounded(
        HOST_SOURCE,
        "pub(crate) fn compile_host_gc_builtin(",
        "pub(crate) fn compile_host_parse_int_builtin(",
    );
    assert!(owner.contains("if heap_collector_is_executable() {"));
    assert!(owner.contains("heap collector is marked executable but host gc emitter is not wired"));
    assert!(owner.contains("gc requires a real collector in wasm-aot"));
    assert!(owner.contains("self.set_completion_kind(CompletionKind::Throw, function);"));
}
