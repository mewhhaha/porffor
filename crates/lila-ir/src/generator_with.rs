//! With retains its analyzed Object Environment Record across the whole body.

use crate::generator_loop_control::collect_suspensions;
use crate::lowering_helpers::GeneratorWithSourceStates;
use crate::with_object_environment::{CheckedWithObjectEnvironmentIr, WithObjectEnvironmentError};
use crate::{
    GeneratorLoopExpressionIr, GeneratorLoopRegionIr, LexicalEnvironmentIr, OwnedEnvBindingIr,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorWithControlError {
    InvalidStates,
    ForeignHead,
    UnallocatedHead,
    ForeignObjectEnvironment,
    UnconsumedSourceSuspension,
}

impl From<WithObjectEnvironmentError> for GeneratorWithControlError {
    fn from(error: WithObjectEnvironmentError) -> Self {
        match error {
            WithObjectEnvironmentError::ForeignHead => Self::ForeignHead,
            WithObjectEnvironmentError::UnallocatedHead => Self::UnallocatedHead,
            WithObjectEnvironmentError::ForeignObjectEnvironment => Self::ForeignObjectEnvironment,
        }
    }
}

/// The head completes outside this record. Fresh and resumed body entries use
/// the same original hidden object cell, with one outward completion cleanup.
#[must_use = "the complete With environment lifetime must reach its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorWithIr {
    head: GeneratorLoopExpressionIr,
    object_environment: CheckedWithObjectEnvironmentIr,
    body: GeneratorLoopRegionIr,
    exit_state: u32,
}

impl OrdinaryGeneratorWithIr {
    pub(crate) fn new(
        states: GeneratorWithSourceStates,
        head: GeneratorLoopExpressionIr,
        head_binding: OwnedEnvBindingIr,
        object_binding: OwnedEnvBindingIr,
        lexical_environment: LexicalEnvironmentIr,
        body: GeneratorLoopRegionIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, GeneratorWithControlError> {
        let head_range = states.head();
        let body_range = states.body();
        if head_range.entry != states.entry()
            || head_range.end.checked_add(1) != Some(body_range.entry)
            || body_range.end.checked_add(1) != Some(states.exit())
            || head.region().entry_state() != head_range.entry
            || head.region().end_state() != head_range.end
            || body.entry_state() != body_range.entry
            || body.end_state() != body_range.end
            || head.region().block().lexical_environment.is_some()
            || body.block().lexical_environment.is_some()
        {
            return Err(GeneratorWithControlError::InvalidStates);
        }
        let object_environment = CheckedWithObjectEnvironmentIr::new(
            head.region().block(),
            head.value(),
            head_binding,
            object_binding,
            lexical_environment,
            inventory,
        )?;
        let mut points = Vec::new();
        collect_suspensions(&head.region().block().statements, &mut points);
        collect_suspensions(&body.block().statements, &mut points);
        if points != states.suspensions() {
            return Err(GeneratorWithControlError::UnconsumedSourceSuspension);
        }
        Ok(Self {
            head,
            object_environment,
            body,
            exit_state: states.exit(),
        })
    }

    pub fn entry_state(&self) -> u32 {
        self.head.region().entry_state()
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn head(&self) -> &GeneratorLoopExpressionIr {
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
    pub fn body(&self) -> &GeneratorLoopRegionIr {
        &self.body
    }
}
