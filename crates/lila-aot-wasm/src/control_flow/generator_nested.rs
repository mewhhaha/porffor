use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_generator_structured_loop(
        &mut self,
        init: Option<&ForInitIr>,
        test: Option<&TypedExpr>,
        update: Option<&TypedExpr>,
        plan: &GeneratorStructuredLoopPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entry_state = plan.entry_state();
        let body_entry_state = plan.body_entry_state();
        let body_exit_state = plan.body_exit_state();
        let exit_state = plan.exit_state();
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator loop requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(body_exit_state)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
            self.emit_dispatch_current_completion(function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            self.emit_dispatch_current_completion(function)?;
            function.instruction(&Instruction::I32Eqz);
            function.branch_if_to_label(break_frame.label);
        }
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(body_entry_state),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.compile_statement(plan.body(), function)?;
        self.emit_dispatch_current_completion(function)?;
        if let Some(update) = update {
            self.compile_expr_payload(update, function)?;
            function.instruction(&Instruction::Drop);
            self.emit_dispatch_current_completion(function)?;
        }
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(entry_state),
            function,
        );
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::LocalSet(state_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(exit_state),
            function,
        );
        self.emit_statement_result(function, ValueKind::Undefined);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(state_local);
        Ok(())
    }

    pub(crate) fn compile_generator_structured_if(
        &mut self,
        condition: &TypedExpr,
        plan: &GeneratorStructuredIfPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entry_state = plan.entry_state();
        let then_entry_state = plan.then_entry_state();
        let then_exit_state = plan.then_exit_state();
        let else_entry_state = plan.else_entry_state();
        let else_exit_state = plan.else_exit_state();
        let exit_state = plan.exit_state();
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator branch requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(else_exit_state)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_i32(condition, function)?;
        self.emit_dispatch_current_completion(function)?;
        self.open_frame(ControlFrameKind::If, function);
        for (state, branch) in [(then_entry_state, true), (else_entry_state, false)] {
            if !branch {
                function.instruction(&Instruction::Else);
            }
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                u64::from(state),
                function,
            );
            function.instruction(&Instruction::I64Const(i64::from(state)));
            function.instruction(&Instruction::LocalSet(state_local));
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(then_entry_state)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(then_exit_state)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_statement(plan.then_branch(), function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(else_entry_state)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(else_exit_state)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(else_branch) = plan.else_branch() {
            self.compile_statement(else_branch, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(exit_state),
            function,
        );
        self.emit_statement_result(function, ValueKind::Undefined);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(state_local);
        Ok(())
    }
}
