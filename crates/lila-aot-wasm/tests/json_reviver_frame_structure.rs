const JSON_SOURCE: &str = include_str!("../src/builtins/json.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_json_parse_dynamic_reviver_frame.js");

fn unique_bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    assert_eq!(source.matches(start).count(), 1, "unique start `{start}`");
    assert_eq!(source.matches(end).count(), 1, "unique end `{end}`");
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn mask_line_and_block_comments(source: &str) -> String {
    let mut characters = source.chars().peekable();
    let mut masked = String::with_capacity(source.len());
    let mut quote = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;

    while let Some(character) = characters.next() {
        if line_comment {
            if character == '\n' {
                masked.push(character);
                line_comment = false;
            } else {
                masked.push(' ');
            }
            continue;
        }

        if block_comment_depth > 0 {
            if character == '/' && characters.peek() == Some(&'*') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth += 1;
                continue;
            }
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                masked.push_str("  ");
                block_comment_depth -= 1;
                continue;
            }
            masked.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }

        if let Some(delimiter) = quote {
            masked.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }

        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            masked.push(character);
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            characters.next();
            masked.push_str("  ");
            line_comment = true;
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            masked.push_str("  ");
            block_comment_depth = 1;
            continue;
        }
        masked.push(character);
    }

    assert_eq!(block_comment_depth, 0, "unterminated block comment");
    masked
}

fn anchored_offsets(source: &str, declaration: &str) -> Vec<usize> {
    source
        .match_indices(declaration)
        .filter_map(|(offset, _)| {
            let line_start = source[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            source[line_start..offset]
                .chars()
                .all(char::is_whitespace)
                .then_some(offset)
        })
        .collect()
}

fn braced_rust_function<'a>(source: &'a str, declaration: &str) -> &'a str {
    let offsets = anchored_offsets(source, declaration);
    assert_eq!(offsets.len(), 1, "exact Rust owner `{declaration}`");
    let start = offsets[0];
    let mut depth = 0;
    let mut body_started = false;
    for (relative_offset, character) in source[start..].char_indices() {
        match character {
            '{' => {
                depth += 1;
                body_started = true;
            }
            '}' => {
                depth -= 1;
                if body_started && depth == 0 {
                    return &source[start..start + relative_offset + character.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated Rust owner `{declaration}`");
}

fn assert_live_wasm_cli_test(source: &str, name: &str, fixture: &str) {
    let declaration = format!("fn {name}() {{");
    let offsets = anchored_offsets(source, &declaration);
    assert_eq!(offsets.len(), 1, "exact CLI test owner `{name}`");

    let attached_source = source[..offsets[0]]
        .rsplit_once("\n}\n")
        .expect("preceding top-level CLI test")
        .1;
    let normalized_attached_source = without_whitespace(attached_source);
    assert_eq!(
        normalized_attached_source.matches("#[test]").count(),
        1,
        "`{name}` must remain a live Rust test"
    );
    for disabling_attribute in ["#[cfg", "#[cfg_attr", "#[ignore"] {
        assert!(
            !normalized_attached_source.contains(disabling_attribute),
            "`{name}` must not carry `{disabling_attribute}`"
        );
    }

    let body = braced_rust_function(source, &declaration);
    for marker in [
        "let output = Command::new(env!(\"CARGO_BIN_EXE_lila\"))",
        ".arg(\"run\")",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "assert!(output.status.success());",
        "assert!(stdout.contains(\"backend_used: WasmAot\"));",
        "assert!(stdout.contains(\"boolean(true)\"));",
    ] {
        assert_eq!(
            body.lines().filter(|line| line.trim() == marker).count(),
            1,
            "`{name}` must retain CLI marker `{marker}`"
        );
    }
    let fixture_marker = format!(".arg(fixture_path(\"{fixture}\"))");
    assert_eq!(
        body.lines()
            .filter(|line| line.trim() == fixture_marker)
            .count(),
        1,
        "`{name}` must run its exact fixture"
    );
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn unique_normalized_position(source: &str, snippet: &str, label: &str) -> usize {
    let snippet = without_whitespace(snippet);
    assert_eq!(
        source.matches(&snippet).count(),
        1,
        "fixture must contain one {label}"
    );
    source
        .find(&snippet)
        .unwrap_or_else(|| panic!("missing {label}"))
}

#[test]
fn reviver_frame_wire_domains_fix_four_states_and_two_property_roles() {
    let wire_domain = unique_bounded(
        JSON_SOURCE,
        "macro_rules! json_wire_domain {",
        "json_wire_domain!(JsonReviverFrameState {",
    );
    for proof in [
        "const ALL: &'static [Self] = &[$(Self::$variant),+];",
        "const fn word(self) -> u64 {",
        "$(Self::$variant => $word),+",
        "assert!(all[index].word() == index as u64);",
    ] {
        assert!(
            wire_domain.contains(proof),
            "missing wire-domain proof `{proof}`"
        );
    }

    let states = unique_bounded(
        JSON_SOURCE,
        "json_wire_domain!(JsonReviverFrameState {",
        "json_wire_domain!(JsonReviverPropertyRole {",
    );
    for state in [
        "Enter = 0,",
        "ArrayChildren = 1,",
        "ObjectChildren = 2,",
        "Apply = 3,",
    ] {
        assert_eq!(states.matches(state).count(), 1, "state word `{state}`");
    }
    assert_eq!(states.matches(" = ").count(), 4, "exact state domain");

    let roles = unique_bounded(
        JSON_SOURCE,
        "json_wire_domain!(JsonReviverPropertyRole {",
        "const JSON_PARSE_ARRAY_FIRST_OR_END",
    );
    for role in ["Nested = 0,", "Root = 1,"] {
        assert_eq!(roles.matches(role).count(), 1, "role word `{role}`");
    }
    assert_eq!(roles.matches(" = ").count(), 2, "exact role domain");
}

#[test]
fn reviver_frames_persist_typed_words_and_trap_invalid_dispatch_values() {
    assert_eq!(
        JSON_SOURCE
            .matches("JSON_REVIVER_FRAME_STATE_OFFSET")
            .count(),
        3,
        "one declaration, typed write and dynamic read"
    );
    assert_eq!(
        JSON_SOURCE
            .matches("JSON_REVIVER_FRAME_ROLE_OFFSET")
            .count(),
        3,
        "one declaration, typed write and dynamic read"
    );

    let state_writer = unique_bounded(
        JSON_SOURCE,
        "    fn emit_store_json_reviver_state(",
        "    fn emit_push_json_reviver_frame(",
    );
    for proof in [
        "state: JsonReviverFrameState,",
        "JSON_REVIVER_FRAME_STATE_OFFSET,",
        "state.word(),",
    ] {
        assert_eq!(
            state_writer.matches(proof).count(),
            1,
            "state write `{proof}`"
        );
    }

    let frame_push = unique_bounded(
        JSON_SOURCE,
        "    fn emit_push_json_reviver_frame(",
        "    fn emit_json_reviver_metadata_child(",
    );
    for proof in [
        "role: JsonReviverPropertyRole,",
        "JSON_REVIVER_FRAME_ROLE_OFFSET,",
        "role.word(),",
        "JsonReviverFrameState::Enter,",
    ] {
        assert_eq!(
            frame_push.matches(proof).count(),
            1,
            "frame write `{proof}`"
        );
    }
    assert_eq!(
        frame_push.matches("emit_store_json_reviver_state(").count(),
        1
    );

    let dynamic_walk = unique_bounded(
        JSON_SOURCE,
        "    pub(crate) fn emit_json_internalize_dynamic(",
        "    pub(crate) fn emit_try_parse_json_text(",
    );
    for persisted_read in [
        "(JSON_REVIVER_FRAME_STATE_OFFSET, state_local),",
        "(JSON_REVIVER_FRAME_ROLE_OFFSET, role_local),",
    ] {
        assert_eq!(
            dynamic_walk.matches(persisted_read).count(),
            1,
            "persisted frame read `{persisted_read}`"
        );
    }
    for dispatch in [
        "for state in JsonReviverFrameState::ALL.iter().copied() {",
        "for role in JsonReviverPropertyRole::ALL.iter().copied() {",
    ] {
        assert_eq!(
            dynamic_walk.matches(dispatch).count(),
            1,
            "dispatch `{dispatch}`"
        );
    }
    for arm in [
        "JsonReviverFrameState::Enter =>",
        "JsonReviverFrameState::ArrayChildren =>",
        "JsonReviverFrameState::ObjectChildren =>",
        "JsonReviverFrameState::Apply =>",
    ] {
        assert_eq!(
            dynamic_walk.matches(arm).count(),
            1,
            "state dispatch arm `{arm}`"
        );
    }
    assert!(
        !dynamic_walk.contains("_ =>"),
        "state dispatch must remain exhaustive"
    );
    assert_eq!(
        dynamic_walk
            .matches("function.instruction(&Instruction::Unreachable);")
            .count(),
        2,
        "invalid role and invalid state words each trap"
    );
    assert_before(
        dynamic_walk,
        "(JSON_REVIVER_FRAME_STATE_OFFSET, state_local),",
        "for state in JsonReviverFrameState::ALL.iter().copied() {",
    );
    assert_before(
        dynamic_walk,
        "(JSON_REVIVER_FRAME_ROLE_OFFSET, role_local),",
        "for role in JsonReviverPropertyRole::ALL.iter().copied() {",
    );
}

#[test]
fn static_and_dynamic_revivers_share_one_post_call_result_owner() {
    assert_eq!(
        JSON_SOURCE
            .matches("emit_json_apply_reviver_result(")
            .count(),
        3,
        "one definition and exactly two callers"
    );

    let result_owner = unique_bounded(
        JSON_SOURCE,
        "    fn emit_json_apply_reviver_result(",
        "    fn emit_json_create_data_property(",
    );
    assert_eq!(
        result_owner
            .matches("role: JsonReviverPropertyRole,")
            .count(),
        1
    );
    for arm in [
        "JsonReviverPropertyRole::Root => {}",
        "JsonReviverPropertyRole::Nested => {",
    ] {
        assert_eq!(
            result_owner.matches(arm).count(),
            1,
            "result role arm `{arm}`"
        );
    }
    assert!(
        !result_owner.contains("_ =>"),
        "result role match must be exhaustive"
    );
    for operation in [
        "self.emit_array_delete(",
        "self.emit_object_delete(",
        "self.emit_array_create_data_property_silent(",
        "self.emit_json_create_data_property(",
    ] {
        assert!(
            result_owner.contains(operation),
            "missing nested result operation `{operation}`"
        );
    }

    let static_caller = unique_bounded(
        JSON_SOURCE,
        "    fn emit_json_apply_reviver_with_source(",
        "    /// Applies the result of a completed reviver call.",
    );
    assert_eq!(
        static_caller
            .matches("emit_json_apply_reviver_result(")
            .count(),
        1
    );
    assert_before(
        static_caller,
        "self.emit_indirect_call_from_locals(",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        static_caller,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.set_completion_kind(CompletionKind::Normal, function);",
    );
    assert_before(
        static_caller,
        "self.set_completion_kind(CompletionKind::Normal, function);",
        "self.emit_json_apply_reviver_result(",
    );

    let dynamic_caller = unique_bounded(
        JSON_SOURCE,
        "                JsonReviverFrameState::Apply => {",
        "                    function.instruction(&Instruction::LocalGet(frame_len_local));",
    );
    assert_eq!(
        dynamic_caller
            .matches("emit_json_apply_reviver_result(")
            .count(),
        1
    );
    assert_before(
        dynamic_caller,
        "self.emit_indirect_call_from_locals(",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        dynamic_caller,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.set_completion_kind(CompletionKind::Normal, function);",
    );
    assert_before(
        dynamic_caller,
        "self.set_completion_kind(CompletionKind::Normal, function);",
        "self.emit_json_apply_reviver_result(",
    );
}

#[test]
fn dynamic_reviver_fixture_is_actively_registered_and_covers_frame_observables() {
    const CLI_TEST_NAME: &str =
        "run_wasm_backend_succeeds_for_json_parse_dynamic_reviver_frame_fixture";
    let declaration = format!("fn {CLI_TEST_NAME}() {{");
    for commented_registration in [
        format!("// #[test]\n// {declaration}\n// }}"),
        format!("/*\n#[test]\n{declaration}\n}}\n*/"),
    ] {
        let active_registration = mask_line_and_block_comments(&commented_registration);
        assert!(
            anchored_offsets(&active_registration, &declaration).is_empty(),
            "commented CLI owner must not count as active"
        );
    }

    let active_cli_tests = mask_line_and_block_comments(CLI_TESTS);
    assert_live_wasm_cli_test(
        &active_cli_tests,
        CLI_TEST_NAME,
        "wasm_json_parse_dynamic_reviver_frame.js",
    );
    assert_eq!(
        active_cli_tests
            .matches("fixture_path(\"wasm_json_parse_dynamic_reviver_frame.js\")")
            .count(),
        1,
        "fixture has one exact CLI owner"
    );

    let executable_fixture = mask_line_and_block_comments(CLI_FIXTURE);
    let fixture = without_whitespace(&executable_fixture);
    let executable_assertion =
        without_whitespace(r#"if (calls.length !== 7) fail("postorder call count");"#);
    for commented_assertion in [
        r#"// if (calls.length !== 7) fail("postorder call count");"#,
        r#"/* if (calls.length !== 7) fail("postorder call count"); */"#,
    ] {
        assert!(
            !without_whitespace(&mask_line_and_block_comments(commented_assertion))
                .contains(&executable_assertion),
            "commented fixture assertion must not count as executable"
        );
    }
    let fail_boundary = unique_normalized_position(
        &fixture,
        r#"function fail(message) { throw message; }"#,
        "throwing failure boundary",
    );
    let walk_setup = unique_normalized_position(
        &fixture,
        r#"let result = parse('{"":{"leaf":1e+2},"array":[2,3],"later":4}', function (key, value, context) {"#,
        "dynamic walk setup",
    );
    let call_record =
        unique_normalized_position(&fixture, "calls.push(key);", "reviver call recording");
    assert!(fail_boundary < walk_setup && walk_setup < call_record);

    for (snippet, label) in [
        (
            r#"if (calls.length !== 7) fail("postorder call count");"#,
            "postorder call count assertion",
        ),
        (
            r#"if (calls[0] !== "leaf") fail("postorder leaf");"#,
            "postorder leaf assertion",
        ),
        (
            r#"if (calls[1] !== "") fail("nested empty-string key");"#,
            "nested empty-key assertion",
        ),
        (
            r#"if (calls[2] !== "0" || calls[3] !== "1") fail("array snapshot order");"#,
            "array snapshot assertion",
        ),
        (
            r#"if (calls[4] !== "array" || calls[5] !== "later" || calls[6] !== "") { fail("object snapshot order"); }"#,
            "object snapshot assertion",
        ),
        (
            r#"if (nestedHolder !== wrapped) fail("nested holder identity");"#,
            "nested holder assertion",
        ),
        (
            r#"if (rootHolder === nestedHolder) fail("root holder role");"#,
            "root role assertion",
        ),
        (
            r#"if (rootHolder[""] !== wrapped) fail("synthetic root holder");"#,
            "synthetic root assertion",
        ),
        (
            r#"if (wrapped[""] !== "nested-empty-key") fail("nested empty-string replacement");"#,
            "nested empty-key replacement assertion",
        ),
        (
            r#"if (wrapped.added !== 5) fail("object snapshot mutation");"#,
            "object snapshot mutation assertion",
        ),
        (
            r#"if ("later" in wrapped) fail("nested deletion");"#,
            "nested deletion assertion",
        ),
        (
            r#"if (wrapped.array.length !== 3 || wrapped.array[0] !== 2 || wrapped.array[1] !== 30 || wrapped.array[2] !== 40) { fail("array snapshot mutation"); }"#,
            "array snapshot mutation assertion",
        ),
        (
            r#"if (context.source !== "1e+2") fail("nested primitive source");"#,
            "primitive source assertion",
        ),
        (
            r#"if (context.source !== "2") fail("array primitive source");"#,
            "array element source assertion",
        ),
        (
            r#"if (context.source !== undefined) fail("mutated value source eligibility");"#,
            "mutated source assertion",
        ),
        (
            r#"if (context.source !== undefined) fail("root object source eligibility");"#,
            "root source assertion",
        ),
        (
            r#"if (sourceChecks !== 3) fail("source checks");"#,
            "source assertion count",
        ),
        (
            r#"if (rootUndefined !== undefined) fail("root undefined result");"#,
            "root undefined assertion",
        ),
    ] {
        unique_normalized_position(&fixture, snippet, label);
    }

    let forward_write =
        unique_normalized_position(&fixture, "this[1] = 30;", "forward array write");
    let forward_read = unique_normalized_position(
        &fixture,
        r#"if (value !== 30) fail("forward array mutation");"#,
        "forward array read assertion",
    );
    let forward_source = unique_normalized_position(
        &fixture,
        r#"if (context.source !== undefined) fail("mutated value source eligibility");"#,
        "forward mutation source assertion",
    );
    assert!(forward_write < forward_read && forward_read < forward_source);

    let deletion_request = unique_normalized_position(
        &fixture,
        r#"if (key === "later") return undefined;"#,
        "nested deletion request",
    );
    let deletion_assertion = unique_normalized_position(
        &fixture,
        r#"if ("later" in wrapped) fail("nested deletion");"#,
        "nested deletion result",
    );
    assert!(deletion_request < deletion_assertion);

    let abrupt_setup = unique_normalized_position(
        &fixture,
        r#"parse('{"a":{"b":1},"c":2}', function (key, value) {"#,
        "abrupt walk setup",
    );
    let abrupt_throw = unique_normalized_position(
        &fixture,
        "if (key === \"b\") throw sentinel;",
        "abrupt throw",
    );
    let abrupt_provenance = unique_normalized_position(
        &fixture,
        "caught = error === sentinel;",
        "abrupt provenance",
    );
    let abrupt_assertion = unique_normalized_position(
        &fixture,
        r#"if (!caught || abruptCalls !== 1) fail("abrupt reviver order");"#,
        "abrupt propagation assertion",
    );
    assert!(
        abrupt_setup < abrupt_throw
            && abrupt_throw < abrupt_provenance
            && abrupt_provenance < abrupt_assertion
    );

    let final_success = unique_normalized_position(&fixture, "true;", "final success value");
    assert_eq!(final_success + "true;".len(), fixture.len());
}
