const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/string.rs");
const CLI_FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_string_repeat_core.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_normalized_position(source: &str, snippet: &str, label: &str) -> usize {
    let snippet = without_whitespace(snippet);
    assert_eq!(
        source.matches(snippet.as_str()).count(),
        1,
        "{label} must occur exactly once"
    );
    source
        .find(snippet.as_str())
        .unwrap_or_else(|| panic!("missing normalized sentinel: {label}"))
}

#[test]
fn repeat_fixture_and_cli_registration_pin_total_count_and_error_realm_cases() {
    let fixture = without_whitespace(CLI_FIXTURE);
    for (label, snippet) in [
        (
            "fail-loud assertion boundary",
            r#"
            function check(value, label) {
              if (!value) {
                throw "String repeat fixture failed: " + label;
              }
            }
            "#,
        ),
        (
            "empty enormous finite count",
            r#"check("".repeat(1e100) === "", "empty enormous finite count");"#,
        ),
        (
            "negative fractional count",
            r#"check("foo".repeat(-0.5) === "", "negative fractional count one");"#,
        ),
        (
            "created Realm repeat method",
            "var otherRepeat = otherRealm.String.prototype.repeat;",
        ),
        (
            "created Realm negative fraction",
            r#"check(otherRepeat.call("x", -0.5) === "", "other realm negative fractional count");"#,
        ),
        (
            "created Realm empty enormous finite count",
            r#"check(otherRepeat.call("", 1e100) === "", "other realm empty enormous finite count");"#,
        ),
    ] {
        unique_normalized_position(&fixture, snippet, label);
    }

    let enormous_call = unique_normalized_position(
        &fixture,
        r#""x".repeat(1e100);"#,
        "nonempty enormous finite call",
    );
    let enormous_false_sentinel = unique_normalized_position(
        &fixture,
        r#"check(false, "enormous finite count did not throw");"#,
        "nonempty enormous finite false sentinel",
    );
    let enormous_error = unique_normalized_position(
        &fixture,
        r#"check(e instanceof RangeError, "enormous finite count RangeError");"#,
        "nonempty enormous finite RangeError",
    );
    assert!(
        enormous_call < enormous_false_sentinel && enormous_false_sentinel < enormous_error,
        "the enormous finite call must precede its false sentinel and caught RangeError"
    );

    let invalid_count_call = unique_normalized_position(
        &fixture,
        r#"otherRepeat.call("x", -1);"#,
        "created Realm invalid-count call",
    );
    let invalid_count_false_sentinel = unique_normalized_position(
        &fixture,
        r#"check(false, "other realm negative count did not throw");"#,
        "created Realm invalid-count false sentinel",
    );
    let invalid_count_error = unique_normalized_position(
        &fixture,
        r#"check(e instanceof otherRealm.RangeError, "other realm negative count RangeError");"#,
        "created Realm invalid-count error",
    );
    let invalid_count_provenance = unique_normalized_position(
        &fixture,
        r#"check((e instanceof RangeError) === false, "other realm negative count not main RangeError");"#,
        "created Realm invalid-count provenance",
    );
    assert!(
        invalid_count_call < invalid_count_false_sentinel
            && invalid_count_false_sentinel < invalid_count_error
            && invalid_count_error < invalid_count_provenance,
        "the created-Realm invalid-count call must precede its false sentinel, caught error, and provenance check"
    );

    let result_limit_call = unique_normalized_position(
        &fixture,
        r#"otherRepeat.call("x", 1e100);"#,
        "created Realm result-limit call",
    );
    let result_limit_false_sentinel = unique_normalized_position(
        &fixture,
        r#"check(false, "other realm enormous finite count did not throw");"#,
        "created Realm result-limit false sentinel",
    );
    let result_limit_error = unique_normalized_position(
        &fixture,
        r#"check(e instanceof otherRealm.RangeError, "other realm enormous finite count RangeError");"#,
        "created Realm result-limit error",
    );
    let result_limit_provenance = unique_normalized_position(
        &fixture,
        r#"
        check(
          (e instanceof RangeError) === false,
          "other realm enormous finite count not main RangeError",
        );
        "#,
        "created Realm result-limit provenance",
    );
    assert!(
        result_limit_call < result_limit_false_sentinel
            && result_limit_false_sentinel < result_limit_error
            && result_limit_error < result_limit_provenance,
        "the created-Realm result-limit call must precede its false sentinel, caught error, and provenance check"
    );

    let final_publication =
        unique_normalized_position(&fixture, "true;", "final success publication");
    assert_eq!(
        final_publication + "true;".len(),
        fixture.len(),
        "the unique success publication must terminate the fixture"
    );

    const REGISTRATION_START: &str =
        "#[test]\nfn run_wasm_backend_succeeds_for_string_repeat_fixture()";
    const REGISTRATION_END: &str =
        "#[test]\nfn run_wasm_backend_succeeds_for_string_code_point_at_surrogates_fixture()";
    assert_eq!(CLI_TESTS.matches(REGISTRATION_START).count(), 1);
    let registration_start = CLI_TESTS
        .find(REGISTRATION_START)
        .expect("repeat CLI registration");
    let preceding_attributes = CLI_TESTS[..registration_start]
        .rsplit_once("\n}\n")
        .expect("the CLI test preceding the repeat registration")
        .1;
    for inactive_attribute in ["#[ignore", "#[cfg"] {
        assert!(
            !preceding_attributes.contains(inactive_attribute),
            "the repeat CLI registration must not have {inactive_attribute} attached"
        );
    }
    let registration = bounded(CLI_TESTS, REGISTRATION_START, REGISTRATION_END);
    assert_eq!(
        CLI_TESTS.matches("wasm_string_repeat_core.js").count(),
        1,
        "the repeat fixture must have exactly one CLI registration"
    );
    for required in [
        "Command::new(env!(\"CARGO_BIN_EXE_lila\"))",
        ".arg(\"run\")",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "wasm_string_repeat_core.js",
        "output.status.success()",
        "backend_used: WasmAot",
        "boolean(true)",
    ] {
        assert!(
            registration.contains(required),
            "the repeat CLI registration must retain {required}"
        );
    }
}
