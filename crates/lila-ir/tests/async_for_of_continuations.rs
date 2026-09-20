use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncFunctionForOfIteratorPlanIr, FunctionIr, ResumableLoopIterationEnvironmentIr,
    ScriptIr, StatementIr,
};

fn lower_script(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("async loop parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

fn function(script: &ScriptIr) -> &FunctionIr {
    script
        .functions
        .iter()
        .find(|function| function.name == "task")
        .expect("async function")
}

fn loop_plan(function: &FunctionIr) -> &AsyncFunctionForOfIteratorPlanIr {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionForOfIterator { plan, .. } => Some(plan),
            _ => None,
        })
        .expect("resumable synchronous iterator")
}

#[test]
fn catch_clause_states_join_before_the_following_statement() {
    let script = lower_script(
        "async function task() { for (const value of [1, 2]) { try { await value; } catch (error) {} } await 1; }",
    );
    let function = function(&script);
    let plan = loop_plan(function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.body().entry_state(), 0);
    assert_eq!(plan.body().exit_state(), 3);
    assert_eq!(plan.exit_state(), 4);
    let try_plan = plan
        .body()
        .statements()
        .iter()
        .find_map(|statement| match statement {
            StatementIr::TryCatch { async_plan, .. } => *async_plan,
            _ => None,
        })
        .expect("body retains its try/catch owner");
    assert_eq!(try_plan.try_exit_state, 2);
    assert_eq!(try_plan.catch_entry_state, Some(2));
    assert_eq!(try_plan.catch_exit_state, Some(3));
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 4,
            resume_state: 5,
            ..
        }
    )));
    for name in [
        plan.record().iterator().as_str(),
        plan.record().next_method().as_str(),
        plan.record().done().as_str(),
    ] {
        assert!(
            function
                .owned_env_bindings
                .iter()
                .any(|owned| owned.name == name),
            "Iterator Record slot {name} must survive every clause await"
        );
    }
}

#[test]
fn head_body_and_catch_parameter_environments_remain_distinct() {
    let script = lower_script(
        "async function task() { const readers = []; for (let index of [1, 2]) { let local = index * 10; readers.push(() => index + local); try { await Promise.reject(index); } catch (error) { readers.push(() => error); await 0; local++; } finally { await 0; } } return readers; }",
    );
    let plan = loop_plan(function(&script));
    let ResumableLoopIterationEnvironmentIr::FreshPerIteration(head_environment) =
        plan.iteration_environment()
    else {
        panic!("captured head owns a fresh environment")
    };
    let captured_name = |source_name| {
        script
            .functions
            .iter()
            .flat_map(|function| &function.captured_bindings)
            .find(|binding| binding.source_name == source_name)
            .map(|binding| binding.name.as_str())
            .expect("closure capture retains its source binding")
    };
    assert!(head_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("index")));
    let body = plan
        .body()
        .statements()
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(block) if block.lexical_environment.is_some() => Some(block),
            _ => None,
        })
        .expect("captured body lexical block is retained");
    let body_environment = body.lexical_environment.as_ref().unwrap();
    assert!(body_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("local")));
    let catch_environment = body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::TryCatchFinally {
                catch_parameter_environment,
                ..
            } => catch_parameter_environment.as_ref(),
            _ => None,
        })
        .expect("captured catch parameter has its own environment");
    assert!(catch_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("error")));
}

#[test]
fn conditional_awaits_and_finally_keep_their_nested_owners() {
    let script = lower_script(
        "async function task(flag) { for (const value of [1, 2]) { try { if (flag) await value; else await 0; } finally { await 1; } } }",
    );
    let plan = loop_plan(function(&script));
    let (try_block, finally_block, try_plan) = plan
        .body()
        .statements()
        .iter()
        .find_map(|statement| match statement {
            StatementIr::TryFinally {
                try_block,
                finally_block,
                async_plan: Some(plan),
                ..
            } => Some((try_block, finally_block, plan)),
            _ => None,
        })
        .expect("try/finally owner");
    assert!(try_block
        .statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert!(finally_block
        .statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    assert_eq!(try_plan.exit_state, plan.body().exit_state());
    assert_eq!(plan.exit_state(), plan.body().exit_state() + 1);
}

#[test]
fn unrelated_control_shapes_keep_explicit_capability_diagnostics() {
    for (source, expected) in [
        (
            "async function task() { for (const value of [1]) { await value; break; } }",
            "body without break or continue",
        ),
        (
            "async function task() { for await (const value of [1]) { await value; } }",
            "explicit await in for-await-of body",
        ),
        (
            "async function task() { for (const value of [1]) { label: { await value; } } }",
            "invalid async for-of body continuation",
        ),
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid source");
        let program = lower(&parsed);
        assert!(!program.is_wasm_supported());
        assert!(
            format!("{:?}", program.diagnostics).contains(expected),
            "{:?}",
            program.diagnostics
        );
    }
}
