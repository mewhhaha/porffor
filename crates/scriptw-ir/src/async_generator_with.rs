//! Complete mixed With keeps the original checked Object Environment Record.

use crate::async_generator_loop_control::{validate_tape, AsyncGeneratorControlError};
use crate::async_generator_source::AsyncGeneratorWithSourceStates;
use crate::generator_loop_control::{collect_mixed_suspensions, has_unowned_head_branch};
use crate::with_object_environment::{CheckedWithObjectEnvironmentIr, WithObjectEnvironmentError};
use crate::{
    AsyncGeneratorLoopExpressionIr, AsyncGeneratorLoopRegionIr, LexicalEnvironmentIr,
    OwnedEnvBindingIr, ResumableSuspensionPointIr, TypedExpr,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorWithControlError {
    InvalidStates,
    ForeignContinuation,
    ObjectEnvironment(WithObjectEnvironmentError),
    SourceTape(AsyncGeneratorControlError),
}

/// The original boxed head is complete before the body enters its one record.
/// The private environment proof cannot be replaced by unvalidated raw rows.
#[must_use = "the checked mixed With must reach its actual statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorWithIr {
    head: AsyncGeneratorLoopExpressionIr,
    object_environment: CheckedWithObjectEnvironmentIr,
    body: AsyncGeneratorLoopRegionIr,
    exit_state: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorWithIr {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        states: AsyncGeneratorWithSourceStates,
        head: AsyncGeneratorLoopExpressionIr,
        head_binding: OwnedEnvBindingIr,
        object_binding: OwnedEnvBindingIr,
        lexical_environment: LexicalEnvironmentIr,
        body: AsyncGeneratorLoopRegionIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorWithControlError> {
        let head_range = states.head();
        let body_range = states.body();
        if head_range.end().checked_add(1) != Some(body_range.entry())
            || body_range.end().checked_add(1) != Some(states.exit())
            || head.region().entry_state() != head_range.entry()
            || head.region().end_state() != head_range.end()
            || body.entry_state() != body_range.entry()
            || body.end_state() != body_range.end()
            || head.region().block().lexical_environment.is_some()
            || body.block().lexical_environment.is_some()
        {
            return Err(AsyncGeneratorWithControlError::InvalidStates);
        }
        if head
            .region()
            .block()
            .statements
            .iter()
            .any(has_unowned_head_branch)
        {
            return Err(AsyncGeneratorWithControlError::ForeignContinuation);
        }
        let object_environment = CheckedWithObjectEnvironmentIr::new(
            head.region().block(),
            head.value(),
            head_binding,
            object_binding,
            lexical_environment,
            inventory,
        )
        .map_err(AsyncGeneratorWithControlError::ObjectEnvironment)?;
        let mut suspensions = Vec::new();
        collect_mixed_suspensions(&head.region().block().statements, &mut suspensions);
        collect_mixed_suspensions(&body.block().statements, &mut suspensions);
        validate_tape(states.suspensions(), &suspensions)
            .map_err(AsyncGeneratorWithControlError::SourceTape)?;
        Ok(Self {
            head,
            object_environment,
            body,
            exit_state: states.exit(),
            suspensions,
        })
    }

    pub fn head(&self) -> &AsyncGeneratorLoopExpressionIr {
        &self.head
    }
    pub fn head_binding(&self) -> &OwnedEnvBindingIr {
        self.object_environment.head_binding()
    }
    pub fn object_binding(&self) -> &OwnedEnvBindingIr {
        self.object_environment.object_binding()
    }
    pub fn lexical_environment(&self) -> &LexicalEnvironmentIr {
        self.object_environment.lexical_environment()
    }
    pub fn body(&self) -> &AsyncGeneratorLoopRegionIr {
        &self.body
    }
    pub fn entry_state(&self) -> u32 {
        self.head.region().entry_state()
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn regions(&self) -> impl Iterator<Item = &AsyncGeneratorLoopRegionIr> {
        [self.head.region(), &self.body].into_iter()
    }
    pub fn expressions(&self) -> impl Iterator<Item = &TypedExpr> {
        std::iter::once(self.head.value())
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}
