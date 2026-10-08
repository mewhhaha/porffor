use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

/// Every case executes both the literal's static descriptor and a descriptor
/// compiled from a runtime character loop. Expected values are independent of
/// either compiler, including undefined captures and UTF-16 boundaries.
fn assert_pairs(cases: &[(&str, &str, &str)]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let mut source = String::from(
        "function computed(units) { var text = ''; for (var i = 0; i < units.length; i++) { text += String.fromCharCode(units[i]); } return new RegExp(text, 'd'); }\n",
    );
    for (pattern, input, expected) in cases {
        let units: Vec<u16> = pattern.encode_utf16().collect();
        source.push_str(&format!(
            "var expressions = [/{pattern}/d, computed({units:?})];\nfor (var e = 0; e < expressions.length; e++) {{ var m = expressions[e].exec({input:?}); if (!({expected})) throw new Error('counted case failed: ' + e); }}\n",
        ));
    }
    source.push_str("true;");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            &source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("counted quantifier execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn tiny_sources_keep_exact_large_finite_bounds_without_expansion() {
    assert_pairs(&[
        ("^(a){32768}$", "", "m === null"),
        ("^(){32768}$", "", "m !== null && m[0] === '' && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        ("^(a?){0,32768}$", "", "m !== null && m[0] === '' && m[1] === undefined && m.indices[1] === undefined"),
    ]);
}

#[test]
fn required_empty_and_optional_empty_have_distinct_capture_semantics() {
    assert_pairs(&[
        ("^(a?){2,3}$", "", "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        ("^(a?){0,2}$", "", "m !== null && m[1] === undefined && m.indices[1] === undefined"),
        ("^((()){2,3}){2,4}$", "", "m !== null && m.length === 4 && m[1] === '' && m[2] === '' && m[3] === ''"),
        ("^(|a){1,2}b$", "ab", "m !== null && m[0] === 'ab' && m[1] === 'a' && m.indices[1][0] === 0 && m.indices[1][1] === 1"),
        ("^(|a){1,2}?b$", "ab", "m !== null && m[0] === 'ab' && m[1] === 'a'"),
    ]);
}

#[test]
fn required_nullable_choices_grow_work_at_the_same_input_cursor() {
    assert_pairs(&[
        (
            "^(?:()|a){40}$",
            "",
            "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0",
        ),
        ("^(?:(?:()|a){3}){20}$", "", "m !== null && m[1] === ''"),
    ]);
}

#[test]
fn backtracking_restores_repeat_counts_and_cleared_capture_ranges() {
    assert_pairs(&[
        ("^((a)|(b)){2,3}b$", "abb", "m !== null && m[0] === 'abb' && m[1] === 'b' && m[2] === undefined && m[3] === 'b' && m.indices[1][0] === 1 && m.indices[1][1] === 2 && m.indices[2] === undefined"),
        ("^((a)|(b)){2,3}?b$", "abb", "m !== null && m[1] === 'b' && m[2] === undefined && m[3] === 'b'"),
        ("^((a)|b){2}$", "ab", "m !== null && m[1] === 'b' && m[2] === undefined"),
        ("^(a|aa){2}b$", "aaab", "m !== null && m[0] === 'aaab' && m[1] === 'aa' && m.indices[1][0] === 1 && m.indices[1][1] === 3"),
    ]);
}

#[test]
fn finite_greedy_and_lazy_choices_preserve_ordered_alternatives() {
    assert_pairs(&[
        (
            "^(a|aa){1,2}a",
            "aaaa",
            "m !== null && m[0] === 'aaa' && m[1] === 'a'",
        ),
        (
            "^(a|aa){1,2}?a",
            "aaaa",
            "m !== null && m[0] === 'aa' && m[1] === 'a'",
        ),
        (r"^(?=(a|aa){1,2})\1b$", "aab", "m === null"),
    ]);
}

#[test]
fn reverse_and_assertion_completion_restore_enclosing_repeat_state() {
    assert_pairs(&[
        ("(?<=^((a)|b){2,3})c", "abc", "m !== null && m[0] === 'c' && m.index === 2 && m[1] === 'a' && m[2] === 'a' && m.indices[1][0] === 0 && m.indices[1][1] === 1"),
        ("^(?=((a|b){2,3})$)(a|b){2}$", "ab", "m !== null && m[1] === 'ab' && m[2] === 'b' && m[3] === 'b' && m.indices[1][0] === 0 && m.indices[1][1] === 2"),
        ("^(?!(?:a|b){2,3}c$)((a)|b){2,3}$", "ab", "m !== null && m[1] === 'b' && m[2] === undefined"),
        ("^(?:(?!(?:a|b){2,3}$)(a|b){2}|((a)|b){2})$", "ab", "m !== null && m[1] === undefined && m[2] === 'b' && m[3] === undefined"),
        ("^(?:(?<=^(?:a?){0,2})a){2}$", "aa", "m !== null && m[0] === 'aa'"),
    ]);
}

#[test]
fn exact_finite_maxima_borrow_across_decimal_limbs_on_bounded_real_matches() {
    assert_pairs(&[
        ("^(a?){1,1000000000}b$", "aaab", "m !== null && m[0] === 'aaab' && m[1] === 'a' && m.indices[1][0] === 2 && m.indices[1][1] === 3"),
        ("^(a?){1,1000000000000000000}b$", "aaab", "m !== null && m[1] === 'a'"),
        ("^(a?){1,18446744073709551615}b$", "aaab", "m !== null && m[1] === 'a'"),
        ("^(a?){1,18446744073709551616}b$", "aaab", "m !== null && m[1] === 'a'"),
        ("^(a?){1,1000000000000000000000000000000000000}b$", "aaab", "m !== null && m[1] === 'a'"),
        ("^(a?){1,0001000000000000000000000000000000000000}?b$", "aaab", "m !== null && m[0] === 'aaab' && m[1] === 'a'"),
        ("^(){0,1000000000000000000000000000000000000}$", "", "m !== null && m[1] === undefined && m.indices[1] === undefined"),
        ("^(){1,1000000000000000000000000000000000000}$", "", "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
    ]);
}

#[test]
fn exact_natural_counter_snapshots_preserve_nested_choices_and_assertion_completion() {
    assert_pairs(&[
        ("^((a)|(b)){1,1000000000000000000000000000}b$", "abb", "m !== null && m[0] === 'abb' && m[1] === 'b' && m[2] === undefined && m[3] === 'b' && m.indices[1][0] === 1 && m.indices[1][1] === 2"),
        ("^((a)|(b)){1,1000000000000000000000000000}?b$", "abb", "m !== null && m[1] === 'b' && m[2] === undefined && m[3] === 'b'"),
        ("^(?=((a?){1,1000000000000000000})b$)(a?){1,1000000000000000000000000000}b$", "aab", "m !== null && m[1] === 'aa' && m[2] === 'a' && m[3] === 'a'"),
        ("^(?!(?:a?){1,1000000000000000000000000000}c$)((a)|b){1,1000000000}$", "ab", "m !== null && m[1] === 'b' && m[2] === undefined"),
        ("(?<=^((a)|b){1,1000000000000000000000000000})c", "abc", "m !== null && m.index === 2 && m[1] === 'a' && m[2] === 'a' && m.indices[1][0] === 0 && m.indices[1][1] === 1"),
    ]);
}
