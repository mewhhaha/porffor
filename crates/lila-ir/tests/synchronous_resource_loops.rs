use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncTryPlanIr, BlockIr, FunctionProtocolIr, ScriptIr, StatementIr,
    SynchronousLoopBodyError, SynchronousLoopBodyIr, TypedExpr, ValueKind,
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
fn eager_loop_clauses_do_not_reserve_states_between_adjacent_awaits() {
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
        let states: Vec<_> = function
            .body
            .statements
            .iter()
            .filter_map(|statement| {
                if let StatementIr::AsyncAwait {
                    suspend_state,
                    resume_state,
                    ..
                } = statement
                {
                    Some((*suspend_state, *resume_state))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(states, [(0, 1), (1, 2)], "{loop_source}");
        let body = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::For { body, .. } | StatementIr::ForOfIterator { body, .. } => {
                    Some(body)
                }
                _ => None,
            })
            .expect("resource loop remains synchronous");
        assert!(SynchronousLoopBodyIr::new(body).is_ok());
    }
}

#[test]
fn every_eager_source_suspension_is_rejected_before_region_lowering() {
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
        let parsed = parse(&source, ParseOptions::script()).expect(&source);
        let program = lower(&parsed);
        assert!(!program.is_wasm_supported(), "{source}");
        assert!(
            program.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("suspension inside a synchronous resource loop")),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn nested_function_and_deferred_field_owners_do_not_suspend_the_loop() {
    lower_script(
        "async function task() { await 0; for (using value of [null]) { async function inner() { await 0; } const arrow = async () => { await 0; }; class Local { field = async () => { await 0; }; async method() { await 0; } } inner; arrow; Local; } await 0; } task();",
    );
}

#[test]
fn generator_owners_do_not_acquire_the_plain_async_admission() {
    for prefix in ["function*", "async function*"] {
        let source = format!("{prefix} task() {{ for (using value of [null]) {{}} }}");
        let parsed = parse(&source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(!program.is_wasm_supported());
        assert!(
            program.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("synchronous resource loop in a generator")),
            "{:?}",
            program.diagnostics
        );
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
