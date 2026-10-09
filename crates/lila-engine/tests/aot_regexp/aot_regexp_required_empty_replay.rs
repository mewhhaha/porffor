use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_literal_and_computed(cases: &[(&str, &str, &str, &str)]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let mut source = String::from(
        "function computed(units, flags) { var text = ''; for (var i = 0; i < units.length; i++) text += String.fromCharCode(units[i]); return new RegExp(text, flags); }\n",
    );
    for (pattern, flags, input, expected) in cases {
        let units: Vec<u16> = pattern.encode_utf16().collect();
        source.push_str(&format!(
            "var expressions = [/{pattern}/{flags}, computed({units:?}, {flags:?})];\nfor (var e = 0; e < expressions.length; e++) {{ var m = expressions[e].exec({input:?}); if (!({expected})) throw new Error('required empty replay failed: ' + e); }}\n",
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
            .unwrap_or_else(|error| panic!("required empty replay failed: {error}\n{script}"));
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}",
        );
    }
}

#[test]
fn required_assertions_keep_the_original_input_cursor_and_modifiers() {
    assert_literal_and_computed(&[
        (
            "^(?:^$){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[0] === '' && m.index === 0",
        ),
        (
            r"^a(?:\b){18446744073709551616} $",
            "d",
            "a ",
            "m !== null && m[0] === 'a ' && m.indices[0][1] === 2",
        ),
        (
            r"^(?:\B){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[0] === ''",
        ),
        (
            "^(?:^){18446744073709551616}x$",
            "dm",
            "\nx",
            "m !== null && m[0] === 'x' && m.index === 1",
        ),
        (
            r"^\ud83d(?:\B){18446744073709551616}\ude00$",
            "d",
            "😀",
            "m !== null && m[0] === '😀' && m.indices[0][1] === 2",
        ),
        (
            r"^ſ(?:\b){18446744073709551616}$",
            "diu",
            "ſ",
            "m !== null && m[0] === 'ſ'",
        ),
        (
            "^(^$){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0",
        ),
    ]);
}

#[test]
fn stable_empty_references_keep_participation_aliases_and_outer_fallbacks() {
    assert_literal_and_computed(&[
        (r"^()(?:\1){18446744073709551616}$", "d", "", "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        (r"^(a?){0}(?:\1){18446744073709551616}$", "d", "", "m !== null && m[1] === undefined && m.indices[1] === undefined"),
        (r"^(?<x>)(?:\k<x>){18446744073709551616}$", "d", "", "m !== null && m.groups.x === '' && m.indices.groups.x[0] === 0"),
        (r"^(?<x>a?){0}(?:\k<x>){18446744073709551616}$", "d", "", "m !== null && m.groups.x === undefined && m.indices.groups.x === undefined"),
        (r"^(?:(?<x>)|(?<x>a?))(?:\k<x>){18446744073709551616}$", "d", "", "m !== null && m[1] === '' && m[2] === undefined && m.groups.x === ''"),
        (r"^(?:(?<x>)(?:\k<x>){18446744073709551616}a|b)$", "d", "b", "m !== null && m[0] === 'b' && m.groups.x === undefined && m.indices.groups.x === undefined"),
        // A referenced capture written in the region keeps normal execution.
        (r"^(?:()\1){2}$", "d", "", "m !== null && m[1] === '' && m.indices[1][1] === 0"),
    ]);
}

#[test]
fn exact_nested_empty_regions_preserve_child_state_reverse_and_normal_choices() {
    assert_literal_and_computed(&[
        (
            "^(?:(){18446744073709551616}){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0",
        ),
        (
            "^a(?:(){18446744073709551616}){18446744073709551616}b$",
            "d",
            "ab",
            "m !== null && m[0] === 'ab' && m.indices[1][0] === 1 && m.indices[1][1] === 1",
        ),
        (
            "^(?:(){18446744073709551616}?){3}?$",
            "d",
            "",
            "m !== null && m[1] === ''",
        ),
        (
            "^(?:(){2}){18446744073709551616}(?:(){3}){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[1] === '' && m[2] === ''",
        ),
        (
            "(?<=^(?:(){18446744073709551616}){18446744073709551616})x",
            "d",
            "x",
            "m !== null && m[0] === 'x' && m.indices[1][0] === 0 && m.indices[1][1] === 0",
        ),
        (
            r"^()(?:(?:\1){2}){18446744073709551616}$",
            "d",
            "",
            "m !== null && m[1] === ''",
        ),
        // Unequal/zero bounds and body alternatives do not mint this proof.
        (
            "^(?:(){0}){2}$",
            "d",
            "",
            "m !== null && m[1] === undefined",
        ),
        (
            "^(?:(a?){1,2}){2}$",
            "d",
            "a",
            "m !== null && m[1] === '' && m.indices[1][0] === 1 && m.indices[1][1] === 1",
        ),
        (
            "^(?:()|a){2}b$",
            "d",
            "ab",
            "m !== null && m[0] === 'ab' && m[1] === undefined",
        ),
    ]);
}
