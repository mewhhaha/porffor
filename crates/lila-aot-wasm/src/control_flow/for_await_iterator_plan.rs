use super::*;

/// Existing async-generator/eager-body plans and the checked plain
/// async body remain paired with their actual head and body at dispatch.
#[derive(Clone, Copy)]
pub(crate) struct ForAwaitIteratorPlan<'a> {
    execution: ForAwaitIteratorExecution<'a>,
}

#[derive(Clone, Copy)]
enum ForAwaitIteratorExecution<'a> {
    Existing {
        binding: &'a ForOfAssignmentIr,
        body: &'a StatementIr,
        head_environment: Option<&'a ForInOfEnvironmentIr>,
        states: &'a AsyncForOfIteratorPlanIr,
    },
    Awaited(lila_ir::AsyncFunctionForAwaitOfIteratorIr<'a>),
}

impl<'a> ForAwaitIteratorPlan<'a> {
    pub(super) fn existing(
        binding: &'a ForOfAssignmentIr,
        body: &'a StatementIr,
        head_environment: Option<&'a ForInOfEnvironmentIr>,
        states: &'a AsyncForOfIteratorPlanIr,
    ) -> Self {
        Self {
            execution: ForAwaitIteratorExecution::Existing {
                binding,
                body,
                head_environment,
                states,
            },
        }
    }

    pub(super) fn awaited(plan: lila_ir::AsyncFunctionForAwaitOfIteratorIr<'a>) -> Self {
        Self {
            execution: ForAwaitIteratorExecution::Awaited(plan),
        }
    }

    pub(super) fn requires_plain_async_activation(self) -> bool {
        match self.execution {
            ForAwaitIteratorExecution::Existing { .. } => false,
            ForAwaitIteratorExecution::Awaited(_) => true,
        }
    }

    pub(super) fn entry_state(self) -> u32 {
        match self.execution {
            ForAwaitIteratorExecution::Existing { states, .. } => states.entry_state,
            ForAwaitIteratorExecution::Awaited(plan) => plan.plan().entry_state(),
        }
    }
    pub(super) fn value_resume_state(self) -> u32 {
        match self.execution {
            ForAwaitIteratorExecution::Existing { states, .. } => states.value_resume_state,
            ForAwaitIteratorExecution::Awaited(plan) => plan.value_resume_state(),
        }
    }
    pub(super) fn close_resume_state(self) -> u32 {
        match self.execution {
            ForAwaitIteratorExecution::Existing { states, .. } => states.close_resume_state,
            ForAwaitIteratorExecution::Awaited(plan) => plan.close_resume_state(),
        }
    }
    pub(super) fn exit_state(self) -> u32 {
        match self.execution {
            ForAwaitIteratorExecution::Existing { states, .. } => states.exit_state,
            ForAwaitIteratorExecution::Awaited(plan) => plan.plan().exit_state(),
        }
    }
    pub(super) fn value_mode(self) -> BindingMode {
        match self.execution {
            ForAwaitIteratorExecution::Existing { binding, .. } => binding.mode,
            ForAwaitIteratorExecution::Awaited(plan) => plan.plan().value_mode(),
        }
    }
    pub(super) fn value_name(self) -> &'a str {
        match self.execution {
            ForAwaitIteratorExecution::Existing { binding, .. } => &binding.name,
            ForAwaitIteratorExecution::Awaited(plan) => plan.plan().value_name(),
        }
    }
    pub(super) fn head_environment(self) -> Option<&'a ForInOfEnvironmentIr> {
        match self.execution {
            ForAwaitIteratorExecution::Existing {
                head_environment, ..
            } => head_environment,
            ForAwaitIteratorExecution::Awaited(plan) => plan.plan().head_environment(),
        }
    }
    pub(super) fn body_suspends(self) -> bool {
        match self.execution {
            ForAwaitIteratorExecution::Existing { body, .. } => {
                FunctionBuilder::async_statement_entry_state(body).is_some()
            }
            ForAwaitIteratorExecution::Awaited(_) => true,
        }
    }
    pub(super) fn has_unowned_body_environment(self) -> bool {
        match self.execution {
            ForAwaitIteratorExecution::Existing { body, .. } => {
                matches!(body, StatementIr::Block(block) if block.lexical_environment.is_some())
            }
            // The sole checked body constructor rejects environments at every
            // nested body/catch site before this view can exist.
            ForAwaitIteratorExecution::Awaited(_) => false,
        }
    }
    pub(super) fn compile_body(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.execution {
            ForAwaitIteratorExecution::Existing { body, .. } => {
                builder.compile_statement(body, function)
            }
            ForAwaitIteratorExecution::Awaited(plan) => builder
                .compile_resumable_statement_sequence(
                    plan.plan().body().statements(),
                    plan.plan().body().entry_state(),
                    function,
                ),
        }
    }
}
