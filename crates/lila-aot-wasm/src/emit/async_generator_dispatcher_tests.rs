use super::async_generator_dispatcher_unsupported_feature;
use lila_front::{parse, ParseOptions};
use lila_ir::{AsyncResumeModeIr, ResumableLoopIterationEnvironmentIr, StatementIr, TypedExpr};

#[test]
fn async_generator_dispatcher_retains_checked_targets_through_catch_and_finally() {
    let parsed = parse(
        include_str!("../../../lila-engine/tests/fixtures/async_generator_classic_regions/completions_and_environments.js"),
        ParseOptions::script(),
    ).expect("the original completion fixture parses");
    let program =
        lila_ir::lower_with_host_surface_policy(&parsed, lila_ir::HostSurfacePolicy::Test262);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    for function in &program.script.as_ref().unwrap().functions {
        if function.protocol.execution_kind() == lila_ir::FunctionExecutionKind::AsyncGenerator {
            assert_eq!(
                function
                    .body
                    .statements
                    .iter()
                    .find_map(async_generator_dispatcher_unsupported_feature),
                None,
                "{}",
                function.name,
            );
        }
    }
}

#[test]
fn async_generator_dispatcher_rejects_unowned_abrupt_targets() {
    for label in [None, Some("missing".to_owned())] {
        assert_eq!(
            async_generator_dispatcher_unsupported_feature(&StatementIr::Break {
                label: label.clone()
            }),
            Some("break without a checked control owner"),
        );
        assert_eq!(
            async_generator_dispatcher_unsupported_feature(&StatementIr::Continue { label }),
            Some("continue without a checked iteration owner"),
        );
    }
}

// This raw linear carrier remains independently validated. Current source
// lowering produces opaque complete-loop owners, exercised by the original
// completion fixture above; it cannot be used to manufacture malformed states.
fn linear_await_loop() -> StatementIr {
    let suspension = |suspend_state| StatementIr::AsyncAwait {
        value: TypedExpr::undefined(),
        suspend_state,
        resume_state: suspend_state + 1,
        resume_mode: AsyncResumeModeIr::Ignore,
    };
    StatementIr::GeneratorLoop {
        init: None,
        test: None,
        update: None,
        iteration_environment: ResumableLoopIterationEnvironmentIr::StorageOnly,
        before_suspension: vec![],
        suspension_statement: Box::new(suspension(0)),
        after_suspension: vec![suspension(1)],
        entry_state: 0,
        resume_state: 2,
        exit_state: 2,
    }
}

#[test]
fn async_generator_dispatcher_accepts_a_sequential_await_loop() {
    let statement = linear_await_loop();
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        None
    );
}

#[test]
fn async_generator_dispatcher_rejects_a_truncated_loop_resume_range() {
    let mut statement = linear_await_loop();
    let StatementIr::GeneratorLoop {
        resume_state,
        exit_state,
        ..
    } = &mut statement
    else {
        unreachable!("the fixture is a resumable loop");
    };
    *resume_state = 1;
    *exit_state = 1;
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable loops with non-linear suspension states")
    );
}

#[test]
fn async_generator_dispatcher_rejects_a_discontinuous_await_state() {
    let mut statement = linear_await_loop();
    let StatementIr::GeneratorLoop {
        after_suspension,
        resume_state,
        exit_state,
        ..
    } = &mut statement
    else {
        unreachable!("the fixture is a resumable loop");
    };
    let StatementIr::AsyncAwait {
        suspend_state,
        resume_state: await_resume_state,
        ..
    } = &mut after_suspension[0]
    else {
        panic!("the loop tail should start with its second await");
    };
    *suspend_state = 2;
    *await_resume_state = 3;
    *resume_state = 3;
    *exit_state = 3;
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable loops with non-linear suspension states")
    );
}

#[test]
fn async_generator_dispatcher_rejects_a_nested_await_region() {
    let mut statement = linear_await_loop();
    let StatementIr::GeneratorLoop {
        after_suspension, ..
    } = &mut statement
    else {
        unreachable!("the fixture is a resumable loop");
    };
    let nested = StatementIr::LexicalBlock(std::mem::take(after_suspension));
    after_suspension.push(nested);
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable await loops containing nested suspensions")
    );
}

#[test]
fn async_generator_dispatcher_rejects_a_suspending_loop_prelude() {
    let mut statement = linear_await_loop();
    let StatementIr::GeneratorLoop {
        before_suspension,
        suspension_statement,
        ..
    } = &mut statement
    else {
        unreachable!("the fixture is a resumable loop");
    };
    before_suspension.push(suspension_statement.as_ref().clone());
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable loops with a suspending prelude")
    );
}

#[test]
fn async_generator_dispatcher_rejects_an_unplanned_loop_exit() {
    let mut statement = linear_await_loop();
    let StatementIr::GeneratorLoop { exit_state, .. } = &mut statement else {
        unreachable!("the fixture is a resumable loop");
    };
    *exit_state = 3;
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable loops with an unplanned exit state")
    );
}
