use std::fs;
use std::path::Path;

const DATE_PARENT_SOURCE: &str = include_str!("../src/builtins/date.rs");
const DATE_LOCAL_STRING_SOURCE: &str = include_str!("../src/builtins/date/local_string.rs");
const DATE_RECURSIVE_SOURCE: &str = concat!(
    include_str!("../src/builtins/date.rs"),
    include_str!("../src/builtins/date/local_string.rs")
);
const CLI_DATE_TESTS: &str = include_str!("../../lila-cli/tests/cli/date.rs");
const CLI_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_date_locale_strings.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/date-current-time-source.md");
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

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
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
fn date_local_string_format_is_the_exact_private_no_capability_domain() {
    let declaration_region = bounded(
        DATE_LOCAL_STRING_SOURCE,
        concat!(
            "enum DateTimeValueSource {\n",
            "    ReceiverSlot { payload_local: u32, tag_local: u32 },\n",
            "    RealmHostClock,\n",
            "}\n\n",
        ),
        "impl<'a> FunctionBuilder<'a>",
    );
    assert_eq!(
        normalized(declaration_region),
        "enumDateLocalStringFormat{Date,Time,DateAndTime,}"
    );
    assert!(!declaration_region.contains("#["));
    assert!(!declaration_region.contains("pub enum DateLocalStringFormat"));
    assert!(!declaration_region.contains("pub(crate) enum DateLocalStringFormat"));
    assert!(!DATE_RECURSIVE_SOURCE.contains("impl DateLocalStringFormat"));
    assert_eq!(DATE_PARENT_SOURCE.matches("mod local_string;").count(), 1);
    assert!(!DATE_PARENT_SOURCE.contains("DateLocalStringFormat"));
    assert!(!DATE_PARENT_SOURCE.contains("local_string::"));
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches("DateLocalStringFormat")
            .count(),
        9
    );
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches("emit_date_local_string(")
            .count(),
        5
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "DateLocalStringFormat"),
        9,
        "the declaration, typed consumer, three exhaustive arms and four producers own every mention"
    );
}

#[test]
fn date_time_value_source_is_the_exact_private_no_capability_domain() {
    let declaration_region = bounded(
        DATE_LOCAL_STRING_SOURCE,
        "use super::*;",
        "enum DateLocalStringFormat",
    );
    assert_eq!(
        normalized(declaration_region),
        concat!(
            "enumDateTimeValueSource{",
            "ReceiverSlot{payload_local:u32,tag_local:u32},",
            "RealmHostClock,}"
        )
    );
    assert!(!declaration_region.contains("#["));
    assert!(!declaration_region.contains("pub enum DateTimeValueSource"));
    assert!(!declaration_region.contains("pub(crate) enum DateTimeValueSource"));
    assert!(!DATE_PARENT_SOURCE.contains("DateTimeValueSource"));
    assert!(!DATE_PARENT_SOURCE.contains("emit_date_time_value_from_source("));
    assert!(!DATE_PARENT_SOURCE.contains(".wall_clock_millis_import_function_index()"));
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches("DateTimeValueSource")
            .count(),
        10
    );
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches("DateTimeValueSource::")
            .count(),
        7
    );
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches("emit_date_time_value_from_source(")
            .count(),
        3
    );
    assert_eq!(
        DATE_LOCAL_STRING_SOURCE
            .matches(".wall_clock_millis_import_function_index()")
            .count(),
        1
    );

    let normalized_source = normalized(DATE_RECURSIVE_SOURCE);
    for forbidden in [
        "implDateTimeValueSource",
        "forDateTimeValueSource",
        "DateTimeValueSource==",
        "DateTimeValueSource!=",
        "matches!(source",
    ] {
        assert!(
            !normalized_source.contains(forbidden),
            "found forbidden Date time-value source capability `{forbidden}`"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "DateTimeValueSource"),
        10,
        "the declaration, two typed parameters, two exhaustive arms and five producers own every mention"
    );
}

#[test]
fn date_time_value_source_exhaustively_selects_receiver_or_clock() {
    let consumer = normalized(bounded(
        DATE_LOCAL_STRING_SOURCE,
        "    fn emit_date_time_value_from_source(",
        "    pub(crate) fn emit_date_current_time_payload(",
    ));
    assert_eq!(
        consumer,
        concat!(
            "&mutself,source:DateTimeValueSource,dest_payload_local:u32,",
            "function:&mutFunction,)->Result<(),EmitError>{matchsource{",
            "DateTimeValueSource::ReceiverSlot{payload_local,tag_local,}=>{",
            "self.emit_date_value_payload(payload_local,tag_local,dest_payload_local,function)}",
            "DateTimeValueSource::RealmHostClock=>{",
            "letwall_clock_millis_import_function_index=self.functions.",
            "wall_clock_millis_import_function_index().ok_or_else(||{",
            "EmitError::unsupported(\"Datecurrenttimerequiresthelila_host.wall_clock_millisimport\",)})?;",
            "function.instruction(&Instruction::Call(wall_clock_millis_import_function_index));",
            "function.instruction(&Instruction::I64ReinterpretF64);",
            "function.instruction(&Instruction::LocalSet(dest_payload_local));Ok(())}}}"
        )
    );
    assert_eq!(consumer.matches("matchsource{").count(), 1);
    assert!(!consumer.contains("_=>"));
}

#[test]
fn exactly_five_date_time_value_producers_select_their_source() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "DateTimeValueSource::"),
        7,
        "two exhaustive arms and five typed producers own every qualified mention"
    );

    for (source, start, end, expected_call) in [
        (
            DATE_LOCAL_STRING_SOURCE,
            "    pub(crate) fn emit_date_current_time_payload(",
            "    pub(crate) fn emit_date_function_call(",
            "self.emit_date_time_value_from_source(DateTimeValueSource::RealmHostClock,dest_payload_local,function,)",
        ),
        (
            DATE_LOCAL_STRING_SOURCE,
            "    pub(crate) fn emit_date_function_call(",
            "    fn emit_date_local_string(",
            "self.emit_date_local_string(DateTimeValueSource::RealmHostClock,DateLocalStringFormat::DateAndTime,function,)",
        ),
        (
            DATE_LOCAL_STRING_SOURCE,
            "    pub(crate) fn emit_date_to_date_string(",
            "    pub(crate) fn emit_date_to_time_string(",
            "self.emit_date_local_string(DateTimeValueSource::ReceiverSlot{payload_local:self.this_payload_local.unwrap(),tag_local:self.this_tag_local.unwrap(),},DateLocalStringFormat::Date,function,)",
        ),
        (
            DATE_LOCAL_STRING_SOURCE,
            "    pub(crate) fn emit_date_to_time_string(",
            "    pub(crate) fn emit_date_to_string(",
            "self.emit_date_local_string(DateTimeValueSource::ReceiverSlot{payload_local:self.this_payload_local.unwrap(),tag_local:self.this_tag_local.unwrap(),},DateLocalStringFormat::Time,function,)",
        ),
        (
            DATE_LOCAL_STRING_SOURCE,
            "    pub(crate) fn emit_date_to_string(",
            "\n    }\n}",
            "self.emit_date_local_string(DateTimeValueSource::ReceiverSlot{payload_local:self.this_payload_local.unwrap(),tag_local:self.this_tag_local.unwrap(),},DateLocalStringFormat::DateAndTime,function,)",
        ),
    ] {
        let producer = normalized(bounded(source, start, end));
        assert_eq!(
            producer.matches(expected_call).count(),
            1,
            "missing exact Date time-value source call `{expected_call}`"
        );
        assert_eq!(producer.matches("DateTimeValueSource::").count(), 1);
    }
}

#[test]
fn date_and_time_inclusion_are_one_exhaustive_projection() {
    let consumer = bounded(
        DATE_LOCAL_STRING_SOURCE,
        "    fn emit_date_local_string(",
        "    pub(crate) fn emit_date_to_date_string(",
    );
    let projection = normalized(bounded(
        consumer,
        ") -> Result<(), EmitError> {",
        "let time_payload_local = self.reserve_temp_local();",
    ));
    assert_eq!(
        projection,
        concat!(
            "let(includes_date,includes_time)=matchformat{",
            "DateLocalStringFormat::Date=>(true,false),",
            "DateLocalStringFormat::Time=>(false,true),",
            "DateLocalStringFormat::DateAndTime=>(true,true),};"
        )
    );
    assert_eq!(consumer.matches("match format {").count(), 1);
    assert_eq!(consumer.matches("if includes_date {").count(), 1);
    assert_eq!(
        consumer
            .matches("if includes_date && includes_time {")
            .count(),
        1
    );
    assert_eq!(consumer.matches("if includes_time {").count(), 1);
    for forbidden in [
        "_ =>",
        "matches!",
        "format ==",
        "format !=",
        ".includes_date()",
        ".includes_time()",
    ] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }
    assert_before(consumer, "match format {", "if includes_date {");
    assert_before(
        consumer,
        "if includes_date {",
        "if includes_date && includes_time {",
    );
    assert_before(
        consumer,
        "if includes_date && includes_time {",
        "if includes_time {",
    );
}

#[test]
fn exactly_four_date_surfaces_choose_their_string_format() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_date_local_string("),
        5,
        "the definition and four Date surfaces are the complete call census"
    );

    for (start, end, format) in [
        (
            "    pub(crate) fn emit_date_function_call(",
            "    fn emit_date_local_string(",
            "DateAndTime",
        ),
        (
            "    pub(crate) fn emit_date_to_date_string(",
            "    pub(crate) fn emit_date_to_time_string(",
            "Date",
        ),
        (
            "    pub(crate) fn emit_date_to_time_string(",
            "    pub(crate) fn emit_date_to_string(",
            "Time",
        ),
        (
            "    pub(crate) fn emit_date_to_string(",
            "\n    }\n}",
            "DateAndTime",
        ),
    ] {
        let producer = bounded(DATE_LOCAL_STRING_SOURCE, start, end);
        assert_eq!(producer.matches("self.emit_date_local_string(").count(), 1);
        assert_eq!(producer.matches("DateLocalStringFormat::").count(), 1);
        assert_eq!(
            producer
                .matches(&format!("DateLocalStringFormat::{format},"))
                .count(),
            1
        );
    }
}

#[test]
fn contract_and_existing_cli_witness_pin_all_three_formats() {
    assert!(CONTRACT.contains("DateTimeValueSource"));
    assert!(CONTRACT.contains("DateLocalStringFormat"));
    assert!(
        CONTRACT.contains("cargo test -p lila-aot-wasm --test date_local_string_format_structure")
    );
    assert!(CONTRACT.contains(
        "tests::wasm_backend_uses_one_injected_clock_for_date_temporal_and_monotonic_reads"
    ));
    assert!(TASK.contains("DateTimeValueSource"));
    assert!(TASK.contains("DateLocalStringFormat"));
    assert!(
        CLI_DATE_TESTS.contains("fn run_wasm_backend_succeeds_for_date_locale_strings_fixture()")
    );
    for marker in [
        "date.toDateString()",
        "date.toString()",
        "date.toTimeString()",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
