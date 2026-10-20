use super::*;
use crate::AsyncResumeModeIr;

fn suspension(suspend_state: u32, resume_state: u32) -> StatementIr {
    StatementIr::AsyncAwait {
        value: TypedExpr::undefined(),
        suspend_state,
        resume_state,
        resume_mode: AsyncResumeModeIr::Ignore,
    }
}

#[test]
fn contiguous_condition_awaits_own_a_distinct_exit() {
    let plan = AsyncFunctionWhileConditionIr::new(
        vec![suspension(4, 5), suspension(5, 6)],
        TypedExpr::undefined(),
        StatementIr::Empty,
        4,
        6,
    )
    .expect("contiguous Await prefix");
    assert_eq!(
        (plan.entry_state(), plan.ready_state(), plan.exit_state()),
        (4, 6, 7)
    );
}

#[test]
fn transposed_and_overflowing_states_cannot_form_a_condition() {
    for (prefix, entry, ready, expected) in [
        (
            vec![suspension(0, 2)],
            0,
            2,
            AsyncWhileConditionError::StateOrder,
        ),
        (
            vec![suspension(1, 2)],
            0,
            2,
            AsyncWhileConditionError::StateOrder,
        ),
        (
            vec![suspension(0, 1)],
            0,
            2,
            AsyncWhileConditionError::StateOrder,
        ),
        (
            vec![suspension(u32::MAX - 1, u32::MAX)],
            u32::MAX - 1,
            u32::MAX,
            AsyncWhileConditionError::StateOverflow,
        ),
    ] {
        assert_eq!(
            AsyncFunctionWhileConditionIr::new(
                prefix,
                TypedExpr::undefined(),
                StatementIr::Empty,
                entry,
                ready,
            ),
            Err(expected)
        );
    }
}

#[test]
fn an_awaited_body_cannot_reuse_the_eager_body_owner() {
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![suspension(0, 1)],
            TypedExpr::undefined(),
            suspension(1, 2),
            0,
            1,
        ),
        Err(AsyncWhileConditionError::SuspendingBody)
    );
}

#[test]
fn an_await_hidden_inside_a_prefix_block_cannot_bypass_state_validation() {
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![StatementIr::LexicalBlock(vec![suspension(0, 1)])],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            1,
        ),
        Err(AsyncWhileConditionError::SuspendingPrefix)
    );
}

fn value_branch(
    entry: u32,
    then_prefix: Vec<StatementIr>,
    then_ready: u32,
    else_prefix: Vec<StatementIr>,
    else_ready: u32,
) -> StatementIr {
    StatementIr::AsyncFunctionIf {
        condition: TypedExpr::undefined(),
        then_branch: Box::new(StatementIr::LexicalBlock(then_prefix)),
        else_branch: Some(Box::new(StatementIr::LexicalBlock(else_prefix))),
        plan: crate::AsyncFunctionIfPlanIr::new(entry, then_ready, else_ready)
            .expect("ordered branch ranges")
            .expect("one arm has a continuation"),
    }
}

#[test]
fn nested_value_branches_join_before_the_condition_ready_and_loop_exit() {
    let inner = value_branch(1, vec![suspension(2, 3)], 3, vec![], 4);
    let outer = value_branch(0, vec![inner], 5, vec![suspension(6, 7)], 7);
    let plan = AsyncFunctionWhileConditionIr::new(
        vec![outer],
        TypedExpr::undefined(),
        StatementIr::Empty,
        0,
        8,
    )
    .expect("recursively associated value prefixes");
    assert_eq!((plan.ready_state(), plan.exit_state()), (8, 9));
}

#[test]
fn a_short_arm_cannot_claim_unused_states_before_its_join() {
    // The general span census permits an arm ending before the branch
    // limit. This condition owner requires its actual final ready state.
    let branch = value_branch(0, vec![suspension(1, 2)], 4, vec![], 5);
    assert_eq!(
        crate::async_switch::sequence_exit(&[branch.clone()], 0),
        Ok(6)
    );
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![branch],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            6,
        ),
        Err(AsyncWhileConditionError::StateOrder)
    );
}

#[test]
fn value_arms_require_their_own_containers_and_selected_state_association() {
    let shifted = value_branch(0, vec![suspension(2, 3)], 3, vec![], 4);
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![shifted],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            5,
        ),
        Err(AsyncWhileConditionError::StateOrder)
    );
    let mut unwrapped = value_branch(0, vec![suspension(1, 2)], 2, vec![], 3);
    let StatementIr::AsyncFunctionIf { then_branch, .. } = &mut unwrapped else {
        unreachable!("value_branch constructs If")
    };
    *then_branch = Box::new(suspension(1, 2));
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![unwrapped],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            4,
        ),
        Err(AsyncWhileConditionError::SuspendingPrefix)
    );
    let mut missing_else = value_branch(0, vec![suspension(1, 2)], 2, vec![], 3);
    let StatementIr::AsyncFunctionIf { else_branch, .. } = &mut missing_else else {
        unreachable!("value_branch constructs If")
    };
    *else_branch = None;
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![missing_else],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            4,
        ),
        Err(AsyncWhileConditionError::SuspendingPrefix)
    );
}

#[test]
fn a_valid_nested_loop_span_is_not_a_restartable_expression_value_prefix() {
    let nested = StatementIr::AsyncFunctionWhile(
        AsyncFunctionWhileConditionIr::new(
            vec![suspension(1, 2)],
            TypedExpr::undefined(),
            StatementIr::Empty,
            1,
            2,
        )
        .expect("valid independent loop owner"),
    );
    let branch = value_branch(0, vec![nested], 3, vec![], 4);
    assert_eq!(
        crate::async_switch::sequence_exit(&[branch.clone()], 0),
        Ok(5)
    );
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![branch],
            TypedExpr::undefined(),
            StatementIr::Empty,
            0,
            5,
        ),
        Err(AsyncWhileConditionError::SuspendingPrefix)
    );
}

#[test]
fn statically_erased_awaits_keep_the_restartable_entry_and_a_fresh_exit() {
    let plan = AsyncFunctionWhileConditionIr::new(
        vec![StatementIr::Expression(TypedExpr::undefined())],
        TypedExpr::undefined(),
        StatementIr::Empty,
        7,
        7,
    )
    .expect("an eager prefix still belongs inside the back edge");
    assert_eq!(
        (plan.entry_state(), plan.ready_state(), plan.exit_state()),
        (7, 7, 8)
    );
    assert_eq!(
        AsyncFunctionWhileConditionIr::new(
            vec![],
            TypedExpr::undefined(),
            StatementIr::Empty,
            u32::MAX,
            u32::MAX,
        ),
        Err(AsyncWhileConditionError::StateOverflow)
    );
}
