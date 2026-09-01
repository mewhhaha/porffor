const DURATION_SOURCE: &str = include_str!("../src/builtins/temporal_duration.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn duration_field_transform_is_a_closed_domain() {
    let declaration = bounded(
        DURATION_SOURCE,
        "enum TemporalDurationFieldTransform {",
        "\n}\n\npub(crate) const TEMPORAL_DURATION_FIELD_OFFSETS",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Negate,", "AbsoluteValue,"]);
    assert!(!declaration.contains("Default"));
}

#[test]
fn duration_field_transform_projects_exhaustively() {
    let emitter = bounded(
        DURATION_SOURCE,
        "    fn emit_temporal_duration_with_field_transform(",
        "    pub(crate) fn emit_temporal_duration_value_of(",
    );

    assert!(emitter.contains("transform: TemporalDurationFieldTransform"));
    assert!(emitter.contains("match transform {"));
    assert_eq!(
        emitter
            .matches("TemporalDurationFieldTransform::Negate =>")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("TemporalDurationFieldTransform::AbsoluteValue =>")
            .count(),
        1
    );
    assert!(!emitter.contains("_ =>"));
    assert!(!emitter.contains("unreachable!"));
    assert!(!emitter.contains(": bool"));
    assert!(!emitter.contains("if negate"));
}

#[test]
fn duration_unary_wrappers_and_dispatch_pin_both_transforms() {
    let negated = bounded(
        DURATION_SOURCE,
        "    pub(crate) fn emit_temporal_duration_negated(",
        "    pub(crate) fn emit_temporal_duration_abs(",
    );
    assert_eq!(
        negated
            .matches("TemporalDurationFieldTransform::Negate")
            .count(),
        1
    );
    assert!(!negated.contains("AbsoluteValue"));

    let abs = bounded(
        DURATION_SOURCE,
        "    pub(crate) fn emit_temporal_duration_abs(",
        "    /// Both unary transforms rebuild the duration",
    );
    assert_eq!(
        abs.matches("TemporalDurationFieldTransform::AbsoluteValue")
            .count(),
        1
    );
    assert!(!abs.contains("::Negate"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TemporalDurationPrototypeNegated => {",
        "            StandardBuiltinId::TemporalDurationPrototypeAdd => {",
    );
    assert_eq!(
        dispatch.matches("emit_temporal_duration_negated(").count(),
        1
    );
    assert_eq!(dispatch.matches("emit_temporal_duration_abs(").count(), 1);
    assert!(!dispatch.contains("emit_temporal_duration_negated_or_abs"));
    assert!(!dispatch.contains("true, function"));
    assert!(!dispatch.contains("false, function"));

    assert_eq!(
        DURATION_SOURCE
            .matches("emit_temporal_duration_with_field_transform(")
            .count(),
        3,
        "one private emitter and two named wrappers own the transform"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("emit_temporal_duration_negated(")
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("emit_temporal_duration_abs(")
            .count(),
        1
    );
}
