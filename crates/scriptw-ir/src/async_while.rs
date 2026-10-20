mod condition_prefix;

use crate::{StatementIr, SynchronousLoopBodyIr, TypedExpr};

/// An awaited condition with an eager body. The private owner proves that the
/// condition's Await/If prefixes have exact associated state ranges and its
/// body cannot suspend. A statically skipped Await may leave an eager prefix.
/// The back edge must restart the condition; completion must advance past it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionWhileConditionIr {
    condition_prefix: Vec<StatementIr>,
    condition: TypedExpr,
    body: Box<StatementIr>,
    entry_state: u32,
    ready_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncWhileConditionError {
    StateOrder,
    StateOverflow,
    SuspendingPrefix,
    SuspendingBody,
}

impl AsyncFunctionWhileConditionIr {
    pub(crate) fn new(
        condition_prefix: Vec<StatementIr>,
        condition: TypedExpr,
        body: StatementIr,
        entry_state: u32,
        ready_state: u32,
    ) -> Result<Self, AsyncWhileConditionError> {
        condition_prefix::validate_condition_prefix(&condition_prefix, entry_state, ready_state)?;
        SynchronousLoopBodyIr::new(&body).map_err(|_| AsyncWhileConditionError::SuspendingBody)?;
        Ok(Self {
            condition_prefix,
            condition,
            body: Box::new(body),
            entry_state,
            ready_state,
            exit_state: ready_state
                .checked_add(1)
                .ok_or(AsyncWhileConditionError::StateOverflow)?,
        })
    }

    pub fn condition_prefix(&self) -> &[StatementIr] {
        &self.condition_prefix
    }
    pub fn condition(&self) -> &TypedExpr {
        &self.condition
    }
    pub fn body(&self) -> &StatementIr {
        &self.body
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn ready_state(&self) -> u32 {
        self.ready_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

#[cfg(test)]
mod tests;
