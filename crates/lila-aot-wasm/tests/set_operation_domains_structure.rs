const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn enum_variants(name: &str) -> Vec<&'static str> {
    let marker = format!("enum {name} {{");
    COLLECTIONS_SOURCE
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing enum `{name}`"))
        .1
        .split_once('}')
        .unwrap_or_else(|| panic!("missing end of enum `{name}`"))
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn set_operation_region() -> &'static str {
    bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_predicate_iterate_receiver(",
        "    pub(crate) fn emit_set_prototype_for_each(",
    )
}

#[test]
fn set_operation_domains_are_exact_and_have_no_equality_capability() {
    assert_eq!(
        enum_variants("SetPredicateOperation"),
        ["IsDisjointFrom,", "IsSubsetOf,", "IsSupersetOf,"]
    );
    assert_eq!(
        enum_variants("SetPredicateReceiverIterationOperation"),
        ["IsDisjointFrom,", "IsSubsetOf,"]
    );
    assert_eq!(
        enum_variants("SetPredicateOtherIterationOperation"),
        ["IsDisjointFrom,", "IsSupersetOf,"]
    );
    assert_eq!(
        enum_variants("SetAlgebraOperation"),
        [
            "Difference,",
            "Intersection,",
            "SymmetricDifference,",
            "Union,"
        ]
    );
    assert_eq!(
        enum_variants("SetAlgebraReceiverIterationOperation"),
        ["Difference,", "Intersection,"]
    );

    for name in [
        "SetPredicateOperation",
        "SetPredicateReceiverIterationOperation",
        "SetPredicateOtherIterationOperation",
        "SetAlgebraOperation",
        "SetAlgebraReceiverIterationOperation",
    ] {
        assert!(
            COLLECTIONS_SOURCE.contains(&format!("#[derive(Clone, Copy)]\nenum {name} {{")),
            "{name} must derive only Clone and Copy"
        );
    }
}

#[test]
fn set_predicate_orchestration_constructs_only_restricted_operations() {
    let wrappers = bounded(
        COLLECTIONS_SOURCE,
        "    pub(crate) fn emit_set_prototype_is_disjoint_from(",
        "    fn emit_set_predicate(",
    );
    assert_eq!(wrappers.matches("self.emit_set_predicate(").count(), 3);
    for operation in ["IsDisjointFrom", "IsSubsetOf", "IsSupersetOf"] {
        assert_eq!(
            wrappers
                .matches(&format!("SetPredicateOperation::{operation}"))
                .count(),
            1,
            "predicate producer `{operation}`"
        );
    }

    let orchestration = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_predicate(",
        "    fn emit_set_algebra_iterate_receiver(",
    );
    assert_eq!(
        orchestration
            .matches("emit_set_predicate_iterate_receiver(")
            .count(),
        2
    );
    assert_eq!(
        orchestration
            .matches("emit_set_predicate_iterate_other(")
            .count(),
        2
    );
    for operation in ["IsDisjointFrom", "IsSubsetOf"] {
        assert_eq!(
            orchestration
                .matches(&format!(
                    "SetPredicateReceiverIterationOperation::{operation}"
                ))
                .count(),
            1,
            "receiver predicate operation `{operation}`"
        );
    }
    for operation in ["IsDisjointFrom", "IsSupersetOf"] {
        assert_eq!(
            orchestration
                .matches(&format!("SetPredicateOtherIterationOperation::{operation}"))
                .count(),
            1,
            "other predicate operation `{operation}`"
        );
    }
    assert_eq!(orchestration.matches("match operation").count(), 1);
    assert!(!orchestration.contains("_ =>"));
}

#[test]
fn set_iteration_helpers_exhaustively_consume_their_legal_domains() {
    let receiver_predicate = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_predicate_iterate_receiver(",
        "    fn emit_set_predicate_iterate_other(",
    );
    assert!(receiver_predicate.contains("operation: SetPredicateReceiverIterationOperation,"));
    assert_eq!(receiver_predicate.matches("match operation").count(), 1);
    assert_eq!(
        receiver_predicate
            .matches("SetPredicateReceiverIterationOperation::")
            .count(),
        2
    );

    let other_predicate = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_predicate_iterate_other(",
        "    pub(crate) fn emit_set_prototype_is_disjoint_from(",
    );
    assert!(other_predicate.contains("operation: SetPredicateOtherIterationOperation,"));
    assert_eq!(other_predicate.matches("match operation").count(), 1);
    assert_eq!(
        other_predicate
            .matches("SetPredicateOtherIterationOperation::")
            .count(),
        2
    );

    let receiver_algebra = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_algebra_iterate_receiver(",
        "    fn emit_set_algebra_iterate_other(",
    );
    assert!(receiver_algebra.contains("operation: SetAlgebraReceiverIterationOperation,"));
    assert_eq!(receiver_algebra.matches("match operation").count(), 1);
    assert_eq!(
        receiver_algebra
            .matches("SetAlgebraReceiverIterationOperation::")
            .count(),
        2
    );

    for helper in [receiver_predicate, other_predicate, receiver_algebra] {
        for forbidden in [
            "debug_assert!",
            "operation ==",
            "operation !=",
            "_ =>",
            "unreachable!",
        ] {
            assert!(
                !helper.contains(forbidden),
                "restricted helper contains `{forbidden}`"
            );
        }
    }
}

#[test]
fn set_algebra_policies_are_exhaustive_and_other_iteration_stays_complete() {
    let other_iteration = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_algebra_iterate_other(",
        "    pub(crate) fn emit_set_prototype_difference(",
    );
    assert!(other_iteration.contains("operation: SetAlgebraOperation,"));
    assert_eq!(other_iteration.matches("match operation").count(), 1);
    for operation in ["Difference", "Intersection", "SymmetricDifference", "Union"] {
        assert_eq!(
            other_iteration
                .matches(&format!("SetAlgebraOperation::{operation}"))
                .count(),
            1,
            "other algebra operation `{operation}`"
        );
    }
    assert!(!other_iteration.contains("_ =>"));

    let wrappers = bounded(
        COLLECTIONS_SOURCE,
        "    pub(crate) fn emit_set_prototype_difference(",
        "    fn emit_set_algebra(",
    );
    assert_eq!(wrappers.matches("self.emit_set_algebra(").count(), 4);
    for operation in ["Difference", "Intersection", "SymmetricDifference", "Union"] {
        assert_eq!(
            wrappers
                .matches(&format!("SetAlgebraOperation::{operation}"))
                .count(),
            1,
            "algebra producer `{operation}`"
        );
    }

    let algebra = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_algebra(",
        "    pub(crate) fn emit_set_prototype_for_each(",
    );
    let initialization = bounded(
        algebra,
        "        match operation {",
        "        let receiver_iteration = match operation {",
    );
    for operation in ["Difference", "Intersection", "SymmetricDifference", "Union"] {
        assert_eq!(
            initialization
                .matches(&format!("SetAlgebraOperation::{operation}"))
                .count(),
            1,
            "algebra initialization `{operation}`"
        );
    }
    assert_eq!(initialization.matches("emit_copy_set_record(").count(), 1);
    let receiver_projection = bounded(
        algebra,
        "        let receiver_iteration = match operation {",
        "        match receiver_iteration {",
    );
    for operation in ["Difference", "Intersection", "SymmetricDifference", "Union"] {
        assert_eq!(
            receiver_projection
                .matches(&format!("SetAlgebraOperation::{operation}"))
                .count(),
            1,
            "receiver-iteration projection `{operation}`"
        );
    }
    assert_eq!(
        receiver_projection
            .matches("Some(SetAlgebraReceiverIterationOperation::")
            .count(),
        2
    );
    assert_eq!(receiver_projection.matches("None").count(), 1);
    assert!(!receiver_projection.contains("_ =>"));
    assert_eq!(
        algebra
            .matches("emit_set_algebra_iterate_receiver(")
            .count(),
        1
    );
    assert_eq!(
        algebra.matches("emit_set_algebra_iterate_other(").count(),
        2
    );

    let region = set_operation_region();
    for forbidden in [
        "operation ==",
        "operation !=",
        "debug_assert!",
        "receiver iteration is only",
    ] {
        assert!(
            !region.contains(forbidden),
            "Set operation region contains `{forbidden}`"
        );
    }
}
