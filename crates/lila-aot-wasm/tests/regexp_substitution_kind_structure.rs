use std::fs;
use std::path::Path;

const STRING_PARENT: &str = include_str!("../src/builtins/string.rs");
const REGEXP_SUBSTITUTION: &str = include_str!("../src/builtins/string/regexp_substitution.rs");
const STRING_RECURSIVE: &str = concat!(
    include_str!("../src/builtins/string.rs"),
    include_str!("../src/builtins/string/regexp_substitution.rs")
);

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
fn regexp_substitution_kind_owns_six_private_rows_and_the_runtime_codes() {
    let domain = bounded(
        REGEXP_SUBSTITUTION,
        "enum RegExpSubstitutionKind {",
        "\n\nimpl RegExpSubstitutionKind {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "LiteralDollar,",
            "MatchedSubstring,",
            "Prefix,",
            "Suffix,",
            "NumberedCapture,",
            "NamedCapture,",
        ]
    );

    let declaration_start = REGEXP_SUBSTITUTION
        .find("enum RegExpSubstitutionKind {")
        .expect("missing substitution-kind domain");
    let preceding_declaration = REGEXP_SUBSTITUTION[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "use super::*;");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(
            !REGEXP_SUBSTITUTION.contains(&format!("impl {capability} for RegExpSubstitutionKind"))
        );
    }
    assert!(!REGEXP_SUBSTITUTION.contains("pub enum RegExpSubstitutionKind"));
    assert!(!REGEXP_SUBSTITUTION.contains("pub(crate) enum RegExpSubstitutionKind"));
    assert!(!REGEXP_SUBSTITUTION.contains("pub(super) enum RegExpSubstitutionKind"));
    assert_eq!(STRING_PARENT.matches("mod regexp_substitution;").count(), 1);
    assert!(!STRING_PARENT.contains("RegExpSubstitutionKind"));
    assert!(!STRING_PARENT.contains("regexp_substitution::"));
    assert_eq!(
        REGEXP_SUBSTITUTION
            .matches("RegExpSubstitutionKind")
            .count(),
        15
    );
    assert_eq!(
        STRING_RECURSIVE.matches("RegExpSubstitutionKind").count(),
        15
    );
    assert_eq!(
        STRING_PARENT
            .matches("self.emit_regexp_get_substitution(")
            .count(),
        1
    );

    let authority = without_whitespace(bounded(
        REGEXP_SUBSTITUTION,
        "impl RegExpSubstitutionKind {",
        "\n\nimpl<'a> FunctionBuilder<'a> {",
    ));
    assert!(authority.contains(
        "constALL:[Self;6]=[Self::LiteralDollar,Self::MatchedSubstring,Self::Prefix,Self::Suffix,Self::NumberedCapture,Self::NamedCapture,];"
    ));
    assert!(authority.contains("constfnruntime_code(&self)->i64{"));
    for mapping in [
        "Self::LiteralDollar=>1",
        "Self::MatchedSubstring=>2",
        "Self::Prefix=>3",
        "Self::Suffix=>4",
        "Self::NumberedCapture=>5",
        "Self::NamedCapture=>6",
    ] {
        assert_eq!(authority.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert!(!authority.contains("_=>"));
    assert!(!authority.contains("unreachable!"));
}

#[test]
fn recognizers_store_only_named_runtime_codes_and_keep_zero_as_the_sentinel() {
    let emitter = bounded(
        REGEXP_SUBSTITUTION,
        "    pub(super) fn emit_regexp_get_substitution(",
        "\n    }\n}",
    );
    let normalized = without_whitespace(emitter);

    for mapping in [
        "(b'$',RegExpSubstitutionKind::LiteralDollar)",
        "(b'&',RegExpSubstitutionKind::MatchedSubstring)",
        "(b'`',RegExpSubstitutionKind::Prefix)",
        "(b'\\'',RegExpSubstitutionKind::Suffix)",
    ] {
        assert_eq!(
            normalized.matches(mapping).count(),
            1,
            "byte row `{mapping}`"
        );
    }
    assert_eq!(normalized.matches("kind.runtime_code()").count(), 2);
    assert!(normalized.contains(
        "Instruction::I64Const(RegExpSubstitutionKind::NumberedCapture.runtime_code(),));function.instruction(&Instruction::LocalSet(substitution_kind_local))"
    ));
    assert!(normalized.contains(
        "Instruction::I64Const(RegExpSubstitutionKind::NamedCapture.runtime_code(),));function.instruction(&Instruction::LocalSet(substitution_kind_local))"
    ));
    assert_eq!(
        normalized
            .matches("Instruction::I64Const(0));function.instruction(&Instruction::LocalSet(substitution_kind_local))")
            .count(),
        1,
        "zero must remain the sole no-recognized-substitution sentinel"
    );
    for raw_code in 1..=6 {
        let raw_store = format!(
            "Instruction::I64Const({raw_code}));function.instruction(&Instruction::LocalSet(substitution_kind_local))"
        );
        assert!(!normalized.contains(&raw_store), "raw store `{raw_store}`");
    }
    assert!(!emitter.contains("for (byte, kind) in [(b'$', 1)"));
    assert!(!emitter.contains("Instruction::I64Const(kind)"));
}

#[test]
fn handler_walks_all_rows_and_matches_semantics_exhaustively_in_order() {
    let emitter = bounded(
        REGEXP_SUBSTITUTION,
        "    pub(super) fn emit_regexp_get_substitution(",
        "\n    }\n}",
    );
    let handler = bounded(
        emitter,
        "        for kind in RegExpSubstitutionKind::ALL {",
        "        function.instruction(&Instruction::LocalGet(replacement_index_local));\n        function.instruction(&Instruction::LocalGet(consumed_local));",
    );

    assert_eq!(handler.matches("kind.runtime_code()").count(), 1);
    assert_eq!(handler.matches("match &kind {").count(), 1);
    let mut previous = 0;
    for variant in [
        "LiteralDollar",
        "MatchedSubstring",
        "Prefix",
        "Suffix",
        "NumberedCapture",
        "NamedCapture",
    ] {
        let marker = format!("RegExpSubstitutionKind::{variant} => {{");
        assert_eq!(handler.matches(&marker).count(), 1, "handler `{variant}`");
        let offset = handler.find(&marker).expect("missing semantic arm");
        assert!(
            previous < offset,
            "semantic arm `{variant}` moved out of order"
        );
        previous = offset;
    }
    assert!(!handler.contains("for kind in 1..=6"));
    assert!(!handler.contains("match kind"));
    assert!(!handler.contains("_ =>"));
    assert!(!handler.contains("unreachable!"));

    let normalized = without_whitespace(handler);
    for semantic_anchor in [
        "RegExpSubstitutionKind::LiteralDollar=>{function.instruction(&Instruction::I64Const(self.strings.payload(\"$\")))",
        "RegExpSubstitutionKind::MatchedSubstring=>{function.instruction(&Instruction::LocalGet(match_string_local))",
        "RegExpSubstitutionKind::Prefix=>{self.emit_utf16_code_unit_range_payload_from_locals(input_string_local,zero_local,position_local,function,)?;",
        "RegExpSubstitutionKind::Suffix=>{function.instruction(&Instruction::LocalGet(position_local))",
        "RegExpSubstitutionKind::NumberedCapture=>{self.emit_index_to_flat_map_key_local(capture_index_local,number_payload_local,key_local,function,)?;",
        "RegExpSubstitutionKind::NamedCapture=>{function.instruction(&Instruction::LocalGet(replacement_index_local))",
    ] {
        assert!(normalized.contains(semantic_anchor), "semantic anchor `{semantic_anchor}`");
    }
}

#[test]
fn consumed_and_source_updates_remain_after_the_typed_handler() {
    let emitter = without_whitespace(bounded(
        REGEXP_SUBSTITUTION,
        "    pub(super) fn emit_regexp_get_substitution(",
        "\n    }\n}",
    ));

    assert!(emitter.contains(
        "Instruction::I64Const(2));function.instruction(&Instruction::LocalSet(consumed_local))"
    ));
    assert!(emitter.contains(
        "Instruction::I64Const(3));function.instruction(&Instruction::LocalSet(consumed_local))"
    ));
    assert!(emitter.contains(
        "Instruction::LocalGet(group_end_local));function.instruction(&Instruction::LocalGet(replacement_index_local));function.instruction(&Instruction::I64Sub);function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(consumed_local))"
    ));
    assert!(emitter.contains(
        "Instruction::LocalGet(replacement_index_local));function.instruction(&Instruction::LocalGet(consumed_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(replacement_index_local));function.instruction(&Instruction::LocalGet(replacement_index_local));function.instruction(&Instruction::LocalSet(literal_start_local))"
    ));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "RegExpSubstitutionKind"),
        15,
        "the domain, projections and every recognizer/handler use must stay inventoried"
    );
    assert_eq!(
        REGEXP_SUBSTITUTION.matches(".runtime_code()").count(),
        4,
        "all runtime code writes and comparisons must use the one projection"
    );
}
