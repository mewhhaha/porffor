/// A plain async non-loop label owns the continuation after its complete body.
/// Normal completion and a matching labelled break both commit this fresh exit;
/// an early break must not depend on reaching a child's await or condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsyncFunctionLabelledPlanIr {
    entry_state: u32,
    body_exit_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncFunctionLabelledPlanError {
    BodyExitBeforeContinuation,
    StateOverflow,
}

impl AsyncFunctionLabelledPlanIr {
    pub(crate) fn new(
        entry_state: u32,
        body_exit_state: u32,
    ) -> Result<Self, AsyncFunctionLabelledPlanError> {
        if body_exit_state <= entry_state {
            return Err(AsyncFunctionLabelledPlanError::BodyExitBeforeContinuation);
        }
        let exit_state = body_exit_state
            .checked_add(1)
            .ok_or(AsyncFunctionLabelledPlanError::StateOverflow)?;
        Ok(Self {
            entry_state,
            body_exit_state,
            exit_state,
        })
    }

    pub const fn entry_state(self) -> u32 {
        self.entry_state
    }

    pub const fn body_exit_state(self) -> u32 {
        self.body_exit_state
    }

    pub const fn exit_state(self) -> u32 {
        self.exit_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_exit_is_distinct_from_the_last_child_segment() {
        let plan = AsyncFunctionLabelledPlanIr::new(4, 7).expect("ordered body");
        assert_eq!(
            (
                plan.entry_state(),
                plan.body_exit_state(),
                plan.exit_state()
            ),
            (4, 7, 8)
        );
    }

    #[test]
    fn missing_or_reversed_body_continuations_cannot_claim_a_region() {
        for states in [(4, 4), (4, 3)] {
            assert_eq!(
                AsyncFunctionLabelledPlanIr::new(states.0, states.1),
                Err(AsyncFunctionLabelledPlanError::BodyExitBeforeContinuation)
            );
        }
    }

    #[test]
    fn an_unrepresentable_region_exit_cannot_form_a_plan() {
        assert_eq!(
            AsyncFunctionLabelledPlanIr::new(0, u32::MAX),
            Err(AsyncFunctionLabelledPlanError::StateOverflow)
        );
    }
}
