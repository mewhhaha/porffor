use std::fs;
use std::path::Path;

const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const STRING: &str = include_str!("../src/builtins/string.rs");
const SYMBOL_METHOD: &str = include_str!("../src/builtins/string/symbol_method.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/string-symbol-hook-operation.md");
const TASK: &str = include_str!("../../../tasks/18-strings-unicode.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
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
fn string_symbol_hook_operation_is_the_five_row_non_copyable_shared_domain() {
    let domain = bounded(
        STRING,
        "enum StringSymbolHookOperation {",
        "\n\nenum RegExpFlagGetter {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(
        variants,
        ["Match,", "MatchAll,", "Replace,", "ReplaceAll,", "Search,"]
    );
    assert!(!domain.contains("Split"));
    let declaration_start = STRING
        .find("enum StringSymbolHookOperation {")
        .expect("missing String symbol-hook operation domain");
    let preceding_declaration = STRING[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "}");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!STRING.contains(&format!("impl {capability} for StringSymbolHookOperation")));
    }
    assert!(!STRING.contains("pub enum StringSymbolHookOperation"));
    assert!(!STRING.contains("pub(crate) enum StringSymbolHookOperation"));
    assert!(!STRING.contains("pub(super) enum StringSymbolHookOperation"));
}

#[test]
fn symbol_hook_emitter_keeps_closed_policies_and_consumes_one_optional_method() {
    let emitter = bounded(
        STRING,
        "    fn emit_string_symbol_hook_builtin(",
        "    pub(crate) fn emit_string_validate_regexp_global_flags(",
    );
    assert!(emitter.contains("operation: StringSymbolHookOperation,"));
    for forbidden in [
        "builtin: StandardBuiltinId",
        "StandardBuiltinId::StringPrototype",
        "passes_second_arg",
        ": bool",
        "matches!(operation",
        "operation ==",
        "operation !=",
        "_ =>",
        "unreachable!",
        "emit_object_own_property_present(",
        "emit_ordinary_get_prototype_of(",
        "emit_function_handle_call(",
    ] {
        assert!(!emitter.contains(forbidden), "forbidden `{forbidden}`");
    }
    let object_gate = emitter
        .find("self.emit_is_heap_object_like_tag_i32(")
        .unwrap();
    let flags = emitter
        .find("self.emit_string_validate_regexp_global_flags(")
        .unwrap();
    let method = emitter
        .find("NullableStringSymbolMethod::get_method(")
        .unwrap();
    let dispatch = emitter.find("method.call_or_fallback(").unwrap();
    assert!(object_gate < flags && flags < method && method < dispatch);
    assert!(emitter
        .contains("StringSymbolHookOperation::MatchAll | StringSymbolHookOperation::ReplaceAll"));

    let split = bounded(
        STRING,
        "    pub(crate) fn emit_string_split_builtin(",
        "    fn emit_string_symbol_hook_fallback(",
    );
    assert!(split.contains("self.emit_is_heap_object_like_tag_i32("));
    assert!(split.contains("StringSymbolMethodKey::Split"));
    assert!(split.contains("NullableStringSymbolMethod::get_method("));
    assert!(split.contains("method.call_or_fallback("));
    assert!(!split.contains("emit_function_handle_call("));

    // The observed receiver/method record remains private, and a required Invoke
    // cannot choose the nullable protocol's fallback operation.
    let observed = bounded(
        SYMBOL_METHOD,
        "struct ObservedStringSymbolMethod {",
        "impl ObservedStringSymbolMethod {",
    );
    assert!(!observed.contains("pub"));
    assert!(!SYMBOL_METHOD.contains("derive(Clone"));
    assert!(!SYMBOL_METHOD.contains("derive(Copy"));
    assert!(SYMBOL_METHOD.contains("emit_is_callable_i32("));
    assert!(SYMBOL_METHOD.contains("emit_function_or_proxy_call_leave_throw_completion("));
    assert!(SYMBOL_METHOD.contains("emit_propagate_throw_from_locals_if_needed("));
    assert!(SYMBOL_METHOD.contains("emit_throw_current_function_realm_type_error("));
    let required = bounded(
        SYMBOL_METHOD,
        "impl RequiredStringSymbolMethod {",
        "// RegExpCreate uses",
    );
    assert!(!required.contains("fallback"));
    assert!(required.contains("pub(super) fn invoke(\n        self,"));
}

#[test]
fn private_fallback_matches_all_five_operations_to_their_exact_algorithms() {
    let fallback = bounded(
        STRING,
        "    fn emit_string_symbol_hook_fallback(",
        "    pub(crate) fn emit_string_search_regexp_fallback_from_string_locals(",
    );
    let normalized = without_whitespace(fallback);

    assert!(fallback.contains("operation: &StringSymbolHookOperation,"));
    assert_eq!(fallback.matches("match operation {").count(), 1);
    for variant in ["Match", "MatchAll", "Replace", "ReplaceAll", "Search"] {
        let arm = format!("StringSymbolHookOperation::{variant} => {{");
        assert_eq!(
            fallback.matches(&arm).count(),
            1,
            "fallback arm `{variant}`"
        );
    }
    for semantic in [
        "StringSymbolHookOperation::Match=>{self.emit_string_match_literal_fallback_from_string_locals(",
        "StringSymbolHookOperation::MatchAll=>{self.emit_string_match_all_literal_fallback_from_string_locals(",
        "StringSymbolHookOperation::Replace=>{self.emit_string_replace_literal_first_occurrence_from_string_locals(",
        "StringSymbolHookOperation::ReplaceAll=>{self.emit_string_replace_literal_all_occurrences_from_string_locals(",
        "StringSymbolHookOperation::Search=>{self.emit_string_search_regexp_fallback_from_string_locals(",
    ] {
        assert!(normalized.contains(semantic), "fallback semantic `{semantic}`");
    }
    for forbidden in [
        "StringSymbolHookOperation::Split",
        "StandardBuiltinId",
        "matches!(operation",
        "_ =>",
        "unreachable!",
        "RegExp/string fallback is unsupported",
    ] {
        assert!(!fallback.contains(forbidden), "forbidden `{forbidden}`");
    }
}

#[test]
fn standard_dispatch_names_five_operations_and_routes_split_directly() {
    assert!(!STANDARD.contains("StringSymbolHookOperation"));
    assert!(!STANDARD.contains("emit_string_symbol_hook_builtin("));
    let dispatch_start = STANDARD
        .find("            StandardBuiltinId::StringPrototypeMatch => {")
        .expect("missing first String symbol-hook producer");
    let dispatch_end = STANDARD[dispatch_start..]
        .find("            StandardBuiltinId::RegExpConstructor => {")
        .map(|offset| dispatch_start + offset)
        .expect("missing end of String symbol-hook dispatch");
    let dispatch = &STANDARD[dispatch_start..dispatch_end];
    let normalized = without_whitespace(dispatch).replace(",)", ")");

    for (builtin, entry, variant) in [
        ("Match", "match", "Match"),
        ("MatchAll", "match_all", "MatchAll"),
        ("Replace", "replace", "Replace"),
        ("ReplaceAll", "replace_all", "ReplaceAll"),
        ("Search", "search", "Search"),
    ] {
        let producer = format!(
            "StandardBuiltinId::StringPrototype{builtin}=>{{self.emit_string_{entry}_builtin(function)?;}}"
        );
        assert_eq!(
            normalized.matches(&producer).count(),
            1,
            "producer `{builtin}`"
        );
        assert_eq!(
            STRING
                .matches(&format!(
                    "self.emit_string_symbol_hook_builtin(StringSymbolHookOperation::{variant}, function)"
                ))
                .count(),
            1,
            "fixed entry `{entry}`"
        );
    }
    assert!(normalized.contains(
        "StandardBuiltinId::StringPrototypeSplit=>{self.emit_string_split_builtin(function)?;}"
    ));
    assert_eq!(dispatch.matches("emit_string_").count(), 6);
    assert_eq!(dispatch.matches("emit_string_split_builtin(").count(), 1);
    assert!(!dispatch.contains("| StandardBuiltinId::StringPrototype"));
    assert!(!dispatch.contains("emit_string_symbol_hook_builtin(builtin"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_string_symbol_hook_builtin("),
        6,
        "the typed emitter definition and exactly five calls must stay inventoried"
    );
}

#[test]
fn contract_and_task_record_the_private_dispatcher_boundary() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("Batch AY"));
        assert!(evidence.contains("five fixed String symbol-hook entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new String behavior"));
    }
}
