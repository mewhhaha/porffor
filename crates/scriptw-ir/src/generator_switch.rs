//! Checked ordinary-generator Switch selection and CaseBlock ownership.
use crate::generator_loop_control::{
    checked_next_state, collect_suspensions, has_unowned_head_branch, sequence_end,
    GeneratorLoopControlError, GeneratorLoopSourceRange,
};
use crate::lowering_helpers::{CheckedEmptyStatementCompletionSource, GeneratorSwitchSourceStates};
use crate::switch_storage::CheckedSwitchStorageIr;
use crate::{
    GeneratorLoopExpressionIr, GeneratorLoopRegionIr, LexicalEnvironmentIr, OwnedEnvBindingIr,
    StatementIr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorSwitchControlError {
    Continuation(GeneratorLoopControlError),
    CaseCount,
    CaseOrder,
    DuplicateDefault,
    UnallocatedBinding,
    AliasedBindings,
    MissingDiscriminantPublication,
    PerCaseEnvironment,
    InvalidCaseBlockEnvironment,
    UnownedHeadBranch,
}

impl From<GeneratorLoopControlError> for GeneratorSwitchControlError {
    fn from(error: GeneratorLoopControlError) -> Self {
        Self::Continuation(error)
    }
}

/// An actual source item whose normal completion has no value. The original
/// lowered item remains intact, including every nested suspension and abrupt
/// completion; only the source-owned normal Empty result is recorded here.
#[must_use = "the source-owned Empty item must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyStatementCompletionIr {
    statement: StatementIr,
}

impl EmptyStatementCompletionIr {
    pub(crate) fn new(
        _source: CheckedEmptyStatementCompletionSource<'_>,
        statement: StatementIr,
    ) -> Self {
        Self { statement }
    }

    pub fn statement(&self) -> &StatementIr {
        &self.statement
    }
}

/// A complete body in the single Switch CaseBlock environment. A default owns
/// no selector region; it is considered only after all actual selectors fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorSwitchCaseIr {
    selector: Option<GeneratorLoopExpressionIr>,
    body: GeneratorLoopRegionIr,
}

impl OrdinaryGeneratorSwitchCaseIr {
    pub(crate) fn new(
        selector: Option<GeneratorLoopExpressionIr>,
        body: GeneratorLoopRegionIr,
    ) -> Result<Self, GeneratorSwitchControlError> {
        if body.block().lexical_environment.is_some() {
            return Err(GeneratorSwitchControlError::PerCaseEnvironment);
        }
        Ok(Self { selector, body })
    }

    pub fn selector(&self) -> Option<&GeneratorLoopExpressionIr> {
        self.selector.as_ref()
    }

    pub fn body(&self) -> &GeneratorLoopRegionIr {
        &self.body
    }
}

/// The source plan and actual allocated cells are consumed before this owner
/// can reach the backend. Selection commits an actual body entry, and normal
/// fallthrough follows bodies in source order without evaluating selectors.
#[must_use = "the checked Switch must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorSwitchIr {
    discriminant: GeneratorLoopExpressionIr,
    storage: CheckedSwitchStorageIr,
    lexical_declarations: Vec<StatementIr>,
    cases: Vec<OrdinaryGeneratorSwitchCaseIr>,
    case_block_entry_state: u32,
    fallback_state: u32,
    exit_state: u32,
}

impl OrdinaryGeneratorSwitchIr {
    pub(crate) fn new(
        source: GeneratorSwitchSourceStates,
        discriminant: GeneratorLoopExpressionIr,
        discriminant_binding: OwnedEnvBindingIr,
        lexical_environment: Option<LexicalEnvironmentIr>,
        lexical_declarations: Vec<StatementIr>,
        cases: Vec<OrdinaryGeneratorSwitchCaseIr>,
        value_binding: OwnedEnvBindingIr,
        owned_bindings: &[OwnedEnvBindingIr],
    ) -> Result<Self, GeneratorSwitchControlError> {
        if source.selectors().len() != source.bodies().len() || cases.len() != source.bodies().len()
        {
            return Err(GeneratorSwitchControlError::CaseCount);
        }
        if cases.iter().filter(|case| case.selector.is_none()).count() > 1 {
            return Err(GeneratorSwitchControlError::DuplicateDefault);
        }
        let storage = CheckedSwitchStorageIr::new(
            discriminant.region().block(),
            discriminant.value(),
            discriminant_binding,
            value_binding,
            lexical_environment,
            owned_bindings,
        )?;
        let discriminant_statements = &discriminant.region().block().statements;
        if cases
            .iter()
            .filter_map(|case| case.selector.as_ref())
            .any(|selector| selector.region().block().lexical_environment.is_some())
        {
            return Err(GeneratorSwitchControlError::InvalidCaseBlockEnvironment);
        }
        if discriminant_statements.iter().any(has_unowned_head_branch)
            || lexical_declarations.iter().any(has_unowned_head_branch)
            || cases
                .iter()
                .filter_map(|case| case.selector.as_ref())
                .any(|selector| {
                    selector
                        .region()
                        .block()
                        .statements
                        .iter()
                        .any(has_unowned_head_branch)
                })
        {
            return Err(GeneratorSwitchControlError::UnownedHeadBranch);
        }
        if range_of(discriminant.region()) != source.discriminant()
            || checked_next_state(discriminant.region().end_state())? != source.case_block_entry()
            || sequence_end(&lexical_declarations, source.case_block_entry())?
                != source.case_block_entry()
        {
            return Err(GeneratorSwitchControlError::CaseOrder);
        }
        let mut cursor = source.case_block_entry();
        for (case, expected) in cases.iter().zip(source.selectors()) {
            match (case.selector.as_ref(), expected) {
                (Some(selector), Some(expected))
                    if range_of(selector.region()) == *expected
                        && selector.region().entry_state() == cursor =>
                {
                    cursor = checked_next_state(selector.region().end_state())?;
                }
                (None, None) => {}
                _ => return Err(GeneratorSwitchControlError::CaseOrder),
            }
        }
        if cursor != source.fallback_state() {
            return Err(GeneratorSwitchControlError::CaseOrder);
        }
        cursor = checked_next_state(cursor)?;
        for (case, expected) in cases.iter().zip(source.bodies()) {
            if case.body.block().lexical_environment.is_some() {
                return Err(GeneratorSwitchControlError::PerCaseEnvironment);
            }
            if range_of(&case.body) != *expected || case.body.entry_state() != cursor {
                return Err(GeneratorSwitchControlError::CaseOrder);
            }
            cursor = checked_next_state(case.body.end_state())?;
        }
        if cursor != source.exit() {
            return Err(GeneratorSwitchControlError::CaseOrder);
        }
        let plan = Self {
            discriminant,
            storage,
            lexical_declarations,
            cases,
            case_block_entry_state: source.case_block_entry(),
            fallback_state: source.fallback_state(),
            exit_state: source.exit(),
        };
        let mut actual_suspensions = Vec::new();
        for region in plan.regions() {
            collect_suspensions(&region.block().statements, &mut actual_suspensions);
        }
        if actual_suspensions != source.suspensions() {
            return Err(GeneratorLoopControlError::UnconsumedSourceSuspension.into());
        }
        let resumed: std::collections::BTreeSet<_> = actual_suspensions
            .iter()
            .map(|point| point.resume_state)
            .collect();
        if resumed.len() != actual_suspensions.len() {
            return Err(GeneratorLoopControlError::InvalidSuspension.into());
        }
        Ok(plan)
    }

    pub fn discriminant(&self) -> &GeneratorLoopExpressionIr {
        &self.discriminant
    }
    pub fn discriminant_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.discriminant_binding()
    }
    pub fn lexical_environment(&self) -> Option<&LexicalEnvironmentIr> {
        self.storage.lexical_environment()
    }
    pub fn lexical_declarations(&self) -> &[StatementIr] {
        &self.lexical_declarations
    }
    pub fn cases(&self) -> &[OrdinaryGeneratorSwitchCaseIr] {
        &self.cases
    }
    pub fn entry_state(&self) -> u32 {
        self.discriminant.region().entry_state()
    }
    pub const fn case_block_entry_state(&self) -> u32 {
        self.case_block_entry_state
    }
    pub const fn fallback_state(&self) -> u32 {
        self.fallback_state
    }
    pub fn value_binding(&self) -> &OwnedEnvBindingIr {
        self.storage.value_binding()
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn regions(&self) -> impl Iterator<Item = &GeneratorLoopRegionIr> {
        std::iter::once(self.discriminant.region())
            .chain(
                self.cases
                    .iter()
                    .filter_map(|case| case.selector.as_ref().map(|selector| selector.region())),
            )
            .chain(self.cases.iter().map(|case| &case.body))
    }
}

fn range_of(region: &GeneratorLoopRegionIr) -> GeneratorLoopSourceRange {
    GeneratorLoopSourceRange {
        entry: region.entry_state(),
        end: region.end_state(),
    }
}

#[cfg(test)]
mod tests;
