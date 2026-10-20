//! Complete mixed enumeration consumes its original head and four-cell proof.

use crate::async_generator_loop_control::validate_tape;
use crate::async_generator_source::{AsyncGeneratorForInSourceStates, AsyncGeneratorSourceRange};
use crate::for_in_storage::{CheckedForInStorageIr, ForInStorageError};
use crate::generator_loop_control::has_unowned_head_branch;
use crate::lowering::CheckedAsyncGeneratorForInInitializer;
use crate::lowering_helpers::GeneratorForInHeadProof;
use crate::{
    BindingMode, BlockIr, ForInOfEnvironmentIr, OwnedEnvBindingIr, ResumableExpressionIr,
    ResumableRegionIr, ResumableRegionProtocolIr, ResumableSuspensionPointIr,
};

#[cfg(test)]
#[path = "async_for_in/tests.rs"]
mod async_protocol_tests;
#[cfg(test)]
#[path = "generator_for_in/tests.rs"]
mod generator_protocol_tests;
#[cfg(test)]
mod protocol_test_support;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorForInControlError {
    ForeignSourceHead,
    InvalidStates,
    UnallocatedBinding,
    AliasedBindings,
    MissingHeadPublication,
    InvalidInitialization,
    ForeignLexicalEnvironment,
    UnconsumedSourceSuspension,
}

#[must_use = "the checked mixed enumeration must reach its actual statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorForInIr {
    execution: ResumableRegionProtocolIr,
    head: ResumableExpressionIr,
    storage: CheckedForInStorageIr,
    body: ResumableRegionIr,
    head_mode: BindingMode,
    advance_state: u32,
    exit_state: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorForInIr {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        states: AsyncGeneratorForInSourceStates,
        head_proof: GeneratorForInHeadProof,
        head: impl Into<ResumableExpressionIr>,
        head_binding: OwnedEnvBindingIr,
        initialization: CheckedAsyncGeneratorForInInitializer,
        lexical_environment: Option<ForInOfEnvironmentIr>,
        enumerator_binding: OwnedEnvBindingIr,
        key_binding: OwnedEnvBindingIr,
        value_binding: OwnedEnvBindingIr,
        body: impl Into<ResumableRegionIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, GeneratorForInControlError> {
        let head = head.into();
        let body = body.into();
        let execution = states.execution();
        if head.region().protocol() != execution
            || body.protocol() != execution
            || initialization.region().protocol() != execution
        {
            return Err(GeneratorForInControlError::InvalidStates);
        }
        if !states.matches_head(&head_proof)
            || states.head_kind() != head_proof.kind()
            || !initialization.matches_source(&states)
        {
            return Err(GeneratorForInControlError::ForeignSourceHead);
        }
        if states.head().entry() != states.entry()
            || states.head().end().checked_add(1) != Some(states.advance_state())
            || states.advance_state().checked_add(1) != Some(states.initialization().entry())
            || states.initialization().end().checked_add(1) != Some(states.body().entry())
            || states.body().end().checked_add(1) != Some(states.exit())
            || states.exit().checked_add(1).is_none()
            || !matches_range(head.region(), states.head())
            || !matches_range(&body, states.body())
            || !matches_range(initialization.region(), states.initialization())
            || initialization
                .region()
                .block()
                .lexical_environment
                .is_some()
            || initialization
                .region()
                .block()
                .statements
                .iter()
                .any(has_unowned_head_branch)
            || body.block().lexical_environment.is_some()
            || head
                .region()
                .block()
                .statements
                .iter()
                .any(has_unowned_head_branch)
        {
            return Err(GeneratorForInControlError::InvalidStates);
        }
        let storage = CheckedForInStorageIr::new(
            &head_proof,
            head.region().block(),
            head.value(),
            head_binding,
            initialization,
            lexical_environment,
            enumerator_binding,
            key_binding,
            value_binding,
            inventory,
        )
        .map_err(|error| match error {
            ForInStorageError::InvalidHeadEnvironment => GeneratorForInControlError::InvalidStates,
            ForInStorageError::UnallocatedBinding => GeneratorForInControlError::UnallocatedBinding,
            ForInStorageError::AliasedBindings => GeneratorForInControlError::AliasedBindings,
            ForInStorageError::MissingHeadPublication => {
                GeneratorForInControlError::MissingHeadPublication
            }
            ForInStorageError::ForeignSourceHead => GeneratorForInControlError::ForeignSourceHead,
            ForInStorageError::InvalidInitialization => {
                GeneratorForInControlError::InvalidInitialization
            }
            ForInStorageError::ForeignLexicalEnvironment => {
                GeneratorForInControlError::ForeignLexicalEnvironment
            }
        })?;
        let mut actual = Vec::new();
        head.region().collect_suspensions(&mut actual);
        storage
            .initialization()
            .region()
            .collect_suspensions(&mut actual);
        body.collect_suspensions(&mut actual);
        validate_tape(states.suspensions(), &actual)
            .map_err(|_| GeneratorForInControlError::UnconsumedSourceSuspension)?;
        Ok(Self {
            execution,
            head,
            storage,
            body,
            head_mode: states.head_mode(),
            advance_state: states.advance_state(),
            exit_state: states.exit(),
            suspensions: actual,
        })
    }

    pub fn execution(&self) -> ResumableRegionProtocolIr {
        self.execution
    }
    pub fn head(&self) -> &ResumableExpressionIr {
        &self.head
    }
    pub fn body(&self) -> &ResumableRegionIr {
        &self.body
    }
    pub fn head_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.head_binding()
    }
    pub fn initialization(&self) -> &BlockIr {
        self.initialization_region().block()
    }
    pub fn initialization_region(&self) -> &ResumableRegionIr {
        self.storage.initialization().region()
    }
    pub fn lexical_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.storage.lexical_environment()
    }
    pub fn enumerator_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.enumerator_binding()
    }
    pub fn key_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.key_binding()
    }
    pub fn value_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.value_binding()
    }
    pub const fn head_mode(&self) -> BindingMode {
        self.head_mode
    }
    pub fn entry_state(&self) -> u32 {
        self.head.region().entry_state()
    }
    pub const fn advance_state(&self) -> u32 {
        self.advance_state
    }
    pub const fn continue_state(&self) -> u32 {
        self.advance_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

fn matches_range(region: &ResumableRegionIr, range: AsyncGeneratorSourceRange) -> bool {
    region.entry_state() == range.entry() && region.end_state() == range.end()
}
