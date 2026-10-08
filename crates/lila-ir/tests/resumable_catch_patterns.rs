use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, ResumableRegionProtocolIr, StatementIr};

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

fn has_array(items: &[StatementIr], asynchronous: bool) -> bool {
    items.iter().any(|item| match item {
        StatementIr::OrdinaryGeneratorArrayDestructuring(_) => !asynchronous,
        StatementIr::AsyncFunctionArrayDestructuring(_) => asynchronous,
        StatementIr::EmptyStatementCompletion(item) => {
            has_array(std::slice::from_ref(item.statement()), asynchronous)
        }
        StatementIr::LexicalBlock(items) => has_array(items, asynchronous),
        StatementIr::Block(block) => has_array(&block.statements, asynchronous),
        _ => false,
    })
}

fn catch_rows(items: &[StatementIr]) -> Vec<&StatementIr> {
    fn visit<'a>(items: &'a [StatementIr], rows: &mut Vec<&'a StatementIr>) {
        for item in items {
            rows.push(item);
            match item {
                StatementIr::Block(block) => visit(&block.statements, rows),
                StatementIr::LexicalBlock(items) => visit(items, rows),
                StatementIr::EmptyStatementCompletion(item) => {
                    visit(std::slice::from_ref(item.statement()), rows)
                }
                StatementIr::AsyncGeneratorForOf(plan) => {
                    visit(&plan.initialization().block().statements, rows);
                    visit(&plan.body().block().statements, rows);
                }
                StatementIr::TryCatch {
                    try_block,
                    catch_block,
                    ..
                } => {
                    visit(&try_block.statements, rows);
                    visit(&catch_block.statements, rows);
                }
                StatementIr::TryFinally {
                    try_block,
                    finally_block,
                    ..
                } => {
                    visit(&try_block.statements, rows);
                    visit(&finally_block.statements, rows);
                }
                StatementIr::TryCatchFinally {
                    try_block,
                    catch_block,
                    finally_block,
                    ..
                } => {
                    visit(&try_block.statements, rows);
                    visit(&catch_block.statements, rows);
                    visit(&finally_block.statements, rows);
                }
                _ => {}
            }
        }
    }
    let mut rows = Vec::new();
    visit(items, &mut rows);
    rows
}

#[test]
fn ordinary_generator_catch_binding_owns_thrown_value_pattern_close_and_parameter_record() {
    let function = values("function* values(input){try{throw input;}catch([received=yield ()=>received]){yield ()=>received;}finally{yield 'finally';}}");
    let StatementIr::TryCatchFinally {
        catch_name,
        catch_parameter_environment,
        catch_block,
        generator_plan: Some(plan),
        ..
    } = catch_rows(&function.body.statements)
        .into_iter()
        .find(|item| matches!(item, StatementIr::TryCatchFinally { .. }))
        .unwrap()
    else {
        panic!("original generator Try owner");
    };
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == catch_name)
            .count(),
        1
    );
    assert!(catch_parameter_environment.is_some());
    assert!(has_array(&catch_block.statements, false));
    assert!(matches!(
        catch_block.statements.first(),
        Some(StatementIr::EmptyStatementCompletion(_))
    ));
    assert!(function
        .generator_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .iter()
        .any(
            |point| point.suspend_state >= plan.catch_entry_state.unwrap()
                && point.resume_state < plan.catch_exit_state.unwrap()
        ));
    assert!(plan.finally_entry_state.unwrap() >= plan.catch_exit_state.unwrap());
}

#[test]
fn plain_async_catch_binding_uses_original_await_array_and_separate_catch_body() {
    let function = values("async function values(input,p){try{throw input;}catch([received=await p]){let body='body';return()=>[body,received];}finally{await p;}}");
    let StatementIr::TryCatchFinally {
        catch_name,
        catch_parameter_environment,
        catch_block,
        async_plan: Some(plan),
        ..
    } = catch_rows(&function.body.statements)
        .into_iter()
        .find(|item| matches!(item, StatementIr::TryCatchFinally { .. }))
        .unwrap()
    else {
        panic!("original Async Try owner");
    };
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == catch_name)
            .count(),
        1
    );
    let parameter = catch_parameter_environment.as_ref().unwrap();
    let [StatementIr::EmptyStatementCompletion(_), StatementIr::Block(body)] =
        catch_block.statements.as_slice()
    else {
        panic!("complete parameter initialization before body");
    };
    let body_environment = body.lexical_environment.as_ref().unwrap();
    assert!(parameter.bindings.iter().all(|binding| body_environment
        .bindings
        .iter()
        .all(|other| other.name != binding.name)));
    assert!(has_array(&catch_block.statements, true));
    assert!(plan.catch_exit_state.unwrap() > plan.catch_entry_state.unwrap());
    assert!(plan.finally_entry_state.unwrap() >= plan.catch_exit_state.unwrap());
}

#[test]
fn ordinary_and_plain_async_object_catch_parameters_own_computed_key_before_default() {
    for source in [
        "function* values(input){try{throw input;}catch({[yield 'key']:received=yield ()=>received}){yield received;}}",
        "async function values(input,key,value){try{throw input;}catch({[await key]:received=await value}){return ()=>received;}}",
    ] {
        let function = values(source);
        let StatementIr::TryCatch { catch_name, catch_block, .. } = catch_rows(&function.body.statements).into_iter().find(|item| matches!(item, StatementIr::TryCatch { .. })).unwrap() else { panic!("actual catch owner"); };
        assert!(function.owned_env_bindings.iter().any(|binding| &binding.name == catch_name));
        assert!(matches!(catch_block.statements.first(), Some(StatementIr::EmptyStatementCompletion(_))));
    }
}

#[test]
fn iterator_body_catch_defaults_keep_the_original_iterator_and_catch_clause_owners() {
    let generator = values("function* values(list,input){for(const item of list){try{throw input;}catch([received=yield item]){yield received;}}}");
    let StatementIr::AsyncGeneratorForOf(iterator) = catch_rows(&generator.body.statements)
        .into_iter()
        .find(|item| matches!(item, StatementIr::AsyncGeneratorForOf(_)))
        .unwrap()
    else {
        panic!("original iterator owner");
    };
    assert_eq!(iterator.execution(), ResumableRegionProtocolIr::Generator);
    assert!(iterator.initialization().end_state() < iterator.body().entry_state());
    let StatementIr::TryCatch {
        catch_name,
        catch_block,
        generator_plan: Some(plan),
        ..
    } = catch_rows(&iterator.body().block().statements)
        .into_iter()
        .find(|item| matches!(item, StatementIr::TryCatch { .. }))
        .unwrap()
    else {
        panic!("original catch clause");
    };
    assert!(has_array(&catch_block.statements, false));
    assert!(generator
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name.as_str() == catch_name.as_str()));
    assert!(plan.catch_exit_state.unwrap() > plan.catch_entry_state.unwrap());
    for source in [
        "async function values(list,input){for(const item of list){try{throw input;}catch([received=await item]){return received;}}}",
        "async function values(list,input){for await(const item of list){try{throw input;}catch([received=await item]){return received;}}}",
    ] {
        let asynchronous=values(source);
        let StatementIr::AsyncGeneratorForOf(iterator)=catch_rows(&asynchronous.body.statements).into_iter().find(|item| matches!(item,StatementIr::AsyncGeneratorForOf(_))).unwrap() else {panic!("original async iterator body owner");};
        assert_eq!(iterator.execution(), ResumableRegionProtocolIr::Async);
        assert!(iterator.initialization().end_state() < iterator.body().entry_state());
        let StatementIr::TryCatch {catch_block,async_plan:Some(plan),..}=catch_rows(&iterator.body().block().statements).into_iter().find(|item| matches!(item,StatementIr::TryCatch {..})).unwrap() else {panic!("original async catch clause");};
        assert!(has_array(&catch_block.statements,true));
        assert!(plan.catch_exit_state.unwrap()>plan.catch_entry_state.unwrap());
    }
}
