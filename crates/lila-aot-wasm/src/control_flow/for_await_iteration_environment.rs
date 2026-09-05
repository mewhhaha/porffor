use super::*;

impl ForAwaitActivationLayout {
    pub(super) const fn lexical_environment_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_ENV_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_for_await_iteration_done_check(
        &mut self,
        body_suspends: bool,
        iteration_environment_present: bool,
        state_local: u32,
        value_resume_state: u32,
        done_storage: BindingStorage,
        done_payload_local: u32,
        done_tag_local: u32,
        break_frame: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_path_only = body_suspends && iteration_environment_present;
        if value_path_only {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        self.read_binding_to_locals(done_storage, done_payload_local, done_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.branch_if_to_label(break_frame.label);
        if value_path_only {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    pub(super) fn emit_enter_for_await_iteration_environment(
        &mut self,
        environment: Option<&LexicalEnvironmentIr>,
        body_suspends: bool,
        activation_local: u32,
        activation_environment_offset: u64,
        state_local: u32,
        value_resume_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(environment) = environment else {
            return Ok(());
        };
        if !body_suspends {
            return self.emit_enter_lexical_environment(environment, function);
        }

        let cleanup_frame = self.open_frame(ControlFrameKind::Block, function);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(value_resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_allocate_lexical_environment_record(environment, function)?;
        self.store_i64_local_at_offset(
            activation_local,
            activation_environment_offset,
            self.current_env_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.push_scope();
        self.begin_existing_lexical_environment_scope(environment);
        self.finally_stack.push(ControlTarget {
            environment_depth: self.environment_depth,
            ..cleanup_frame
        });
        Ok(())
    }

    pub(super) fn emit_leave_for_await_iteration_environment(
        &mut self,
        iteration_environment_present: bool,
        body_suspends: bool,
        activation_local: u32,
        activation_environment_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !body_suspends || !iteration_environment_present {
            return Ok(());
        }
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_leave_lexical_environment(function);
        self.pop_scope();
        self.store_i64_local_at_offset(
            activation_local,
            activation_environment_offset,
            self.current_env_local,
            function,
        );
        self.emit_dispatch_async_completion(function)
    }
}
