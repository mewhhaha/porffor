const DATE_METHODS_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date_methods.rs");
const MONTH_DAY_SOURCE: &str = include_str!("../src/builtins/temporal_plain_month_day.rs");

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
fn temporal_date_field_read_mode_is_a_closed_four_variant_domain() {
    let type_declaration = bounded(
        DATE_METHODS_SOURCE,
        "#[derive(",
        "\n\n/// `ISO_REFERENCE_YEAR`",
    );
    let declaration = bounded(
        DATE_METHODS_SOURCE,
        "pub(super) enum TemporalDateFieldReadMode {",
        "\n}\n\n/// `ISO_REFERENCE_YEAR`",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(
        variants,
        [
            "DateConversion,",
            "DateWith,",
            "MonthDayConversion,",
            "MonthDayWith,",
        ]
    );
    assert!(!type_declaration.contains("Default"));
}

#[test]
fn temporal_date_field_reader_projects_both_policies_exhaustively() {
    let reader = bounded(
        DATE_METHODS_SOURCE,
        "    pub(super) fn emit_temporal_plain_date_read_fields(",
        "    /// `CalendarResolveFields` + `RegulateISODate`.",
    );

    assert!(reader.contains("mode: TemporalDateFieldReadMode,"));
    assert_eq!(reader.matches("match mode {").count(), 2);
    for variant in [
        "TemporalDateFieldReadMode::DateConversion",
        "TemporalDateFieldReadMode::DateWith",
        "TemporalDateFieldReadMode::MonthDayConversion",
        "TemporalDateFieldReadMode::MonthDayWith",
    ] {
        assert_eq!(reader.matches(variant).count(), 2, "variant `{variant}`");
    }
    assert!(!reader.contains("read_calendar"));
    assert!(!reader.contains("strict_month_code"));
    assert!(!reader.contains(": bool"));
    assert!(!reader.contains("matches!(mode"));
    assert!(!reader.contains("=> true"));
    assert!(!reader.contains("=> false"));
    assert!(!reader.contains("_ =>"));
    assert!(!reader.contains("unreachable!"));
}

#[test]
fn exactly_four_producers_select_their_named_field_read_modes() {
    let date_conversion = bounded(
        DATE_METHODS_SOURCE,
        "    pub(super) fn emit_temporal_to_temporal_date(",
        "    pub(crate) fn emit_temporal_plain_date_from(",
    );
    assert_eq!(
        date_conversion
            .matches("TemporalDateFieldReadMode::DateConversion")
            .count(),
        1
    );
    assert!(!date_conversion.contains("TemporalDateFieldReadMode::DateWith"));

    let date_with = bounded(
        DATE_METHODS_SOURCE,
        "    pub(crate) fn emit_temporal_plain_date_with(",
        "    pub(super) fn emit_temporal_plain_date_add_or_subtract(",
    );
    assert_eq!(
        date_with
            .matches("TemporalDateFieldReadMode::DateWith")
            .count(),
        1
    );
    assert!(!date_with.contains("TemporalDateFieldReadMode::DateConversion"));

    let month_day_conversion = bounded(
        MONTH_DAY_SOURCE,
        "    pub(super) fn emit_temporal_to_temporal_month_day(",
        "    /// Temporal proposal 10.2.2 `Temporal.PlainMonthDay.from`.",
    );
    assert_eq!(
        month_day_conversion
            .matches("TemporalDateFieldReadMode::MonthDayConversion")
            .count(),
        1
    );
    assert!(!month_day_conversion.contains("TemporalDateFieldReadMode::MonthDayWith"));

    let month_day_with = bounded(
        MONTH_DAY_SOURCE,
        "    pub(crate) fn emit_temporal_plain_month_day_with(",
        "    /// `Temporal.PlainMonthDay.prototype.toLocaleString`.",
    );
    assert_eq!(
        month_day_with
            .matches("TemporalDateFieldReadMode::MonthDayWith")
            .count(),
        1
    );
    assert!(!month_day_with.contains("TemporalDateFieldReadMode::MonthDayConversion"));

    assert_eq!(
        DATE_METHODS_SOURCE
            .matches("self.emit_temporal_plain_date_read_fields(")
            .count()
            + MONTH_DAY_SOURCE
                .matches("self.emit_temporal_plain_date_read_fields(")
                .count(),
        4
    );
}
