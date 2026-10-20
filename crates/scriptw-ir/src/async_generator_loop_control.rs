use crate::async_generator_source::{
    AsyncGeneratorIfSourceStates, AsyncGeneratorLoopSourceStates, AsyncGeneratorSourceRange,
};
use crate::generator_loop_control::{
    checked_next_state, has_unowned_head_branch, mixed_sequence_end,
    validate_classic_loop_environment, GeneratorLoopControlError,
};
use crate::{
    BlockIr, ForLexicalEnvironmentIr, GeneratorLoopKindIr, OwnedEnvBindingIr,
    ResumableExpressionIr, ResumableRegionIr, ResumableRegionProtocolIr,
    ResumableSuspensionPointIr, TypedExpr,
};

pub(crate) type AsyncGeneratorControlError = GeneratorLoopControlError;

fn require_state(expected: u32, actual: u32) -> Result<(), AsyncGeneratorControlError> {
    if expected == actual {
        Ok(())
    } else {
        Err(AsyncGeneratorControlError::StateMismatch { expected, actual })
    }
}

/// A complete mixed-protocol region minted from the actual source range.
/// Ordinary-generator and plain-async carriers cannot supply this region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorLoopRegionIr {
    block: BlockIr,
    entry_state: u32,
    end_state: u32,
}

impl AsyncGeneratorLoopRegionIr {
    pub(crate) fn from_resource_scope(
        body: crate::async_generator_resource::CheckedResourceBody,
    ) -> Self {
        let (block, range) = body.into_parts();
        Self {
            block,
            entry_state: range.entry(),
            end_state: range.end(),
        }
    }

    pub(crate) fn from_array_pattern(
        body: crate::async_generator_array_destructuring::CheckedArrayBody,
    ) -> Self {
        let (block, range) = body.into_parts();
        Self {
            block,
            entry_state: range.entry(),
            end_state: range.end(),
        }
    }

    pub(crate) fn new(
        block: BlockIr,
        range: AsyncGeneratorSourceRange,
    ) -> Result<Self, AsyncGeneratorControlError> {
        require_state(
            range.end(),
            mixed_sequence_end(&block.statements, range.entry())?,
        )?;
        Ok(Self {
            block,
            entry_state: range.entry(),
            end_state: range.end(),
        })
    }

    pub fn block(&self) -> &BlockIr {
        &self.block
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn end_state(&self) -> u32 {
        self.end_state
    }

    fn matches(&self, range: AsyncGeneratorSourceRange) -> bool {
        self.entry_state == range.entry() && self.end_state == range.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorLoopExpressionIr {
    region: AsyncGeneratorLoopRegionIr,
    value: TypedExpr,
}

impl AsyncGeneratorLoopExpressionIr {
    pub(crate) fn new(region: AsyncGeneratorLoopRegionIr, value: TypedExpr) -> Self {
        Self { region, value }
    }
    pub fn region(&self) -> &AsyncGeneratorLoopRegionIr {
        &self.region
    }
    pub fn value(&self) -> &TypedExpr {
        &self.value
    }
}

/// A whole classic loop, including every eager phase and its exact protocol
/// tape. Distinct checked source constructors admit Async and AsyncGenerator.
#[must_use = "the checked complete loop must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorLoopIr {
    execution: ResumableRegionProtocolIr,
    kind: GeneratorLoopKindIr,
    initialization: Option<ResumableRegionIr>,
    test: ResumableExpressionIr,
    body: ResumableRegionIr,
    update: Option<ResumableExpressionIr>,
    lexical_environment: Option<ForLexicalEnvironmentIr>,
    value_binding: OwnedEnvBindingIr,
    resource: Option<crate::AsyncGeneratorScopedResourceIr>,
    exit_state: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorLoopIr {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source: AsyncGeneratorLoopSourceStates,
        initialization: Option<AsyncGeneratorLoopRegionIr>,
        test: AsyncGeneratorLoopExpressionIr,
        body: AsyncGeneratorLoopRegionIr,
        update: Option<AsyncGeneratorLoopExpressionIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<crate::AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorControlError> {
        if source.execution() != ResumableRegionProtocolIr::AsyncGenerator {
            return Err(AsyncGeneratorControlError::ForeignContinuation);
        }
        Self::checked(
            source,
            initialization.map(Into::into),
            test.into(),
            body.into(),
            update.map(Into::into),
            lexical_environment,
            value_binding,
            resource,
            inventory,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_plain_async(
        source: AsyncGeneratorLoopSourceStates,
        initialization: Option<ResumableRegionIr>,
        test: ResumableExpressionIr,
        body: ResumableRegionIr,
        update: Option<ResumableExpressionIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<crate::AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorControlError> {
        if source.execution() != ResumableRegionProtocolIr::Async {
            return Err(AsyncGeneratorControlError::ForeignContinuation);
        }
        Self::checked(
            source,
            initialization,
            test,
            body,
            update,
            lexical_environment,
            value_binding,
            resource,
            inventory,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_generator_resource(
        source: AsyncGeneratorLoopSourceStates,
        initialization: Option<ResumableRegionIr>,
        test: ResumableExpressionIr,
        body: ResumableRegionIr,
        update: Option<ResumableExpressionIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
        value_binding: OwnedEnvBindingIr,
        resource: crate::AsyncGeneratorScopedResourceIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorControlError> {
        if source.execution() != ResumableRegionProtocolIr::Generator || source.resource().is_none()
        {
            return Err(AsyncGeneratorControlError::ForeignContinuation);
        }
        Self::checked(
            source,
            initialization,
            test,
            body,
            update,
            lexical_environment,
            value_binding,
            Some(resource),
            inventory,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn checked(
        source: AsyncGeneratorLoopSourceStates,
        initialization: Option<ResumableRegionIr>,
        test: ResumableExpressionIr,
        body: ResumableRegionIr,
        update: Option<ResumableExpressionIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
        value_binding: OwnedEnvBindingIr,
        resource: Option<crate::AsyncGeneratorScopedResourceIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncGeneratorControlError> {
        let plan = Self {
            execution: source.execution(),
            kind: source.kind(),
            initialization,
            test,
            body,
            update,
            lexical_environment,
            value_binding,
            resource,
            exit_state: source.exit(),
            suspensions: source.suspensions().to_vec(),
        };
        if !matches_complete_optional_region(plan.initialization.as_ref(), source.initialization())
            || !matches_complete_region(plan.test.region(), source.test())
            || !matches_complete_region(&plan.body, source.body())
            || !matches_complete_optional_region(
                plan.update.as_ref().map(|value| value.region()),
                source.update(),
            )
        {
            return Err(AsyncGeneratorControlError::InvalidPhases);
        }
        match plan.kind {
            GeneratorLoopKindIr::For if plan.initialization.is_some() && plan.update.is_some() => {}
            GeneratorLoopKindIr::While | GeneratorLoopKindIr::DoWhile
                if plan.initialization.is_none()
                    && plan.update.is_none()
                    && plan.lexical_environment.is_none() => {}
            _ => return Err(AsyncGeneratorControlError::InvalidPhases),
        }
        let retained: Vec<_> = inventory
            .iter()
            .filter(|binding| {
                binding.name == plan.value_binding.name || binding.slot == plan.value_binding.slot
            })
            .collect();
        if retained.len() != 1 || retained[0] != &plan.value_binding {
            return Err(AsyncGeneratorControlError::InvalidPhases);
        }
        let mut previous_end = None;
        let mut actual = Vec::new();
        for region in plan.regions() {
            if region.protocol() != plan.execution {
                return Err(AsyncGeneratorControlError::ForeignContinuation);
            }
            if let Some(end) = previous_end {
                require_state(checked_next_state(end)?, region.entry_state())?;
            }
            previous_end = Some(region.end_state());
            region.collect_suspensions(&mut actual);
        }
        let source_end = previous_end.ok_or(AsyncGeneratorControlError::InvalidPhases)?;
        match (source.resource(), plan.resource.as_ref()) {
            (None, None) => require_state(checked_next_state(source_end)?, plan.exit_state)?,
            (Some(states), Some(resource))
                if plan.kind == GeneratorLoopKindIr::For
                    && resource.execution() == plan.execution
                    && resource.matches_resumable_regions(
                        states,
                        &plan.regions().collect::<Vec<_>>(),
                        inventory,
                    )
                    && resource.entry_state() == plan.entry_state()
                    && resource.body_end_state() == source_end
                    && resource.exit_state() == plan.exit_state
                    && resource.capability_binding().name != plan.value_binding.name
                    && resource.capability_binding().slot != plan.value_binding.slot => {}
            _ => return Err(AsyncGeneratorControlError::InvalidPhases),
        }
        validate_tape(source.suspensions(), &actual)?;
        for head in plan
            .initialization
            .iter()
            .chain(std::iter::once(plan.test.region()))
            .chain(plan.update.iter().map(ResumableExpressionIr::region))
        {
            if head.block().statements.iter().any(has_unowned_head_branch) {
                return Err(AsyncGeneratorControlError::ForeignContinuation);
            }
        }
        if let Some(environment) = &plan.lexical_environment {
            if plan.resource().is_some() && !environment.per_iteration_slots.is_empty() {
                return Err(AsyncGeneratorControlError::InvalidPhases);
            }
            if plan.resource().is_some_and(|resource| {
                environment
                    .bindings
                    .iter()
                    .any(|binding| binding.name == resource.capability_binding().name)
            }) {
                return Err(AsyncGeneratorControlError::InvalidPhases);
            }
            validate_classic_loop_environment(
                &plan
                    .initialization
                    .as_ref()
                    .ok_or(AsyncGeneratorControlError::InvalidPhases)?
                    .block()
                    .statements,
                environment,
                &plan.value_binding,
            )?;
        }
        Ok(plan)
    }

    pub const fn kind(&self) -> GeneratorLoopKindIr {
        self.kind
    }
    pub const fn execution(&self) -> ResumableRegionProtocolIr {
        self.execution
    }
    pub fn initialization(&self) -> Option<&ResumableRegionIr> {
        self.initialization.as_ref()
    }
    pub fn test(&self) -> &ResumableExpressionIr {
        &self.test
    }
    pub fn body(&self) -> &ResumableRegionIr {
        &self.body
    }
    pub fn update(&self) -> Option<&ResumableExpressionIr> {
        self.update.as_ref()
    }
    pub fn lexical_environment(&self) -> Option<&ForLexicalEnvironmentIr> {
        self.lexical_environment.as_ref()
    }
    pub fn value_binding(&self) -> &OwnedEnvBindingIr {
        &self.value_binding
    }
    pub fn resource(&self) -> Option<&crate::AsyncGeneratorScopedResourceIr> {
        self.resource.as_ref()
    }
    pub fn value_binding_name(&self) -> &str {
        &self.value_binding.name
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn entry_state(&self) -> u32 {
        match self.kind {
            GeneratorLoopKindIr::For => self
                .initialization
                .as_ref()
                .expect("checked For initialization")
                .entry_state(),
            GeneratorLoopKindIr::While => self.test.region().entry_state(),
            GeneratorLoopKindIr::DoWhile => self.body.entry_state(),
        }
    }
    pub fn continue_state(&self) -> u32 {
        self.update
            .as_ref()
            .map_or(self.test.region().entry_state(), |value| {
                value.region().entry_state()
            })
    }
    pub fn regions(&self) -> impl Iterator<Item = &ResumableRegionIr> {
        let (first, second) = match self.kind {
            GeneratorLoopKindIr::DoWhile => (&self.body, self.test.region()),
            GeneratorLoopKindIr::For | GeneratorLoopKindIr::While => {
                (self.test.region(), &self.body)
            }
        };
        self.initialization
            .iter()
            .chain([first, second])
            .chain(self.update.iter().map(ResumableExpressionIr::region))
    }
    pub fn expressions(&self) -> impl Iterator<Item = &TypedExpr> {
        std::iter::once(self.test.value())
            .chain(self.update.iter().map(ResumableExpressionIr::value))
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

fn matches_complete_region(region: &ResumableRegionIr, range: AsyncGeneratorSourceRange) -> bool {
    region.entry_state() == range.entry() && region.end_state() == range.end()
}
fn matches_complete_optional_region(
    region: Option<&ResumableRegionIr>,
    range: Option<AsyncGeneratorSourceRange>,
) -> bool {
    match (region, range) {
        (None, None) => true,
        (Some(region), Some(range)) => matches_complete_region(region, range),
        (None, Some(_)) | (Some(_), None) => false,
    }
}

/// The condition is completed exactly once before selecting one retained branch.
#[must_use = "the checked mixed conditional must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorIfIr {
    condition: AsyncGeneratorLoopExpressionIr,
    then_branch: AsyncGeneratorLoopRegionIr,
    else_branch: AsyncGeneratorLoopRegionIr,
    exit_state: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorIfIr {
    pub(crate) fn new(
        source: AsyncGeneratorIfSourceStates,
        condition: AsyncGeneratorLoopExpressionIr,
        then_branch: AsyncGeneratorLoopRegionIr,
        else_branch: AsyncGeneratorLoopRegionIr,
    ) -> Result<Self, AsyncGeneratorControlError> {
        if !condition.region.matches(source.condition())
            || !then_branch.matches(source.then_branch())
            || !else_branch.matches(source.else_branch())
        {
            return Err(AsyncGeneratorControlError::InvalidPhases);
        }
        require_state(
            checked_next_state(condition.region.end_state())?,
            then_branch.entry_state(),
        )?;
        require_state(
            checked_next_state(then_branch.end_state())?,
            else_branch.entry_state(),
        )?;
        require_state(checked_next_state(else_branch.end_state())?, source.exit())?;
        let mut actual = Vec::new();
        for region in [condition.region(), &then_branch, &else_branch] {
            super::generator_loop_control::collect_mixed_suspensions(
                &region.block.statements,
                &mut actual,
            );
        }
        validate_tape(source.suspensions(), &actual)?;
        if condition
            .region
            .block
            .statements
            .iter()
            .any(has_unowned_head_branch)
        {
            return Err(AsyncGeneratorControlError::ForeignContinuation);
        }
        Ok(Self {
            condition,
            then_branch,
            else_branch,
            exit_state: source.exit(),
            suspensions: actual,
        })
    }
    pub fn condition(&self) -> &AsyncGeneratorLoopExpressionIr {
        &self.condition
    }
    pub fn then_branch(&self) -> &AsyncGeneratorLoopRegionIr {
        &self.then_branch
    }
    pub fn else_branch(&self) -> &AsyncGeneratorLoopRegionIr {
        &self.else_branch
    }
    pub fn entry_state(&self) -> u32 {
        self.condition.region.entry_state()
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn regions(&self) -> impl Iterator<Item = &AsyncGeneratorLoopRegionIr> {
        [
            self.condition.region(),
            &self.then_branch,
            &self.else_branch,
        ]
        .into_iter()
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

pub(crate) fn validate_tape(
    source: &[ResumableSuspensionPointIr],
    actual: &[ResumableSuspensionPointIr],
) -> Result<(), AsyncGeneratorControlError> {
    if source != actual {
        return Err(AsyncGeneratorControlError::UnconsumedSourceSuspension);
    }
    let resumed: std::collections::BTreeSet<_> =
        actual.iter().map(|point| point.resume_state).collect();
    if resumed.len() != actual.len() {
        return Err(AsyncGeneratorControlError::InvalidSuspension);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
