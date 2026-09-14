const SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
const ZONED_SOURCE: &str = include_str!("../src/builtins/temporal_zoned_date_time_with.rs");
const YEAR_MONTH_SOURCE: &str =
    include_str!("../src/builtins/temporal_plain_year_month_methods.rs");

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
fn date_time_field_read_mode_expresses_the_zoned_offset_destination() {
    let declaration = bounded(
        SOURCE,
        "pub(super) enum TemporalDateTimeFieldReadMode {",
        "\n}",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Conversion,",
            "With,",
            "ZonedWith {",
            "offset_nanoseconds_local: u32,",
            "},"
        ]
    );
    assert!(!declaration.contains(": bool"));
    assert!(!declaration.contains("Option<"));
}

#[test]
fn field_reader_projects_calendar_and_offset_modes_exhaustively() {
    let reader = bounded(
        SOURCE,
        "    pub(super) fn emit_temporal_date_time_read_fields(",
        "    /// `ToTemporalDateTime`.",
    );
    assert!(reader.contains("mode: TemporalDateTimeFieldReadMode,"));
    assert_eq!(reader.matches("match &mode {").count(), 2);
    let calendar_projection = bounded(
        reader,
        "match &mode {",
        "for key in TemporalDateTimeFieldKey::ALL",
    );
    assert!(calendar_projection.contains("TemporalDateTimeFieldReadMode::Conversion => {"));
    assert!(calendar_projection.contains("TemporalDateTimeFieldReadMode::With"));
    assert!(calendar_projection.contains("TemporalDateTimeFieldReadMode::ZonedWith"));
    assert_eq!(
        calendar_projection
            .matches("self.strings.payload(\"calendar\")")
            .count(),
        1
    );
    assert_eq!(
        calendar_projection
            .matches("self.emit_temporal_to_temporal_calendar_identifier(")
            .count(),
        1
    );
    let offset_projection = bounded(
        reader,
        "TemporalDateTimeFieldRead::Offset => {",
        "TemporalDateTimeFieldRead::EraPair => {",
    );
    assert!(offset_projection.contains("TemporalDateTimeFieldReadMode::ZonedWith"));
    assert_eq!(
        offset_projection
            .matches("self.emit_temporal_offset_string(")
            .count(),
        1
    );
    assert_eq!(
        offset_projection
            .matches("self.emit_temporal_utc_offset_nanoseconds(")
            .count(),
        1
    );
    for absent in ["read_calendar", ": bool", "_ =>", "unreachable!"] {
        assert!(!reader.contains(absent));
    }
}

#[test]
fn three_producers_select_plain_conversion_plain_with_and_zoned_with() {
    let conversion = bounded(
        SOURCE,
        "    pub(super) fn emit_to_temporal_date_time(",
        "    pub(crate) fn emit_temporal_plain_date_time_from(",
    );
    assert_eq!(
        conversion
            .matches("TemporalDateTimeFieldReadMode::Conversion,")
            .count(),
        1
    );
    assert!(!conversion.contains("TemporalDateTimeFieldReadMode::With"));

    let with = bounded(
        SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_time_with(",
        "    /// Temporal proposal 5.3.x `withPlainTime`.",
    );
    assert_eq!(
        with.matches("TemporalDateTimeFieldReadMode::With,").count(),
        1
    );
    assert!(!with.contains("TemporalDateTimeFieldReadMode::Conversion"));
    assert_eq!(
        SOURCE
            .matches("self.emit_temporal_date_time_read_fields(")
            .count(),
        2
    );
    assert_eq!(
        ZONED_SOURCE
            .matches("self.emit_temporal_date_time_read_fields(")
            .count(),
        1
    );
    assert_eq!(
        ZONED_SOURCE
            .matches("TemporalDateTimeFieldReadMode::ZonedWith {")
            .count(),
        1
    );
    assert!(!YEAR_MONTH_SOURCE.contains("read_calendar: bool,"));
    assert!(YEAR_MONTH_SOURCE.contains("enum TemporalPlainYearMonthFieldReadMode {"));
    assert_eq!(
        SOURCE
            .matches("fn emit_temporal_plain_year_month_read_fields(")
            .count(),
        0,
        "PlainYearMonth reader must remain in its own module"
    );
}

#[test]
fn with_reads_both_forbidden_temporal_properties_before_the_field_sweep() {
    let with = bounded(
        SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_time_with(",
        "    /// Temporal proposal 5.3.x `withPlainTime`.",
    );
    let forbidden_property_reads = bounded(
        with,
        "        // `RejectTemporalLikeObject` reads both keys with `Get`, not with a",
        "\n\n        for local in present_locals.iter()",
    );

    assert!(forbidden_property_reads.contains("for property in [\"calendar\", \"timeZone\"]"));
    assert_eq!(
        forbidden_property_reads
            .matches("self.emit_object_read(")
            .count(),
        1
    );
    assert_eq!(
        forbidden_property_reads
            .matches("self.emit_return_current_completion_if_throw(function);")
            .count(),
        1
    );
    assert!(forbidden_property_reads.contains("ValueKind::Undefined.tag()"));
    assert!(!forbidden_property_reads.contains("emit_object_own_property_present"));
    assert!(
        with.find(forbidden_property_reads).unwrap()
            < with
                .find("self.emit_temporal_date_time_read_fields(")
                .unwrap()
    );
}
