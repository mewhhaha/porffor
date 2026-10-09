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
            "ZonedWith { offset_nanoseconds_local: I64Local },"
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
    assert_eq!(reader.matches("match &mode {").count(), 1);
    assert!(reader.contains("calendar: &TemporalCalendarSlotLocals,"));
    assert!(!reader.contains("\"calendar\""));
    let conversion = bounded(
        SOURCE,
        "    pub(super) fn emit_to_temporal_date_time(",
        "    pub(crate) fn emit_temporal_plain_date_time_from(",
    );
    let get = conversion.find("self.emit_temporal_duration_option_get(argument, \"calendar\", &calendar_value, function)?;").unwrap();
    let canonicalize = conversion
        .find("let calendar = self.emit_temporal_to_temporal_calendar_identifier(")
        .unwrap();
    let sweep = conversion
        .find("self.emit_temporal_date_time_read_fields(")
        .unwrap();
    assert!(get < canonicalize && canonicalize < sweep);
    let offset_projection = bounded(
        reader,
        "TemporalDateTimeFieldRead::Offset => {",
        "TemporalDateTimeFieldRead::EraPair => {",
    );
    assert!(offset_projection.contains("TemporalDateTimeFieldReadMode::ZonedWith"));
    assert!(offset_projection.contains("TemporalDateTimeFieldReadMode::Conversion"));
    assert!(offset_projection.contains("TemporalDateTimeFieldReadMode::With"));
    assert!(offset_projection.contains("self.emit_tagged_to_primitive_locals("));
    assert!(offset_projection.contains("ToPrimitiveHint::String,"));
    assert!(offset_projection.contains("TEMPORAL_ZONEDDATETIME_OFFSET_MUST_BE_A_STRING"));
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
        "    pub(crate) fn emit_temporal_plain_date_time_with_plain_time(",
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
    assert!(YEAR_MONTH_SOURCE.contains("calendar: &TemporalCalendarSlotLocals,"));
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
        "    pub(crate) fn emit_temporal_plain_date_time_with_plain_time(",
    );
    let forbidden_property_reads = bounded(
        with,
        "        // Both observable Gets precede the ordered field sweep.",
        "        acquired_month_code.set_undefined(function);",
    );

    assert!(forbidden_property_reads.contains("for property in [\"calendar\", \"timeZone\"]"));
    assert_eq!(
        forbidden_property_reads
            .matches("self.emit_temporal_duration_option_get(")
            .count(),
        1
    );
    assert_eq!(
        forbidden_property_reads
            .matches("self.emit_temporal_error_and_return(")
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
