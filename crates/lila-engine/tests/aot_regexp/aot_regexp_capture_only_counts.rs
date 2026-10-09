use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_literal_and_computed(cases: &[(&str, &str, &str)]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    // The character loop keeps the second producer genuinely runtime-owned.
    // Both results are checked against independent capture and UTF16 expectations.
    let mut source = String::from(
        "function computed(units) { var text = ''; for (var i = 0; i < units.length; i++) text += String.fromCharCode(units[i]); return new RegExp(text, 'd'); }\n",
    );
    for (pattern, input, expected) in cases {
        let units: Vec<u16> = pattern.encode_utf16().collect();
        source.push_str(&format!(
            "var expressions = [/{pattern}/d, computed({units:?})];\nfor (var e = 0; e < expressions.length; e++) {{ var m = expressions[e].exec({input:?}); if (!({expected})) throw new Error('capture-only count failed: ' + e); }}\n",
        ));
    }
    source.push_str("true;");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &script,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| {
                panic!("capture-only count execution failed: {error}\n{script}")
            });
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}",
        );
    }
}

#[test]
fn exact_huge_capture_only_counts_finish_with_last_required_capture_state() {
    assert_literal_and_computed(&[
        ("^(){18446744073709551616}$", "", "m !== null && m[0] === '' && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        ("^(?:){18446744073709551616}$", "", "m !== null && m.length === 1 && m[0] === ''"),
        ("^a(()){1000000000000000000000000000000}b$", "ab", "m !== null && m[1] === '' && m[2] === '' && m.indices[1][0] === 1 && m.indices[1][1] === 1 && m.indices[2][0] === 1 && m.indices[2][1] === 1"),
        ("^(){1000000000000000000,1000000000000000001}$", "", "m !== null && m[1] === ''"),
        ("^(){999999999999999999,1000000000000000000}?$", "", "m !== null && m[1] === ''"),
        ("^(){18446744073709551616,}$", "", "m !== null && m[1] === ''"),
        ("^(){0,18446744073709551616}$", "", "m !== null && m[1] === undefined && m.indices[1] === undefined"),
        ("^(){1,18446744073709551616}$", "", "m !== null && m[1] === ''"),
        ("^(){18446744073709551616}(a){2,3}b$", "aab", "m !== null && m[0] === 'aab' && m[1] === '' && m[2] === 'a' && m.indices[2][0] === 1 && m.indices[2][1] === 2"),
    ]);
}

#[test]
fn capture_only_batches_preserve_reverse_assertion_and_outer_backtracking_boundaries() {
    assert_literal_and_computed(&[
        ("(?<=^(()){18446744073709551616})x", "x", "m !== null && m[0] === 'x' && m[1] === '' && m[2] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        ("^a(?=(()){18446744073709551616}b)b$", "ab", "m !== null && m[1] === '' && m[2] === '' && m.indices[2][0] === 1 && m.indices[2][1] === 1"),
        ("^a(?!(()){18446744073709551616}b)c$", "ac", "m !== null && m[1] === undefined && m[2] === undefined && m.indices[1] === undefined"),
        ("^(?:(()){18446744073709551616}a|b)$", "b", "m !== null && m[0] === 'b' && m[1] === undefined && m[2] === undefined"),
        ("^(?:(){18446744073709551616}){2}$", "", "m !== null && m[1] === ''"),
        // These small domains retain the ordinary path for each excluded opcode
        // family. Their results expose real choice/progress and capture clearing.
        ("^(?:()|a){2}b$", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined"),
        ("^(?:()|a){2}b$", "b", "m !== null && m[1] === ''"),
        ("^(?:(?=a)){2}a$", "a", "m !== null && m[0] === 'a'"),
        (r"^(a?)\1{2}b$", "b", "m !== null && m[1] === ''"),
        ("^(?:(){2}){2}$", "", "m !== null && m[1] === ''"),
    ]);
}
