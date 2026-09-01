use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/temporal_plain_year_month_methods.rs");

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
fn plain_year_month_field_read_mode_is_a_private_capability_free_two_row_domain() {
    let domain = bounded(
        SOURCE,
        "enum TemporalPlainYearMonthFieldReadMode {",
        "\n}\n\n/// Which partial-date goal",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Conversion,", "With,"]);
    assert!(!SOURCE.contains("pub enum TemporalPlainYearMonthFieldReadMode"));
    assert!(!SOURCE.contains("pub(crate) enum TemporalPlainYearMonthFieldReadMode"));
    assert!(!SOURCE.contains("pub(super) enum TemporalPlainYearMonthFieldReadMode"));
    let declaration_start = SOURCE
        .find("enum TemporalPlainYearMonthFieldReadMode {")
        .expect("missing PlainYearMonth field-read mode");
    let preceding_declaration = SOURCE[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "};");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!SOURCE.contains(&format!(
            "impl {capability} for TemporalPlainYearMonthFieldReadMode"
        )));
    }
}

#[test]
fn field_reader_borrows_one_exhaustive_mode_before_the_shared_field_sweep() {
    let reader = bounded(
        SOURCE,
        "    fn emit_temporal_year_month_read_fields(",
        "    fn emit_temporal_year_month_resolve_fields(",
    );

    assert!(reader.contains("mode: TemporalPlainYearMonthFieldReadMode,"));
    assert_eq!(reader.matches("match &mode {").count(), 1);
    assert_eq!(
        reader
            .matches("TemporalPlainYearMonthFieldReadMode::Conversion => {")
            .count(),
        1
    );
    assert_eq!(
        reader
            .matches("TemporalPlainYearMonthFieldReadMode::With => {}")
            .count(),
        1
    );
    let conversion = bounded(
        reader,
        "TemporalPlainYearMonthFieldReadMode::Conversion => {",
        "\n            }\n            TemporalPlainYearMonthFieldReadMode::With => {}",
    );
    assert_eq!(
        conversion
            .matches("self.strings.payload(\"calendar\")")
            .count(),
        1
    );
    assert_eq!(conversion.matches("self.emit_object_read(").count(), 1);
    assert_eq!(
        conversion
            .matches("self.emit_temporal_to_temporal_calendar_identifier(")
            .count(),
        1
    );
    assert!(
        reader.find("match &mode {").unwrap()
            < reader
                .find("let era = self.emit_temporal_read_era_fields(")
                .unwrap()
    );
    for forbidden in [
        "read_calendar",
        ": bool",
        "matches!(mode",
        "if mode",
        "_ =>",
        "unreachable!",
        "Default::default",
    ] {
        assert!(!reader.contains(forbidden), "forbidden `{forbidden}`");
    }
}

#[test]
fn conversion_and_with_are_the_exact_two_field_reader_producers() {
    let conversion = bounded(
        SOURCE,
        "    pub(super) fn emit_temporal_to_temporal_year_month(",
        "    pub(crate) fn emit_temporal_parse_year_month_string(",
    );
    assert_eq!(
        conversion
            .matches("TemporalPlainYearMonthFieldReadMode::Conversion,")
            .count(),
        1
    );
    assert!(!conversion.contains("TemporalPlainYearMonthFieldReadMode::With"));

    let with = bounded(
        SOURCE,
        "    pub(crate) fn emit_temporal_plain_year_month_with(",
        "    pub(super) fn emit_temporal_plain_year_month_add_or_subtract(",
    );
    assert_eq!(
        with.matches("TemporalPlainYearMonthFieldReadMode::With,")
            .count(),
        1
    );
    assert!(!with.contains("TemporalPlainYearMonthFieldReadMode::Conversion"));
    assert!(with.contains("for property in [\"calendar\", \"timeZone\"]"));
    assert!(
        with.find("for property in [\"calendar\", \"timeZone\"]")
            .unwrap()
            < with
                .find("self.emit_temporal_year_month_read_fields(")
                .unwrap()
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "TemporalPlainYearMonthFieldReadMode"),
        6,
        "the domain, reader and exactly two producers must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_temporal_year_month_read_fields("),
        3,
        "the reader definition and exactly two calls must stay inventoried"
    );
}
