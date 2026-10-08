//! Resume ancestry comes only from the actual checked source certificate.

use super::*;

pub(super) fn emit_scope_outer_resume_test(
    plan: &lila_ir::AsyncGeneratorResumeEnvironmentPlanIr,
    point: crate::gc_types::I32Local,
    function: &mut Function,
) {
    function.instruction(&Instruction::I32Const(0));
    for state in plan
        .invocation_resume_states()
        .iter()
        .chain(plan.enclosing_scope_resume_states())
    {
        point.load(function);
        function.instruction(&Instruction::I32Const(*state as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
    }
}

fn emit_enclosing_saved_resume_test(
    plan: &lila_ir::AsyncGeneratorResumeEnvironmentPlanIr,
    point: crate::gc_types::I32Local,
    function: &mut Function,
) {
    function.instruction(&Instruction::I32Const(0));
    for state in plan.enclosing_scope_resume_states() {
        point.load(function);
        function.instruction(&Instruction::I32Const(*state as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
    }
    function.instruction(&Instruction::I32Const(0));
    for state in plan.invocation_resume_states() {
        point.load(function);
        function.instruction(&Instruction::I32Const(*state as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
    }
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::I32And);
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_checked_async_generator_scope_resume(
        &self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let plan = self
            .async_generator_resume_environment_plan
            .as_ref()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: mixed environment has no checked source certificate",
                )
            })?;
        let point = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_point())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: mixed environment has no entry resume point",
                )
            })?;
        emit_scope_outer_resume_test(plan, point, function);
        Ok(())
    }

    pub(crate) fn emit_checked_async_generator_enclosing_saved_resume(
        &self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let plan = self
            .async_generator_resume_environment_plan
            .as_ref()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: foreign scope has no checked source certificate",
                )
            })?;
        let point = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_point())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: foreign scope has no entry resume point",
                )
            })?;
        emit_enclosing_saved_resume_test(plan, point, function);
        Ok(())
    }

    /// The enclosing source scopes were reconstructed before this foreign
    /// owner. Its exact Saved body resume recovers the iteration child of
    /// that parent before the existing detach reads current_environment.
    pub(crate) fn emit_reattach_checked_async_generator_foreign_environment(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.current_function_meta().is_some_and(|meta| {
            meta.protocol().execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            return Ok(());
        }
        let Some(plan) = self
            .async_generator_resume_environment_plan
            .as_ref()
            .filter(|plan| !plan.enclosing_scope_resume_states().is_empty())
        else {
            return Ok(());
        };
        let point = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_point())
            .expect("mixed source owns its entry resume point");
        emit_enclosing_saved_resume_test(plan, point, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_reattach_saved_child_lexical_environment(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn with_checked_async_generator_source_environment<T>(
        &mut self,
        emit: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        if previous.is_none()
            && self.current_function_meta().is_some_and(|meta| {
                meta.protocol().execution_kind() == FunctionExecutionKind::AsyncGenerator
            })
        {
            self.checked_async_generator_environment_owner = self
                .async_generator_resume_environment_plan
                .as_ref()
                .and_then(crate::control_flow::CheckedAsyncGeneratorEnvironmentOwner::for_source);
        }
        let result = emit(self);
        self.checked_async_generator_environment_owner = previous;
        result
    }
}
