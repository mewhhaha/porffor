use std::fs;
use std::path::Path;

const MONTH_DAY_SOURCE: &str = include_str!("../src/builtins/temporal_plain_month_day.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/temporal-plain-month-day-parsed-year-privacy.md"
);
const TASK: &str = include_str!("../../../tasks/22-date-temporal.md");

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
fn parsed_year_carrier_and_raw_parser_are_owner_private() {
    assert!(MONTH_DAY_SOURCE.contains("\nstruct TemporalParsedMonthDayYear {"));
    assert!(!MONTH_DAY_SOURCE.contains("pub(crate) struct TemporalParsedMonthDayYear"));
    let carrier = bounded(
        MONTH_DAY_SOURCE,
        "struct TemporalParsedMonthDayYear {",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(!carrier.contains("pub("));
    assert_eq!(carrier.matches("year_local:").count(), 1);
    assert_eq!(carrier.matches("year_present_local:").count(), 1);

    assert_eq!(
        MONTH_DAY_SOURCE
            .matches("    fn emit_temporal_parse_month_day_string(")
            .count(),
        1
    );
    assert!(!MONTH_DAY_SOURCE.contains("pub(crate) fn emit_temporal_parse_month_day_string("));
}

#[test]
fn parsed_year_moves_from_the_only_parser_to_the_reference_year_step() {
    assert!(MONTH_DAY_SOURCE.contains("#[must_use]\nstruct TemporalParsedMonthDayYear {"));
    let parser = normalized(bounded(
        MONTH_DAY_SOURCE,
        "fn emit_temporal_parse_month_day_string(",
        "/// The three things `ToTemporalMonthDay`'s string branch",
    ));
    assert!(parser.contains(")->Result<TemporalParsedMonthDayYear,EmitError>{"));
    assert_eq!(parser.matches("Ok(TemporalParsedMonthDayYear{").count(), 1);
    assert!(parser.contains("year_local,year_present_local,"));

    let consumer = normalized(bounded(
        MONTH_DAY_SOURCE,
        "fn emit_temporal_month_day_string_reference_year(",
        "/// Temporal proposal 10.3.x `equals`.",
    ));
    assert!(consumer.contains("parsed:TemporalParsedMonthDayYear,"));
    assert_eq!(
        consumer.matches("letTemporalParsedMonthDayYear{").count(),
        1
    );
    assert!(consumer.contains("year_local,year_present_local,}=parsed;"));

    let parse = MONTH_DAY_SOURCE
        .find("        let parsed = self.emit_temporal_parse_month_day_string(")
        .expect("parsed-year producer call");
    let overflow = MONTH_DAY_SOURCE[parse..]
        .find("        match overflow_options {")
        .expect("overflow option dispatch")
        + parse;
    let consume = MONTH_DAY_SOURCE[overflow..]
        .find("        self.emit_temporal_month_day_string_reference_year(")
        .expect("parsed-year consumer call")
        + overflow;
    assert!(parse < overflow && overflow < consume);
}

#[test]
fn parsed_year_authority_has_one_recursive_owner_and_frozen_evidence() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, count) in [
        ("TemporalParsedMonthDayYear", 7),
        ("emit_temporal_parse_month_day_string", 4),
    ] {
        assert_eq!(
            MONTH_DAY_SOURCE.matches(name).count(),
            count,
            "owner `{name}`"
        );
        assert_eq!(
            count_in_rust_sources(&source_root, name),
            count,
            "recursive `{name}`"
        );
    }
    assert_eq!(
        MONTH_DAY_SOURCE
            .matches("        let parsed = self.emit_temporal_parse_month_day_string(")
            .count(),
        1
    );

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("owner-private `TemporalParsedMonthDayYear`"));
        assert!(
            evidence.contains("edd8d04d5cf6ec69edd44225d78506a09d49e857a028ad52071a39d78417a4be")
        );
        assert!(
            evidence.contains("a6f4eeae8728f7f922afac564ea96b845164c0115682f0821fabdb76d0cac6ff")
        );
        assert!(evidence.contains("no new Temporal behavior"));
    }
    assert!(CONTRACT.contains("does not close T22"));
}
