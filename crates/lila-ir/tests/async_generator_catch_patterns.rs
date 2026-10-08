use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, FunctionIr, ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, StatementIr,
};

fn values(source: &str) -> FunctionIr {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
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

fn contains_array(items: &[StatementIr]) -> bool {
    items.iter().any(|item| match item {
        StatementIr::AsyncGeneratorArrayDestructuring(_) => true,
        StatementIr::EmptyStatementCompletion(item) => {
            contains_array(std::slice::from_ref(item.statement()))
        }
        StatementIr::LexicalBlock(items) => contains_array(items),
        StatementIr::Block(block) => contains_array(&block.statements),
        _ => false,
    })
}

fn contains_value_yield(items: &[StatementIr]) -> bool {
    items.iter().any(|item| match item {
        StatementIr::GeneratorYield { value, .. } => !matches!(value.expr, ExprIr::Undefined),
        StatementIr::EmptyStatementCompletion(item) => {
            contains_value_yield(std::slice::from_ref(item.statement()))
        }
        StatementIr::LexicalBlock(items) => contains_value_yield(items),
        StatementIr::Block(block) => contains_value_yield(&block.statements),
        _ => false,
    })
}

#[test]
fn mixed_catch_binding_retains_original_thrown_cell_parameter_and_body_records() {
    let function = values("async function* values(input){let outside='outside';try{throw input;}catch({[await(yield 'key')]:received=await(yield ()=>received)}){let outside='body';yield ()=>[outside,received];}}");
    let StatementIr::TryCatch {
        catch_name,
        catch_parameter_environment,
        catch_block,
        generator_plan: Some(plan),
        ..
    } = function
        .body
        .statements
        .iter()
        .find(|item| matches!(item, StatementIr::TryCatch { .. }))
        .unwrap()
    else {
        panic!("original Try/catch graph");
    };
    let thrown = function
        .owned_env_bindings
        .iter()
        .find(|binding| &binding.name == catch_name)
        .expect("original thrown value survives catch parameter suspension");
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == catch_name)
            .count(),
        1
    );
    let parameter = catch_parameter_environment
        .as_ref()
        .expect("original captured parameter record");
    assert!(parameter
        .bindings
        .iter()
        .all(|binding| binding.name != thrown.name));
    let [StatementIr::EmptyStatementCompletion(initialization), StatementIr::Block(body)] =
        catch_block.statements.as_slice()
    else {
        panic!("whole Empty initialization before body environment");
    };
    assert!(matches!(
        initialization.statement(),
        StatementIr::LexicalBlock(_)
    ));
    let body_environment = body
        .lexical_environment
        .as_ref()
        .expect("separate captured catch body record");
    assert!(parameter.bindings.iter().all(|binding| body_environment
        .bindings
        .iter()
        .all(|other| other.name != binding.name)));
    let source = function.resumable_plan.as_ref().unwrap();
    assert_eq!(
        source
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
        ]
    );
    assert!(source.suspension_points.iter().all(|point| {
        point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter
            && point.suspend_state >= plan.catch_entry_state.unwrap()
            && point.resume_state < plan.catch_exit_state.unwrap()
            && source
                .resume_environment_plan()
                .enclosing_scope_resume_states()
                .contains(&point.resume_state)
    }));
}

#[test]
fn mixed_array_catch_default_has_original_iterator_owner_before_catch_body() {
    let function = values("async function* values(input){try{throw input;}catch([received=await(yield ()=>received)]){yield received;}finally{await 0;yield 'finally';}}");
    let StatementIr::TryCatchFinally {
        catch_name,
        catch_block,
        generator_plan: Some(plan),
        ..
    } = function
        .body
        .statements
        .iter()
        .find(|item| matches!(item, StatementIr::TryCatchFinally { .. }))
        .unwrap()
    else {
        panic!("original whole-finalizer route");
    };
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == catch_name));
    assert!(contains_array(&catch_block.statements));
    let [StatementIr::EmptyStatementCompletion(_), StatementIr::Block(body)] =
        catch_block.statements.as_slice()
    else {
        panic!("binding initialization is one Empty source operation");
    };
    assert!(contains_value_yield(&body.statements));
    assert!(plan.finally_entry_state.unwrap() >= plan.catch_exit_state.unwrap());
}
