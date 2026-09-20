use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, BindingMode, EnvironmentIdentifierOperationIr, ExprIr, ForInitIr, ForOfIteratorHeadIr,
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
        let mut units = script
            .prepared_script_units()
            .filter(|unit| matches!(unit.kind, PreparedScriptKind::DirectEval(_)));
        let unit = units.next().expect("prepared direct eval unit");
        assert!(units.next().is_none(), "one direct eval context");
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
    let mut units = script
        .prepared_script_units()
        .filter(|unit| matches!(unit.kind, PreparedScriptKind::DirectEval(_)));
    let unit = units.next().expect("prepared direct eval unit");
    assert!(units.next().is_none(), "one direct eval context");
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
    for (source, direct) in [
        (
            "eval('\"use strict\"; for (var value of [7, 9]) { value; }');",
            true,
        ),
        ("(0, eval)('for (var value of [7, 9]) { value; }');", false),
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let mut units = script
            .prepared_script_units()
            .filter(|unit| match unit.kind {
                PreparedScriptKind::DirectEval(_) => direct,
                PreparedScriptKind::IndirectEval => !direct,
                PreparedScriptKind::RealmScript => false,
            });
        let unit = units
            .next()
            .expect("prepared eval unit of the intended kind");
        assert!(
            units.next().is_none(),
            "one eval context of the intended kind"
        );
        assert_eq!(unit.strict, direct);
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

#[test]
fn borrowed_classic_for_head_publishes_declarations_in_source_order() {
    let program = lower(
        &parse(
            "eval('for (var first = 7, untouched, second = first;;) { break; }');",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script
        .prepared_script_units()
        .find(|unit| matches!(unit.kind, PreparedScriptKind::DirectEval(_)))
        .expect("prepared direct eval unit");
    assert_eq!(
        unit.declarations.var_names,
        ["first", "untouched", "second"]
    );
    let StatementIr::For {
        init: Some(ForInitIr::Statements(statements)),
        ..
    } = &unit.body.statements[0]
    else {
        panic!("borrowed classic heads must use declaration evaluation");
    };
    let [StatementIr::LexicalBlock(declarations)] = statements.as_slice() else {
        panic!("initialized declarations must retain their source order");
    };
    assert_eq!(declarations.len(), 2, "uninitialized var performs no write");
    let (first, value) = environment_assignment(&declarations[0]);
    assert_eq!(first, "first");
    assert!(matches!(value.expr, ExprIr::Number(bits) if bits == 7f64.to_bits()));
    let (second, value) = environment_assignment(&declarations[1]);
    assert_eq!(second, "second");
    assert!(matches!(
        &value.expr,
        ExprIr::EnvironmentIdentifier(reference)
            if reference.name == "first"
                && matches!(reference.operation, EnvironmentIdentifierOperationIr::Read)
    ));
}

#[test]
fn borrowed_classic_for_without_an_initializer_only_instantiates_its_var() {
    let program = lower(
        &parse(
            "eval('for (var value; false;) {}');",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script
        .prepared_script_units()
        .find(|unit| matches!(unit.kind, PreparedScriptKind::DirectEval(_)))
        .expect("prepared direct eval unit");
    assert_eq!(unit.declarations.var_names, ["value"]);
    let StatementIr::For {
        init: Some(ForInitIr::Statements(statements)),
        ..
    } = &unit.body.statements[0]
    else {
        panic!("borrowed classic heads must use declaration evaluation");
    };
    assert!(
        matches!(statements.as_slice(), [StatementIr::LexicalBlock(declarations)] if declarations.is_empty())
    );
}

#[test]
fn owned_classic_for_heads_keep_their_declared_storage() {
    for (source, direct) in [
        (
            "eval('\"use strict\"; for (var value = 7; false;) {}');",
            true,
        ),
        ("(0, eval)('for (var value = 7; false;) {}');", false),
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let unit = script
            .prepared_script_units()
            .find(|unit| match unit.kind {
                PreparedScriptKind::DirectEval(_) => direct,
                PreparedScriptKind::IndirectEval => !direct,
                PreparedScriptKind::RealmScript => false,
            })
            .expect("prepared eval unit of the intended kind");
        let statement = unit
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::For { .. }))
            .expect("prepared eval retains its loop");
        let StatementIr::For {
            init: Some(ForInitIr::Var(declarations)),
            ..
        } = statement
        else {
            panic!("owned classic head retains its declaration storage");
        };
        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0].name, "value");
        assert!(
            matches!(declarations[0].init.as_ref().map(|value| &value.expr), Some(ExprIr::Number(bits)) if *bits == 7f64.to_bits())
        );
    }
}
