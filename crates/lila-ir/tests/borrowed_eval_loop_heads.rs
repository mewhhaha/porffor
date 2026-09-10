use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, BindingMode, EnvironmentIdentifierOperationIr, ExprIr, ForOfIteratorHeadIr,
    PreparedScriptKind, StatementIr, TypedExpr,
};

fn loop_head(statement: &StatementIr) -> (BindingMode, &str, &StatementIr) {
    match statement {
        StatementIr::ForOfIterator {
            head: ForOfIteratorHeadIr::Assignment { binding, .. },
            body,
            ..
        } => (binding.mode, &binding.name, body),
        StatementIr::ForInObject {
            mode, name, body, ..
        } => (*mode, name, body),
        _ => panic!("expected an ordinary loop: {statement:?}"),
    }
}

fn environment_assignment(statement: &StatementIr) -> (&str, &TypedExpr) {
    let StatementIr::DeclarationEvaluation(expression) = statement else {
        panic!("loop binding evaluation must preserve the prior completion: {statement:?}");
    };
    let ExprIr::EnvironmentIdentifier(reference) = &expression.expr else {
        panic!("borrowed var must resolve through the caller environment");
    };
    let EnvironmentIdentifierOperationIr::Assign { value } = &reference.operation else {
        panic!("loop head must assign its current value");
    };
    (&reference.name, value)
}

#[test]
fn borrowed_eval_loop_heads_publish_each_value_before_the_body() {
    for source in [
        "eval('for (var value of [7, 9]) { value; }');",
        "eval('for (var value in {a: 1, b: 2}) { value; }');",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let unit = script.prepared_script_units().next().unwrap();
        assert!(matches!(unit.kind, PreparedScriptKind::DirectEval(_)));
        assert_eq!(unit.declarations.var_names, ["value"]);
        let (mode, name, body) = loop_head(&unit.body.statements[0]);
        assert_eq!(mode, BindingMode::Let);
        assert_ne!(name, "value", "iteration storage is private to the loop");
        let StatementIr::Block(body) = body else {
            panic!("binding publication must prefix the original loop body");
        };
        assert_eq!(body.statements.len(), 2);
        let (source_name, value) = environment_assignment(&body.statements[0]);
        assert_eq!(source_name, "value");
        assert!(matches!(&value.expr, ExprIr::Identifier(storage) if storage == name));
        let StatementIr::Block(original_body) = &body.statements[1] else {
            panic!("the original body must remain after binding publication");
        };
        assert!(matches!(
            &original_body.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(reference),
                ..
            }) if reference.name == "value"
                && matches!(reference.operation, EnvironmentIdentifierOperationIr::Read)
        ));
    }
}

#[test]
fn borrowed_for_in_initializer_also_uses_the_caller_environment() {
    let program = lower(
        &parse(
            "eval('for (var value = 5 in {}) {}');",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script.prepared_script_units().next().unwrap();
    let StatementIr::Block(sequence) = &unit.body.statements[0] else {
        panic!("Annex B initializer must precede enumeration");
    };
    let (name, value) = environment_assignment(&sequence.statements[0]);
    assert_eq!(name, "value");
    assert!(matches!(value.expr, ExprIr::Number(bits) if bits == 5f64.to_bits()));
    assert_eq!(loop_head(&sequence.statements[1]).0, BindingMode::Let);
}

#[test]
fn owned_eval_loop_heads_keep_their_declared_storage() {
    for source in [
        "eval('\"use strict\"; for (var value of [7, 9]) { value; }');",
        "(0, eval)('for (var value of [7, 9]) { value; }');",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let unit = script.prepared_script_units().next().unwrap();
        let statement = unit
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::ForOfIterator { .. }))
            .expect("prepared eval retains its loop");
        let (mode, name, _) = loop_head(statement);
        assert_eq!(mode, BindingMode::Var);
        assert_eq!(name, "value");
    }
}
