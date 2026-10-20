const DATE_METHODS_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_date_methods.rs");
const DATE_CONVERSION: &str =
    include_str!("../../src/builtins/temporal_plain_date_methods/convert.rs");
const MONTH_DAY_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_month_day.rs");
const DATE_SOURCE: &str = include_str!("../../src/builtins/temporal_plain_date.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end: {end}"))
        .0
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn in_order(source: &str, markers: &[&str]) {
    let mut rest = source;
    for marker in markers {
        rest = rest
            .split_once(marker)
            .unwrap_or_else(|| panic!("missing ordered marker: {marker}"))
            .1;
    }
}

#[test]
fn temporal_date_field_reader_consumes_the_checked_calendar_owner() {
    let reader = bounded(
        DATE_METHODS_SOURCE,
        "fn emit_temporal_plain_date_read_fields(",
        "/// `CalendarResolveFields` + `RegulateISODate`.",
    );
    assert!(reader.contains("calendar: &TemporalCalendarSlotLocals"));
    assert!(reader.contains("calendar.calendar_id()"));
    for raw_policy in [
        "read_calendar: bool",
        "strict_month_code: bool",
        "TemporalDateFieldReadMode",
    ] {
        assert!(!reader.contains(raw_policy));
    }
    let era = bounded(
        DATE_SOURCE,
        "fn emit_temporal_read_era_fields(",
        "fn emit_temporal_resolve_era_to_calendar_year(",
    );
    assert!(era.contains("self.emit_temporal_calendar_has_eras_i32(calendar_id, function)"));
    in_order(
        era,
        &[
            "self.emit_temporal_calendar_has_eras_i32(",
            "self.open_frame(ControlFrameKind::If",
            "\"era\"",
            "\"eraYear\"",
        ],
    );
}

#[test]
fn temporal_date_field_reader_shares_month_code_validation_in_original_order() {
    let reader = bounded(
        DATE_METHODS_SOURCE,
        "fn emit_temporal_plain_date_read_fields(",
        "/// `CalendarResolveFields` + `RegulateISODate`.",
    );
    in_order(
        reader,
        &[
            "self.reserve_temporal_era_slots(",
            "\"day\"",
            "self.emit_temporal_read_era_fields(",
            "\"month\"",
            "\"monthCode\"",
            "self.emit_temporal_month_code_string(",
            "\"year\"",
        ],
    );
    assert!(!reader.contains("self.emit_temporal_property_bag_string("));
    assert!(!reader.contains("_ =>"));
}

#[test]
fn date_and_month_day_producers_pass_their_actual_calendar_before_field_reads() {
    let kernel = compact(
        DATE_CONVERSION
            .split_once("fn emit_temporal_to_temporal_date_kernel(")
            .unwrap()
            .1,
    );
    let bag = kernel
        .split_once("letcalendar_value=schema.reserve_value_local(function);")
        .unwrap()
        .1;
    in_order(
        bag,
        &[
            "self.emit_temporal_duration_option_get(argument,\"calendar\"",
            "self.emit_temporal_to_temporal_calendar_identifier(",
            "self.emit_temporal_plain_date_read_fields(",
            "overflow_options.emit_if_present(",
            "self.emit_temporal_resolve_era_to_calendar_year(",
            "self.emit_temporal_plain_date_resolve_fields(",
        ],
    );
    let date_with = compact(bounded(
        DATE_METHODS_SOURCE,
        "fn emit_temporal_plain_date_with(",
        "fn emit_temporal_plain_date_with_calendar(",
    ));
    assert!(date_with.contains("self.emit_temporal_plain_date_read_fields(&argument,&calendar"));
    for (start, end) in [
        (
            "fn emit_temporal_to_temporal_month_day(",
            "fn emit_temporal_plain_month_day_from(",
        ),
        (
            "fn emit_temporal_plain_month_day_with(",
            "fn emit_temporal_plain_month_day_to_locale_string(",
        ),
    ] {
        let body = compact(bounded(MONTH_DAY_SOURCE, start, end));
        assert!(
            body.contains("self.emit_temporal_plain_date_read_fields(argument,&calendar")
                || body.contains("self.emit_temporal_plain_date_read_fields(&argument,&calendar")
        );
    }
}
