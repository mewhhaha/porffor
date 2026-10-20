use super::*;

#[test]
fn static_regexp_eval_proof_requires_only_one_valid_literal_and_inert_preparation() {
    for strictness in [Strictness::Sloppy, Strictness::Strict] {
        let caller = caller(context(strictness, EvalInvocationContext::Function));
        for policy in [HostSurfacePolicy::default(), HostSurfacePolicy::Test262] {
            let compile = |source: &str| {
                lower_runtime_direct_eval(
                    &caller,
                    &source.encode_utf16().collect::<Vec<_>>(),
                    policy,
                )
            };
            for (source, pattern, flags) in [
                ("/a/", "a", ""),
                ("(/a/g)", "a", "g"),
                ("/* comment */ /a/;", "a", ""),
                ("/\\ud800/u", "\\ud800", "u"),
            ] {
                let program = compile(source).unwrap();
                let proof = program.static_regexp_literal().unwrap();
                assert_eq!(
                    proof.source_units(),
                    pattern.encode_utf16().collect::<Vec<_>>()
                );
                assert_eq!(
                    proof.flags_units(),
                    flags.encode_utf16().collect::<Vec<_>>()
                );
                assert!(!proof.compiled_bytes().is_empty());
                assert!(program.empty_completion().is_none());
            }
            for source in [
                "",
                "/a/; 1",
                "1; /a/",
                "var x; /a/",
                "let x; /a/",
                "function f() {} /a/",
                "({value: /a/})",
                "new RegExp('a')",
                "eval('/a/')",
                "/a/, /b/",
                "/[/",
                "/a/gg",
            ] {
                if let Ok(program) = compile(source) {
                    assert!(program.static_regexp_literal().is_none(), "{source}");
                }
            }
        }
    }
}

#[test]
fn static_regexp_eval_proof_rejects_extra_effects_and_corrupt_program_bytes() {
    let caller = caller(context(Strictness::Sloppy, EvalInvocationContext::Function));
    for mutation in 0..4 {
        let mut program = compile(&caller, "/a/").unwrap();
        let script = program.program.script.as_mut().unwrap();
        let PreparedScriptOutcome::Executable(unit) = &mut script.prepared_scripts[0].outcome
        else {
            panic!()
        };
        match mutation {
            0 => unit.body.statements.push(StatementIr::Empty),
            1 => unit.owned_env_bindings[0].name = "visible".into(),
            2 => unit.declarations.var_names.push("visible".into()),
            3 => {
                let StatementIr::Expression(expression) = &mut unit.body.statements[0] else {
                    panic!()
                };
                let ExprIr::RegExpLiteral {
                    static_compilation, ..
                } = &mut expression.expr
                else {
                    panic!()
                };
                *static_compilation = None;
            }
            _ => unreachable!(),
        }
        assert!(program.static_regexp_literal().is_none());
    }
}
