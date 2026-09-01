use std::fs;
use std::path::Path;

const STRING: &str = include_str!("../src/builtins/string.rs");
const STRING_LITERAL_REPLACEMENT_SCOPE: &str =
    include_str!("../src/builtins/string/string_literal_replacement_scope.rs");

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
fn string_literal_replacement_scope_is_a_private_non_copyable_two_variant_domain() {
    let domain = bounded(
        STRING_LITERAL_REPLACEMENT_SCOPE,
        "enum StringLiteralReplacementScope {",
        "\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["FirstOccurrence,", "AllOccurrences,"]);
    let declaration_start = STRING_LITERAL_REPLACEMENT_SCOPE
        .find("enum StringLiteralReplacementScope {")
        .expect("missing literal-replacement scope");
    let preceding_declaration = STRING_LITERAL_REPLACEMENT_SCOPE[..declaration_start].trim();
    assert_eq!(preceding_declaration, "use super::*;");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!STRING_LITERAL_REPLACEMENT_SCOPE.contains(&format!(
            "impl {capability} for StringLiteralReplacementScope"
        )));
    }
    assert!(!STRING_LITERAL_REPLACEMENT_SCOPE.contains("pub enum StringLiteralReplacementScope"));
    assert!(
        !STRING_LITERAL_REPLACEMENT_SCOPE.contains("pub(crate) enum StringLiteralReplacementScope")
    );
    assert!(
        !STRING_LITERAL_REPLACEMENT_SCOPE.contains("pub(super) enum StringLiteralReplacementScope")
    );
    assert_eq!(
        STRING
            .matches("mod string_literal_replacement_scope;")
            .count(),
        1
    );
    assert!(!STRING.contains("mod string_literal_replacement_scope {"));
    assert!(!STRING.contains("string_literal_replacement_scope::"));
    assert!(!STRING.contains("StringLiteralReplacementScope"));
    assert!(!STRING.contains("emit_string_replace_literal_from_string_locals"));
    assert_eq!(
        STRING_LITERAL_REPLACEMENT_SCOPE
            .matches("StringLiteralReplacementScope")
            .count(),
        6
    );
    assert_eq!(
        STRING_LITERAL_REPLACEMENT_SCOPE
            .matches("FirstOccurrence")
            .count(),
        3
    );
    assert_eq!(
        STRING_LITERAL_REPLACEMENT_SCOPE
            .matches("AllOccurrences")
            .count(),
        3
    );
}

#[test]
fn literal_replace_helper_projects_break_or_continuation_once_in_instruction_order() {
    let emitter = bounded(
        STRING_LITERAL_REPLACEMENT_SCOPE,
        "    fn emit_string_replace_literal_from_string_locals(",
        "\n}\n",
    );

    assert!(emitter.contains("scope: StringLiteralReplacementScope,"));
    assert!(!emitter.contains("builtin: StandardBuiltinId"));
    assert_eq!(emitter.matches("match &scope {").count(), 1);
    assert_eq!(
        emitter
            .matches("StringLiteralReplacementScope::FirstOccurrence => {")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("StringLiteralReplacementScope::AllOccurrences => {")
            .count(),
        1
    );
    for forbidden in [
        ": bool",
        "scope ==",
        "scope !=",
        "matches!(scope",
        "_ =>",
        "unreachable!",
        "Default::default",
        "StandardBuiltinId::StringPrototypeReplace",
    ] {
        assert!(!emitter.contains(forbidden));
    }

    let first = without_whitespace(bounded(
        emitter,
        "StringLiteralReplacementScope::FirstOccurrence => {",
        "            StringLiteralReplacementScope::AllOccurrences => {",
    ));
    assert_eq!(first, "function.instruction(&Instruction::Br(2));}");

    let all = without_whitespace(bounded(
        emitter,
        "StringLiteralReplacementScope::AllOccurrences => {",
        "        }\n        function.instruction(&Instruction::End);",
    ));
    assert_eq!(
        all,
        "function.instruction(&Instruction::LocalGet(last_end_local));function.instruction(&Instruction::LocalSet(scan_index_local));function.instruction(&Instruction::LocalGet(search_len_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::LocalGet(scan_index_local));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(scan_index_local));function.instruction(&Instruction::End);}"
    );

    let common_last_end = emitter
        .find("function.instruction(&Instruction::LocalSet(last_end_local));")
        .expect("missing common last-end update");
    let scope_match = emitter
        .find("match &scope {")
        .expect("missing scope projection");
    let continuation = emitter[scope_match..]
        .find("function.instruction(&Instruction::Br(0));")
        .map(|offset| scope_match + offset)
        .expect("missing outer scan continuation");
    assert!(common_last_end < scope_match);
    assert!(scope_match < continuation);
}

#[test]
fn replace_and_replace_all_fallbacks_choose_their_exact_scopes() {
    let fallback = bounded(
        STRING,
        "    fn emit_string_symbol_hook_fallback(",
        "    pub(crate) fn emit_string_search_regexp_fallback_from_string_locals(",
    );
    let normalized = without_whitespace(fallback);

    assert!(fallback.contains("operation: &StringSymbolHookOperation,"));
    assert_eq!(
        fallback
            .matches("StringSymbolHookOperation::Replace => {")
            .count(),
        1
    );
    assert_eq!(
        fallback
            .matches("StringSymbolHookOperation::ReplaceAll => {")
            .count(),
        1
    );
    assert!(!fallback.contains("StandardBuiltinId::StringPrototypeReplace"));
    assert_eq!(
        normalized
            .matches("self.emit_string_replace_literal_first_occurrence_from_string_locals(string_local,arg_payload_local,arg_tag_local,second_payload_local,second_tag_local,function,)?;")
            .count(),
        1
    );
    assert_eq!(
        normalized
            .matches("self.emit_string_replace_literal_all_occurrences_from_string_locals(string_local,arg_payload_local,arg_tag_local,second_payload_local,second_tag_local,function,)?;")
            .count(),
        1
    );
    assert_eq!(fallback.matches("emit_string_replace_literal_").count(), 2);

    let owner = without_whitespace(STRING_LITERAL_REPLACEMENT_SCOPE);
    assert_eq!(
        owner.matches("self.emit_string_replace_literal_from_string_locals(StringLiteralReplacementScope::FirstOccurrence,string_local,search_payload_local,search_tag_local,replacement_payload_local,replacement_tag_local,function,)").count(),
        1
    );
    assert_eq!(
        owner.matches("self.emit_string_replace_literal_from_string_locals(StringLiteralReplacementScope::AllOccurrences,string_local,search_payload_local,search_tag_local,replacement_payload_local,replacement_tag_local,function,)").count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_replace_literal_from_string_locals(",
        ),
        3,
        "the private helper definition and exactly two semantic-wrapper calls must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_replace_literal_first_occurrence_from_string_locals(",
        ),
        2,
        "the first-occurrence wrapper and its sole parent call must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_replace_literal_all_occurrences_from_string_locals(",
        ),
        2,
        "the all-occurrences wrapper and its sole parent call must stay inventoried"
    );
}
