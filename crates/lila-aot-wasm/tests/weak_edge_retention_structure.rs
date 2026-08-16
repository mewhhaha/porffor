const HEAP_SOURCE: &str = include_str!("../src/heap.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn variants(source: &str, declaration: &str) -> Vec<&str> {
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

#[test]
fn weak_edge_kind_owns_one_closed_retention_projection() {
    assert_eq!(
        variants(HEAP_SOURCE, "pub(crate) enum HeapWeakEdgeKind {"),
        [
            "EphemeronKey,",
            "EphemeronValue,",
            "WeakTarget,",
            "FinalizerHoldings,",
            "FinalizerToken,",
        ],
    );
    assert_eq!(
        variants(HEAP_SOURCE, "pub(crate) enum HeapWeakEdgeRetention {"),
        [
            "DoesNotRetain,",
            "ConditionalOnReachableEphemeronKey,",
            "StrongUntilCleanup,",
        ],
    );

    let projection = between(
        HEAP_SOURCE,
        "impl HeapWeakEdgeKind {",
        "pub(crate) struct HeapWeakEdgeSlot {",
    );
    assert!(projection.contains("pub(crate) const fn retention(self) -> HeapWeakEdgeRetention {"));
    assert!(projection.contains("match self {"));
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
    assert_eq!(
        projection
            .matches("HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey")
            .count(),
        1,
    );
    assert_eq!(
        projection
            .matches("HeapWeakEdgeRetention::StrongUntilCleanup")
            .count(),
        1,
    );
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));
    assert!(!projection.contains("todo!"));
}

#[test]
fn weak_edge_slots_cannot_override_kind_retention() {
    assert!(!HEAP_SOURCE.contains("keeps_target_alive"));

    let slot = between(
        HEAP_SOURCE,
        "pub(crate) struct HeapWeakEdgeSlot {",
        "\n}",
    );
    let fields = slot
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "pub record: &'static str,",
            "pub name: &'static str,",
            "pub kind: HeapWeakEdgeKind,",
        ],
    );

    let inventory = between(
        HEAP_SOURCE,
        "pub(crate) const HEAP_WEAK_EDGE_SLOTS: &[HeapWeakEdgeSlot] = &[",
        "pub(crate) const HEAP_COLLECTOR_PHASES: &[HeapCollectorPhase] = &[",
    );
    assert_eq!(inventory.matches("HeapWeakEdgeSlot {").count(), 7);
    for (kind, count) in [
        ("EphemeronKey", 2),
        ("EphemeronValue", 1),
        ("WeakTarget", 2),
        ("FinalizerHoldings", 1),
        ("FinalizerToken", 1),
    ] {
        assert_eq!(
            inventory
                .matches(&format!("kind: HeapWeakEdgeKind::{kind},"))
                .count(),
            count,
            "unexpected slot count for {kind}",
        );
    }
}
