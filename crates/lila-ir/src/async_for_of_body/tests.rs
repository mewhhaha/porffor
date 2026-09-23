use super::*;
use lila_front::{parse, ParseOptions};

fn statements(source: &str) -> Vec<StatementIr> {
    let parsed = parse(source, ParseOptions::script()).expect("async source parses");
    let program = crate::lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "task")
        .expect("async function")
        .body
        .statements
}

fn direct_await() -> StatementIr {
    statements("async function task() { await 0; }")
        .into_iter()
        .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
        .expect("direct await")
}

#[test]
fn nested_clauses_and_conditional_branches_form_one_continuation_body() {
    let statements = statements(
        "async function task(flag) { await 0; try { if (flag) { await 1; } else { await 2; } } catch (error) { await error; } finally { await 3; } await 4; }",
    );
    let body =
        AsyncFunctionForOfBodyIr::new(statements.clone(), 0).expect("all nested owners compose");
    assert_eq!(body.entry_state(), 0);
    assert_eq!(body.exit_state(), 12);
    assert_eq!(body.statements(), statements);
}

#[test]
fn an_eager_try_still_owns_clause_states_before_a_following_await() {
    let body = AsyncFunctionForOfBodyIr::new(
        statements("async function task() { try { 0; } catch (error) { 1; } await 2; }"),
        0,
    )
    .expect("eager clauses reserve states in a plain async owner");
    assert_eq!(body.exit_state(), 3);
}

#[test]
fn malformed_await_ranges_and_overflow_cannot_form_a_body() {
    for (suspend, resume, entry, expected) in [
        (
            1,
            2,
            0,
            AsyncFunctionForOfBodyError::StateMismatch {
                expected: 0,
                actual: 1,
            },
        ),
        (
            0,
            2,
            0,
            AsyncFunctionForOfBodyError::StateMismatch {
                expected: 1,
                actual: 2,
            },
        ),
        (
            u32::MAX,
            0,
            u32::MAX,
            AsyncFunctionForOfBodyError::StateOverflow { state: u32::MAX },
        ),
    ] {
        let mut statement = direct_await();
        let StatementIr::AsyncAwait {
            suspend_state,
            resume_state,
            ..
        } = &mut statement
        else {
            unreachable!()
        };
        *suspend_state = suspend;
        *resume_state = resume;
        assert_eq!(
            AsyncFunctionForOfBodyIr::new(vec![statement], entry),
            Err(expected)
        );
    }
}

#[test]
fn missing_catch_state_and_overlapping_finally_range_are_rejected() {
    let original = statements(
        "async function task() { try { await 0; } catch (error) { await error; } finally { await 1; } }",
    );
    for missing in [true, false] {
        let mut statements = original.clone();
        let StatementIr::TryCatchFinally { async_plan, .. } = statements
            .iter_mut()
            .find(|statement| matches!(statement, StatementIr::TryCatchFinally { .. }))
            .expect("try statement")
        else {
            unreachable!()
        };
        let plan = async_plan.as_mut().expect("plain async clause owner");
        if missing {
            plan.catch_exit_state = None;
        } else {
            plan.finally_entry_state = Some(plan.entry_state);
        }
        let error = AsyncFunctionForOfBodyIr::new(statements, 0).unwrap_err();
        if missing {
            assert_eq!(error, AsyncFunctionForOfBodyError::TryClauseLayout);
        } else {
            assert!(matches!(
                error,
                AsyncFunctionForOfBodyError::StateMismatch { .. }
            ));
        }
    }
}

#[test]
fn ordinary_label_dispatch_cannot_hide_a_continuation() {
    let statement = StatementIr::Labelled {
        labels: vec!["label".into()],
        statement: Box::new(direct_await()),
    };
    assert_eq!(
        AsyncFunctionForOfBodyIr::new(vec![statement], 0),
        Err(AsyncFunctionForOfBodyError::StateMismatch {
            expected: 0,
            actual: 1
        })
    );
}

#[test]
fn missing_async_owner_and_a_body_without_await_are_rejected() {
    let mut statements = statements("async function task() { try { await 0; } catch (error) {} }");
    let StatementIr::TryCatch { async_plan, .. } = statements
        .iter_mut()
        .find(|statement| matches!(statement, StatementIr::TryCatch { .. }))
        .expect("try statement")
    else {
        unreachable!()
    };
    *async_plan = None;
    assert_eq!(
        AsyncFunctionForOfBodyIr::new(statements, 0),
        Err(AsyncFunctionForOfBodyError::TryClauseLayout)
    );
    assert_eq!(
        AsyncFunctionForOfBodyIr::new(vec![StatementIr::Empty], 0),
        Err(AsyncFunctionForOfBodyError::AwaitRequired)
    );
}
