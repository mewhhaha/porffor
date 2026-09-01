const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const OWNER: &str = include_str!("../src/heap_temporal_duration_layout.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/temporal-duration-heap-slot-authority.md");
const TASK: &str = include_str!("../../../tasks/05-values-heap-gc.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn temporal_duration_heap_slot_is_the_exact_capability_free_domain() {
    let variants = bounded(
        OWNER,
        "pub(crate) enum TemporalDurationHeapSlot {",
        "\n}\n\nstruct TemporalDurationHeapSlotMetadata",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Years,",
            "Months,",
            "Weeks,",
            "Days,",
            "Hours,",
            "Minutes,",
            "Seconds,",
            "Milliseconds,",
            "Microseconds,",
            "Nanoseconds,",
        ]
    );
    assert!(!OWNER.contains("#[derive("));
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
        assert!(!OWNER.contains(&format!("impl {capability} for TemporalDurationHeapSlot")));
    }
}

#[test]
fn one_exhaustive_projection_owns_ten_exact_scalar_rows() {
    let projection = bounded(
        OWNER,
        "    const fn metadata(&self) -> TemporalDurationHeapSlotMetadata {",
        "\n    pub(crate) const fn layout(&self)",
    );
    for (variant, name, offset) in [
        ("Years", "years", "HEAP_TEMPORAL_DURATION_YEARS_OFFSET"),
        ("Months", "months", "HEAP_TEMPORAL_DURATION_MONTHS_OFFSET"),
        ("Weeks", "weeks", "HEAP_TEMPORAL_DURATION_WEEKS_OFFSET"),
        ("Days", "days", "HEAP_TEMPORAL_DURATION_DAYS_OFFSET"),
        ("Hours", "hours", "HEAP_TEMPORAL_DURATION_HOURS_OFFSET"),
        (
            "Minutes",
            "minutes",
            "HEAP_TEMPORAL_DURATION_MINUTES_OFFSET",
        ),
        (
            "Seconds",
            "seconds",
            "HEAP_TEMPORAL_DURATION_SECONDS_OFFSET",
        ),
        (
            "Milliseconds",
            "milliseconds",
            "HEAP_TEMPORAL_DURATION_MILLISECONDS_OFFSET",
        ),
        (
            "Microseconds",
            "microseconds",
            "HEAP_TEMPORAL_DURATION_MICROSECONDS_OFFSET",
        ),
        (
            "Nanoseconds",
            "nanoseconds",
            "HEAP_TEMPORAL_DURATION_NANOSECONDS_OFFSET",
        ),
    ] {
        let arm = bounded(
            projection,
            &format!("            Self::{variant} => TemporalDurationHeapSlotMetadata {{"),
            "            },",
        );
        assert!(arm.contains("record: \"temporal-duration-record\""));
        assert!(arm.contains(&format!("name: \"{name}\"")));
        assert!(arm.contains(&format!("offset: {offset}")));
        assert!(arm.contains("width: 8"));
        assert!(arm.contains("pointer: false"));
    }
    assert_eq!(projection.matches("Self::").count(), 10);
    assert!(!projection.contains("_ =>"));
}

#[test]
fn typed_registry_preserves_duration_component_order() {
    let registry = bounded(
        OWNER,
        "pub(crate) const HEAP_TEMPORAL_DURATION_RECORD_LAYOUT:",
        "];",
    );
    for variant in [
        "Years",
        "Months",
        "Weeks",
        "Days",
        "Hours",
        "Minutes",
        "Seconds",
        "Milliseconds",
        "Microseconds",
        "Nanoseconds",
    ] {
        assert_eq!(
            registry
                .matches(&format!("TemporalDurationHeapSlot::{variant}"))
                .count(),
            1
        );
    }
    assert_eq!(registry.matches("TemporalDurationHeapSlot::").count(), 10);
}

#[test]
fn temporal_duration_layout_has_one_private_owner() {
    assert_eq!(
        LIB_SOURCE
            .matches("mod heap_temporal_duration_layout;")
            .count(),
        1
    );
    assert!(!LIB_SOURCE.contains("pub mod heap_temporal_duration_layout;"));
    assert!(!HEAP_SOURCE.contains("record: \"temporal-duration-record\""));
    assert!(!HEAP_SOURCE.contains("HEAP_TEMPORAL_DURATION_RECORD_LAYOUT: &[HeapLayoutSlot]",));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("TemporalDurationHeapSlot"));
        assert!(evidence.contains("passive metadata migration"));
        assert!(evidence.contains("no new Temporal behavior"));
    }
}
