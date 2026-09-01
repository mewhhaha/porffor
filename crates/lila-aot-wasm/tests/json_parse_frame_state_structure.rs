const JSON_SOURCE: &str = include_str!("../src/builtins/json.rs");
const PARSE_FRAME_STATE_SOURCE: &str = include_str!("../src/builtins/json/parse_frame_state.rs");

fn unique_bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    assert_eq!(source.matches(start).count(), 1, "unique start `{start}`");
    assert_eq!(source.matches(end).count(), 1, "unique end `{end}`");
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start `{}`", start))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end `{}` after `{}`", end, start))
        .0
}

#[test]
fn parser_frame_state_has_one_closed_eight_word_domain() {
    let domain = unique_bounded(
        JSON_SOURCE,
        "json_wire_domain!(JsonParseFrameState {",
        "\n});\n\nmod parse_frame_state;",
    );
    for (variant, word) in [
        ("ArrayFirstOrEnd", 0),
        ("ArrayValue", 1),
        ("ArrayCommaOrEnd", 2),
        ("ObjectFirstKeyOrEnd", 3),
        ("ObjectKey", 4),
        ("ObjectColon", 5),
        ("ObjectValue", 6),
        ("ObjectCommaOrEnd", 7),
    ] {
        let row = format!("{variant} = {word},");
        assert_eq!(domain.matches(&row).count(), 1, "state row `{row}`");
    }
    assert_eq!(domain.matches(" = ").count(), 8, "exact state domain");

    for obsolete_constant in [
        "JSON_PARSE_ARRAY_FIRST_OR_END",
        "JSON_PARSE_ARRAY_VALUE",
        "JSON_PARSE_ARRAY_COMMA_OR_END",
        "JSON_PARSE_OBJECT_FIRST_KEY_OR_END",
        "JSON_PARSE_OBJECT_KEY",
        "JSON_PARSE_OBJECT_COLON",
        "JSON_PARSE_OBJECT_VALUE",
        "JSON_PARSE_OBJECT_COMMA_OR_END",
    ] {
        assert!(
            !JSON_SOURCE.contains(obsolete_constant),
            "raw frame-state constant remains `{}`",
            obsolete_constant
        );
    }
}

#[test]
fn persisted_state_admission_returns_one_move_only_local_authority() {
    let authority = unique_bounded(
        PARSE_FRAME_STATE_SOURCE,
        "#[must_use = \"validated JSON parse frame state must drive typed dispatch\"]",
        "\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(JSON_SOURCE.matches("\nmod parse_frame_state;\n").count(), 1);
    assert!(!JSON_SOURCE.contains("\npub mod parse_frame_state;\n"));
    assert!(!JSON_SOURCE.contains("\npub(crate) mod parse_frame_state;\n"));
    assert!(!JSON_SOURCE.contains("parse_frame_state::"));
    assert_eq!(
        PARSE_FRAME_STATE_SOURCE
            .matches("pub(super) struct ValidatedJsonParseFrameStateLocal(u32);")
            .count(),
        1
    );
    assert_eq!(
        authority
            .matches("pub(super) struct ValidatedJsonParseFrameStateLocal(u32);")
            .count(),
        1
    );
    assert!(!authority.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!authority.contains(&format!(
            "impl {capability} for ValidatedJsonParseFrameStateLocal"
        )));
    }
    assert!(!authority.contains("pub(super) struct ValidatedJsonParseFrameStateLocal(pub"));
    assert_eq!(authority.matches("const fn local(&self) -> u32").count(), 1);
    assert_eq!(
        authority
            .matches("const fn into_local(self) -> u32")
            .count(),
        1
    );
    assert_eq!(
        JSON_SOURCE
            .matches("ValidatedJsonParseFrameStateLocal")
            .count(),
        0
    );
    assert_eq!(
        PARSE_FRAME_STATE_SOURCE
            .matches("ValidatedJsonParseFrameStateLocal")
            .count(),
        7
    );
    assert_eq!(PARSE_FRAME_STATE_SOURCE.matches("self.0").count(), 2);

    let admission = unique_bounded(
        PARSE_FRAME_STATE_SOURCE,
        "    pub(super) fn emit_validate_json_parse_frame_state_local(",
        "    pub(super) fn emit_json_parse_frame_state_is_i32(",
    );
    for proof in [
        "for state in JsonParseFrameState::ALL.iter()",
        "Instruction::LocalGet(state_local)",
        "Instruction::I64Const(state.word() as i64)",
        "Instruction::I64Eq",
        "Instruction::I32Or",
        "Instruction::I32Eqz",
        "Instruction::Unreachable",
        "ValidatedJsonParseFrameStateLocal(state_local)",
    ] {
        assert_eq!(admission.matches(proof).count(), 1, "admission `{proof}`");
    }
    assert!(!admission.contains(".copied()"));
    assert!(!admission.contains("emit_json_parse_syntax_error"));

    let comparison = unique_bounded(
        PARSE_FRAME_STATE_SOURCE,
        "    pub(super) fn emit_json_parse_frame_state_is_i32(",
        "    pub(super) fn emit_push_json_parse_frame(",
    );
    for proof in [
        "state: &ValidatedJsonParseFrameStateLocal,",
        "expected: JsonParseFrameState,",
        "Instruction::LocalGet(state.local())",
        "Instruction::I64Const(expected.word() as i64)",
    ] {
        assert_eq!(comparison.matches(proof).count(), 1, "comparison `{proof}`");
    }
}

#[test]
fn frame_persistence_requires_a_validated_state_local() {
    let typed_write = unique_bounded(
        JSON_SOURCE,
        "    fn emit_store_json_parse_frame_state(",
        "    fn emit_json_literal_matches_i32(",
    );
    for proof in [
        "state: JsonParseFrameState,",
        "JSON_PARSE_FRAME_STATE_OFFSET,",
        "state.word(),",
    ] {
        assert_eq!(typed_write.matches(proof).count(), 1, "write `{proof}`");
    }

    let push = unique_bounded(
        PARSE_FRAME_STATE_SOURCE,
        "    pub(super) fn emit_push_json_parse_frame(",
        "    pub(super) fn release_validated_json_parse_frame_state_local(",
    );
    for proof in [
        "state: ValidatedJsonParseFrameStateLocal,",
        "state.local(),",
    ] {
        assert_eq!(push.matches(proof).count(), 1, "persistence `{proof}`");
    }
    assert_eq!(
        push.matches("JSON_PARSE_FRAME_STATE_OFFSET,").count(),
        2,
        "one frame-copy row and one typed new-frame write"
    );
    assert!(!push.contains("state_local: u32"));

    let parser = unique_bounded(
        JSON_SOURCE,
        "    pub(crate) fn emit_try_parse_json_text(",
        "    pub(crate) fn emit_try_parse_json_string_text(",
    );
    assert_eq!(
        parser
            .matches("emit_validate_json_parse_frame_state_local(")
            .count(),
        4,
        "root, persisted load, array child and object child admissions"
    );
    assert_eq!(parser.matches("emit_push_json_parse_frame(").count(), 3);
    for validated_state in [
        "root_frame_state,",
        "array_child_frame_state,",
        "object_child_frame_state,",
    ] {
        assert_eq!(
            parser.matches(validated_state).count(),
            1,
            "{validated_state}"
        );
    }
    assert_eq!(
        parser.matches("emit_store_json_parse_frame_state(").count(),
        8,
        "every transition uses the typed state writer"
    );
    assert!(parser.contains("self.release_validated_json_parse_frame_state_local(frame_state);"));
    assert!(!parser.contains("frame_state.into_local()"));
    assert!(!parser.contains("self.release_temp_local(frame_state_local);"));

    let recursive_source = format!("{JSON_SOURCE}{PARSE_FRAME_STATE_SOURCE}");
    for (method, expected) in [
        ("emit_validate_json_parse_frame_state_local(", 5),
        ("emit_json_parse_frame_state_is_i32(", 9),
        ("emit_push_json_parse_frame(", 4),
        ("release_validated_json_parse_frame_state_local(", 2),
    ] {
        assert_eq!(
            recursive_source.matches(method).count(),
            expected,
            "recursive method census `{method}`"
        );
        assert_eq!(
            PARSE_FRAME_STATE_SOURCE
                .matches(&format!("fn {method}"))
                .count(),
            1,
            "private child method owner `{method}`"
        );
        assert_eq!(
            JSON_SOURCE.matches(&format!("fn {method}")).count(),
            0,
            "parent method copy `{method}`"
        );
    }
}

#[test]
fn parser_dispatch_borrows_typed_states_and_traps_an_invalid_word() {
    let parser = unique_bounded(
        JSON_SOURCE,
        "    pub(crate) fn emit_try_parse_json_text(",
        "    pub(crate) fn emit_try_parse_json_string_text(",
    );
    assert_eq!(
        parser
            .matches("emit_json_parse_frame_state_is_i32(")
            .count(),
        8,
        "one dispatch comparison per state"
    );
    assert!(!parser.contains("Instruction::LocalGet(frame_state_local)"));
    for variant in [
        "ArrayFirstOrEnd",
        "ArrayValue",
        "ArrayCommaOrEnd",
        "ObjectFirstKeyOrEnd",
        "ObjectKey",
        "ObjectColon",
        "ObjectValue",
        "ObjectCommaOrEnd",
    ] {
        let expected = format!("JsonParseFrameState::{variant}");
        assert!(
            parser.contains(&expected),
            "missing typed state `{}`",
            expected
        );
    }

    let invalid_dispatch = "function.instruction(&Instruction::End);\n\n        function.instruction(&Instruction::Unreachable);\n        function.instruction(&Instruction::Br(0));";
    assert_eq!(
        parser.matches(invalid_dispatch).count(),
        1,
        "unmatched persisted state must trap before the loop back-edge"
    );
}
