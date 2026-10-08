use super::*;
use lila_ir::AsyncFunctionLabelledPlanIr;

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_function_labelled(
        &mut self,
        labels: &[String],
        statement: &StatementIr,
        plan: AsyncFunctionLabelledPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "labelled continuation region requires a plain async activation",
            ));
        }
        self.emit_async_state_in_range(plan.entry_state(), plan.exit_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.push_labels(labels, break_frame, None);
        self.compile_statement(statement, function)?;
        self.pop_labels(labels.len());
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Matching Break (including dispatch after an awaited finalizer) lands
        // here even if it bypassed the condition. Suspension and other abrupt
        // completions leave the function/region before this epilogue.
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
