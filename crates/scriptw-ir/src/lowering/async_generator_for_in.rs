//! The analyzed mixed owner consumes the same physical original ForIn head.
use super::*;
use crate::analysis::CompleteResumableForInOwner;
use crate::async_generator_source::{AsyncGeneratorForInSource, AsyncGeneratorForInSourceStates};
use crate::lowering_helpers::ForInSourceIdentity;

/// Only lowering this actual initializer against its selected-key cell mints
/// the per-key continuation. Callers cannot replace it with a bare key read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedAsyncGeneratorForInInitializer {
    source: ForInSourceIdentity,
    key_binding: OwnedEnvBindingIr,
    region: ResumableRegionIr,
    environment: Option<ForInOfEnvironmentIr>,
    mode: BindingMode,
}
impl CheckedAsyncGeneratorForInInitializer {
    pub(crate) fn region(&self) -> &ResumableRegionIr {
        &self.region
    }
    pub(crate) fn key_binding(&self) -> &OwnedEnvBindingIr {
        &self.key_binding
    }
    pub(crate) fn environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.environment.as_ref()
    }
    pub(crate) fn mode(&self) -> BindingMode {
        self.mode
    }
    pub(crate) fn matches_source(&self, states: &AsyncGeneratorForInSourceStates) -> bool {
        self.source == states.identity()
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_checked_async_generator_for_in_initializer(
        &mut self,
        source: &boa_ast::statement::iteration::ForInLoop,
        states: &AsyncGeneratorForInSourceStates,
        head: &super::for_in::ForInIterationHead,
        key_binding: &OwnedEnvBindingIr,
        environment: Option<&ForInOfEnvironmentIr>,
    ) -> Option<(
        CheckedAsyncGeneratorForInInitializer,
        Option<crate::reference::IgnoredIterationIdentifierWriteIr>,
    )> {
        if states.identity() != ForInSourceIdentity::from_source(source)
            || !self.generated_owned_env_bindings.contains(key_binding)
        {
            return None;
        }
        let execution = states.execution();
        let protocol = super::resumable_operand::ResumableOperandProtocol::for_execution(execution);
        protocol.set_phase(self, states.initialization().entry());
        let continuation = match execution {
            ResumableRegionProtocolIr::Generator => {
                super::pattern_target::PatternContinuation::Generator
            }
            ResumableRegionProtocolIr::Async => super::pattern_target::PatternContinuation::Async,
            ResumableRegionProtocolIr::AsyncGenerator => {
                super::pattern_target::PatternContinuation::AsyncGenerator
            }
        };
        let (statements, ignored) = self.lower_for_in_iteration_initialization_with_continuation(
            head,
            Some(TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::Identifier(key_binding.name.clone()),
            )),
            Some(continuation),
        )?;
        (protocol.entry_state(self)? == states.initialization().end()).then_some(())?;
        let region = ResumableRegionIr::new(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            states.initialization(),
            execution,
        )
        .ok()?;
        Some((
            CheckedAsyncGeneratorForInInitializer {
                source: states.identity(),
                key_binding: key_binding.clone(),
                region,
                environment: environment.cloned(),
                mode: states.head_mode(),
            },
            ignored,
        ))
    }
    pub(super) fn lower_async_generator_for_in(
        &mut self,
        source: &boa_ast::statement::iteration::ForInLoop,
        owner: CompleteResumableForInOwner,
    ) -> (StatementIr, ValueKind) {
        let Some(source) = owner.checked_source(source) else {
            self.unsupported("ForIn source identity differs from its analyzed execution owner");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        self.lower_complete_resumable_for_in(source)
    }

    pub(super) fn lower_complete_resumable_for_in(
        &mut self,
        checked: AsyncGeneratorForInSource<'_>,
    ) -> (StatementIr, ValueKind) {
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        let execution = checked.execution();
        let protocol = super::resumable_operand::ResumableOperandProtocol::for_execution(execution);
        let previous_context = self.async_value_branch_context;
        let previous_generator_depth = self.ordinary_generator_region_depth;
        if execution == ResumableRegionProtocolIr::Generator {
            self.ordinary_generator_region_depth += 1;
        }
        if execution == ResumableRegionProtocolIr::Async {
            self.async_value_branch_context = AsyncValueBranchContext::ForInHead {
                loop_depth: self.loop_depth,
            };
        }
        let lowered = (|| {
            let states = checked.states(protocol.entry_state(self)?)?;
            let proof = checked.checked_head(self.interner)?;
            self.lower_resumable_for_in(checked, states, proof)
        })();
        self.async_value_branch_context = previous_context;
        self.ordinary_generator_region_depth = previous_generator_depth;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        match lowered {
            Some(plan) => plan,
            None => {
                self.unsupported("complete ForIn differs from its checked source, original per-key initializer, or environment plan");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }
}
