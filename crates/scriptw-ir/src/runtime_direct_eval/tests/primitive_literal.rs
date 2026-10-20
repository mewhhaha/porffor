use super::*;
use crate::{RuntimeDirectEvalPrimitive, ValueKind};

#[test]
fn primitive_eval_proof_requires_one_actual_literal_and_inert_instantiation() {
    for strictness in [Strictness::Sloppy, Strictness::Strict] {
        let caller = caller(context(strictness, EvalInvocationContext::Function));
        for (source, expected) in [
            (
                "/* arbitrary source */ 23.5;",
                RuntimeDirectEvalPrimitive::Number(23.5f64.to_bits()),
            ),
            (
                "0x80",
                RuntimeDirectEvalPrimitive::Number(128.0f64.to_bits()),
            ),
            (
                "1e309",
                RuntimeDirectEvalPrimitive::Number(f64::INFINITY.to_bits()),
            ),
            ("true", RuntimeDirectEvalPrimitive::Boolean(true)),
            ("(false);", RuntimeDirectEvalPrimitive::Boolean(false)),
            ("null;", RuntimeDirectEvalPrimitive::Null),
        ] {
            let program = compile(&caller, source).unwrap();
            assert_eq!(
                program.primitive_literal().unwrap().value(),
                expected,
                "{source}"
            );
            assert!(program.empty_completion().is_none());
        }
        for source in [
            "",
            "'text';",
            "123n;",
            "({})",
            "/pattern/",
            "name;",
            "name = 1",
            "void name",
            "name();",
            "1; 2;",
            "var name; 1;",
            "let name; 1;",
            "function name(){}; 1;",
            "throw 1;",
        ] {
            if let Ok(program) = compile(&caller, source) {
                assert!(program.primitive_literal().is_none(), "{source}");
            }
        }
        assert!(compile(&caller, "1; let x; let x;").is_err());
    }
}

#[test]
fn primitive_eval_proof_rejects_effectful_or_inconsistent_preparation() {
    let caller = caller(context(Strictness::Sloppy, EvalInvocationContext::Function));
    for mutation in 0..5 {
        let mut program = compile(&caller, "42;").unwrap();
        assert!(program.primitive_literal().is_some());
        let script = program.program.script.as_mut().unwrap();
        let PreparedScriptOutcome::Executable(unit) = &mut script.prepared_scripts[0].outcome
        else {
            panic!()
        };
        match mutation {
            0 => unit.body.statements.push(StatementIr::Empty),
            1 => unit.owned_env_bindings[0].name = "observable".into(),
            2 => unit.declarations.var_names.push("observable".into()),
            3 => {
                let StatementIr::Expression(expression) = &mut unit.body.statements[0] else {
                    panic!()
                };
                expression.kind = ValueKind::Boolean;
            }
            4 => {
                let StatementIr::Expression(expression) = &mut unit.body.statements[0] else {
                    panic!()
                };
                expression.possible_kinds = crate::KindSet::all_runtime_tags();
            }
            _ => unreachable!(),
        }
        assert!(program.primitive_literal().is_none());
    }
}
