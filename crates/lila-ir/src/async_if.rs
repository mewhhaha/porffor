/// Disjoint continuation ranges for one plain async function's conditional.
/// The condition owns `entry_state`; each branch starts in a fresh state and
/// normal completion advances to the shared exit without testing it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsyncFunctionIfPlanIr {
    entry_state: u32,
    then_entry_state: u32,
    else_entry_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncIfBranch {
    Then,
    Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncFunctionIfPlanError {
    StateOverflow {
        state: u32,
    },
    BranchExitBeforeEntry {
        branch: AsyncIfBranch,
        entry_state: u32,
        exit_state: u32,
    },
}

impl AsyncFunctionIfPlanIr {
    pub(crate) fn new(
        entry_state: u32,
        then_exit_state: u32,
        else_exit_state: u32,
    ) -> Result<Option<Self>, AsyncFunctionIfPlanError> {
        let successor = |state: u32| {
            state
                .checked_add(1)
                .ok_or(AsyncFunctionIfPlanError::StateOverflow { state })
        };
        let then_entry_state = successor(entry_state)?;
        if then_exit_state < then_entry_state {
            return Err(AsyncFunctionIfPlanError::BranchExitBeforeEntry {
                branch: AsyncIfBranch::Then,
                entry_state: then_entry_state,
                exit_state: then_exit_state,
            });
        }
        let else_entry_state = successor(then_exit_state)?;
        if else_exit_state < else_entry_state {
            return Err(AsyncFunctionIfPlanError::BranchExitBeforeEntry {
                branch: AsyncIfBranch::Else,
                entry_state: else_entry_state,
                exit_state: else_exit_state,
            });
        }
        if then_exit_state == then_entry_state && else_exit_state == else_entry_state {
            return Ok(None);
        }
        Ok(Some(Self {
            entry_state,
            then_entry_state,
            else_entry_state,
            exit_state: successor(else_exit_state)?,
        }))
    }

    pub const fn entry_state(self) -> u32 {
        self.entry_state
    }

    pub const fn then_entry_state(self) -> u32 {
        self.then_entry_state
    }

    pub const fn else_entry_state(self) -> u32 {
        self.else_entry_state
    }

    pub const fn exit_state(self) -> u32 {
        self.exit_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_resumable_branches_do_not_claim_continuation_states() {
        assert_eq!(AsyncFunctionIfPlanIr::new(7, 8, 9), Ok(None));
    }

    #[test]
    fn branch_ranges_and_join_are_disjoint_even_when_one_branch_is_empty() {
        for (then_exit, else_exit, expected) in [
            (2, 4, [0, 1, 3, 5]),
            (1, 3, [0, 1, 2, 4]),
            (2, 3, [0, 1, 3, 4]),
        ] {
            let plan = AsyncFunctionIfPlanIr::new(0, then_exit, else_exit)
                .expect("ordered branches")
                .expect("at least one resumable branch");
            assert_eq!(
                [
                    plan.entry_state(),
                    plan.then_entry_state(),
                    plan.else_entry_state(),
                    plan.exit_state(),
                ],
                expected
            );
        }
    }

    #[test]
    fn invalid_branch_order_and_state_overflow_cannot_form_a_plan() {
        assert!(matches!(
            AsyncFunctionIfPlanIr::new(5, 5, 7),
            Err(AsyncFunctionIfPlanError::BranchExitBeforeEntry {
                branch: AsyncIfBranch::Then,
                ..
            })
        ));
        assert!(matches!(
            AsyncFunctionIfPlanIr::new(5, 7, 7),
            Err(AsyncFunctionIfPlanError::BranchExitBeforeEntry {
                branch: AsyncIfBranch::Else,
                ..
            })
        ));
        for states in [
            (u32::MAX, u32::MAX, u32::MAX),
            (0, u32::MAX, u32::MAX),
            (0, 2, u32::MAX),
        ] {
            assert!(matches!(
                AsyncFunctionIfPlanIr::new(states.0, states.1, states.2),
                Err(AsyncFunctionIfPlanError::StateOverflow { .. })
            ));
        }
    }
}
