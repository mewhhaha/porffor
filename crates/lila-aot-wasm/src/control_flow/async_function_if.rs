use super::*;
use lila_ir::AsyncFunctionIfPlanIr;

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_function_if(
        &mut self,
        condition: &TypedExpr,
        then_branch: &StatementIr,
        else_branch: Option<&StatementIr>,
        plan: AsyncFunctionIfPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "async conditional continuation requires a plain async function",
            ));
        }
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_i32(condition, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        self.emit_statement_result(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_resumable_resume_point(plan.then_entry_state(), function)?;
        function.instruction(&Instruction::Else);
        self.emit_set_resumable_resume_point(plan.else_entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_async_state_in_range(plan.then_entry_state(), plan.else_entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_statement(then_branch, function)?;
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_async_state_in_range(plan.else_entry_state(), plan.exit_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        if let Some(else_branch) = else_branch {
            self.compile_statement(else_branch, function)?;
        }
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
