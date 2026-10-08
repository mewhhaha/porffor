use super::{AsyncWhileConditionError, StatementIr, SynchronousLoopBodyIr};

/// Admit only the expression value owners this loop can restart. A general
/// continuation census also admits Try/Switch/class/loop owners, which do not
/// belong to this condition lifecycle. Blocks containing hidden Await remain
/// refused except for the checked value owner's own lexical arm containers.
pub(super) fn validate_condition_prefix(
    statements: &[StatementIr],
    entry: u32,
    ready: u32,
) -> Result<(), AsyncWhileConditionError> {
    let mut state = entry;
    for statement in statements {
        match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => {
                let successor = state
                    .checked_add(1)
                    .ok_or(AsyncWhileConditionError::StateOverflow)?;
                if *suspend_state != state || *resume_state != successor {
                    return Err(AsyncWhileConditionError::StateOrder);
                }
                state = successor;
            }
            StatementIr::AsyncFunctionIf {
                plan,
                then_branch,
                else_branch,
                ..
            } => {
                if plan.entry_state() != state
                    || state.checked_add(1) != Some(plan.then_entry_state())
                {
                    return Err(AsyncWhileConditionError::StateOrder);
                }
                let then_ready = plan
                    .else_entry_state()
                    .checked_sub(1)
                    .ok_or(AsyncWhileConditionError::StateOrder)?;
                let else_ready = plan
                    .exit_state()
                    .checked_sub(1)
                    .ok_or(AsyncWhileConditionError::StateOrder)?;
                validate_value_arm(then_branch, plan.then_entry_state(), then_ready)?;
                let branch = else_branch
                    .as_deref()
                    .ok_or(AsyncWhileConditionError::SuspendingPrefix)?;
                validate_value_arm(branch, plan.else_entry_state(), else_ready)?;
                state = plan.exit_state();
            }
            eager => {
                SynchronousLoopBodyIr::new(eager)
                    .map_err(|_| AsyncWhileConditionError::SuspendingPrefix)?;
            }
        }
    }
    if state != ready || crate::async_switch::sequence_exit(statements, entry).ok() != Some(ready) {
        return Err(AsyncWhileConditionError::StateOrder);
    }
    Ok(())
}

fn validate_value_arm(
    branch: &StatementIr,
    entry: u32,
    ready: u32,
) -> Result<(), AsyncWhileConditionError> {
    let StatementIr::LexicalBlock(statements) = branch else {
        return Err(AsyncWhileConditionError::SuspendingPrefix);
    };
    validate_condition_prefix(statements, entry, ready)
}
