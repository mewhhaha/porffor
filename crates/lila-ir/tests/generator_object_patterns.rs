use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, BindingMode, DestructuringTargetIr, EnvironmentIdentifierOperationIr, ExprIr,
    FunctionIr, ObjectDestructuringOperationView, SpecOperationIr, StatementIr,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("object pattern parses");
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
        .expect("ordinary generator")
}

fn rows<'a>(source: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in source {
        output.push(statement);
        match statement {
            StatementIr::LexicalBlock(body) => rows(body, output),
            StatementIr::Block(body) => rows(&body.statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                rows(&plan.then_branch().block().statements, output);
                rows(&plan.else_branch().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    rows(&region.block().statements, output);
                }
            }
            _ => {}
        }
    }
}

fn expression(statement: &StatementIr) -> Option<&lila_ir::TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. } => Some(init),
        StatementIr::Expression(value)
        | StatementIr::DeclarationEvaluation(value)
        | StatementIr::Return(value) => Some(value),
        _ => None,
    }
}

#[test]
fn object_pattern_acquisition_reference_and_lazy_defaults_have_exact_source_order() {
    let function = values("function* values(rhs) { var chosen, rest; return ({[yield 'key']: chosen = (yield 'first', yield 'second'), ...rest} = rhs); }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let index = |predicate: &dyn Fn(&StatementIr) -> bool| {
        ordered
            .iter()
            .position(|statement| predicate(statement))
            .expect("semantic source event")
    };
    let boxed = index(&|statement| {
        matches!(
            expression(statement).map(|v| &v.expr),
            Some(ExprIr::SpecOperation {
                operation: SpecOperationIr::ToObject,
                ..
            })
        )
    });
    let key_yield = index(&|statement| {
        matches!(statement, StatementIr::GeneratorYield {
        value, .. } if matches!(&value.expr, ExprIr::String(name) if name == "key"))
    });
    let key_conversion = index(&|statement| {
        matches!(
            expression(statement).map(|v| &v.expr),
            Some(ExprIr::SpecOperation {
                operation: SpecOperationIr::ToPropertyKey,
                ..
            })
        )
    });
    let capture = index(&|statement| {
        matches!(expression(statement).map(|v| &v.expr),
        Some(ExprIr::EnvironmentIdentifier(reference)) if matches!(reference.operation,
            EnvironmentIdentifierOperationIr::CaptureAssignmentReference {..}))
    });
    let get = index(&|statement| {
        matches!(expression(statement).map(|v| &v.expr),
        Some(ExprIr::ObjectDestructuringOperation(operation)) if matches!(operation.use_view(),
            ObjectDestructuringOperationView::GetV {..}))
    });
    let default = index(&|statement| matches!(statement, StatementIr::OrdinaryGeneratorIf(_)));
    let put = index(&|statement| {
        matches!(expression(statement).map(|v| &v.expr),
        Some(ExprIr::EnvironmentIdentifier(reference)) if matches!(reference.operation,
            EnvironmentIdentifierOperationIr::PutCapturedReference {..}))
    });
    let rest = index(&|statement| {
        matches!(expression(statement).map(|v| &v.expr),
        Some(ExprIr::ObjectDestructuringOperation(operation)) if matches!(operation.use_view(),
            ObjectDestructuringOperationView::Rest {..}))
    });
    assert!(boxed < key_yield && key_yield < key_conversion && key_conversion < capture);
    assert!(capture < get && get < default && default < put && put < rest);
    let StatementIr::OrdinaryGeneratorIf(plan) = ordered[default] else {
        unreachable!()
    };
    assert_eq!(
        (
            plan.entry_state(),
            plan.then_branch().entry_state(),
            plan.then_branch().end_state(),
            plan.else_branch().entry_state(),
            plan.else_branch().end_state(),
            plan.exit_state()
        ),
        (1, 2, 4, 5, 5, 6)
    );
    let suspension_points = &function
        .generator_plan
        .as_ref()
        .expect("whole source plan")
        .suspension_points;
    assert_eq!(
        suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        vec![(0, 1), (2, 3), (3, 4)]
    );
    for statement in ordered {
        if let Some(lila_ir::TypedExpr {
            expr: ExprIr::ObjectDestructuringOperation(operation),
            ..
        }) = expression(statement)
        {
            operation.visit_expressions(&mut |operand| {
                if let ExprIr::Identifier(name) = &operand.expr {
                    assert!(function
                        .owned_env_bindings
                        .iter()
                        .any(|binding| &binding.name == name));
                }
            });
        }
    }
}

#[test]
fn object_pattern_member_put_keeps_raw_keys_and_lexical_for_cells_are_original() {
    let function = values("function* values(source, object) { return ({x: (yield object)[yield 'raw-key'] = yield 'default'} = source); }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let target = ordered
        .iter()
        .filter_map(|statement| expression(statement))
        .find_map(|value| match &value.expr {
            ExprIr::ObjectDestructuringOperation(operation) => match operation.use_view() {
                ObjectDestructuringOperationView::PutTarget { target, .. } => Some(target),
                _ => None,
            },
            _ => None,
        })
        .expect("actual deferred Put consumer");
    let DestructuringTargetIr::AssignmentProperty { target, key, .. } = target else {
        panic!("real Member target")
    };
    assert!(matches!(&target.expr, ExprIr::Identifier(_)));
    assert!(
        matches!(key,lila_ir::DestructuringPropertyKeyIr::Computed(value)
        if matches!(&value.expr,ExprIr::Identifier(_)))
    );
    let function = values("function* values(source) { for (let {[yield 'key']: i = yield 'default', box: {step}} = source; i < 2; i++) { yield () => [i, step]; } }");
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorLoop(plan) => Some(plan),
            _ => None,
        })
        .expect("actual classic For");
    let environment = plan
        .lexical_environment()
        .expect("actual captured head record");
    ordered.clear();
    rows(
        &plan.initialization().unwrap().block().statements,
        &mut ordered,
    );
    let mut targets = Vec::new();
    for statement in ordered {
        if let StatementIr::DeclarationEvaluation(value) = statement {
            if let ExprIr::ObjectDestructuringOperation(operation) = &value.expr {
                operation.visit_bindings(&mut |mode, name| targets.push((mode, name.to_owned())));
            }
        }
    }
    assert_eq!(targets.len(), 2);
    for (mode, name) in targets {
        assert_eq!(mode, BindingMode::Let);
        let binding = environment
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .expect("Initialize and capture share the exact head cell");
        assert!(environment.per_iteration_slots.contains(&binding.slot));
    }
}

#[test]
fn object_owned_suspension_uses_complete_defaults_and_preserves_iterator_boundaries() {
    for source in [
        "function* values(source) { let {[yield 1]: first = (yield 2, yield 3), inner: {[yield 4]: later = yield* source}, ...rest} = source; return [first,later,rest]; }",
        "function* values(source) { const {[yield 1]: first = function(){}, array: [later]} = source; return [first,later]; }",
        "function* values(source,object) { ({x: object[yield 1] = yield 2, ...object[yield 3]} = source); return object; }",
        "function* values(source) { var {[yield 1]: first = yield 2} = source; return first; }",
        "function* values(scope,source) { var x; with(scope) { ({x: x = yield 1} = source); } return x; }",
    ] { values(source); }
    for source in [
        "function* values(source) { let [first = yield 1] = source; }",
        "function* values(source) { let {[yield 1]: [first = yield 2]} = source; }",
        "function* values(items) { for (var item of items) { let {[yield 1]: first} = item; } }",
        "async function* values(source) { let {[yield 1]: first} = source; }",
        "class Parent {} class Child extends Parent { *values(source) { ({x: super[yield 1]} = source); } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("complete pattern parses");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
}
