use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, ModuleEntryEvaluationKindIr, StatementIr};

#[test]
fn module_entry_has_one_private_completion_operation_in_both_execution_modes() {
    for (source, kind) in [
        ("42;", ModuleEntryEvaluationKindIr::Synchronous),
        ("await 0; 42;", ModuleEntryEvaluationKindIr::Promise),
        (
            "throw undefined; await 0;",
            ModuleEntryEvaluationKindIr::Promise,
        ),
    ] {
        let parsed = parse(source, ParseOptions::module()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().unwrap();
        let entry = script
            .module_entry_evaluation()
            .expect("Module owns its entry result");
        assert_eq!(entry.kind(), kind);
        let entries = script
            .body
            .statements
            .iter()
            .filter(|statement| {
                matches!(statement, StatementIr::Expression(expression)
                if matches!(expression.expr, ExprIr::ModuleEntryEvaluation(_)))
            })
            .count();
        assert_eq!(entries, 1);
        match kind {
            ModuleEntryEvaluationKindIr::Synchronous => {
                assert!(matches!(entry.evaluation().expr, ExprIr::ModuleEvaluate(_)));
            }
            ModuleEntryEvaluationKindIr::Promise => {
                assert!(matches!(
                    entry.evaluation().expr,
                    ExprIr::CallIndirect { .. }
                ));
            }
        }
    }
}

#[test]
fn source_spelling_does_not_grant_script_calls_the_private_module_operation() {
    for source in [
        "void (() => { 42; })();",
        "void (async () => { await 0; })();",
        "void (async () => { await new Promise(() => {}); })();",
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program
            .script
            .as_ref()
            .unwrap()
            .module_entry_evaluation()
            .is_none());
    }
}
