const JSON_SOURCE: &str = include_str!("../src/builtins/json.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_json_parse_dynamic_reviver_frame.js");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/json-reviver-frame.md");
const PARSE_CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/json-parse-frame-state.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

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
fn runtime_parser_and_iterative_walk_are_the_only_reviver_owners() {
    for source in [JSON_SOURCE, EXPRESSIONS_SOURCE, IR_SOURCE, LOWERING_SOURCE] {
        for retired in [
            "JsonParseStaticReviver",
            "JsonStaticValueIr",
            "compile_json_static_reviver_to_locals",
            "mod static_reviver;",
            "mod static_json_parse;",
        ] {
            assert!(!source.contains(retired), "retired JSON owner `{retired}`");
        }
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!manifest
        .join("src/builtins/json/static_reviver.rs")
        .exists());
    assert!(!manifest
        .join("../lila-ir/src/lowering/static_json_parse.rs")
        .exists());
}

#[test]
fn reviver_frame_wire_domains_fix_four_states_and_two_property_roles() {
    for evidence in [CONTRACT, PARSE_CONTRACT, TASK] {
        let words = evidence.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("Batch AM"));
        assert!(words.contains("capability-free JSON wire domains"));
    }
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
