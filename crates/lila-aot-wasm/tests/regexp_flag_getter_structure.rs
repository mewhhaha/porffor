use std::fs;
use std::path::Path;

const STANDARD: &str = include_str!("../src/builtins/standard.rs");
const STRING: &str = include_str!("../src/builtins/string.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/regexp-flag-getter.md");
const TASK: &str = include_str!("../../../tasks/19-regexp.md");

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
fn regexp_flag_getter_is_an_eight_row_sibling_visible_non_copyable_domain() {
    let domain = bounded(
        STRING,
        "enum RegExpFlagGetter {",
        "\n\nmod duplicate_named_group_pattern;",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(
        variants,
        [
            "HasIndices,",
            "Global,",
            "IgnoreCase,",
            "Multiline,",
            "DotAll,",
            "Unicode,",
            "UnicodeSets,",
            "Sticky,",
        ]
    );
    let declaration_start = STRING
        .find("enum RegExpFlagGetter {")
        .expect("missing RegExp flag-getter domain");
    let preceding_declaration = STRING[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "}");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!STRING.contains(&format!("impl {capability} for RegExpFlagGetter")));
    }
    assert!(!STRING.contains("pub enum RegExpFlagGetter"));
    assert!(!STRING.contains("pub(crate) enum RegExpFlagGetter"));
    assert!(!STRING.contains("pub(super) enum RegExpFlagGetter"));
}

#[test]
fn regexp_flag_getter_helper_projects_all_bytes_once_before_the_shared_algorithm() {
    let emitter = bounded(
        STRING,
        "    fn emit_regexp_prototype_flag_getter_builtin(",
        "    fn emit_regexp_builtin_exec_update_last_index_after_compact_match(",
    );

    assert!(emitter.contains("getter: RegExpFlagGetter,"));
    assert!(!emitter.contains("builtin: StandardBuiltinId"));
    assert_eq!(emitter.matches("match &getter {").count(), 1);
    let normalized = without_whitespace(emitter);
    for projection in [
        "RegExpFlagGetter::HasIndices=>b'd'",
        "RegExpFlagGetter::Global=>b'g'",
        "RegExpFlagGetter::IgnoreCase=>b'i'",
        "RegExpFlagGetter::Multiline=>b'm'",
        "RegExpFlagGetter::DotAll=>b's'",
        "RegExpFlagGetter::Unicode=>b'u'",
        "RegExpFlagGetter::UnicodeSets=>b'v'",
        "RegExpFlagGetter::Sticky=>b'y'",
    ] {
        assert_eq!(
            normalized.matches(projection).count(),
            1,
            "projection `{projection}`"
        );
    }
    for forbidden in [
        ": bool",
        "StandardBuiltinId::RegExpPrototype",
        "getter ==",
        "getter !=",
        "matches!(getter",
        "_ =>",
        "unreachable!",
        "Default::default",
        "invalid RegExp flag getter",
    ] {
        assert!(!emitter.contains(forbidden), "forbidden `{forbidden}`");
    }

    let projection_end = emitter
        .find("        };\n        let receiver_payload_local")
        .expect("missing end of flag-byte projection");
    let shared = &emitter[projection_end..];
    for anchor in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET",
        "REGEXP_PROTOTYPE_GLOBAL_INDEX",
        "emit_require_regexp_internal_slots",
        "HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET",
        "emit_string_payload_contains_ascii_byte_i32",
        "ValueKind::Boolean.tag()",
    ] {
        assert!(
            shared.contains(anchor),
            "shared algorithm anchor `{anchor}`"
        );
    }
}

#[test]
fn standard_dispatch_has_exactly_eight_named_flag_getter_producers() {
    assert!(!STANDARD.contains("RegExpFlagGetter"));
    assert!(!STANDARD.contains("emit_regexp_prototype_flag_getter_builtin("));
    let dispatch = bounded(
        STANDARD,
        "            StandardBuiltinId::RegExpPrototypeSourceGetter => {",
        "            StandardBuiltinId::RegExpPrototypeSymbolMatch => {",
    );
    let normalized = without_whitespace(dispatch).replace(",)", ")");

    for (builtin, entry, variant) in [
        ("HasIndices", "has_indices", "HasIndices"),
        ("Global", "global", "Global"),
        ("IgnoreCase", "ignore_case", "IgnoreCase"),
        ("Multiline", "multiline", "Multiline"),
        ("DotAll", "dot_all", "DotAll"),
        ("Unicode", "unicode", "Unicode"),
        ("UnicodeSets", "unicode_sets", "UnicodeSets"),
        ("Sticky", "sticky", "Sticky"),
    ] {
        let producer = format!(
            "StandardBuiltinId::RegExpPrototype{builtin}Getter=>{{self.emit_regexp_prototype_{entry}_getter_builtin(function)?;}}"
        );
        assert_eq!(
            normalized.matches(&producer).count(),
            1,
            "producer `{builtin}`"
        );
        assert_eq!(
            STRING
                .matches(&format!(
                    "self.emit_regexp_prototype_flag_getter_builtin(RegExpFlagGetter::{variant}, function)"
                ))
                .count(),
            1,
            "fixed entry `{entry}`"
        );
    }
    assert_eq!(dispatch.matches("_getter_builtin(").count(), 9);
    assert!(!dispatch.contains("| StandardBuiltinId::RegExpPrototype"));
    assert!(!dispatch.contains("emit_regexp_prototype_flag_getter_builtin(builtin"));
    assert!(!dispatch.contains("_ =>"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpFlagGetter"),
        18,
        "the private domain, byte projections and eight fixed producers must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_regexp_prototype_flag_getter_builtin(",),
        9,
        "the helper definition and exactly eight calls must stay inventoried"
    );
}

#[test]
fn contract_and_task_record_the_private_dispatcher_boundary() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("Batch AZ"));
        assert!(evidence.contains("eight fixed RegExp flag-getter entries"));
        assert!(evidence.contains("source-equivalent"));
        assert!(evidence.contains("no new RegExp behavior"));
    }
}
