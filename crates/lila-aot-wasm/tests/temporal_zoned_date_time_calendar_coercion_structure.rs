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
fn zoned_date_time_calendar_coercion_is_a_closed_domain() {
    let declaration = bounded(
        TEMPORAL_SOURCE,
        "enum ZonedDateTimeCalendarCoercion {",
        "\n}\n\n#[derive(Clone, Copy)]",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(
        variants,
        ["ToTemporalCalendarIdentifier,", "CanonicalizeCalendar,"]
    );
    assert!(TEMPORAL_SOURCE.contains("enum ZonedDateTimeCalendarCoercion"));
    assert!(!declaration.contains("Default"));
}

#[test]
fn property_bag_and_constructor_select_their_spec_operations() {
    let property_bag = bounded(
        TEMPORAL_SOURCE,
        "    fn emit_temporal_zoned_date_time_from_property_bag(",
        "    fn emit_temporal_regulate_property_bag_date_time(",
    );
    assert_eq!(
        property_bag
            .matches("ZonedDateTimeCalendarCoercion::ToTemporalCalendarIdentifier")
            .count(),
        1
    );
    assert!(!property_bag.contains("ZonedDateTimeCalendarCoercion::CanonicalizeCalendar"));

    let constructor = bounded(
        TEMPORAL_SOURCE,
        "    pub(crate) fn emit_temporal_zoned_date_time_constructor(",
        "    pub(crate) fn emit_temporal_zoned_date_time_time_zone(",
    );
    assert_eq!(
        constructor
            .matches("ZonedDateTimeCalendarCoercion::CanonicalizeCalendar")
            .count(),
        1
    );
    assert!(!constructor.contains("ZonedDateTimeCalendarCoercion::ToTemporalCalendarIdentifier"));

    assert_eq!(
        TEMPORAL_SOURCE
            .matches("emit_temporal_zoned_date_time_calendar(")
            .count(),
        3,
        "exactly two producers and one consumer own this coercion"
    );
}

#[test]
fn zoned_date_time_calendar_coercion_projects_exhaustively() {
    let projection = bounded(
        TEMPORAL_SOURCE,
        "    fn emit_temporal_zoned_date_time_calendar(",
        "    pub(crate) fn emit_alloc_temporal_zoned_date_time(",
    );

    assert!(projection.contains("coercion: ZonedDateTimeCalendarCoercion"));
    assert!(projection.contains("match coercion {"));
    assert_eq!(
        projection
            .matches("ZonedDateTimeCalendarCoercion::ToTemporalCalendarIdentifier =>")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("ZonedDateTimeCalendarCoercion::CanonicalizeCalendar =>")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("emit_temporal_to_temporal_calendar_identifier(")
            .count(),
        1
    );
    assert_eq!(
        projection
            .matches("emit_temporal_canonicalize_calendar(")
            .count(),
        1
    );
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));
    assert!(!projection.contains(": bool"));
    assert!(!projection.contains("if coercion"));
}
