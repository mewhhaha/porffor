//! One closed operand protocol feeds the existing semantic construction owners.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum ResumableOperandProtocol {
    Generator,
    Async,
    Mixed,
}

impl ResumableOperandProtocol {
    pub(super) fn for_execution(execution: ResumableRegionProtocolIr) -> Self {
        match execution {
            ResumableRegionProtocolIr::Generator => Self::Generator,
            ResumableRegionProtocolIr::Async => Self::Async,
            ResumableRegionProtocolIr::AsyncGenerator => Self::Mixed,
        }
    }
    pub(super) fn entry_state(self, lowerer: &ScriptLowerer<'_>) -> Option<u32> {
        match self {
            Self::Generator => lowerer.plain_generator_entry_state(),
            Self::Async => lowerer.plain_async_entry_state(),
            Self::Mixed => lowerer.async_generator_entry_state(),
        }
    }
    pub(super) fn set_phase(self, lowerer: &mut ScriptLowerer<'_>, state: u32) {
        match self {
            Self::Generator => lowerer.current_generator_resume_state = Some(state),
            Self::Async => lowerer.current_async_resume_state = Some(state),
            Self::Mixed => lowerer.set_async_generator_phase(state),
        }
    }
    pub(super) fn current(lowerer: &ScriptLowerer<'_>) -> Option<Self> {
        if lowerer.async_generator_entry_state().is_some() {
            Some(Self::Mixed)
        } else if lowerer.current_generator_resume_state.is_some() {
            Some(Self::Generator)
        } else if lowerer.current_async_resume_state.is_some() {
            Some(Self::Async)
        } else {
            None
        }
    }
    pub(super) fn for_source(lowerer: &ScriptLowerer<'_>, source: &Expression) -> Option<Self> {
        if lowerer.async_generator_entry_state().is_some() {
            Some(Self::Mixed)
        } else if contains(source, ContainsSymbol::YieldExpression)
            && lowerer.current_generator_resume_state.is_some()
        {
            Some(Self::Generator)
        } else if contains(source, ContainsSymbol::AwaitExpression)
            && lowerer.current_async_resume_state.is_some()
        {
            Some(Self::Async)
        } else {
            None
        }
    }
    pub(super) fn lower(
        self,
        lowerer: &mut ScriptLowerer<'_>,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if !contains(source, ContainsSymbol::AwaitExpression)
            && !contains(source, ContainsSymbol::YieldExpression)
        {
            return Some((Vec::new(), lowerer.lower_expression(source)));
        }
        match self {
            Self::Generator
                if lowerer.current_generator_resume_state.is_some()
                    && !contains(source, ContainsSymbol::AwaitExpression) =>
            {
                lowerer.lower_staged_generator_expression_legacy(source)
            }
            Self::Async
                if lowerer.current_async_resume_state.is_some()
                    && !contains(source, ContainsSymbol::YieldExpression) =>
            {
                lowerer.lower_async_prefixed_expression(source)
            }
            Self::Mixed if lowerer.async_generator_entry_state().is_some() => {
                lowerer.lower_mixed_generator_value(source)
            }
            Self::Generator | Self::Async | Self::Mixed => None,
        }
    }
}
