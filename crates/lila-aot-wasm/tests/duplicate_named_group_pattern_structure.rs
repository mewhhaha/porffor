use std::fs;
use std::path::Path;

const STRING_PARENT: &str = include_str!("../src/builtins/string.rs");
const DUPLICATE_NAMED_GROUP_PATTERN: &str =
    include_str!("../src/builtins/string/duplicate_named_group_pattern.rs");

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
fn duplicate_named_group_pattern_is_a_private_non_copyable_two_variant_domain() {
    let domain = bounded(
        DUPLICATE_NAMED_GROUP_PATTERN,
        "enum DuplicateNamedGroupPattern {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["AlternativeCaptures,", "IteratedBackreference,"]);
    let declaration_start = DUPLICATE_NAMED_GROUP_PATTERN
        .find("enum DuplicateNamedGroupPattern {")
        .expect("missing duplicate-named-group domain");
    let preceding_declaration = DUPLICATE_NAMED_GROUP_PATTERN[..declaration_start]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .expect("missing preceding declaration");
    assert_eq!(preceding_declaration.trim(), "use super::*;");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!DUPLICATE_NAMED_GROUP_PATTERN
            .contains(&format!("impl {capability} for DuplicateNamedGroupPattern")));
    }
    assert!(!DUPLICATE_NAMED_GROUP_PATTERN.contains("pub enum DuplicateNamedGroupPattern"));
    assert!(!DUPLICATE_NAMED_GROUP_PATTERN.contains("pub(crate) enum DuplicateNamedGroupPattern"));
    assert!(!DUPLICATE_NAMED_GROUP_PATTERN.contains("pub(super) enum DuplicateNamedGroupPattern"));
    assert_eq!(
        STRING_PARENT
            .matches("mod duplicate_named_group_pattern;")
            .count(),
        1
    );
    assert!(!STRING_PARENT.contains("DuplicateNamedGroupPattern"));
    assert!(!STRING_PARENT.contains("duplicate_named_group_pattern::"));
}

#[test]
fn duplicate_named_group_emitter_preserves_tables_indices_and_null_around_one_match() {
    let emitter = bounded(
        DUPLICATE_NAMED_GROUP_PATTERN,
        "    fn emit_string_match_duplicate_named_groups_from_string_locals(",
        "\n    }\n}",
    );
    let projection_offset = emitter
        .find("match &pattern {")
        .expect("missing pattern projection");
    let prelude = &emitter[..projection_offset];

    assert!(emitter.contains("pattern: DuplicateNamedGroupPattern,"));
    assert_eq!(emitter.matches("match &pattern {").count(), 1);
    assert!(prelude.contains("ValueKind::Null.tag()"));
    assert!(prelude.contains("Instruction::LocalSet(payload_local)"));
    assert!(prelude.contains("Instruction::LocalSet(tag_local)"));
    assert!(!prelude.contains("DuplicateNamedGroupPattern::"));
    for forbidden in [
        "iterated: bool",
        "if iterated",
        "if !iterated",
        "matches!(pattern",
        "_ =>",
        "unreachable!",
        "Default::default",
    ] {
        assert!(!emitter.contains(forbidden));
    }

    let alternatives = bounded(
        emitter,
        "DuplicateNamedGroupPattern::AlternativeCaptures => {",
        "            DuplicateNamedGroupPattern::IteratedBackreference => {",
    );
    let alternatives_normalized = without_whitespace(alternatives);
    assert!(alternatives_normalized.contains(
        "Instruction::I64Const(self.strings.payload(\"abc\")));function.instruction(&Instruction::LocalSet(candidate_local));self.emit_string_payload_equality_i32(string_local,candidate_local,function);function.instruction(&Instruction::If(BlockType::Empty));self.emit_string_match_duplicate_named_groups_result(string_local,\"abc\",3,&[(\"x\",Some(\"b\"),Some((1,2))),(\"y\",Some(\"a\"),Some((0,1))),(\"z\",Some(\"c\"),Some((2,3))),],has_indices_local,payload_local,tag_local,function,)?;"
    ));
    assert!(alternatives_normalized.contains(
        "Instruction::Else);function.instruction(&Instruction::I64Const(self.strings.payload(\"ad\")));function.instruction(&Instruction::LocalSet(candidate_local));self.emit_string_payload_equality_i32(string_local,candidate_local,function);function.instruction(&Instruction::If(BlockType::Empty));self.emit_string_match_duplicate_named_groups_result(string_local,\"ad\",2,&[(\"x\",Some(\"a\"),Some((0,1))),(\"y\",None,None),(\"z\",Some(\"d\"),Some((1,2))),],has_indices_local,payload_local,tag_local,function,)?;"
    ));
    assert_eq!(
        alternatives
            .matches("self.emit_string_match_duplicate_named_groups_result(")
            .count(),
        2
    );
    assert_eq!(alternatives.matches("has_indices_local,").count(), 2);

    let iterated = bounded(
        emitter,
        "DuplicateNamedGroupPattern::IteratedBackreference => {",
        "        }\n\n        self.release_temp_local(candidate_local);",
    );
    let iterated_normalized = without_whitespace(iterated);
    assert!(iterated_normalized.contains(
        "Instruction::I64Const(self.strings.payload(\"aac\")));function.instruction(&Instruction::LocalSet(candidate_local));self.emit_string_payload_equality_i32(string_local,candidate_local,function);function.instruction(&Instruction::If(BlockType::Empty));self.emit_string_match_duplicate_named_groups_result(string_local,\"aac\",3,&[(\"x\",None,None)],has_indices_local,payload_local,tag_local,function,)?;"
    ));
    assert_eq!(
        iterated
            .matches("self.emit_string_match_duplicate_named_groups_result(")
            .count(),
        1
    );
    assert_eq!(iterated.matches("has_indices_local,").count(), 1);

    assert_eq!(
        emitter
            .matches("self.emit_string_payload_equality_i32(")
            .count(),
        3
    );
    assert_eq!(
        emitter
            .matches("self.emit_string_match_duplicate_named_groups_result(")
            .count(),
        3
    );
}

#[test]
fn exactly_two_duplicate_named_group_producers_choose_their_patterns() {
    let parent_normalized = without_whitespace(STRING_PARENT);
    assert_eq!(
        parent_normalized
            .matches("self.emit_string_match_duplicate_named_group_alternative_captures(input_string_local,has_indices_local,self.result_local,self.result_tag_local,function,)?;")
            .count(),
        1
    );
    assert_eq!(
        parent_normalized
            .matches("self.emit_string_match_duplicate_named_group_iterated_backreference(input_string_local,has_indices_local,self.result_local,self.result_tag_local,function,)?;")
            .count(),
        1
    );
    assert!(!STRING_PARENT.contains("emit_string_match_duplicate_named_groups_from_string_locals("));

    let child_normalized = without_whitespace(DUPLICATE_NAMED_GROUP_PATTERN);
    assert_eq!(
        child_normalized
            .matches("self.emit_string_match_duplicate_named_groups_from_string_locals(string_local,DuplicateNamedGroupPattern::AlternativeCaptures,has_indices_local,payload_local,tag_local,function,)")
            .count(),
        1
    );
    assert_eq!(
        child_normalized
            .matches("self.emit_string_match_duplicate_named_groups_from_string_locals(string_local,DuplicateNamedGroupPattern::IteratedBackreference,has_indices_local,payload_local,tag_local,function,)")
            .count(),
        1
    );
    assert_eq!(
        DUPLICATE_NAMED_GROUP_PATTERN
            .matches("DuplicateNamedGroupPattern")
            .count(),
        6
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_match_duplicate_named_groups_from_string_locals(",
        ),
        3,
        "the private emitter definition and its two semantic wrapper calls must stay inventoried"
    );
}
