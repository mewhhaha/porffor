//! A loop or CaseBlock owns one capability across its actual source regions.
use super::*;
use crate::async_generator_source::AsyncGeneratorScopedResourceSourceStates;

#[must_use = "the resource lifetime must be consumed by its whole loop or CaseBlock"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorScopedResourceIr {
    execution: ResumableRegionProtocolIr,
    capability_binding: OwnedEnvBindingIr,
    capability: AsyncGeneratorResourceCapabilityIr,
    registrations: Vec<AsyncGeneratorResourceRegistrationSource>,
    entry_state: u32,
    body_end_state: u32,
    exit_state: u32,
}

impl AsyncGeneratorScopedResourceIr {
    /// Direct registrations are legal only in the real source region containing
    /// them. The ordinary region constructor continues to reject bare operations.
    pub(crate) fn checked_region(
        states: &AsyncGeneratorScopedResourceSourceStates,
        range: AsyncGeneratorSourceRange,
        capability_binding: &OwnedEnvBindingIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<AsyncGeneratorLoopRegionIr, AsyncGeneratorResourceError> {
        use AsyncGeneratorResourceError as Error;
        if states.execution() != ResumableRegionProtocolIr::AsyncGenerator
            || range.entry() < states.entry()
            || range.end() > states.body_end()
            || range.end() < range.entry()
            || body.lexical_environment.is_some()
        {
            return Err(Error::InvalidStates);
        }
        let mut cells = PatternCells::new(inventory);
        cells
            .insert(capability_binding)
            .map_err(|_| Error::InvalidCapability)?;
        let mut registrations = Vec::new();
        if validate_body(
            &body.statements,
            range.entry(),
            states.execution(),
            capability_binding,
            inventory,
            &mut cells,
            &mut registrations,
        )? != range.end()
        {
            return Err(Error::InvalidStates);
        }
        let expected: Vec<_> = states
            .registrations()
            .iter()
            .filter(|source| {
                range.entry() <= source.register_state() && source.register_state() <= range.end()
            })
            .collect();
        if registrations.iter().collect::<Vec<_>>() != expected {
            return Err(Error::UnconsumedSourceRegistration);
        }
        Ok(AsyncGeneratorLoopRegionIr::from_resource_scope(
            CheckedResourceBody {
                block: body,
                range,
                execution: states.execution(),
            },
        ))
    }

    pub(crate) fn checked_resumable_region(
        states: &AsyncGeneratorScopedResourceSourceStates,
        range: AsyncGeneratorSourceRange,
        capability_binding: &OwnedEnvBindingIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<ResumableRegionIr, AsyncGeneratorResourceError> {
        use AsyncGeneratorResourceError as Error;
        if range.entry() < states.entry()
            || range.end() > states.body_end()
            || range.end() < range.entry()
            || body.lexical_environment.is_some()
        {
            return Err(Error::InvalidStates);
        }
        let mut cells = PatternCells::new(inventory);
        cells
            .insert(capability_binding)
            .map_err(|_| Error::InvalidCapability)?;
        let mut registrations = Vec::new();
        if validate_body(
            &body.statements,
            range.entry(),
            states.execution(),
            capability_binding,
            inventory,
            &mut cells,
            &mut registrations,
        )? != range.end()
        {
            return Err(Error::InvalidStates);
        }
        let expected: Vec<_> = states
            .registrations()
            .iter()
            .filter(|source| {
                range.entry() <= source.register_state() && source.register_state() <= range.end()
            })
            .collect();
        if registrations.iter().collect::<Vec<_>>() != expected {
            return Err(Error::UnconsumedSourceRegistration);
        }
        Ok(ResumableRegionIr::from_checked_resource(
            CheckedResourceBody {
                block: body,
                range,
                execution: states.execution(),
            },
        ))
    }

    pub(crate) fn new(
        states: &AsyncGeneratorScopedResourceSourceStates,
        capability_binding: OwnedEnvBindingIr,
        regions: &[&AsyncGeneratorLoopRegionIr],
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorResourceError> {
        if states.execution() != ResumableRegionProtocolIr::AsyncGenerator {
            return Err(AsyncGeneratorResourceError::InvalidStates);
        }
        let regions: Vec<_> = regions
            .iter()
            .map(|region| ResumableRegionIr::from((*region).clone()))
            .collect();
        Self::new_resumable(
            states,
            capability_binding,
            &regions.iter().collect::<Vec<_>>(),
            inventory,
        )
    }

    pub(crate) fn new_resumable(
        states: &AsyncGeneratorScopedResourceSourceStates,
        capability_binding: OwnedEnvBindingIr,
        regions: &[&ResumableRegionIr],
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorResourceError> {
        use AsyncGeneratorResourceError as Error;
        if states.registrations().is_empty()
            || states.body_end() < states.entry()
            || states.exit().checked_add(1).is_none()
        {
            return Err(Error::InvalidStates);
        }
        let mut cells = PatternCells::new(inventory);
        cells
            .insert(&capability_binding)
            .map_err(|_| Error::InvalidCapability)?;
        let mut registrations = Vec::new();
        let mut previous_end = None;
        for region in regions {
            if region.protocol() != states.execution()
                || region.entry_state() < states.entry()
                || region.end_state() > states.body_end()
                || previous_end.is_some_and(|end| end >= region.entry_state())
                || validate_body(
                    &region.block().statements,
                    region.entry_state(),
                    states.execution(),
                    &capability_binding,
                    inventory,
                    &mut cells,
                    &mut registrations,
                )? != region.end_state()
            {
                return Err(Error::InvalidStates);
            }
            previous_end = Some(region.end_state());
        }
        if registrations != states.registrations() {
            return Err(Error::UnconsumedSourceRegistration);
        }
        let capability = match (states.hint(), states.finalizer()) {
            (ResourceDisposalHintIr::Sync, None)
                if states.body_end().checked_add(1) == Some(states.exit())
                    && registrations
                        .iter()
                        .all(|source| source.hint() == ResourceDisposalHintIr::Sync) =>
            {
                AsyncGeneratorResourceCapabilityIr::Sync(
                    AsyncGeneratorSyncDisposableCapabilityIr::new(capability_binding.name.clone()),
                )
            }
            (ResourceDisposalHintIr::Async, Some(finalizer))
                if states.execution() != ResumableRegionProtocolIr::Generator
                    && AsyncDisposableFinalizerPlanIr::after_source_suffix(
                        states.entry(),
                        states.body_end(),
                    )
                    .as_ref()
                        == Some(finalizer)
                    && finalizer.exit_state() == states.exit()
                    && registrations
                        .iter()
                        .any(|source| source.hint() == ResourceDisposalHintIr::Async) =>
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
        Ok(Self {
            execution: states.execution(),
            capability_binding,
            capability,
            registrations,
            entry_state: states.entry(),
            body_end_state: states.body_end(),
            exit_state: states.exit(),
        })
    }

    pub(crate) fn matches_resumable_regions(
        &self,
        states: &AsyncGeneratorScopedResourceSourceStates,
        regions: &[&ResumableRegionIr],
        inventory: &[OwnedEnvBindingIr],
    ) -> bool {
        Self::new_resumable(states, self.capability_binding.clone(), regions, inventory).as_ref()
            == Ok(self)
    }
    pub const fn execution(&self) -> ResumableRegionProtocolIr {
        self.execution
    }

    pub fn capability_binding(&self) -> &OwnedEnvBindingIr {
        &self.capability_binding
    }
    pub fn capability(&self) -> &AsyncGeneratorResourceCapabilityIr {
        &self.capability
    }
    pub fn capacity(&self) -> usize {
        self.registrations.len()
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn body_end_state(&self) -> u32 {
        self.body_end_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}
