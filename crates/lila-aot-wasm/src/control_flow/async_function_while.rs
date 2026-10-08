use super::*;
use lila_ir::AsyncFunctionWhileConditionIr;

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_function_while_condition(
        &mut self,
        plan: &AsyncFunctionWhileConditionIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "awaited while condition requires a plain async activation",
            ));
        }
        self.emit_async_state_in_range(plan.entry_state(), plan.exit_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_statement_result(function);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);

        // Every iteration evaluates the condition anew. A reaction resumes only
        // its retained prefix state; it cannot repeat an earlier operand effect.
        self.compile_resumable_statement_sequence(
            plan.condition_prefix(),
            plan.entry_state(),
            function,
        )?;
        self.compile_iteration_condition(plan.condition(), function)?;
        function.instruction(&Instruction::I32Eqz);
        function.branch_if_to_label(break_frame.label);

        // Continue leaves the body, then resets the condition state. Branching
        // directly to the loop would keep Ready and reuse yesterday's result.
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(plan.body(), function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.entry_state(), function)?;
        function.branch_to_label(loop_frame.label);

        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
