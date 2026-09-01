const WEAK_EDGE_SOURCE: &str = include_str!("../src/heap_weak_edges.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const POLICY_SOURCE: &str = include_str!("../src/heap_collector_policy.rs");

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
fn weak_edge_kind_owns_one_closed_retention_projection() {
    assert_eq!(
        variants(WEAK_EDGE_SOURCE, "pub(crate) enum HeapWeakEdgeKind {"),
        [
            "EphemeronKey,",
            "EphemeronValue,",
            "WeakTarget,",
            "FinalizerHoldings,",
            "FinalizerToken,",
        ],
    );
    assert_eq!(
        variants(WEAK_EDGE_SOURCE, "pub(crate) enum HeapWeakEdgeRetention {"),
        [
            "DoesNotRetain,",
            "ConditionalOnReachableEphemeronKey,",
            "StrongUntilCleanup,",
        ],
    );

    let projection = bounded(
        WEAK_EDGE_SOURCE,
        "impl HeapWeakEdgeKind {",
        "pub(crate) enum HeapWeakEdge {",
    );
    assert!(projection.contains("pub(crate) const fn retention(self) -> HeapWeakEdgeRetention {"));
    assert_eq!(projection.matches("match self {").count(), 1);
    assert!(
        projection.contains("Self::EphemeronKey | Self::WeakTarget | Self::FinalizerToken => {")
    );
    assert!(projection.contains(
        "Self::EphemeronValue => HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey,"
    ));
    assert!(projection
        .contains("Self::FinalizerHoldings => HeapWeakEdgeRetention::StrongUntilCleanup,"));
    assert_eq!(
        projection
            .matches("HeapWeakEdgeRetention::DoesNotRetain")
            .count(),
        1,
    );
    assert!(!projection.contains("_ =>"));
}

#[test]
fn weak_edge_identity_is_the_exact_capability_free_domain() {
    assert_eq!(
        variants(WEAK_EDGE_SOURCE, "pub(crate) enum HeapWeakEdge {"),
        [
            "WeakMapKey,",
            "WeakMapValue,",
            "WeakSetValue,",
            "WeakRefTarget,",
            "FinalizationRegistryTarget,",
            "FinalizationRegistryHoldings,",
            "FinalizationRegistryUnregisterToken,",
        ],
    );
    assert!(WEAK_EDGE_SOURCE.contains("}\n\npub(crate) enum HeapWeakEdge {"));
    assert!(!WEAK_EDGE_SOURCE.contains("struct HeapWeakEdgeSlot"));
}

#[test]
fn weak_edge_identity_owns_one_exhaustive_metadata_projection() {
    let implementation = bounded(
        WEAK_EDGE_SOURCE,
        "impl HeapWeakEdge {",
        "pub(crate) const HEAP_WEAK_EDGES",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(!implementation.contains("todo!"));

    let normalized_implementation = normalized(implementation);
    for (variant, record, name, kind) in [
        ("WeakMapKey", "weak-map-entry", "key", "EphemeronKey"),
        ("WeakMapValue", "weak-map-entry", "value", "EphemeronValue"),
        ("WeakSetValue", "weak-set-entry", "value", "EphemeronKey"),
        ("WeakRefTarget", "weak-ref-record", "target", "WeakTarget"),
        (
            "FinalizationRegistryTarget",
            "finalization-registry-cell",
            "target",
            "WeakTarget",
        ),
        (
            "FinalizationRegistryHoldings",
            "finalization-registry-cell",
            "holdings",
            "FinalizerHoldings",
        ),
        (
            "FinalizationRegistryUnregisterToken",
            "finalization-registry-cell",
            "unregister-token",
            "FinalizerToken",
        ),
    ] {
        let arm = format!(
            "Self::{variant}=>HeapWeakEdgeMetadata{{record:\"{record}\",name:\"{name}\",kind:HeapWeakEdgeKind::{kind},}},"
        );
        assert!(
            normalized_implementation.contains(&arm),
            "missing exact metadata arm for {variant}"
        );
    }
    for accessor in ["record", "name", "kind"] {
        assert!(
            implementation.contains(&format!("self.metadata().{accessor}")),
            "{accessor} must project through the sole metadata authority"
        );
    }
}

#[test]
fn weak_edge_registry_and_collector_policy_use_only_typed_identities() {
    let registry = normalized(bounded(
        WEAK_EDGE_SOURCE,
        "pub(crate) const HEAP_WEAK_EDGES",
        "];",
    ));
    assert_eq!(
        registry,
        ":&[HeapWeakEdge]=&[HeapWeakEdge::WeakMapKey,HeapWeakEdge::WeakMapValue,HeapWeakEdge::WeakSetValue,HeapWeakEdge::WeakRefTarget,HeapWeakEdge::FinalizationRegistryTarget,HeapWeakEdge::FinalizationRegistryHoldings,HeapWeakEdge::FinalizationRegistryUnregisterToken,"
    );

    let policy = normalized(bounded(
        POLICY_SOURCE,
        "impl HeapCollectorPolicy {",
        "pub(crate) const HEAP_COLLECTOR_POLICY",
    ));
    assert!(policy.contains("constfnweak_edges(&self)->&'static[HeapWeakEdge]"));
    assert!(policy.contains("Self::NonMovingMetadataChecked=>HEAP_WEAK_EDGES,"));
    assert!(!HEAP_SOURCE.contains("HeapWeakEdgeSlot"));
    assert!(!HEAP_SOURCE.contains("HEAP_WEAK_EDGE_SLOTS"));
}
