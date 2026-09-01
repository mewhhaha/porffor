const PLAIN_DATE_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date.rs");
const TEMPORAL_SOURCE: &str = include_str!("../src/builtins/temporal.rs");

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
fn calendar_canonicalization_context_is_a_closed_domain() {
    let declaration = bounded(
        PLAIN_DATE_SOURCE,
        "pub(super) enum TemporalCalendarCanonicalizationContext {",
        "\n}\n\nimpl TemporalCalendarCanonicalizationContext",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["PlainDateFamily,", "ZonedDateTime,"]);
    assert!(!declaration.contains("Default"));
}

#[test]
fn calendar_canonicalization_context_projects_both_diagnostics_exhaustively() {
    let projection = bounded(
        PLAIN_DATE_SOURCE,
        "impl TemporalCalendarCanonicalizationContext {",
        "\n}\n\nimpl TemporalCalendarId",
    );

    assert_eq!(projection.matches("match self {").count(), 2);
    assert_eq!(projection.matches("Self::PlainDateFamily =>").count(), 2);
    assert_eq!(projection.matches("Self::ZonedDateTime =>").count(), 2);
    assert!(projection.contains("Temporal.PlainDate calendar must be a string"));
    assert!(projection.contains("Invalid Temporal.PlainDate calendar"));
    assert!(projection.contains("Temporal.ZonedDateTime calendar must be a string"));
    assert!(projection.contains("Invalid Temporal.ZonedDateTime calendar"));
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));
}

#[test]
fn canonicalization_helper_has_exactly_two_typed_producers() {
    let helper = bounded(
        PLAIN_DATE_SOURCE,
        "    pub(super) fn emit_temporal_canonicalize_calendar(",
        "    pub(crate) fn emit_temporal_plain_date_calendar(",
    );
    assert!(helper.contains("context: TemporalCalendarCanonicalizationContext"));
    assert!(helper.contains("context.type_error_message()"));
    assert!(helper.contains("context.range_error_message()"));
    assert!(!helper.contains("type_error_message: &str"));
    assert!(!helper.contains("range_error_message: &str"));

    let plain_date_family = bounded(
        PLAIN_DATE_SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_calendar(",
        "    pub(crate) fn emit_temporal_calendar_is_default_i32(",
    );
    assert_eq!(
        plain_date_family
            .matches("TemporalCalendarCanonicalizationContext::PlainDateFamily")
            .count(),
        1
    );

    let zoned_date_time = bounded(
        TEMPORAL_SOURCE,
        "    fn emit_temporal_zoned_date_time_calendar(",
        "    pub(crate) fn emit_alloc_temporal_zoned_date_time(",
    );
    assert_eq!(
        zoned_date_time
            .matches("TemporalCalendarCanonicalizationContext::ZonedDateTime")
            .count(),
        1
    );

    assert_eq!(
        PLAIN_DATE_SOURCE
            .matches("emit_temporal_canonicalize_calendar(")
            .count()
            + TEMPORAL_SOURCE
                .matches("emit_temporal_canonicalize_calendar(")
                .count(),
        3,
        "exactly two producers and one consumer own calendar canonicalization"
    );
}
