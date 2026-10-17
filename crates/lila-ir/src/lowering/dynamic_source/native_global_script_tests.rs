use crate::{
    EnvironmentIdentifierOperationIr, ExprIr, HostBuiltinId, HostSurfacePolicy,
    PreparedScriptAdmission, PreparedScriptKind, ProgramIr, ScriptIr, StatementIr, TypedExpr,
    ValueKind,
};
fn lower(source: &str) -> ProgramIr {
    crate::lower_with_host_surface_policy(
        &lila_front::parse(source, lila_front::ParseOptions::script()).unwrap(),
        HostSurfacePolicy::Test262,
    )
}
// Callee/argument staging may wrap the call, and a global Reference is an
// EnvironmentIdentifier Call. Follow only completion-preserving wrappers;
// finding an unrelated call in an operand would not prove this invocation.
fn assert_runtime_call(script: &ScriptIr) {
    fn completion(expression: &TypedExpr) -> &TypedExpr {
        match &expression.expr {
            ExprIr::MaterializeBinding { body, .. } => completion(body),
            ExprIr::Comma { rhs, .. } => completion(rhs),
            _ => expression,
        }
    }
    let Some(StatementIr::Expression(expression)) = script.body.statements.last() else {
        panic!("the source invocation must remain the final expression");
    };
    let expression = completion(expression);
    match &expression.expr {
        ExprIr::CallNamed { .. }
        | ExprIr::CallIndirect {
            direct_eval: None, ..
        }
        | ExprIr::CallMethod { .. } => {}
        ExprIr::EnvironmentIdentifier(identifier)
            if matches!(
                &identifier.operation,
                EnvironmentIdentifierOperationIr::Call {
                    direct_eval: None,
                    ..
                }
            ) => {}
        _ => panic!("actual runtime callee must be retained: {expression:?}"),
    }
}
#[test]
fn opaque_realm_script_source_retains_the_actual_callable_and_runtime_invocation() {
    for source in [
        "__lilaRealmEvalScript(globalThis.source);",
        "__lilaRealmEvalScript(...[globalThis.source]);",
        "var invoke=__lilaRealmEvalScript;invoke(globalThis.source);",
        "__lilaRealmEvalScript.call(undefined,globalThis.source);",
    ] {
        let program = lower(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.unwrap();
        assert!(script.prepared_scripts.is_empty());
        assert_eq!(script.result_kind(), ValueKind::Dynamic);
        assert!(script
            .host_builtins
            .contains(&HostBuiltinId::RealmEvalScript));
        assert_runtime_call(&script);
    }
}
#[test]
fn realm_script_spelling_cannot_admit_a_replacement_ordinary_callable_as_source() {
    let program=lower("var marker={}; var realm=__lilaCreateRealm(); realm.evalScript=function(value){return value;}; realm.evalScript(marker);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    // A speculative source cache entry cannot prove the overwritten
    // property still denotes the intrinsic. Execution must select the actual
    // ordinary callable; only a runtime-guarded candidate may remain cached.
    assert!(script.prepared_scripts.iter().all(|prepared| {
        prepared.kind == PreparedScriptKind::RealmScript
            && prepared.admission == PreparedScriptAdmission::RuntimeCandidate
    }));
    assert!(script.functions.iter().any(|f| f.params.len() == 1));
    assert_runtime_call(&script);
}
#[test]
fn global_invocation_admission_does_not_mint_direct_eval_or_shadow_source_units() {
    for source in [
        "eval(globalThis.source);",
        "new ShadowRealm().evaluate(globalThis.source);",
    ] {
        let program = lower(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.unwrap();
        assert!(!script
            .prepared_scripts
            .iter()
            .any(|p| matches!(p.kind, PreparedScriptKind::RealmScript)));
    }
}
