use super::async_generator_dispatcher_unsupported_feature;
use lila_front::{parse, ParseOptions};
use lila_ir::{lower, StatementIr};

fn lowered_await_loop() -> StatementIr {
    let parsed = parse(
        "async function* sequence() { for (let i = 0; i < 2; i++) { await 0; await 1; } yield 9; }",
        ParseOptions::script(),
    )
    .expect("the async-generator source should parse");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .as_ref()
        .expect("the source should lower to a script")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("the async generator should be collected")
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::GeneratorLoop { .. }))
        .expect("the source should own a resumable loop")
        .clone()
}

#[test]
fn async_generator_dispatcher_accepts_a_sequential_await_loop() {
    let statement = lowered_await_loop();
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        None
    );
}

#[test]
fn async_generator_dispatcher_rejects_a_truncated_loop_resume_range() {
    let mut statement = lowered_await_loop();
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
    let mut statement = lowered_await_loop();
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
    let mut statement = lowered_await_loop();
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
    let mut statement = lowered_await_loop();
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
    let mut statement = lowered_await_loop();
    let StatementIr::GeneratorLoop { exit_state, .. } = &mut statement else {
        unreachable!("the fixture is a resumable loop");
    };
    *exit_state = 3;
    assert_eq!(
        async_generator_dispatcher_unsupported_feature(&statement),
        Some("resumable loops with an unplanned exit state")
    );
}
