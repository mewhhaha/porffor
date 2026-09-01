const DATE_SOURCE: &str = include_str!("../src/builtins/date.rs");
const DATE_STRING_PARSE_SOURCE: &str = include_str!("../src/builtins/date/date_string_parse.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_DATE_SOURCE: &str = include_str!("../../lila-cli/tests/cli/date.rs");
const DATE_PARSE_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_date_parse.js");
const GOLDEN_SOURCE: &str = include_str!("emit_golden.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn ordered(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let next = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: {marker}"));
        cursor += next + marker.len();
    }
}

#[test]
fn date_string_parse_has_one_private_file_owner_and_exact_visibility() {
    assert_eq!(DATE_SOURCE.matches("\nmod date_string_parse;\n").count(), 1);
    assert!(!DATE_SOURCE.contains("\npub mod date_string_parse;\n"));
    assert!(!DATE_SOURCE.contains("\npub(crate) mod date_string_parse;\n"));
    assert!(!DATE_SOURCE.contains("\nmod date_string_parse {\n"));
    assert!(DATE_STRING_PARSE_SOURCE.starts_with("use super::*;\n\n"));

    let expected_methods = [
        "fn emit_date_iso_expect_byte(",
        "fn emit_date_iso_decimal(",
        "pub(crate) fn emit_date_parse_iso_string(",
        "pub(crate) fn emit_date_parse_string(",
    ];
    assert_eq!(
        DATE_STRING_PARSE_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("fn ") || line.starts_with("pub(crate) fn "))
            .collect::<Vec<_>>(),
        expected_methods
    );
    for method in expected_methods {
        assert_eq!(DATE_STRING_PARSE_SOURCE.matches(method).count(), 1);
        assert!(
            !DATE_SOURCE.contains(method),
            "parent retained parser method `{method}`"
        );
    }
    assert_eq!(
        DATE_STRING_PARSE_SOURCE.matches("pub(crate) fn ").count(),
        2
    );
    assert!(!DATE_STRING_PARSE_SOURCE.contains("pub(super) fn "));
    assert!(!DATE_STRING_PARSE_SOURCE.contains("pub fn "));
}

#[test]
fn date_string_parse_internal_and_external_call_maps_are_closed() {
    assert_eq!(
        DATE_STRING_PARSE_SOURCE
            .matches("self.emit_date_iso_expect_byte(")
            .count(),
        8
    );
    assert_eq!(
        DATE_STRING_PARSE_SOURCE
            .matches("self.emit_date_iso_decimal(")
            .count(),
        10
    );
    assert_eq!(
        DATE_STRING_PARSE_SOURCE
            .matches("self.emit_date_parse_iso_string(")
            .count(),
        1
    );
    assert_eq!(
        DATE_STRING_PARSE_SOURCE
            .matches("self.emit_date_parse_string(")
            .count(),
        0
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("self.emit_date_parse_string(")
            .count(),
        2
    );
    assert!(!STANDARD_SOURCE.contains("emit_date_parse_iso_string("));
    assert!(!DATE_SOURCE.contains("emit_date_parse_string("));
    assert!(!DATE_SOURCE.contains("emit_date_parse_iso_string("));

    let date_parse = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::DateParse => {",
        "StandardBuiltinId::DateUtc => {",
    );
    assert_eq!(
        date_parse.matches("self.emit_date_parse_string(").count(),
        1
    );
    ordered(
        date_parse,
        &[
            "self.emit_value_to_string_payload(",
            "self.emit_return_current_completion_if_throw(function);",
            "self.emit_date_parse_string(",
            "ValueKind::Number.tag() as i64",
        ],
    );

    let constructor = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::DateConstructor => {",
        "StandardBuiltinId::DatePrototypeGetTime | StandardBuiltinId::DatePrototypeValueOf",
    );
    assert_eq!(
        constructor.matches("self.emit_date_parse_string(").count(),
        1
    );
    let string_parse = constructor
        .find("self.emit_date_parse_string(")
        .expect("Date constructor string parse should exist");
    assert!(
        constructor[..string_parse]
            .rfind("ValueKind::String.tag() as i64")
            .is_some(),
        "Date constructor must select the string route before parsing"
    );
    assert!(
        constructor[string_parse..]
            .find("self.emit_value_to_number_payload(")
            .is_some(),
        "the non-string constructor route must remain numeric"
    );
}

#[test]
fn date_parse_fixture_and_golden_corpus_cover_the_owned_domain() {
    assert_eq!(
        CLI_DATE_SOURCE
            .matches("fn run_wasm_backend_succeeds_for_date_parse_fixture()")
            .count(),
        1
    );
    assert_eq!(CLI_DATE_SOURCE.matches("wasm_date_parse.js").count(), 1);
    for witness in [
        "Date.parse(\"-271821-04-20T00:00:00.000Z\")",
        "Date.parse(\"+275760-09-13T00:00:00.000Z\")",
        "Date.parse(\"1970-01-01\")",
        "Date.parse(\"1970-01-01T00:00:00\")",
        "Date.parse(\"1970-01-01T01:00:00+01:00\")",
        "Date.parse(\"-000000-03-31T00:45Z\")",
        "Date.parse(epoch.toString())",
        "Date.parse(epoch.toUTCString())",
        "Date.parse(epoch.toISOString())",
    ] {
        assert!(
            DATE_PARSE_FIXTURE.contains(witness),
            "Date parse fixture must retain `{witness}`"
        );
    }

    assert!(GOLDEN_SOURCE.contains(".join(\"../lila-cli/tests/fixtures\")"));
    assert!(GOLDEN_SOURCE.contains("path.extension().is_some_and(|ext| ext == \"js\")"));
}
