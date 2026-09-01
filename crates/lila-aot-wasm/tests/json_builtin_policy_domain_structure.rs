use std::fs;
use std::path::Path;

const JSON_SOURCE: &str = include_str!("../src/builtins/json.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/json-builtin-policy-domain.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

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
fn json_builtin_domain_is_exact_and_capability_free() {
    let declaration = JSON_SOURCE
        .split_once("enum JsonBuiltin {")
        .expect("JsonBuiltin declaration")
        .1
        .split_once('}')
        .expect("JsonBuiltin declaration end")
        .0;
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Parse,", "Stringify,", "RawJson,", "IsRawJson,"]
    );

    let prelude = JSON_SOURCE
        .split_once("mod json_stringify_replacer_invocation {")
        .expect("JSON declaration prelude")
        .0;
    assert!(!prelude.contains("#[derive"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "JsonBuiltin"), 10);
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, &format!("impl {capability} for JsonBuiltin")),
            0,
            "JsonBuiltin must not implement {capability}"
        );
    }
}

#[test]
fn standard_dispatch_can_only_call_four_fixed_json_operations() {
    assert_eq!(STANDARD_SOURCE.matches("JsonBuiltin").count(), 0);
    assert_eq!(STANDARD_SOURCE.matches("emit_json_builtin(").count(), 0);

    for (builtin, wrapper, variant, end_marker) in [
        (
            "JsonParse",
            "emit_json_parse_builtin",
            "Parse",
            "    pub(super) fn emit_json_stringify_builtin(",
        ),
        (
            "JsonStringify",
            "emit_json_stringify_builtin",
            "Stringify",
            "    pub(super) fn emit_json_raw_json_builtin(",
        ),
        (
            "JsonRawJson",
            "emit_json_raw_json_builtin",
            "RawJson",
            "    pub(super) fn emit_json_is_raw_json_builtin(",
        ),
        (
            "JsonIsRawJson",
            "emit_json_is_raw_json_builtin",
            "IsRawJson",
            "    /// Applies the result of a completed reviver call.",
        ),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!(
                    "StandardBuiltinId::{builtin} => self.{wrapper}(function)?"
                ))
                .count(),
            1,
            "standard dispatcher route `{builtin}`"
        );
        assert_eq!(
            JSON_SOURCE
                .matches(&format!("pub(super) fn {wrapper}("))
                .count(),
            1
        );

        let wrapper_body = bounded(
            JSON_SOURCE,
            &format!("    pub(super) fn {wrapper}("),
            end_marker,
        );
        assert_eq!(wrapper_body.matches("self.emit_json_builtin(").count(), 1);
        assert_eq!(
            wrapper_body
                .matches(&format!("JsonBuiltin::{variant}"))
                .count(),
            1
        );
        assert!(!wrapper_body.contains("Instruction::"));
    }
    assert_eq!(JSON_SOURCE.matches("emit_json_builtin(").count(), 5);
}

#[test]
fn json_builtin_selection_has_one_owned_exhaustive_consumer() {
    let dispatcher = JSON_SOURCE
        .split_once("    fn emit_json_builtin(")
        .expect("JSON builtin dispatcher")
        .1
        .split_once("    pub(super) fn emit_json_parse_builtin(")
        .expect("JSON builtin dispatcher end")
        .0;
    assert!(dispatcher.contains("builtin: JsonBuiltin,"));
    assert_eq!(dispatcher.matches("match builtin").count(), 1);
    for variant in ["Parse", "Stringify", "RawJson", "IsRawJson"] {
        assert_eq!(
            dispatcher
                .matches(&format!("JsonBuiltin::{variant}"))
                .count(),
            1,
            "JSON builtin consumer `JsonBuiltin::{variant}`"
        );
    }
    for forbidden in [
        "builtin ==",
        "builtin !=",
        "_ =>",
        "unreachable!",
        "debug_assert!",
        ".clone()",
    ] {
        assert!(
            !dispatcher.contains(forbidden),
            "JSON builtin policy contains `{forbidden}`"
        );
    }
}

#[test]
fn contract_and_task_record_the_json_builtin_ownership_boundary() {
    for evidence in [CONTRACT, TASK] {
        let words = evidence.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("capability-free `JsonBuiltin`"));
        assert!(words.contains("Batch AJ"));
        assert!(words.contains("Batch AO"));
        assert!(words.contains("private fixed semantic wrappers"));
        assert!(words.contains("no new JSON behavior"));
    }
}
