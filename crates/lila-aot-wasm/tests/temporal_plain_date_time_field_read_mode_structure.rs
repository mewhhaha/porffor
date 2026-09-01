const SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_time_methods.rs");
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
fn plain_date_time_field_read_mode_is_a_private_capability_free_domain() {
    let domain = bounded(
        SOURCE,
        "    EraPair,\n}\n\n",
        "\n\nimpl TemporalDateTimeFieldKey",
    );
    let declaration = bounded(domain, "enum TemporalPlainDateTimeFieldReadMode {", "\n}");
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Conversion,", "With,"]);
    assert!(!SOURCE.contains("pub enum TemporalPlainDateTimeFieldReadMode"));
    assert!(!SOURCE.contains("pub(crate) enum TemporalPlainDateTimeFieldReadMode"));
    assert!(!SOURCE.contains("pub(super) enum TemporalPlainDateTimeFieldReadMode"));
    assert!(!domain.contains("#[derive("));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!declaration.contains(capability));
    }
}

#[test]
fn field_reader_projects_the_mode_once_and_exhaustively() {
    let reader = bounded(
        SOURCE,
        "    fn emit_temporal_plain_date_time_read_fields(",
        "    /// `ToTemporalDateTime`.",
    );

    assert!(reader.contains("mode: TemporalPlainDateTimeFieldReadMode,"));
    assert_eq!(reader.matches("match mode {").count(), 1);
    assert_eq!(
        reader
            .matches("TemporalPlainDateTimeFieldReadMode::Conversion => {")
            .count(),
        1
    );
    assert_eq!(
        reader
            .matches("TemporalPlainDateTimeFieldReadMode::With => {}")
            .count(),
        1
    );
    let conversion_arm = bounded(
        reader,
        "TemporalPlainDateTimeFieldReadMode::Conversion => {",
        "\n            }\n            TemporalPlainDateTimeFieldReadMode::With => {}",
    );
    assert_eq!(
        conversion_arm
            .matches("self.strings.payload(\"calendar\")")
            .count(),
        1
    );
    assert_eq!(
        conversion_arm
            .matches("self.emit_temporal_to_temporal_calendar_identifier(")
            .count(),
        1
    );
    assert!(!reader.contains("read_calendar"));
    assert!(!reader.contains(": bool"));
    assert!(!reader.contains("matches!(mode"));
    assert!(!reader.contains("_ =>"));
    assert!(!reader.contains("unreachable!"));
}

#[test]
fn exactly_two_producers_select_conversion_and_with() {
    let conversion = bounded(
        SOURCE,
        "    pub(super) fn emit_to_temporal_date_time(",
        "    pub(crate) fn emit_temporal_plain_date_time_from(",
    );
    assert_eq!(
        conversion
            .matches("TemporalPlainDateTimeFieldReadMode::Conversion,")
            .count(),
        1
    );
    assert!(!conversion.contains("TemporalPlainDateTimeFieldReadMode::With"));

    let with = bounded(
        SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_time_with(",
        "    /// Temporal proposal 5.3.x `withPlainTime`.",
    );
    assert_eq!(
        with.matches("TemporalPlainDateTimeFieldReadMode::With,")
            .count(),
        1
    );
    assert!(!with.contains("TemporalPlainDateTimeFieldReadMode::Conversion"));
    assert_eq!(
        SOURCE
            .matches("self.emit_temporal_plain_date_time_read_fields(")
            .count(),
        2
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
                .find("self.emit_temporal_plain_date_time_read_fields(")
                .unwrap()
    );
}
