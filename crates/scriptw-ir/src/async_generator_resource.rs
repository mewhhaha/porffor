//! Complete source-owned resource registration and disposal across suspension.

use crate::async_array_destructuring::PatternCells;
use crate::async_generator_loop_control::validate_tape;
use crate::async_generator_source::{
    AsyncGeneratorForOfResourceSourceStates, AsyncGeneratorResourceRegistrationSource,
    AsyncGeneratorResourceScopeFinalizationSource, AsyncGeneratorResourceScopeSourceStates,
    AsyncGeneratorSourceRange,
};
use crate::generator_loop_control::{collect_mixed_suspensions, mixed_sequence_end};
use crate::lowering::CheckedAsyncGeneratorResourceRegistration;
use crate::{
    AsyncDisposableFinalizerPlanIr, AsyncGeneratorAsyncDisposableCapabilityIr,
    AsyncGeneratorLoopRegionIr, AsyncGeneratorSyncDisposableCapabilityIr, BlockIr, ExprIr,
    OwnedEnvBindingIr, ResumableRegionIr, ResumableRegionProtocolIr, ResumableSuspensionPointIr,
    StatementIr, TypedExpr,
};

mod scoped;
#[cfg(test)]
mod tests;
pub use scoped::AsyncGeneratorScopedResourceIr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceDisposalHintIr {
    Sync,
    Async,
}

/// These are the original capability representations, selected once for the
/// whole lexical resource scope. A mixed scope uses the async capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncGeneratorResourceCapabilityIr {
    Sync(AsyncGeneratorSyncDisposableCapabilityIr),
    Async(AsyncGeneratorAsyncDisposableCapabilityIr),
}

/// A per-key capability lives across the actual iterator initializer and body.
/// Its original finalizer completes before the iterator can advance or close.
#[must_use = "the per-key capability must stay attached to its complete iterator"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorForOfResourceIr {
    capability_binding: OwnedEnvBindingIr,
    capability: AsyncGeneratorResourceCapabilityIr,
    source: AsyncGeneratorResourceRegistrationSource,
    exit_state: u32,
}

impl AsyncGeneratorForOfResourceIr {
    pub(crate) fn for_iteration(
        states: &AsyncGeneratorForOfResourceSourceStates,
        range: AsyncGeneratorSourceRange,
        capability_binding: OwnedEnvBindingIr,
        incoming_binding: &OwnedEnvBindingIr,
        registration: AsyncGeneratorResourceRegistrationIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<(Self, AsyncGeneratorLoopRegionIr), AsyncGeneratorResourceError> {
        use AsyncGeneratorResourceError as Error;
        if range.entry() != range.end()
            || states.registration().register_state() != range.entry()
            || registration.source() != states.registration()
            || registration.capability_binding() != &capability_binding
            || registration.hint() != states.hint()
            || states.exit() <= range.end()
            || states.exit().checked_add(1).is_none()
        {
            return Err(Error::InvalidRegistration);
        }
        let ExprIr::Identifier(name) = &registration.initializer().expr else {
            return Err(Error::UnallocatedInitializer);
        };
        if name != &incoming_binding.name {
            return Err(Error::UnallocatedInitializer);
        }
        let mut cells = PatternCells::new(inventory);
        cells
            .insert(&capability_binding)
            .map_err(|_| Error::InvalidCapability)?;
        cells
            .insert(incoming_binding)
            .map_err(|_| Error::AliasedStorage)?;
        let capability = match (states.hint(), states.finalizer()) {
            (ResourceDisposalHintIr::Sync, None) => AsyncGeneratorResourceCapabilityIr::Sync(
                AsyncGeneratorSyncDisposableCapabilityIr::new(capability_binding.name.clone()),
            ),
            (ResourceDisposalHintIr::Async, Some(finalizer))
                if finalizer.entry_state() == range.entry()
                    && finalizer.exit_state() == states.exit() =>
            {
                AsyncGeneratorResourceCapabilityIr::Async(
                    AsyncGeneratorAsyncDisposableCapabilityIr::new(
                        capability_binding.name.clone(),
                        finalizer.clone(),
                    ),
                )
            }
            _ => return Err(Error::InvalidStates),
        };
        let owner = Self {
            capability_binding,
            capability,
            source: registration.source().clone(),
            exit_state: states.exit(),
        };
        let body = BlockIr {
            statements: vec![StatementIr::AsyncGeneratorResourceRegistration(Box::new(
                registration,
            ))],
            result_kind: crate::ValueKind::Undefined,
            lexical_environment: None,
        };
        Ok((
            owner,
            AsyncGeneratorLoopRegionIr::from_resource_scope(CheckedResourceBody {
                block: body,
                range,
                execution: ResumableRegionProtocolIr::AsyncGenerator,
            }),
        ))
    }

    pub(crate) fn matches_source(&self, states: &AsyncGeneratorForOfResourceSourceStates) -> bool {
        self.source == *states.registration()
            && self.exit_state == states.exit()
            && match (&self.capability, states.hint(), states.finalizer()) {
                (
                    AsyncGeneratorResourceCapabilityIr::Sync(_),
                    ResourceDisposalHintIr::Sync,
                    None,
                ) => true,
                (
                    AsyncGeneratorResourceCapabilityIr::Async(capability),
                    ResourceDisposalHintIr::Async,
                    Some(finalizer),
                ) => capability.finalizer() == finalizer,
                _ => false,
            }
    }
    pub fn capability_binding(&self) -> &OwnedEnvBindingIr {
        &self.capability_binding
    }
    pub fn capability(&self) -> &AsyncGeneratorResourceCapabilityIr {
        &self.capability
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorResourceRegistrationIr(CheckedAsyncGeneratorResourceRegistration);

impl AsyncGeneratorResourceRegistrationIr {
    pub(crate) fn new(proof: CheckedAsyncGeneratorResourceRegistration) -> Self {
        Self(proof)
    }
    pub fn hint(&self) -> ResourceDisposalHintIr {
        self.0.source().hint()
    }
    pub fn binding_name(&self) -> &str {
        self.0.binding_name()
    }
    pub fn initializer(&self) -> &TypedExpr {
        self.0.initializer()
    }
    pub fn capability_binding(&self) -> &OwnedEnvBindingIr {
        self.0.capability_binding()
    }
    pub(crate) fn source(&self) -> &AsyncGeneratorResourceRegistrationSource {
        self.0.source()
    }
}

pub(crate) struct CheckedResourceBody {
    block: BlockIr,
    range: AsyncGeneratorSourceRange,
    execution: ResumableRegionProtocolIr,
}
impl CheckedResourceBody {
    pub(crate) fn into_parts(self) -> (BlockIr, AsyncGeneratorSourceRange) {
        (self.block, self.range)
    }
    pub(crate) fn into_protocol_parts(
        self,
    ) -> (
        BlockIr,
        AsyncGeneratorSourceRange,
        ResumableRegionProtocolIr,
    ) {
        (self.block, self.range, self.execution)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorResourceError {
    InvalidStates,
    InvalidCapability,
    InvalidRegistration,
    UnallocatedInitializer,
    AliasedStorage,
    UnconsumedSourceRegistration,
    UnconsumedSourceSuspension,
}

#[must_use = "one complete resource scope must own all registrations and disposal"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorResourceScopeIr {
    capability_binding: OwnedEnvBindingIr,
    capability: AsyncGeneratorResourceCapabilityIr,
    body: ResumableRegionIr,
    exit_state: u32,
    capacity: usize,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorResourceScopeIr {
    pub(crate) fn new(
        states: AsyncGeneratorResourceScopeSourceStates,
        capability_binding: OwnedEnvBindingIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorResourceError> {
        use AsyncGeneratorResourceError as Error;
        let range = states.body();
        let execution = states.execution();
        let exit_state = states.exit();
        if range.entry() != states.entry()
            || range.end() < range.entry()
            || body.lexical_environment.is_some()
            || states.registrations().is_empty()
            || states.exit().checked_add(1).is_none()
        {
            return Err(Error::InvalidStates);
        }
        let mut cells = PatternCells::new(inventory);
        cells
            .insert(&capability_binding)
            .map_err(|_| Error::InvalidCapability)?;
        let mut registrations = Vec::new();
        if validate_body(
            &body.statements,
            range.entry(),
            states.execution(),
            &capability_binding,
            inventory,
            &mut cells,
            &mut registrations,
        )? != range.end()
        {
            return Err(Error::InvalidStates);
        }
        if registrations != states.registrations() {
            return Err(Error::UnconsumedSourceRegistration);
        }
        let mut suspensions = Vec::new();
        collect_resource_suspensions(&body.statements, states.execution(), &mut suspensions);
        validate_tape(states.suspensions(), &suspensions)
            .map_err(|_| Error::UnconsumedSourceSuspension)?;
        let capability = match states.into_finalization() {
            AsyncGeneratorResourceScopeFinalizationSource::Sync { exit } => {
                if range.end().checked_add(1) != Some(exit)
                    || registrations
                        .iter()
                        .any(|source| source.hint() != ResourceDisposalHintIr::Sync)
                {
                    return Err(Error::InvalidStates);
                }
                AsyncGeneratorResourceCapabilityIr::Sync(
                    AsyncGeneratorSyncDisposableCapabilityIr::new(capability_binding.name.clone()),
                )
            }
            AsyncGeneratorResourceScopeFinalizationSource::Async(finalizer) => {
                if execution == ResumableRegionProtocolIr::Generator {
                    return Err(Error::InvalidCapability);
                }
                if !registrations
                    .iter()
                    .any(|source| source.hint() == ResourceDisposalHintIr::Async)
                {
                    return Err(Error::InvalidCapability);
                }
                if finalizer.entry_state() != range.entry()
                    || range.end().checked_add(1) != Some(finalizer.dispose_state())
                    || finalizer.exit_state() != exit_state
                {
                    return Err(Error::InvalidStates);
                }
                AsyncGeneratorResourceCapabilityIr::Async(
                    AsyncGeneratorAsyncDisposableCapabilityIr::new(
                        capability_binding.name.clone(),
                        finalizer,
                    ),
                )
            }
        };
        Ok(Self {
            capability_binding,
            capability,
            body: ResumableRegionIr::from_checked_resource(CheckedResourceBody {
                block: body,
                range,
                execution,
            }),
            exit_state,
            capacity: registrations.len(),
            suspensions,
        })
    }

    pub fn capability_binding(&self) -> &OwnedEnvBindingIr {
        &self.capability_binding
    }
    pub fn capability(&self) -> &AsyncGeneratorResourceCapabilityIr {
        &self.capability
    }
    pub fn body(&self) -> &ResumableRegionIr {
        &self.body
    }
    pub fn execution(&self) -> ResumableRegionProtocolIr {
        self.body.protocol()
    }
    pub fn entry_state(&self) -> u32 {
        self.body.entry_state()
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

fn validate_body(
    statements: &[StatementIr],
    mut state: u32,
    execution: ResumableRegionProtocolIr,
    capability: &OwnedEnvBindingIr,
    inventory: &[OwnedEnvBindingIr],
    cells: &mut PatternCells<'_>,
    registrations: &mut Vec<AsyncGeneratorResourceRegistrationSource>,
) -> Result<u32, AsyncGeneratorResourceError> {
    use AsyncGeneratorResourceError as Error;
    for statement in statements {
        state = match statement {
            StatementIr::AsyncGeneratorResourceRegistration(operation) => {
                if operation.capability_binding() != capability
                    || operation.source().register_state() != state
                {
                    return Err(Error::InvalidRegistration);
                }
                let ExprIr::Identifier(name) = &operation.initializer().expr else {
                    return Err(Error::UnallocatedInitializer);
                };
                let binding = inventory
                    .iter()
                    .find(|binding| &binding.name == name)
                    .ok_or(Error::UnallocatedInitializer)?;
                cells.insert(binding).map_err(|_| Error::AliasedStorage)?;
                registrations.push(operation.source().clone());
                state
            }
            StatementIr::EmptyStatementCompletion(item) => validate_body(
                std::slice::from_ref(item.statement()),
                state,
                execution,
                capability,
                inventory,
                cells,
                registrations,
            )?,
            StatementIr::Block(block) if block.lexical_environment.is_none() => validate_body(
                &block.statements,
                state,
                execution,
                capability,
                inventory,
                cells,
                registrations,
            )?,
            StatementIr::LexicalBlock(statements) => validate_body(
                statements,
                state,
                execution,
                capability,
                inventory,
                cells,
                registrations,
            )?,
            // Nested checked owners consume their own operations and tapes;
            // the general reader refuses any unowned registration inside them.
            _ => match execution {
                ResumableRegionProtocolIr::Generator => {
                    crate::generator_loop_control::sequence_end(
                        std::slice::from_ref(statement),
                        state,
                    )
                }
                ResumableRegionProtocolIr::Async => crate::async_switch::sequence_exit(
                    std::slice::from_ref(statement),
                    state,
                )
                .map_err(|_| {
                    crate::generator_loop_control::GeneratorLoopControlError::ForeignContinuation
                }),
                ResumableRegionProtocolIr::AsyncGenerator => {
                    mixed_sequence_end(std::slice::from_ref(statement), state)
                }
            }
            .map_err(|_| Error::InvalidStates)?,
        };
    }
    Ok(state)
}

pub(crate) fn collect_resource_suspensions(
    statements: &[StatementIr],
    execution: ResumableRegionProtocolIr,
    output: &mut Vec<ResumableSuspensionPointIr>,
) {
    match execution {
        ResumableRegionProtocolIr::Generator => {
            let mut points = Vec::new();
            crate::generator_loop_control::collect_suspensions(statements, &mut points);
            output.extend(points.into_iter().map(|point| ResumableSuspensionPointIr {
                kind: crate::ResumableSuspensionKindIr::Yield,
                suspend_state: point.suspend_state,
                resume_state: point.resume_state,
                resume_environment: crate::ResumableResumeEnvironmentIr::InvocationOuter,
            }));
        }
        ResumableRegionProtocolIr::Async => {
            crate::async_with::collect_async_suspensions(statements, output)
        }
        ResumableRegionProtocolIr::AsyncGenerator => collect_mixed_suspensions(statements, output),
    }
}
