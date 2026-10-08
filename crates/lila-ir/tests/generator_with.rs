use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, BindingMode, EvalEnvironmentRoleIr, ExprIr, FunctionIr, OrdinaryGeneratorWithIr,
    SpecOperationIr, StatementIr,
};

fn functions(source: &str) -> Vec<FunctionIr> {
    let program = lower(&parse(source, ParseOptions::script()).expect("With source parses"));
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program.script.unwrap().functions
}

fn walk<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::Block(block) => walk(&block.statements, output),
            StatementIr::LexicalBlock(statements) => walk(statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Labelled { statement, .. } => {
                walk(std::slice::from_ref(statement.as_ref()), output)
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                walk(&plan.then_branch().block().statements, output);
                walk(&plan.else_branch().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization_region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                walk(&plan.body().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, output);
                walk(&catch_block.statements, output);
                walk(&finally_block.statements, output);
            }
            _ => {}
        }
    }
}

fn with_plans(function: &FunctionIr) -> Vec<&OrdinaryGeneratorWithIr> {
    let mut rows = Vec::new();
    walk(&function.body.statements, &mut rows);
    rows.into_iter()
        .filter_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorWith(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .collect()
}

#[test]
fn with_head_conversion_and_body_have_disjoint_complete_source_regions() {
    let functions = functions("function* values() { with (yield 'head') { yield value; } }");
    let function = functions.iter().find(|f| f.name == "values").unwrap();
    let plan = with_plans(function)[0];
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 1, 2, 3, 4)
    );
    assert!(plan.head().region().block().lexical_environment.is_none());
    assert!(plan.body().block().lexical_environment.is_none());
    assert!(function.owned_env_bindings.contains(plan.head_binding()));
    let StatementIr::Lexical {
        mode: BindingMode::Let,
        name,
        init,
    } = plan.head().region().block().statements.last().unwrap()
    else {
        panic!("head publishes actual ToObject result");
    };
    assert_eq!(name, &plan.head_binding().name);
    assert!(
        matches!(&init.expr, ExprIr::SpecOperation { operation: SpecOperationIr::ToObject, operands } if operands.len() == 1)
    );
    assert!(
        matches!(&plan.head().value().expr, ExprIr::Identifier(name) if name == &plan.head_binding().name)
    );
    assert_eq!(plan.head().value().value_info(), init.value_info());
    let environment = plan.lexical_environment();
    assert_eq!(
        environment.bindings.as_slice(),
        std::slice::from_ref(plan.object_binding())
    );
    assert!(
        matches!(&environment.eval_environment, Some(EvalEnvironmentRoleIr::WithObject { object_slot }) if *object_slot == plan.object_binding().slot)
    );
    let points = &function.generator_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        points
            .iter()
            .map(|p| (p.suspend_state, p.resume_state))
            .collect::<Vec<_>>(),
        [(0, 1), (2, 3)]
    );
}

#[test]
fn eager_with_materializes_original_hidden_record_for_escaping_closures() {
    let functions = functions("var value=9; function* values(scope) { var read; with(scope) { let local=2; read=function(){return [value,local];}; } return read; }");
    let function = functions.iter().find(|f| f.name == "values").unwrap();
    let plan = with_plans(function)[0];
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 0, 1, 1, 2)
    );
    assert!(function
        .generator_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .is_empty());
    let hidden = plan.object_binding();
    assert!(
        functions.iter().any(|f| f
            .captured_bindings
            .iter()
            .any(|b| b.name == hidden.name && b.slot == hidden.slot)),
        "escaping closure retains original With binding cell"
    );
    assert!(
        !function
            .owned_env_bindings
            .iter()
            .any(|b| b.name == hidden.name),
        "hidden row belongs to child With environment, not the invocation record"
    );
}

#[test]
fn with_body_consumes_nested_control_and_preserves_foreign_owner_boundaries() {
    let compiled = functions("function* values(scope) { outer: for(let i=0;i<2;i++){ with(scope){ switch(yield 'd'){ case yield 's': with(scope){ if(yield 'if') continue outer; } break; default: try { yield value; } finally { yield 'finally'; } } } } }");
    let function = compiled.iter().find(|f| f.name == "values").unwrap();
    let plans = with_plans(function);
    assert_eq!(plans.len(), 2);
    assert_ne!(
        plans[0].object_binding().name,
        plans[1].object_binding().name
    );
    assert!(plans
        .iter()
        .all(|p| p.body().block().lexical_environment.is_none()));
    let labelled =
        functions("function* values(scope){ stop: with(scope){ yield value; break stop; } }");
    assert_eq!(
        with_plans(labelled.iter().find(|f| f.name == "values").unwrap()).len(),
        1
    );
    let eager = functions("function* values(scope,choice){if(choice){with(scope){value;}}else{switch(choice){case 0:with(scope){value;}break;default:break;}}}");
    let eager = eager.iter().find(|f| f.name == "values").unwrap();
    assert_eq!(
        with_plans(eager).len(),
        2,
        "eager enclosing selection consumes both actual With phase owners"
    );
    assert!(eager
        .generator_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .is_empty());
    let iterator_body =
        functions("function* values(view){for(let item of [1]){with(view){item;}yield item;}}");
    assert_eq!(
        with_plans(iterator_body.iter().find(|f| f.name == "values").unwrap()).len(),
        1,
        "the complete iterator body retains its original checked With record"
    );
    let plain_async = lower(
        &parse(
            "async function values(scope){with(await scope){await value;}}",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(
        plain_async.is_wasm_supported(),
        "plain Async has its own complete With owner: {:?}",
        plain_async.diagnostics
    );
    for source in [
        "async function* values(scope){with(yield scope){yield value;}}",
        "async function* values(scope){with(scope){yield ()=>value;}}",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert!(program
            .script
            .unwrap()
            .functions
            .iter()
            .any(|function| function.name == "values"
                && function
                    .body
                    .statements
                    .iter()
                    .any(|statement| matches!(statement, StatementIr::AsyncGeneratorWith(_)))));
    }
    for source in
        ["function* values(scope){for(const item of [1]){with(yield scope){yield item;}}}"]
    {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "the complete iterator owns its suspended With body: {source}: {:?}",
            program.diagnostics
        );
        let functions = program.script.unwrap().functions;
        let function = functions.iter().find(|f| f.name == "values").unwrap();
        assert_eq!(with_plans(function).len(), 1);
        let mut rows = Vec::new();
        walk(&function.body.statements, &mut rows);
        assert!(rows.iter().any(|row| matches!(row,
            StatementIr::AsyncGeneratorForOf(plan)
                if plan.execution() == lila_ir::ResumableRegionProtocolIr::Generator)));
    }
    assert!(parse(
        "'use strict'; function* values(scope){with(scope){yield value;}}",
        ParseOptions::script()
    )
    .is_err());
}
