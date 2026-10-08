use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncTryPlanIr, BlockIr, FunctionProtocolIr, ResumableRegionProtocolIr, ScriptIr,
    StatementIr, SynchronousLoopBodyError, SynchronousLoopBodyIr, TypedExpr, ValueKind,
};

fn lower_script(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("fixture parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program.script.unwrap()
}

#[test]
fn resource_loop_phases_keep_adjacent_awaits_disjoint() {
    for loop_source in [
        "for (using value = null; false;) { try {} catch (error) {} finally {} }",
        "for (using value of [null]) { try {} catch (error) {} finally {} }",
    ] {
        let script = lower_script(&format!(
            "async function task() {{ await 0; {loop_source} await 1; }} task();"
        ));
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "task")
            .unwrap();
        assert_eq!(function.protocol, FunctionProtocolIr::Async);
        let [StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }, loop_statement, StatementIr::AsyncAwait {
            suspend_state,
            resume_state,
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!("two source awaits surround the complete resource loop");
        };
        let (entry, exit, body) = match loop_statement {
            StatementIr::AsyncGeneratorLoop(plan) => {
                assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
                assert!(plan.resource().is_some());
                (plan.entry_state(), plan.exit_state(), plan.body())
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
                assert!(plan.resource().is_some());
                (plan.entry_state(), plan.exit_state(), plan.body())
            }
            _ => panic!("the original resource loop retains its checked phases"),
        };
        assert_eq!(entry, 1);
        assert_eq!(exit, *suspend_state);
        assert_eq!(*resume_state, exit + 1);
        assert!(
            body.end_state() > body.entry_state(),
            "try clauses own their dispatch phases"
        );
    }
}

#[test]
fn resource_source_suspensions_belong_to_complete_async_loop_owners() {
    for loop_source in [
        "for (using value = await null; false;) {}",
        "for (using value = null; await false;) {}",
        "for (using value = null; false; await 0) {}",
        "for (using value = null; true;) { await 0; }",
        "for (using value of [null]) { await 0; }",
        "for (using value of [null]) { await using nested = null; }",
        "for (using value of [null]) { for await (const nested of []) {} }",
        "for (using value of [null]) { for (await using nested of []) {} }",
        "for (using value of [null]) { class Local extends (await Object) {} }",
        "for (using value of [null]) { class Local { [await 0]() {} } }",
        "for (using value of [null]) { class Local { [await 0] = 1; } }",
    ] {
        let source = format!("async function task() {{ {loop_source} }}");
        let script = lower_script(&source);
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "task")
            .unwrap();
        assert_eq!(function.protocol, FunctionProtocolIr::Async);
        let [statement] = function.body.statements.as_slice() else {
            panic!("one complete source resource loop");
        };
        match statement {
            StatementIr::AsyncGeneratorLoop(plan) => {
                assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
                assert!(plan.resource().is_some());
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
                assert!(plan.resource().is_some());
            }
            _ => panic!("suspensions require their actual whole-loop owner: {source}"),
        }
    }
}

#[test]
fn nested_function_and_deferred_field_owners_do_not_suspend_the_loop() {
    lower_script(
        "async function task() { await 0; for (using value of [null]) { async function inner() { await 0; } const arrow = async () => { await 0; }; class Local { field = async () => { await 0; }; async method() { await 0; } } inner; arrow; Local; } await 0; } task();",
    );
}

#[test]
fn generator_resource_owners_retain_their_actual_execution_protocol() {
    for (prefix, execution) in [
        ("function*", ResumableRegionProtocolIr::Generator),
        ("async function*", ResumableRegionProtocolIr::AsyncGenerator),
    ] {
        let source = format!("{prefix} task() {{ for (using value of [null]) {{}} }}");
        let script = lower_script(&source);
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "task")
            .unwrap();
        let [StatementIr::AsyncGeneratorForOf(plan)] = function.body.statements.as_slice() else {
            panic!("one complete resource iterator");
        };
        assert_eq!(plan.execution(), execution);
        assert!(matches!(
            plan.resource().unwrap().capability(),
            lila_ir::AsyncGeneratorResourceCapabilityIr::Sync(_)
        ));
    }
}

fn block(statements: Vec<StatementIr>) -> BlockIr {
    BlockIr {
        statements,
        result_kind: ValueKind::Undefined,
        lexical_environment: None,
    }
}

#[test]
fn manually_constructed_suspensions_cannot_enter_the_local_disposal_consumer() {
    let statement = StatementIr::Block(block(vec![StatementIr::AsyncAwait {
        value: TypedExpr::undefined(),
        suspend_state: 0,
        resume_state: 1,
        resume_mode: lila_ir::AsyncResumeModeIr::Ignore,
    }]));
    assert_eq!(
        SynchronousLoopBodyIr::new(&statement).unwrap_err(),
        SynchronousLoopBodyError::Suspension
    );
    let boundary = StatementIr::AsyncModuleInstantiation;
    assert_eq!(
        SynchronousLoopBodyIr::new(&boundary).unwrap_err(),
        SynchronousLoopBodyError::ContinuationOwner
    );
}

#[test]
fn even_a_nonawaiting_try_must_not_carry_an_async_dispatch_plan() {
    let ordinary = StatementIr::TryFinally {
        try_block: block(vec![]),
        finally_block: block(vec![]),
        generator_plan: None,
        async_plan: None,
    };
    assert!(SynchronousLoopBodyIr::new(&ordinary).is_ok());
    let resumable = StatementIr::TryFinally {
        try_block: block(vec![]),
        finally_block: block(vec![]),
        generator_plan: None,
        async_plan: Some(AsyncTryPlanIr {
            entry_state: 0,
            try_exit_state: 1,
            catch_entry_state: None,
            catch_exit_state: None,
            finally_entry_state: Some(1),
            finally_exit_state: Some(2),
            exit_state: 2,
        }),
    };
    assert_eq!(
        SynchronousLoopBodyIr::new(&resumable).unwrap_err(),
        SynchronousLoopBodyError::ContinuationOwner
    );
}
