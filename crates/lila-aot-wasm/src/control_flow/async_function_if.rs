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
            .is_some_and(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "async conditional continuation requires a plain async function",
            ));
        }
        let activation_local = self
            .new_target_payload_local()
            .expect("async function bodies use the function call ABI");
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_RESUME_STATE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_i32(condition, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_statement_result(function, ValueKind::Undefined);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_async_resume_state(activation_local, plan.then_entry_state(), function);
        function.instruction(&Instruction::Else);
        self.emit_set_async_resume_state(activation_local, plan.else_entry_state(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_async_state_in_range(
            activation_local,
            plan.then_entry_state(),
            plan.else_entry_state(),
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.compile_statement(then_branch, function)?;
        self.emit_set_async_resume_state(activation_local, plan.exit_state(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_async_state_in_range(
            activation_local,
            plan.else_entry_state(),
            plan.exit_state(),
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        if let Some(else_branch) = else_branch {
            self.compile_statement(else_branch, function)?;
        }
        self.emit_set_async_resume_state(activation_local, plan.exit_state(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
