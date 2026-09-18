use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_enter_resumable_lexical_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (resume_state_offset, saved_environment_offset) = match self
            .current_function_meta()
            .map(|meta| meta.protocol.execution_kind())
        {
            Some(FunctionExecutionKind::Async) => {
                (HEAP_ASYNC_RESUME_STATE_OFFSET, HEAP_ASYNC_ENV_OFFSET)
            }
            Some(FunctionExecutionKind::Generator) => (
                HEAP_GENERATOR_RESUME_STATE_OFFSET,
                HEAP_GENERATOR_LEXICAL_ENV_OFFSET,
            ),
            Some(FunctionExecutionKind::Ordinary | FunctionExecutionKind::AsyncGenerator)
            | None => {
                return Err(EmitError::unsupported(
                    "saved lexical environment requires a plain async function or generator",
                ))
            }
        };
        let activation_local = self
            .new_target_payload_local()
            .expect("a resumable block belongs to an activation");
        self.load_i64_to_local_from_offset(
            activation_local,
            resume_state_offset,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(i64::from(entry_state)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_allocate_lexical_environment_record(environment, function)?;
        function.instruction(&Instruction::Else);

        // An inner block may own the saved record. Reattach this block's
        // existing child of the enclosing record before restoring its bindings.
        let saved_environment_local = self.reserve_temp_local();
        let parent_environment_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            activation_local,
            saved_environment_offset,
            saved_environment_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(saved_environment_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            saved_environment_local,
            ENV_PARENT_OFFSET,
            parent_environment_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_environment_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(parent_environment_local));
        function.instruction(&Instruction::LocalSet(saved_environment_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_environment_local));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.release_temp_local(parent_environment_local);
        self.release_temp_local(saved_environment_local);
        function.instruction(&Instruction::End);
        self.begin_existing_lexical_environment_scope(environment);
        Ok(())
    }
}
