//! A complete mixed iterator loop retains head, initialization and body phases.

use crate::async_generator_loop_control::validate_tape;
use crate::async_generator_source::{AsyncGeneratorForOfSourceStates, AsyncGeneratorSourceRange};
use crate::generator_loop_control::has_unowned_head_branch;
use crate::lowering::CheckedAsyncGeneratorForOfInitializer;
use crate::{
    BindingMode, ExprIr, ForInOfEnvironmentIr, LexicalEnvironmentInitializationIr,
    OwnedEnvBindingIr, ResumableExpressionIr, ResumableRegionIr, ResumableRegionProtocolIr,
    ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, ResumableSuspensionPointIr,
    StatementIr,
};
use std::collections::BTreeSet;

/// The actual syntax selects synchronous iteration or the two implicit Await
/// sites. Only the checked complete carrier authorizes native use of these PCs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncGeneratorIteratorProtocolIr {
    Sync,
    Awaited {
        next_suspend_state: u32,
        next_resume_state: u32,
        close_suspend_state: u32,
        close_resume_state: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorForOfError {
    InvalidStates,
    ForeignInitializer,
    InvalidEnvironment,
    UnallocatedBinding,
    AliasedBindings,
    MissingHeadPublication,
    UnconsumedSourceSuspension,
}

#[must_use = "the complete iterator owner must reach native dispatch"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorForOfIr {
    head: ResumableExpressionIr,
    head_binding: OwnedEnvBindingIr,
    incoming_binding: OwnedEnvBindingIr,
    value_binding: OwnedEnvBindingIr,
    initializer: CheckedAsyncGeneratorForOfInitializer,
    body: ResumableRegionIr,
    execution: ResumableRegionProtocolIr,
    acquisition_state: u32,
    advance_state: u32,
    exit_state: u32,
    protocol: AsyncGeneratorIteratorProtocolIr,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorForOfIr {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        states: AsyncGeneratorForOfSourceStates,
        head: impl Into<ResumableExpressionIr>,
        head_binding: OwnedEnvBindingIr,
        incoming_binding: OwnedEnvBindingIr,
        value_binding: OwnedEnvBindingIr,
        initializer: CheckedAsyncGeneratorForOfInitializer,
        body: impl Into<ResumableRegionIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorForOfError> {
        use AsyncGeneratorForOfError as Error;
        let head = head.into();
        let body = body.into();
        if !initializer.matches_source(&states)
            || initializer.mode() != states.head_mode()
            || initializer.incoming_binding() != &incoming_binding
        {
            return Err(Error::ForeignInitializer);
        }
        let initialization = initializer.region();
        if states.execution() == ResumableRegionProtocolIr::Generator
            && !matches!(states.protocol(), AsyncGeneratorIteratorProtocolIr::Sync)
        {
            return Err(Error::InvalidStates);
        }
        if [head.region(), initialization, &body]
            .iter()
            .any(|region| region.protocol() != states.execution())
            || !matches_range(head.region(), states.head())
            || !matches_range(initialization, states.initialization())
            || !matches_range(&body, states.body())
            || states.head().end().checked_add(1) != Some(states.acquisition_state())
            || states.acquisition_state().checked_add(1) != Some(states.advance_state())
            || states.initialization().end().checked_add(1) != Some(states.body().entry())
            || states.exit().checked_add(1).is_none()
            || [head.region(), initialization, &body]
                .iter()
                .any(|region| region.block().lexical_environment.is_some())
            || [head.region(), initialization].iter().any(|region| {
                region
                    .block()
                    .statements
                    .iter()
                    .any(has_unowned_head_branch)
            })
        {
            return Err(Error::InvalidStates);
        }
        let iteration_exit = match (states.resource(), initializer.resource()) {
            (None, None) => states
                .body()
                .end()
                .checked_add(1)
                .ok_or(Error::InvalidStates)?,
            (Some(source), Some(resource)) if resource.matches_source(source) => {
                match resource.capability() {
                    crate::AsyncGeneratorResourceCapabilityIr::Sync(_) => {
                        if states.body().end().checked_add(1) != Some(resource.exit_state()) {
                            return Err(Error::InvalidStates);
                        }
                    }
                    crate::AsyncGeneratorResourceCapabilityIr::Async(capability) => {
                        if crate::AsyncDisposableFinalizerPlanIr::after_source_suffix(
                            states.initialization().entry(),
                            states.body().end(),
                        )
                        .as_ref()
                            != Some(capability.finalizer())
                        {
                            return Err(Error::InvalidStates);
                        }
                    }
                }
                resource.exit_state()
            }
            _ => return Err(Error::ForeignInitializer),
        };
        match states.protocol() {
            AsyncGeneratorIteratorProtocolIr::Sync => {
                if states.advance_state().checked_add(1) != Some(states.initialization().entry())
                    || iteration_exit != states.exit()
                {
                    return Err(Error::InvalidStates);
                }
            }
            AsyncGeneratorIteratorProtocolIr::Awaited {
                next_suspend_state,
                next_resume_state,
                close_suspend_state,
                close_resume_state,
            } => {
                if next_suspend_state != states.advance_state()
                    || next_suspend_state.checked_add(1) != Some(next_resume_state)
                    || next_resume_state.checked_add(1) != Some(states.initialization().entry())
                    || iteration_exit != close_suspend_state
                    || close_suspend_state.checked_add(1) != Some(close_resume_state)
                    || close_resume_state.checked_add(1) != Some(states.exit())
                {
                    return Err(Error::InvalidStates);
                }
            }
        }
        let mut bindings = vec![&head_binding, &incoming_binding, &value_binding];
        if let Some(resource) = initializer.resource() {
            bindings.push(resource.capability_binding());
        }
        let names: BTreeSet<_> = bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect();
        let slots: BTreeSet<_> = bindings.iter().map(|binding| binding.slot).collect();
        if names.len() != bindings.len() || slots.len() != bindings.len() {
            return Err(Error::AliasedBindings);
        }
        for binding in bindings {
            let mut rows = inventory
                .iter()
                .filter(|row| row.name == binding.name || row.slot == binding.slot);
            if rows.next() != Some(binding) || rows.next().is_some() {
                return Err(Error::UnallocatedBinding);
            }
        }
        let ExprIr::Identifier(read) = &head.value().expr else {
            return Err(Error::MissingHeadPublication);
        };
        let Some(StatementIr::Lexical {
            mode: BindingMode::Let,
            name,
            init,
        }) = head.region().block().statements.last()
        else {
            return Err(Error::MissingHeadPublication);
        };
        if read != &head_binding.name || name != read || init.value_info() != head.value().value_info()
            || head.region().block().statements.iter().filter(|statement| matches!(statement, StatementIr::Lexical { name, .. } if name == read)).count() != 1
        {
            return Err(Error::MissingHeadPublication);
        }
        match (initializer.mode(), initializer.environment()) {
            (BindingMode::Let | BindingMode::Const, Some(environment)) => {
                let tdz: BTreeSet<_> = environment
                    .tdz_binding_names
                    .iter()
                    .map(String::as_str)
                    .collect();
                if tdz.len() != environment.tdz_binding_names.len() || !tdz.is_disjoint(&names) {
                    return Err(Error::InvalidEnvironment);
                }
                for record in [
                    environment.tdz_environment.as_ref(),
                    environment.iteration_environment.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    let record_names: BTreeSet<_> = record
                        .bindings
                        .iter()
                        .map(|binding| binding.name.as_str())
                        .collect();
                    let record_slots: BTreeSet<_> =
                        record.bindings.iter().map(|binding| binding.slot).collect();
                    if record.initialization != LexicalEnvironmentInitializationIr::Uninitialized
                        || matches!(
                            record.eval_environment,
                            Some(crate::EvalEnvironmentRoleIr::WithObject { .. })
                        )
                        || record_names.len() != record.bindings.len()
                        || record_slots.len() != record.bindings.len()
                        || !record_names.is_disjoint(&names)
                    {
                        return Err(Error::InvalidEnvironment);
                    }
                }
            }
            (BindingMode::Var, None) => {}
            _ => return Err(Error::InvalidEnvironment),
        }
        let mut suspensions = Vec::new();
        head.region().collect_suspensions(&mut suspensions);
        if let AsyncGeneratorIteratorProtocolIr::Awaited {
            next_suspend_state,
            next_resume_state,
            ..
        } = states.protocol()
        {
            suspensions.push(implicit_point(
                ResumableSuspensionKindIr::ForAwaitNext,
                next_suspend_state,
                next_resume_state,
            ));
        }
        initialization.collect_suspensions(&mut suspensions);
        body.collect_suspensions(&mut suspensions);
        if let AsyncGeneratorIteratorProtocolIr::Awaited {
            close_suspend_state,
            close_resume_state,
            ..
        } = states.protocol()
        {
            suspensions.push(implicit_point(
                ResumableSuspensionKindIr::ForAwaitClose,
                close_suspend_state,
                close_resume_state,
            ));
        }
        validate_tape(states.suspensions(), &suspensions)
            .map_err(|_| Error::UnconsumedSourceSuspension)?;
        Ok(Self {
            head,
            head_binding,
            incoming_binding,
            value_binding,
            initializer,
            body,
            acquisition_state: states.acquisition_state(),
            advance_state: states.advance_state(),
            exit_state: states.exit(),
            protocol: states.protocol(),
            execution: states.execution(),
            suspensions,
        })
    }

    pub fn head(&self) -> &ResumableExpressionIr {
        &self.head
    }
    pub fn head_binding(&self) -> &OwnedEnvBindingIr {
        &self.head_binding
    }
    pub fn incoming_binding(&self) -> &OwnedEnvBindingIr {
        &self.incoming_binding
    }
    pub fn value_binding(&self) -> &OwnedEnvBindingIr {
        &self.value_binding
    }
    pub fn initialization(&self) -> &ResumableRegionIr {
        self.initializer.region()
    }
    pub fn resource(&self) -> Option<&crate::AsyncGeneratorForOfResourceIr> {
        self.initializer.resource()
    }
    pub fn lexical_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.initializer.environment()
    }
    pub fn head_mode(&self) -> BindingMode {
        self.initializer.mode()
    }
    pub fn body(&self) -> &ResumableRegionIr {
        &self.body
    }
    pub fn execution(&self) -> ResumableRegionProtocolIr {
        self.execution
    }
    pub fn entry_state(&self) -> u32 {
        self.head.region().entry_state()
    }
    pub const fn acquisition_state(&self) -> u32 {
        self.acquisition_state
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
    pub const fn protocol(&self) -> AsyncGeneratorIteratorProtocolIr {
        self.protocol
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

fn matches_range(region: &ResumableRegionIr, range: AsyncGeneratorSourceRange) -> bool {
    region.entry_state() == range.entry() && region.end_state() == range.end()
}

fn implicit_point(
    kind: ResumableSuspensionKindIr,
    suspend_state: u32,
    resume_state: u32,
) -> ResumableSuspensionPointIr {
    ResumableSuspensionPointIr {
        kind,
        suspend_state,
        resume_state,
        resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
    }
}

#[cfg(test)]
mod tests;
