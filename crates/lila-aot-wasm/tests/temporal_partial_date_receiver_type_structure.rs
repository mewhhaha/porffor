const YEAR_MONTH_SOURCE: &str = include_str!("../src/builtins/temporal_plain_year_month.rs");
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

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn partial_date_type_projects_brand_and_receiver_diagnostic_exhaustively() {
    let projections = normalized(bounded(
        YEAR_MONTH_SOURCE,
        "impl TemporalPartialDateType {",
        "/// Where a partial-date object's prototype comes from.",
    ));
    for mapping in [
        "TemporalPartialDateType::PlainYearMonth=>{OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH}",
        "TemporalPartialDateType::PlainMonthDay=>{OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY}",
        "TemporalPartialDateType::PlainYearMonth=>{\"Temporal.PlainYearMonthreceiverdoesnothave[[InitializedTemporalYearMonth]]\"}",
        "TemporalPartialDateType::PlainMonthDay=>{\"Temporal.PlainMonthDayreceiverdoesnothave[[InitializedTemporalMonthDay]]\"}",
    ] {
        assert_eq!(
            projections.matches(mapping).count(),
            1,
            "mapping `{mapping}`"
        );
    }
    assert_eq!(projections.matches("=>").count(), 6);
    assert!(!projections.contains("_=>"));
    assert!(!projections.contains("unreachable!"));
}

#[test]
fn partial_date_receiver_helper_accepts_only_the_existing_closed_type() {
    let signature = bounded(
        YEAR_MONTH_SOURCE,
        "pub(crate) fn emit_temporal_branded_record_from_receiver(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("partial_date_type: TemporalPartialDateType,"));
    assert!(!signature.contains("brand: u64"));
    assert!(!signature.contains("message: &str"));

    let emitter = bounded(
        YEAR_MONTH_SOURCE,
        "pub(crate) fn emit_temporal_branded_record_from_receiver(",
        "pub(crate) fn emit_temporal_partial_date_load_record(",
    );
    assert_eq!(
        emitter
            .matches("partial_date_type.receiver_error_message()")
            .count(),
        1
    );
    assert_eq!(emitter.matches("partial_date_type.brand()").count(), 1);
}

#[test]
fn both_partial_date_wrappers_pass_only_their_named_type() {
    assert_eq!(
        YEAR_MONTH_SOURCE
            .matches("self.emit_temporal_branded_record_from_receiver(")
            .count()
            + MONTH_DAY_SOURCE
                .matches("self.emit_temporal_branded_record_from_receiver(")
                .count(),
        2
    );

    let year_month = normalized(bounded(
        YEAR_MONTH_SOURCE,
        "pub(crate) fn emit_temporal_plain_year_month_record_from_receiver(",
        "pub(crate) fn emit_temporal_plain_year_month_field(",
    ));
    assert_eq!(
        year_month
            .matches("self.emit_temporal_branded_record_from_receiver(TemporalPartialDateType::PlainYearMonth,record_local,function,)")
            .count(),
        1
    );
    assert!(!year_month.contains("OBJECT_INTERNAL_BRAND_"));
    assert!(!year_month.contains("receiverdoesnothave"));

    let month_day = normalized(bounded(
        MONTH_DAY_SOURCE,
        "pub(crate) fn emit_temporal_plain_month_day_record_from_receiver(",
        "pub(crate) fn emit_temporal_plain_month_day_field(",
    ));
    assert_eq!(
        month_day
            .matches("self.emit_temporal_branded_record_from_receiver(TemporalPartialDateType::PlainMonthDay,record_local,function,)")
            .count(),
        1
    );
    assert!(!month_day.contains("OBJECT_INTERNAL_BRAND_"));
    assert!(!month_day.contains("receiverdoesnothave"));
}
