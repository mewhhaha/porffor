use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorWithIr, BindingMode, EvalEnvironmentRoleIr, ExprIr, FunctionIr,
    ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, SpecOperationIr, StatementIr,
};

fn functions(source: &str) -> Vec<FunctionIr> {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program.script.unwrap().functions
}

fn walk<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a AsyncGeneratorWithIr>) {
    for statement in statements {
        match statement {
            StatementIr::AsyncGeneratorWith(plan) => {
                output.push(plan);
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                walk(&plan.body().block().statements, output)
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::Block(block) => walk(&block.statements, output),
            StatementIr::LexicalBlock(statements) => walk(statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Labelled { statement, .. } => {
                walk(std::slice::from_ref(statement.as_ref()), output)
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

#[test]
fn mixed_with_publishes_to_object_once_before_disjoint_head_and_body_regions() {
    let functions =
        functions("async function* values(){with(await (yield 'head')){yield value;await 0;}}");
    let function = functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    let plan = plans[0];
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 2, 3, 5, 6)
    );
    assert!(plan.head().region().block().lexical_environment.is_none());
    assert!(plan.body().block().lexical_environment.is_none());
    let StatementIr::Lexical {
        mode: BindingMode::Let,
        name,
        init,
    } = plan.head().region().block().statements.last().unwrap()
    else {
        panic!("actual ToObject publication");
    };
    assert_eq!(name, &plan.head_binding().name);
    assert!(
        matches!(&init.expr, ExprIr::SpecOperation { operation: SpecOperationIr::ToObject, operands } if operands.len() == 1)
    );
    assert!(
        matches!(&plan.head().value().expr, ExprIr::Identifier(name) if name == &plan.head_binding().name)
    );
    assert_eq!(init.value_info(), plan.head().value().value_info());
    assert!(function.owned_env_bindings.contains(plan.head_binding()));
    assert!(!function.owned_env_bindings.contains(plan.object_binding()));
    assert_eq!(
        plan.lexical_environment().bindings.as_slice(),
        std::slice::from_ref(plan.object_binding())
    );
    assert!(matches!(&plan.lexical_environment().eval_environment,
        Some(EvalEnvironmentRoleIr::WithObject { object_slot }) if *object_slot == plan.object_binding().slot));
    let source = function.resumable_plan.as_ref().unwrap();
    assert_eq!(source.state_count, 7);
    assert_eq!(
        source
            .suspension_points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 0, 1),
            (ResumableSuspensionKindIr::Await, 1, 2),
            (ResumableSuspensionKindIr::Yield, 3, 4),
            (ResumableSuspensionKindIr::Await, 4, 5),
        ]
    );
    assert!(source
        .suspension_points
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert_eq!(
        source
            .resume_environment_plan()
            .enclosing_scope_resume_states(),
        [4, 5]
    );
}

#[test]
fn eager_mixed_with_retains_original_hidden_record_and_nested_block_cells() {
    let functions = functions("var value=9;async function* values(scope){var read;with(scope){let local=2;read=function(){return [value,local];};}return read;}");
    let function = functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    let plan = plans[0];
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
    let StatementIr::Block(block) = &plan.body().block().statements[0] else {
        panic!("original nested source Block");
    };
    assert!(block.lexical_environment.is_some());
    let hidden = plan.object_binding();
    assert!(functions.iter().any(|function| function
        .captured_bindings
        .iter()
        .any(|binding| binding.name == hidden.name && binding.slot == hidden.slot)));
    assert!(!function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == hidden.name));
}

#[test]
fn mixed_with_body_consumes_nested_complete_switch_case_records() {
    let functions = functions("async function* values(scope,key){with(scope){switch(await (yield key)){case await key:with(nested){yield value;}break;default:yield value;}}}");
    let function = functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    assert_eq!(
        plans.len(),
        2,
        "both original With environments are real analyzed records"
    );
    assert_ne!(
        plans[0].object_binding().name,
        plans[1].object_binding().name
    );
    assert!(plans
        .iter()
        .all(|plan| plan.head().region().end_state() + 1 == plan.body().entry_state()));
    let source = function.resumable_plan.as_ref().unwrap();
    assert!(source
        .suspension_points
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert!(!source
        .resume_environment_plan()
        .enclosing_scope_resume_states()
        .is_empty());
}

#[test]
fn nested_mixed_with_controls_own_eager_and_suspended_phases_and_distinct_records() {
    let compiled = functions("async function* values(scope,other){outer:for(let i=0;i<2;i++){with(scope){if(await (yield 'choice')){with(other){yield value;}}else{try{yield value;}finally{await 0;}}}}}");
    let function = compiled
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 2);
    assert_ne!(
        plans[0].object_binding().name,
        plans[1].object_binding().name
    );
    assert!(plans
        .iter()
        .all(|plan| plan.body().block().lexical_environment.is_none()));
    let eager = functions("async function* values(scope,choice){if(choice){with(scope){value;}}else{stop:with(scope){value;break stop;}}}");
    let function = eager
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 2);
    assert!(function
        .resumable_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .is_empty());
}

#[test]
fn mixed_with_preserves_foreign_boundaries_and_strict_early_errors() {
    for source in [
        "async function* values(scope,input){with(scope){for await(const item of input){item;}}}",
        "async function* values(scope,input){with(scope){for(const item of input){yield item;}}}",
        "async function* values(scope){for(const item of [1]){with(yield scope){yield item;}}}",
        "async function* values(scope){with(scope){if(true){using resource=null;yield value;}}}",
        "async function* values(scope){with(scope){for(let i=0;i<1;i++){await using resource=null;yield value;}}}",
        "async function* values(scope){with(scope){for(using resource of [null]){resource;}}}",
    ] { let functions = functions(source); assert!(functions.iter().any(|function| { let mut plans=Vec::new(); walk(&function.body.statements,&mut plans); !plans.is_empty() })); }
    let functions = functions(
        "async function* values(scope){for(const item of [1]){with(scope){item;}yield item;}}",
    );
    let mut plans = Vec::new();
    for function in functions
        .iter()
        .filter(|function| function.name == "values")
    {
        walk(&function.body.statements, &mut plans);
    }
    assert_eq!(
        plans.len(),
        1,
        "accepted iterator keeps actual nested With source ownership"
    );
    assert!(parse(
        "'use strict';async function* values(scope){with(scope){yield value;}}",
        ParseOptions::script()
    )
    .is_err());
}

#[test]
fn mixed_for_in_body_preserves_the_original_nested_with_record() {
    let functions = functions("async function* values(items,scope){for(const key in await (yield items)){with(await scope){yield key;await value;}}}");
    let function = functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let mut plans = Vec::new();
    walk(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 1);
    let StatementIr::AsyncGeneratorForIn(outer) = &function.body.statements[0] else {
        panic!("checked complete enumeration");
    };
    assert!(plans[0].entry_state() >= outer.body().entry_state());
    assert!(plans[0].exit_state() <= outer.body().end_state());
    assert_eq!(
        plans[0].lexical_environment().bindings.as_slice(),
        std::slice::from_ref(plans[0].object_binding())
    );
}
