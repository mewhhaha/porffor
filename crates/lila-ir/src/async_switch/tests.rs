use super::*;
use crate::{AsyncResumeModeIr, ValueKind};

fn body(statements: Vec<StatementIr>) -> BlockIr {
    BlockIr {
        statements,
        result_kind: ValueKind::Undefined,
        lexical_environment: None,
    }
}

fn await_at(state: u32) -> StatementIr {
    StatementIr::AsyncAwait {
        value: TypedExpr::undefined(),
        suspend_state: state,
        resume_state: state + 1,
        resume_mode: AsyncResumeModeIr::Ignore,
    }
}

fn case(condition: Option<TypedExpr>, entry: u32, count: u32) -> AsyncFunctionSwitchCaseIr {
    AsyncFunctionSwitchCaseIr::new(
        condition.map(AsyncFunctionSwitchSelectorIr::Eager),
        body((entry..entry + count).map(await_at).collect()),
        entry,
        entry + count,
    )
    .unwrap()
}

fn disposal(
    statements: Vec<StatementIr>,
    entry: u32,
    dispose: u32,
    resume: u32,
    exit: u32,
) -> StatementIr {
    StatementIr::AsyncDisposableScope {
        execution: AsyncDisposableScopeExecutionIr::AsyncFunction(
            crate::AsyncFunctionAsyncDisposableCapabilityIr::new(
                "capability".into(),
                crate::AsyncDisposableFinalizerPlanIr::new(entry, dispose, resume, exit),
            ),
        ),
        resources: crate::AsyncDisposableResourcesIr::new(
            crate::AsyncDisposableResourceIr::new("resource".into(), TypedExpr::undefined()),
            Vec::new(),
        ),
        body: body(statements),
    }
}

#[test]
fn block_disposal_owns_implicit_and_nested_suffix_states_before_the_next_case() {
    let implicit =
        AsyncFunctionSwitchCaseIr::new(None, body(vec![disposal(Vec::new(), 1, 2, 3, 4)]), 1, 4)
            .unwrap();
    assert_eq!(implicit.next_entry_state(), 5);
    assert_eq!(
        implicit.clone().into_eager_case(),
        Err(AsyncSwitchError::MissingChildOwner)
    );
    let nested = AsyncFunctionSwitchCaseIr::new(
        Some(AsyncFunctionSwitchSelectorIr::Eager(TypedExpr::undefined())),
        body(vec![disposal(
            vec![await_at(5), disposal(vec![await_at(6)], 6, 8, 9, 10)],
            5,
            11,
            12,
            13,
        )]),
        5,
        13,
    )
    .unwrap();
    let plan = AsyncFunctionSwitchIr::new(
        TypedExpr::undefined(),
        None,
        Vec::new(),
        vec![implicit, nested],
        0,
    )
    .unwrap();
    assert_eq!(plan.exit_state(), 14);
}

#[test]
fn disposal_suffix_and_all_finalizer_successors_must_be_owned_without_gaps() {
    for child in [
        disposal(vec![await_at(2)], 1, 4, 5, 6),
        disposal(Vec::new(), 1, 3, 4, 5),
        disposal(vec![await_at(1)], 1, 4, 5, 6),
        disposal(Vec::new(), 1, 2, 4, 5),
        disposal(Vec::new(), 1, 2, 3, 5),
    ] {
        assert_eq!(
            AsyncFunctionSwitchCaseIr::new(None, body(vec![child]), 1, 6),
            Err(AsyncSwitchError::StateOrder)
        );
    }
}

#[test]
fn async_generator_disposal_cannot_claim_a_plain_async_switch_case() {
    let mut child = disposal(Vec::new(), 1, 2, 3, 4);
    let StatementIr::AsyncDisposableScope { execution, .. } = &mut child else {
        unreachable!()
    };
    *execution = AsyncDisposableScopeExecutionIr::AsyncGenerator(
        crate::AsyncGeneratorAsyncDisposableCapabilityIr::new(
            "generator-capability".into(),
            crate::AsyncDisposableFinalizerPlanIr::new(1, 2, 3, 4),
        ),
    );
    assert_eq!(
        AsyncFunctionSwitchCaseIr::new(None, body(vec![child]), 1, 4),
        Err(AsyncSwitchError::UnsupportedChildOwner)
    );
}

#[test]
fn skipped_eager_cases_and_awaited_fallthrough_have_disjoint_owned_segments() {
    let plan = AsyncFunctionSwitchIr::new(
        TypedExpr::undefined(),
        None,
        Vec::new(),
        vec![
            case(Some(TypedExpr::undefined()), 8, 0),
            case(None, 9, 2),
            case(Some(TypedExpr::undefined()), 12, 0),
        ],
        7,
    )
    .unwrap();
    assert_eq!((plan.entry_state(), plan.exit_state()), (7, 13));
    assert_eq!(
        plan.cases()
            .iter()
            .map(|case| (case.entry_state(), case.next_entry_state()))
            .collect::<Vec<_>>(),
        [(8, 9), (9, 12), (12, 13)]
    );
}

#[test]
fn actual_case_body_must_own_every_claimed_continuation_in_order() {
    for (statements, entry, claimed) in [
        (vec![await_at(5)], 4, 6),
        (vec![await_at(4), await_at(6)], 4, 7),
        (vec![await_at(4)], 4, 6),
        (Vec::new(), 4, 5),
    ] {
        assert_eq!(
            AsyncFunctionSwitchCaseIr::new(None, body(statements), entry, claimed),
            Err(AsyncSwitchError::StateOrder)
        );
    }
}

#[test]
fn case_order_duplicate_default_and_overflow_cannot_form_a_switch_owner() {
    let build = |cases, entry| {
        AsyncFunctionSwitchIr::new(TypedExpr::undefined(), None, Vec::new(), cases, entry)
    };
    assert_eq!(
        build(vec![case(None, 2, 0)], 0),
        Err(AsyncSwitchError::StateOrder)
    );
    assert_eq!(
        build(vec![case(None, 1, 0), case(None, 2, 0)], 0),
        Err(AsyncSwitchError::DuplicateDefault)
    );
    assert_eq!(
        build(Vec::new(), u32::MAX),
        Err(AsyncSwitchError::StateOverflow)
    );
    assert_eq!(
        AsyncFunctionSwitchCaseIr::new(None, body(Vec::new()), u32::MAX, u32::MAX),
        Err(AsyncSwitchError::StateOverflow)
    );
}

#[test]
fn an_unowned_branch_or_label_cannot_hide_a_child_await() {
    let children = [
        StatementIr::If {
            condition: TypedExpr::undefined(),
            then_branch: Box::new(await_at(1)),
            else_branch: None,
        },
        StatementIr::Labelled {
            labels: vec!["outer".into()],
            statement: Box::new(await_at(1)),
            async_plan: None,
        },
    ];
    for child in children {
        assert_eq!(
            AsyncFunctionSwitchCaseIr::new(None, body(vec![child]), 1, 2),
            Err(AsyncSwitchError::MissingChildOwner)
        );
    }
}

#[test]
fn eager_cases_can_be_consumed_only_without_child_continuations() {
    assert!(case(None, 1, 0).into_eager_case().is_ok());
    assert_eq!(
        case(None, 1, 1).into_eager_case(),
        Err(AsyncSwitchError::MissingChildOwner)
    );
}

#[test]
fn per_case_environment_and_forged_try_clause_states_cannot_claim_shared_ownership() {
    let mut separate = body(Vec::new());
    separate.lexical_environment = Some(LexicalEnvironmentIr {
        initialization: crate::LexicalEnvironmentInitializationIr::Uninitialized,
        eval_environment: None,
        bindings: Vec::new(),
    });
    assert_eq!(
        AsyncFunctionSwitchCaseIr::new(None, separate, 1, 1),
        Err(AsyncSwitchError::PerCaseEnvironment)
    );
    let forged = StatementIr::TryFinally {
        try_block: body(vec![await_at(1)]),
        finally_block: body(Vec::new()),
        generator_plan: None,
        async_plan: Some(crate::AsyncTryPlanIr {
            entry_state: 1,
            try_exit_state: 2,
            catch_entry_state: None,
            catch_exit_state: None,
            finally_entry_state: Some(2),
            finally_exit_state: Some(3),
            exit_state: 3,
        }),
    };
    assert_eq!(
        AsyncFunctionSwitchCaseIr::new(None, body(vec![forged]), 1, 3),
        Err(AsyncSwitchError::StateOrder)
    );
}

#[test]
fn unrelated_loop_body_owners_refuse_a_checked_async_switch() {
    let plan = AsyncFunctionSwitchIr::new(
        TypedExpr::undefined(),
        None,
        Vec::new(),
        vec![case(None, 1, 1)],
        0,
    )
    .unwrap();
    let statement = StatementIr::AsyncFunctionSwitch(plan);
    assert!(crate::SynchronousLoopBodyIr::new(&statement).is_err());
    assert!(
        !crate::source_call_flow_proof::SourceCallFlowEffects::for_finalized_invocation(
            &[],
            &body(vec![statement.clone()])
        )
        .proves_no_flow_invalidation()
    );
    assert_eq!(
        crate::AsyncFunctionForOfBodyIr::new(vec![statement.clone()], 0),
        Err(crate::async_for_of_body::AsyncFunctionForOfBodyError::UnsupportedContinuation)
    );
    assert_eq!(
        crate::GeneratorForOfBodyIr::new(vec![statement], 0),
        Err(crate::generator_for_of_body::GeneratorForOfBodyError::UnsupportedContinuation)
    );
}

#[test]
fn selector_prefix_must_own_its_actual_ready_state_and_decision_successor() {
    for (prefix, entry, ready) in [
        (vec![await_at(2)], 1, 3),
        (vec![await_at(1)], 1, 3),
        (vec![await_at(1), await_at(3)], 1, 4),
    ] {
        assert_eq!(
            AsyncFunctionSwitchSelectorContinuationIr::new(
                prefix,
                TypedExpr::undefined(),
                entry,
                ready,
            ),
            Err(AsyncSwitchError::StateOrder),
        );
    }
    assert_eq!(
        AsyncFunctionSwitchSelectorContinuationIr::new(
            Vec::new(),
            TypedExpr::undefined(),
            u32::MAX,
            u32::MAX,
        ),
        Err(AsyncSwitchError::StateOverflow),
    );
}

fn retained_selector(entry: u32, ready: u32) -> AsyncFunctionSwitchSelectorIr {
    AsyncFunctionSwitchSelectorIr::Resumable(
        AsyncFunctionSwitchSelectorContinuationIr::new(
            (entry..ready).map(await_at).collect(),
            TypedExpr::undefined(),
            entry,
            ready,
        )
        .unwrap(),
    )
}

fn retained_discriminant() -> TypedExpr {
    TypedExpr::from_info(
        crate::ValueInfo::new(ValueKind::Dynamic),
        crate::ExprIr::Identifier("saved".into()),
    )
}

#[test]
fn retained_selection_and_fallback_are_disjoint_from_all_case_bodies() {
    let plan = AsyncFunctionSwitchIr::new(
        retained_discriminant(),
        None,
        Vec::new(),
        vec![
            AsyncFunctionSwitchCaseIr::new(Some(retained_selector(1, 2)), body(Vec::new()), 6, 6)
                .unwrap(),
            case(None, 7, 1),
            AsyncFunctionSwitchCaseIr::new(Some(retained_selector(3, 4)), body(Vec::new()), 9, 9)
                .unwrap(),
        ],
        0,
    )
    .unwrap();
    assert_eq!(plan.selection_fallback_state(), Some(5));
    assert_eq!((plan.cases()[0].entry_state(), plan.exit_state()), (6, 10));
    assert!(plan.cases().iter().all(|case| case.entry_state() > 5));
}

#[test]
fn mixed_selection_unretained_discriminant_and_selector_gaps_are_refused() {
    let selected = || {
        AsyncFunctionSwitchCaseIr::new(Some(retained_selector(1, 2)), body(Vec::new()), 4, 4)
            .unwrap()
    };
    assert_eq!(
        AsyncFunctionSwitchIr::new(
            TypedExpr::undefined(),
            None,
            Vec::new(),
            vec![selected()],
            0
        ),
        Err(AsyncSwitchError::MissingRetainedDiscriminant),
    );
    assert_eq!(
        AsyncFunctionSwitchIr::new(
            retained_discriminant(),
            None,
            Vec::new(),
            vec![selected(), case(Some(TypedExpr::undefined()), 5, 0)],
            0,
        ),
        Err(AsyncSwitchError::MixedSelectorOwnership),
    );
    let gap = AsyncFunctionSwitchCaseIr::new(Some(retained_selector(2, 3)), body(Vec::new()), 5, 5)
        .unwrap();
    assert_eq!(
        AsyncFunctionSwitchIr::new(retained_discriminant(), None, Vec::new(), vec![gap], 0),
        Err(AsyncSwitchError::StateOrder),
    );
}
