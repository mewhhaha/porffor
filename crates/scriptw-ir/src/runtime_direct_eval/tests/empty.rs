use super::*;

pub(super) fn empty_completion_proof_requires_empty_actual_syntax_and_instantiation() {
    for strict in [Strictness::Sloppy, Strictness::Strict] {
        let caller = caller(context(strict, EvalInvocationContext::Function));
        for policy in [HostSurfacePolicy::default(), HostSurfacePolicy::Test262] {
            let compile = |caller: &RuntimeDirectEvalCaller, source: &str| {
                lower_runtime_direct_eval(
                    caller,
                    &source.encode_utf16().collect::<Vec<_>>(),
                    policy,
                )
            };
            for source in [
                "",
                " \n\t",
                "/* source */",
                "// source",
                "/* \u{0}\u{2028} */",
                ";",
                "{}",
                ";{} /* source */ ;{};",
            ] {
                assert!(
                    compile(&caller, source)
                        .unwrap()
                        .empty_completion()
                        .is_some(),
                    "{source:?}"
                );
            }
            for source in [
                "{;}",
                "'use strict';",
                "'value';",
                "undefined;",
                "var declared;",
                "let lexical;",
                "function retained() {}",
                "{function retained() {}}",
                "name = 1;",
                "// source\nname = 1;",
            ] {
                assert!(
                    compile(&caller, source)
                        .unwrap()
                        .empty_completion()
                        .is_none(),
                    "{source:?}"
                );
            }
            assert!(compile(&caller, "/* not closed").is_err());
            let source = [
                b'/' as u16,
                b'*' as u16,
                0xd800,
                0xdfff,
                b'*' as u16,
                b'/' as u16,
            ];
            let program = lower_runtime_direct_eval(&caller, &source, policy).unwrap();
            assert!(program.empty_completion().is_some());
        }
    }
}

#[test]
fn empty_completion_proof_covers_large_lists_without_erasing_declarations_or_errors() {
    for strict in [Strictness::Sloppy, Strictness::Strict] {
        let caller = caller(context(strict, EvalInvocationContext::Function));
        let source = "{};".repeat(65_536);
        let program = compile(&caller, &source).unwrap();
        assert!(program.empty_completion().is_some());
        assert!(root(&program).body.statements.is_empty());
        assert_eq!(
            program.program.script.as_ref().unwrap().prepared_scripts[0].source,
            source
        );
        for suffix in ["var retained;", "let retained;", "debugger;", "17;"] {
            let program = compile(&caller, &(source.clone() + suffix)).unwrap();
            assert!(program.empty_completion().is_none(), "{suffix}");
        }
        assert!(compile(&caller, &(source + "{let duplicate;let duplicate;}")).is_err());
    }
}

pub(super) fn empty_completion_proof_rejects_observable_or_corrupt_preparation_metadata() {
    let caller = caller(context(Strictness::Sloppy, EvalInvocationContext::Function));
    for mutation in 0..5 {
        let mut program = compile(&caller, "/* empty */").unwrap();
        assert!(program.empty_completion().is_some());
        let crate::PreparedScriptOutcome::Executable(unit) =
            &mut program.program.script.as_mut().unwrap().prepared_scripts[0].outcome
        else {
            unreachable!()
        };
        match mutation {
            0 => unit.owned_env_bindings[0].name = "visible".into(),
            1 => unit.owned_env_bindings[0].slot = 1,
            2 => {
                unit.owned_env_bindings[0].mutability =
                    crate::EnvironmentBindingMutabilityIr::Mutable
            }
            3 => {
                unit.eval_environment =
                    Some(crate::EvalEnvironmentRoleIr::WithObject { object_slot: 0 })
            }
            4 => unit
                .global_bindings
                .insert_initial(crate::ScriptGlobalBindingIr {
                    name: "user".into(),
                    initializer: crate::GlobalPropertyInitializerIr::FreshUndefined,
                    declarations: crate::GlobalDeclarationSetIr::None,
                }),
            _ => unreachable!(),
        }
        assert!(
            program.empty_completion().is_none(),
            "metadata mutation {mutation}"
        );
    }
}
