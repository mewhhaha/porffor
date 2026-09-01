const PHASE_SOURCE: &str = include_str!("../src/heap_collector_phases.rs");
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
fn required_collector_phase_is_the_exact_closed_domain() {
    assert_eq!(
        variants(PHASE_SOURCE, "pub(crate) enum RequiredHeapCollectorPhase {"),
        [
            "StopTheWorld,",
            "RootScan,",
            "MarkStrong,",
            "ProcessEphemerons,",
            "ClearWeakRefs,",
            "QueueFinalizers,",
            "Sweep,",
            "Resume,",
        ]
    );
}

#[test]
fn required_phase_owns_one_exhaustive_name_projection() {
    let implementation = bounded(
        PHASE_SOURCE,
        "impl RequiredHeapCollectorPhase {",
        "pub(crate) const REQUIRED_HEAP_COLLECTOR_PHASES",
    );
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert!(!implementation.contains("_ =>"));
    for (variant, name) in [
        ("StopTheWorld", "stop-the-world"),
        ("RootScan", "scan-roots"),
        ("MarkStrong", "mark-strong-graph"),
        ("ProcessEphemerons", "process-ephemerons"),
        ("ClearWeakRefs", "clear-weakrefs"),
        ("QueueFinalizers", "queue-finalizers"),
        ("Sweep", "sweep-unmarked"),
        ("Resume", "resume"),
    ] {
        assert!(implementation.contains(&format!("Self::{variant} => \"{name}\",")));
        assert_eq!(PHASE_SOURCE.matches(&format!("\"{name}\"")).count(), 1);
    }
}

#[test]
fn required_phase_registry_contains_every_phase_once_in_order() {
    let registry = normalized(bounded(
        PHASE_SOURCE,
        "pub(crate) const REQUIRED_HEAP_COLLECTOR_PHASES",
        "];",
    ));
    assert_eq!(
        registry,
        ":&[RequiredHeapCollectorPhase]=&[RequiredHeapCollectorPhase::StopTheWorld,RequiredHeapCollectorPhase::RootScan,RequiredHeapCollectorPhase::MarkStrong,RequiredHeapCollectorPhase::ProcessEphemerons,RequiredHeapCollectorPhase::ClearWeakRefs,RequiredHeapCollectorPhase::QueueFinalizers,RequiredHeapCollectorPhase::Sweep,RequiredHeapCollectorPhase::Resume,"
    );
    assert!(!PHASE_SOURCE.contains("required_for_gc_builtin"));
    assert!(!PHASE_SOURCE.contains("struct HeapCollectorPhase"));
}

#[test]
fn collector_policy_accepts_only_required_phases() {
    let policy = normalized(bounded(
        POLICY_SOURCE,
        "impl HeapCollectorPolicy {",
        "pub(crate) const HEAP_COLLECTOR_POLICY",
    ));
    assert!(policy.contains("constfnrequired_phases(&self)->&'static[RequiredHeapCollectorPhase]"));
    assert!(policy.contains("Self::NonMovingMetadataChecked=>REQUIRED_HEAP_COLLECTOR_PHASES,"));
    assert!(!POLICY_SOURCE.contains("HeapCollectorPhaseKind"));
    assert!(!POLICY_SOURCE.contains("required_for_gc_builtin"));
}
