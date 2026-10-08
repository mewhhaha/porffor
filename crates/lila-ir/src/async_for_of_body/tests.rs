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
        async_plan: None,
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

#[test]
fn current_loop_branches_survive_the_checked_eager_dispatch_path() {
    let branch = StatementIr::If {
        condition: crate::TypedExpr::undefined(),
        then_branch: Box::new(StatementIr::Break { label: None }),
        else_branch: Some(Box::new(StatementIr::Continue { label: None })),
    };
    assert!(crate::SynchronousLoopBodyIr::new(&branch).is_ok());
    let body = AsyncFunctionForOfBodyIr::new(
        vec![
            StatementIr::Continue { label: None },
            direct_await(),
            branch,
            StatementIr::Break { label: None },
        ],
        0,
    )
    .expect("current for-of owns all unlabelled branches");
    assert_eq!(body.exit_state(), 1);
}

#[test]
fn eager_success_cannot_hide_foreign_or_labelled_branch_owners() {
    for branch in [
        StatementIr::Break { label: None },
        StatementIr::Continue { label: None },
    ] {
        for foreign in [
            StatementIr::While {
                condition: crate::TypedExpr::undefined(),
                body: Box::new(branch.clone()),
            },
            StatementIr::For {
                init: Some(ForInitIr::Statements(vec![branch.clone()])),
                test: None,
                update: None,
                body: Box::new(StatementIr::Empty),
                lexical_environment: None,
            },
            StatementIr::Labelled {
                labels: vec!["other".into()],
                statement: Box::new(branch.clone()),
                async_plan: None,
            },
            StatementIr::Switch {
                discriminant: crate::TypedExpr::undefined(),
                lexical_environment: None,
                lexical_declarations: vec![],
                cases: vec![crate::SwitchCaseIr {
                    condition: None,
                    body: BlockIr {
                        statements: vec![branch.clone()],
                        result_kind: crate::ValueKind::Undefined,
                        lexical_environment: None,
                    },
                }],
            },
            StatementIr::ParameterInitialization {
                parameter_index: 0,
                statements: vec![branch.clone()],
            },
        ] {
            let eager = StatementIr::If {
                condition: crate::TypedExpr::undefined(),
                then_branch: Box::new(foreign),
                else_branch: None,
            };
            assert!(crate::SynchronousLoopBodyIr::new(&eager).is_ok());
            assert_eq!(
                AsyncFunctionForOfBodyIr::new(vec![direct_await(), eager], 0),
                Err(AsyncFunctionForOfBodyError::ForeignBranchOwner),
            );
        }
    }
    for label in [
        StatementIr::Break {
            label: Some("outer".into()),
        },
        StatementIr::Continue {
            label: Some("outer".into()),
        },
    ] {
        assert_eq!(
            AsyncFunctionForOfBodyIr::new(vec![direct_await(), label], 0),
            Err(AsyncFunctionForOfBodyError::ForeignBranchOwner),
        );
    }
}

#[test]
fn awaited_finally_keeps_local_completion_targets_and_exact_states() {
    let mut original =
        statements("async function task() { try { await 0; } finally { await 1; } }");
    let StatementIr::TryFinally {
        try_block,
        finally_block,
        ..
    } = &mut original[0]
    else {
        panic!("source has one direct try/finally owner");
    };
    try_block
        .statements
        .push(StatementIr::Continue { label: None });
    finally_block
        .statements
        .push(StatementIr::Break { label: None });
    let body = AsyncFunctionForOfBodyIr::new(original.clone(), 0)
        .expect("local control does not allocate or overlap clause states");
    assert_eq!(body.exit_state(), 4);
    assert_eq!(body.statements(), original.as_slice());
}

#[test]
fn eager_try_and_synchronous_resource_support_remains_available() {
    let synchronous_try = StatementIr::TryFinally {
        try_block: BlockIr {
            statements: vec![StatementIr::Empty],
            result_kind: crate::ValueKind::Undefined,
            lexical_environment: None,
        },
        finally_block: BlockIr {
            statements: vec![StatementIr::Empty],
            result_kind: crate::ValueKind::Undefined,
            lexical_environment: None,
        },
        generator_plan: None,
        async_plan: None,
    };
    assert!(crate::SynchronousLoopBodyIr::new(&synchronous_try).is_ok());
    assert_eq!(
        AsyncFunctionForOfBodyIr::new(
            vec![
                direct_await(),
                StatementIr::If {
                    condition: crate::TypedExpr::undefined(),
                    then_branch: Box::new(synchronous_try),
                    else_branch: None,
                }
            ],
            0
        )
        .expect("eager try remains supported")
        .exit_state(),
        1
    );
    let resource_statements =
        statements("async function task() { await 0; { using resource = null; 0; } await 1; }");
    let body = AsyncFunctionForOfBodyIr::new(resource_statements, 0)
        .expect("existing eager synchronous resource body remains supported");
    assert_eq!(body.exit_state(), 2);
}

fn empty_materialized_environment() -> crate::LexicalEnvironmentIr {
    crate::LexicalEnvironmentIr {
        initialization: crate::LexicalEnvironmentInitializationIr::Uninitialized,
        eval_environment: None,
        bindings: vec![],
    }
}

fn shifted_await(entry: u32) -> StatementIr {
    let mut statement = direct_await();
    let StatementIr::AsyncAwait {
        suspend_state,
        resume_state,
        ..
    } = &mut statement
    else {
        unreachable!()
    };
    *suspend_state = entry;
    *resume_state = entry.checked_add(1).expect("finite control state");
    statement
}

#[test]
fn for_await_checks_eager_nested_environments_without_claiming_clause_states() {
    let eager_try = StatementIr::TryCatch {
        try_block: BlockIr {
            statements: vec![StatementIr::Empty],
            result_kind: crate::ValueKind::Undefined,
            lexical_environment: None,
        },
        catch_name: "error".into(),
        catch_source_name: "error".into(),
        catch_parameter_environment: None,
        catch_block: BlockIr {
            statements: vec![StatementIr::Empty],
            result_kind: crate::ValueKind::Undefined,
            lexical_environment: None,
        },
        generator_plan: None,
        async_plan: None,
    };
    let branch = |statement| StatementIr::If {
        condition: crate::TypedExpr::undefined(),
        then_branch: Box::new(statement),
        else_branch: None,
    };
    let body = AsyncFunctionForOfBodyIr::new_for_await(
        vec![branch(eager_try.clone()), shifted_await(1)],
        1,
    )
    .expect("ordinary eager try retains ordinary dispatch");
    assert_eq!(body.exit_state(), 2);
    for captured_catch in [true, false] {
        let mut statement = eager_try.clone();
        let StatementIr::TryCatch {
            catch_parameter_environment,
            try_block,
            ..
        } = &mut statement
        else {
            unreachable!()
        };
        if captured_catch {
            *catch_parameter_environment = Some(empty_materialized_environment());
        } else {
            try_block.statements.push(StatementIr::Block(BlockIr {
                statements: vec![],
                result_kind: crate::ValueKind::Undefined,
                lexical_environment: Some(empty_materialized_environment()),
            }));
        }
        assert_eq!(
            AsyncFunctionForOfBodyIr::new_for_await(vec![branch(statement), shifted_await(1)], 1),
            Err(AsyncFunctionForOfBodyError::MaterializedBodyEnvironment)
        );
    }
}

fn awaited_plan(
    statements: Vec<StatementIr>,
    entry: u32,
    async_flag: &str,
    close_flag: &str,
) -> Result<crate::AsyncFunctionForOfIteratorPlanIr, crate::ir::AsyncFunctionForOfIteratorPlanError>
{
    crate::AsyncFunctionForOfIteratorPlanIr::new_for_await(
        crate::ir::AsyncFunctionForOfIteratorHeadIr::Binding {
            source_name: "value".into(),
            binding: crate::ForOfAssignmentIr {
                mode: crate::BindingMode::Var,
                name: "value".into(),
            },
        },
        crate::IteratorRecordIr::new(
            crate::IteratorSlot::new("iterator".into()),
            crate::NextMethodSlot::new("next".into()),
            crate::DoneSlot::new("done".into()),
        ),
        None,
        statements,
        entry,
        async_flag.into(),
        close_flag.into(),
    )
}

#[test]
fn awaited_execution_requires_disjoint_protocol_storage_and_checked_state_successors() {
    use crate::ir::AsyncFunctionForOfIteratorPlanError as Error;
    let plan =
        awaited_plan(vec![shifted_await(1)], 0, "async", "close").expect("checked awaited body");
    let crate::AsyncFunctionForOfIteratorExecutionIr::Awaited(view) = plan.execution() else {
        panic!("awaited protocol view")
    };
    assert_eq!(
        (
            plan.entry_state(),
            view.value_resume_state(),
            plan.body().exit_state(),
            view.close_resume_state(),
            plan.exit_state()
        ),
        (0, 1, 2, 3, 4)
    );
    assert!(std::ptr::eq(view.plan(), &plan));
    for (async_flag, close_flag) in [
        ("async", "async"),
        ("iterator", "close"),
        ("async", "value"),
        ("", "close"),
    ] {
        assert!(matches!(
            awaited_plan(vec![shifted_await(1)], 0, async_flag, close_flag),
            Err(Error::AwaitedProtocolStorageAlias { .. })
        ));
    }
    assert!(matches!(
        awaited_plan(vec![], u32::MAX, "async", "close"),
        Err(Error::AwaitedEntryStateOverflow {
            entry_state: u32::MAX
        })
    ));
    let entry = u32::MAX - 4;
    let last = awaited_plan(vec![shifted_await(entry + 1)], entry, "async", "close")
        .expect("all final successors still fit");
    let crate::AsyncFunctionForOfIteratorExecutionIr::Awaited(last_view) = last.execution() else {
        panic!("only the validated final awaited component is observable")
    };
    assert_eq!(last.body().exit_state(), u32::MAX - 2);
    assert_eq!(last_view.close_resume_state(), u32::MAX - 1);
    assert_eq!(last.exit_state(), u32::MAX);
    for entry in [u32::MAX - 3, u32::MAX - 2] {
        assert!(matches!(
            awaited_plan(vec![shifted_await(entry + 1)], entry, "async", "close"),
            Err(Error::ExitStateOverflow { .. })
        ));
    }
    assert!(matches!(
        awaited_plan(vec![direct_await()], 0, "async", "close"),
        Err(Error::InvalidBody(
            AsyncFunctionForOfBodyError::StateMismatch {
                expected: 1,
                actual: 0
            }
        ))
    ));
    assert!(matches!(
        crate::AsyncFunctionForOfIteratorPlanIr::new_for_await(
            crate::ir::AsyncFunctionForOfIteratorHeadIr::PreparedAssignment {
                value_name: "sink".into()
            },
            crate::IteratorRecordIr::new(
                crate::IteratorSlot::new("iterator".into()),
                crate::NextMethodSlot::new("next".into()),
                crate::DoneSlot::new("done".into())
            ),
            None,
            vec![shifted_await(1)],
            0,
            "async".into(),
            "close".into()
        ),
        Err(Error::AwaitedBindingHeadRequired)
    ));
}
