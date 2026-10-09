use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../../src/builtins/temporal_plain_year_month_methods.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn field_reader_requires_an_already_acquired_calendar_capability() {
    let reader = bounded(
        SOURCE,
        "    fn emit_temporal_year_month_read_fields(",
        "    fn emit_temporal_year_month_resolve_fields(",
    );
    assert!(reader.contains("calendar: &TemporalCalendarSlotLocals,"));
    for forbidden in [
        "read_calendar",
        ": bool",
        "TemporalPlainYearMonthFieldReadMode",
        "\"calendar\"",
        "\"day\"",
        "emit_temporal_to_temporal_calendar_identifier(",
    ] {
        assert!(
            !reader.contains(forbidden),
            "reader cannot reacquire `{forbidden}`"
        );
    }
}

#[test]
fn field_reader_uses_the_borrowed_calendar_before_the_shared_ordered_sweep() {
    let reader = bounded(
        SOURCE,
        "    fn emit_temporal_year_month_read_fields(",
        "    fn emit_temporal_year_month_resolve_fields(",
    );
    let era = reader.find("self.emit_temporal_read_era_fields(slots, argument, calendar.calendar_id(), function)?;").unwrap();
    let month = reader.find("\"month\"").unwrap();
    let month_code = reader.find("\"monthCode\"").unwrap();
    let year = reader.find("\"year\"").unwrap();
    assert!(era < month && month < month_code && month_code < year);
    assert_eq!(
        reader
            .matches("self.emit_temporal_duration_option_get(")
            .count(),
        1
    );
}

#[test]
fn conversion_and_with_are_the_exact_two_completed_calendar_producers() {
    let conversion = bounded(
        SOURCE,
        "    pub(super) fn emit_temporal_to_temporal_year_month(",
        "    pub(crate) fn emit_temporal_parse_year_month_string(",
    );
    let get = conversion.find("self.emit_temporal_duration_option_get(argument, \"calendar\", &calendar_value, function)?;").unwrap();
    let resolve = conversion
        .find("self.emit_temporal_to_temporal_calendar_identifier(")
        .unwrap();
    let read = conversion
        .find("self.emit_temporal_year_month_read_fields(")
        .unwrap();
    assert!(get < resolve && resolve < read);
    assert_eq!(
        conversion
            .matches("self.emit_temporal_year_month_read_fields(")
            .count(),
        1
    );
    let with = bounded(
        SOURCE,
        "    pub(crate) fn emit_temporal_plain_year_month_with(",
        "    pub(super) fn emit_temporal_plain_year_month_add_or_subtract(",
    );
    let receiver = with
        .find("self.emit_temporal_year_month_receiver_fields(")
        .unwrap();
    let calendar = with
        .find("self.emit_temporal_year_month_calendar_slot(")
        .unwrap();
    let reject = with
        .find("for property in [\"calendar\", \"timeZone\"]")
        .unwrap();
    let read = with
        .find("self.emit_temporal_year_month_read_fields(")
        .unwrap();
    assert!(receiver < calendar && calendar < reject && reject < read);
    assert_eq!(
        with.matches("self.emit_temporal_year_month_read_fields(")
            .count(),
        1
    );
    assert!(!with.contains("self.emit_temporal_to_temporal_calendar_identifier("));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_temporal_year_month_read_fields("),
        3
    );
}
