use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArrayDestructuringOperationView, BindingMode, EnvironmentIdentifierOperationIr, ExprIr,
    FunctionIr, StatementIr,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("array pattern parses");
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
        .find(|f| f.name == "values")
        .unwrap()
}

fn rows<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::LexicalBlock(body) => rows(body, output),
            StatementIr::Block(body) => rows(&body.statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                rows(&plan.body().block().statements, output)
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
            StatementIr::AsyncGeneratorForOf(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization().block().statements, output);
                rows(&plan.body().block().statements, output);
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
fn array_pattern_owns_acquisition_complete_body_and_lazy_default_states() {
    let function = values("function* values(input) { var value; return ([, value = (yield 'first', yield 'second')] = input); }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|s| match s {
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 1, 6, 7)
    );
    let ExprIr::Identifier(raw) = &plan.raw_source().expr else {
        panic!("retained original RHS")
    };
    assert_ne!(raw, &plan.storage().binding().name);
    for name in [raw, &plan.storage().binding().name] {
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|b| &b.name == name)
                .count(),
            1
        );
    }
    let body = &plan.body().block().statements;
    let at = |predicate: &dyn Fn(&StatementIr) -> bool| body.iter().position(predicate).unwrap();
    let elision = at(
        &|s| matches!(s, StatementIr::ArrayDestructuringOperation(op) if matches!(op.use_view(), ArrayDestructuringOperationView::Elision(_))),
    );
    let capture = at(
        &|s| matches!(expression(s).map(|e|&e.expr), Some(ExprIr::EnvironmentIdentifier(reference)) if matches!(reference.operation, EnvironmentIdentifierOperationIr::CaptureAssignmentReference{..})),
    );
    let step = at(
        &|s| matches!(s, StatementIr::ArrayDestructuringOperation(op) if matches!(op.use_view(), ArrayDestructuringOperationView::StepValue(_))),
    );
    let default = at(&|s| matches!(s, StatementIr::OrdinaryGeneratorIf(_)));
    let put = at(
        &|s| matches!(expression(s).map(|e|&e.expr), Some(ExprIr::EnvironmentIdentifier(reference)) if matches!(reference.operation, EnvironmentIdentifierOperationIr::PutCapturedReference{..})),
    );
    assert!(elision < capture && capture < step && step < default && default < put);
    let StatementIr::ArrayDestructuringOperation(operation) = &body[step] else {
        panic!("the checked iterator operation publishes one selected cell");
    };
    let result = operation.result_binding().unwrap();
    assert_ne!(result.name, plan.storage().binding().name);
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|b| *b == result)
            .count(),
        1
    );
    let points = &function.generator_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        points
            .iter()
            .map(|p| (p.suspend_state, p.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (3, 4)]
    );
}

#[test]
fn nested_arrays_have_distinct_iterator_cells_and_for_heads_keep_original_bindings() {
    let function = values("function* values(input) { var chosen; return ([{inner: [chosen = yield 'nested']}]=input); }");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let slots = ordered
        .iter()
        .filter_map(|s| match s {
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                Some(plan.storage().binding().slot)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        slots.len(),
        2,
        "each acquired iterator has its own native edge"
    );
    let function=values("function* values(input) { for(let [i = yield 'initial', step] = input; i < 2; i++) { yield () => [i,step]; } }");
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|s| match s {
            StatementIr::OrdinaryGeneratorLoop(p) => Some(p),
            _ => None,
        })
        .unwrap();
    let head = plan.lexical_environment().unwrap();
    ordered.clear();
    rows(
        &plan.initialization().unwrap().block().statements,
        &mut ordered,
    );
    let mut bindings = Vec::new();
    for statement in ordered {
        if let StatementIr::DeclarationEvaluation(value) = statement {
            if let ExprIr::ObjectDestructuringOperation(operation) = &value.expr {
                operation.visit_bindings(&mut |mode, name| bindings.push((mode, name.to_owned())));
            }
        }
    }
    assert_eq!(bindings.len(), 2);
    for (mode, name) in bindings {
        assert_eq!(mode, BindingMode::Let);
        let binding = head.bindings.iter().find(|b| b.name == name).unwrap();
        assert!(head.per_iteration_slots.contains(&binding.slot));
    }
}

#[test]
fn array_patterns_admit_actual_targets_nested_defaults_and_rest_without_foreign_owners() {
    for source in [
        "function* values(input,object) { return ([(yield object)[yield 'key'] = yield 'default'] = input); }",
        "function* values(input,object) { return ([...object[yield 'key']] = input); }",
        "function* values(input) { var value; return ([...[value = yield 'default']] = input); }",
        "function* values(input) { let [{[yield 'key']: inner = yield* input}] = input; return inner; }",
        "function* values(scope,input) { var value; with(scope) { ([value = yield 'default'] = input); } return value; }",
        "async function values(input) { let [value = await 1] = input; }",
        "async function* values(input) { let [value = yield 1] = input; }",
    ] { values(source); }
    let function = values(
        "function* values(items) { for(var item of items) { let [value = yield 1] = item; } }",
    );
    let [StatementIr::AsyncGeneratorForOf(iterator)] = function.body.statements.as_slice() else {
        panic!("the Array pattern belongs to the complete iterator body");
    };
    assert!(iterator.initialization().end_state() < iterator.body().entry_state());
    let mut body = Vec::new();
    rows(&iterator.body().block().statements, &mut body);
    assert!(body.iter().any(|statement| matches!(
        statement,
        StatementIr::OrdinaryGeneratorArrayDestructuring(_)
    )));
    let source = "class Parent {} class Child extends Parent { *values(input) { [super[yield 1]] = input; } }";
    let parsed = parse(source, ParseOptions::script()).expect("Super pattern continuation parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
}
