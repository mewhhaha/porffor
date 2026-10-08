use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, EvalEnvironmentRoleIr, ExprIr, FunctionIr,
    IdentifierReferenceCaptureAccess, IdentifierReferenceCaptureDisposition,
    IdentifierReferenceFallbackDisposition, OrdinaryGeneratorSwitchIr, SpecOperationIr,
    StatementIr, TypedExpr,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("Switch region source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .unwrap()
}

fn walk<'a>(statements: &'a [StatementIr], found: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        found.push(statement);
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), found);
            }
            StatementIr::Block(block) => walk(&block.statements, found),
            StatementIr::LexicalBlock(statements) => walk(statements, found),
            StatementIr::Labelled { statement, .. } => {
                walk(std::slice::from_ref(statement.as_ref()), found);
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, found);
                }
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, found);
                walk(&plan.body().block().statements, found);
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                walk(&plan.then_branch().block().statements, found);
                walk(&plan.else_branch().block().statements, found);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, found);
                }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, found);
                walk(&plan.initialization().block().statements, found);
                walk(&plan.body().block().statements, found);
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                walk(&try_block.statements, found);
                walk(&catch_block.statements, found);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, found);
                walk(&finally_block.statements, found);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, found);
                walk(&catch_block.statements, found);
                walk(&finally_block.statements, found);
            }
            _ => {}
        }
    }
}

fn first_switch(function: &FunctionIr) -> &OrdinaryGeneratorSwitchIr {
    let mut statements = Vec::new();
    walk(&function.body.statements, &mut statements);
    statements
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorSwitch(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual ordinary generator Switch owner")
}

#[test]
fn switch_selection_and_fallthrough_have_complete_disjoint_source_regions() {
    let function = values("function* values() { switch ((yield 'd1', yield 'd2')) { case (yield 's1', yield 's2'): yield 'b1'; break; default: yield 'bd'; case yield 's3': yield 'b3'; } return 9; }");
    let plan = first_switch(&function);
    assert_eq!(
        (plan.entry_state(), plan.discriminant().region().end_state()),
        (0, 2)
    );
    assert_eq!(plan.case_block_entry_state(), 3);
    assert_eq!(plan.fallback_state(), 8);
    assert_eq!(plan.exit_state(), 15);
    assert_eq!(plan.cases().len(), 3);
    let ranges = plan
        .cases()
        .iter()
        .map(|case| {
            (
                case.selector().map(|selector| {
                    (
                        selector.region().entry_state(),
                        selector.region().end_state(),
                    )
                }),
                (case.body().entry_state(), case.body().end_state()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ranges,
        vec![
            (Some((3, 5)), (9, 10)),
            (None, (11, 12)),
            (Some((6, 7)), (13, 14))
        ]
    );
    assert!(function
        .owned_env_bindings
        .contains(plan.discriminant_binding()));
    assert!(function.owned_env_bindings.contains(plan.value_binding()));
    assert_ne!(plan.discriminant_binding().slot, plan.value_binding().slot);
    assert!(plan
        .regions()
        .all(|region| region.block().lexical_environment.is_none()));
    let source = function.generator_plan.as_ref().unwrap();
    assert_eq!(source.state_count, 16);
    assert_eq!(
        source
            .suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        vec![
            (0, 1),
            (1, 2),
            (3, 4),
            (4, 5),
            (6, 7),
            (9, 10),
            (11, 12),
            (13, 14)
        ]
    );
}

#[test]
fn switch_case_block_and_nested_empty_items_retain_actual_completion_and_cells() {
    let function = values("function* values(key) { outer: for (let i = 0; i < 2; i++) { switch (yield key) { case 0: let shared = yield 'shared'; const read = function () { return shared; }; { 6; const fixed = yield 'const'; if (key) var item = yield 'var'; try { let local = yield 'let'; yield read; } finally { yield 'finally'; } continue outer; } default: yield 'default'; break; } } }");
    let plan = first_switch(&function);
    assert!(
        plan.lexical_environment().is_some(),
        "captured shared CaseBlock binding owns one environment"
    );
    assert!(plan
        .cases()
        .iter()
        .all(|case| case.body().block().lexical_environment.is_none()));
    let mut body = Vec::new();
    for case in plan.cases() {
        walk(&case.body().block().statements, &mut body);
    }
    let empty_items = body
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::EmptyStatementCompletion(item) => Some(item.statement()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        empty_items.len(),
        5,
        "shared let/const plus nested const/var/let are exact source Empty items"
    );
    assert!(
        empty_items
            .iter()
            .any(|statement| matches!(statement, StatementIr::LexicalBlock(_))),
        "suspending declaration keeps its whole generated prefix inside the checked Empty item"
    );
    assert!(body.iter().any(|statement| matches!(statement, StatementIr::Continue { label: Some(label) } if label == "outer")));
    assert!(body.iter().any(|statement| matches!(
        statement,
        StatementIr::TryFinally {
            generator_plan: Some(_),
            ..
        }
    )));
    let ordinary = values("function* values() { var outside = yield 1; switch (0) { case 0: var eager = 2; break; } yield outside; }");
    let mut original = Vec::new();
    walk(&ordinary.body.statements, &mut original);
    assert!(
        !original
            .iter()
            .any(|statement| matches!(statement, StatementIr::EmptyStatementCompletion(_))),
        "outside the resumable Switch context retained generator output is unchanged"
    );
    assert!(original
        .iter()
        .any(|statement| matches!(statement, StatementIr::Switch { .. })));
}

#[test]
fn switch_admission_consumes_complete_operands_and_preserves_unowned_context_refusals() {
    for source in [
        "function* values(object) { switch ((yield 1) ? (yield 2, yield 3) : yield* object) { case (yield 4)?.[yield 5]: yield 6; break; default: yield 7; } }",
        "function* values() { label: switch (yield 1) { case (yield 2, yield 3): try { yield 4; break label; } finally { yield 5; } } }",
        "function* values() { switch (yield 1) { case 0: switch (yield 2) { case 1: yield 3; break; } yield 4; } }",
        "function* values() { switch (yield 1) { case 1: throw yield 2; } }",
    ] { let function = values(source); assert!(first_switch(&function).exit_state() > 0); }
    for source in [
        "function* values(items) { for (var item of items) { switch (yield item) { case 1: yield 2; } } }",
        "function* values(scope) { with (scope) { switch (yield 1) { case 1: yield 2; } } }",
    ] {
        let function = values(source);
        assert!(first_switch(&function).exit_state() > 0);
    }
}

fn captured_with_objects<'a>(mut value: &'a TypedExpr, name: &str) -> Vec<&'a str> {
    let mut objects = Vec::new();
    while let ExprIr::Conditional {
        condition,
        else_expr,
        ..
    } = &value.expr
    {
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::WithEnvironmentHasBinding,
            operands,
        } = &condition.expr
        else {
            break;
        };
        if !matches!(&operands[1].expr, ExprIr::String(key) if key == name) {
            return Vec::new();
        }
        let ExprIr::Identifier(object) = &operands[0].expr else {
            panic!("HasBinding must read the original captured object cell");
        };
        objects.push(object.as_str());
        value = else_expr;
    }
    objects
}

#[test]
fn escaped_switch_retains_captured_with_order_and_the_pre_rhs_reference() {
    let source = r#"
        var saved;
        with ({}) {
            with ({}) {
                saved = function* values(local) {
                    switch (yield 'discriminant') {
                        case (yield 'selector', key):
                            let held = 17;
                            const reader = () => [held, local];
                            yield value;
                            value = yield 'write';
                            yield delete victim;
                            yield local;
                            return reader;
                    }
                };
            }
        }
    "#;
    let parsed = parse(source, ParseOptions::script()).expect("captured With source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let mut root_statements = Vec::new();
    walk(&script.body.statements, &mut root_statements);
    let objects = root_statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Block(block) => block.lexical_environment.as_ref(),
            _ => None,
        })
        .filter_map(|environment| match &environment.eval_environment {
            Some(EvalEnvironmentRoleIr::WithObject { object_slot }) => Some(
                environment
                    .bindings
                    .iter()
                    .find(|binding| binding.slot == *object_slot)
                    .expect("the actual With record owns its hidden binding"),
            ),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        objects.len(),
        2,
        "two actual defining Object Environment Records"
    );
    let expected_order = objects
        .iter()
        .rev()
        .map(|binding| binding.name.as_str())
        .collect::<Vec<_>>();
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    for object in &objects {
        assert!(function
            .captured_bindings
            .iter()
            .any(|capture| capture.name == object.name && capture.slot == object.slot));
        assert!(
            !function
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == object.name),
            "invocation must retain the defining object, not allocate a replacement"
        );
    }
    let plan = first_switch(function);
    assert!(
        plan.lexical_environment().is_some(),
        "captured CaseBlock cell has a child record"
    );
    let selector = plan.cases()[0].selector().unwrap();
    assert!(selector.region().end_state() > selector.region().entry_state());
    let mut selector_statements = Vec::new();
    walk(
        &selector.region().block().statements,
        &mut selector_statements,
    );
    let selector_yield = selector_statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::GeneratorYield { value: TypedExpr { expr: ExprIr::String(value), .. }, .. }
            if value == "selector")
        })
        .unwrap();
    let selector_get = selector_statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { init, .. }
            if captured_with_objects(init, "key") == expected_order)
        })
        .unwrap();
    assert!(
        selector_yield < selector_get,
        "captured selector lookup follows its source suspension"
    );
    let mut body = Vec::new();
    walk(&plan.cases()[0].body().block().statements, &mut body);
    for name in ["value", "victim"] {
        assert!(
            body.iter().any(|statement| matches!(statement,
            StatementIr::GeneratorYield { value, .. }
                if captured_with_objects(value, name) == expected_order)),
            "{name} keeps ordered captured GetBindingValue/DeleteBinding resolution"
        );
    }
    let (capture_index, capture) = body
        .iter()
        .enumerate()
        .find_map(|(index, statement)| {
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(identifier),
                ..
            }) = statement
            else {
                return None;
            };
            match &identifier.operation {
                EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture }
                    if identifier.name == "value" =>
                {
                    Some((index, capture))
                }
                _ => None,
            }
        })
        .expect("the suspended write captures its Reference");
    assert_eq!(
        capture.access(),
        IdentifierReferenceCaptureAccess::WriteOnly
    );
    let IdentifierReferenceCaptureDisposition::WithObject {
        selection,
        fallback: IdentifierReferenceFallbackDisposition::Global,
    } = capture.disposition()
    else {
        panic!("captured objects must precede the original global fallback");
    };
    assert_eq!(captured_with_objects(selection, "value"), expected_order);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == capture.reference().storage_name()));
    let yield_index = body
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::GeneratorYield { value: TypedExpr { expr: ExprIr::String(value), .. }, .. }
            if value == "write")
        })
        .unwrap();
    let put_index = body.iter().position(|statement| matches!(statement,
        StatementIr::Expression(TypedExpr { expr: ExprIr::EnvironmentIdentifier(identifier), .. })
            if matches!(&identifier.operation, EnvironmentIdentifierOperationIr::PutCapturedReference { reference, .. }
                if reference == capture.reference()))).unwrap();
    assert!(capture_index < yield_index && yield_index < put_index);
    assert!(
        body.iter().any(|statement| matches!(statement,
        StatementIr::GeneratorYield { value: TypedExpr { expr: ExprIr::Identifier(name), .. }, .. }
            if name == &function.params[0].name)),
        "nearer parameters bypass captured With records"
    );
}
