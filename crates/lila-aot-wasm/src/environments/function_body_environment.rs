use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_initialize_function_body_bindings(
        &mut self,
        environment: &LexicalEnvironmentIr,
        parent_env_local: u32,
        function: &mut Function,
    ) {
        match &environment.initialization {
            lila_ir::LexicalEnvironmentInitializationIr::Uninitialized => {}
            lila_ir::LexicalEnvironmentInitializationIr::FunctionBody { bindings } => {
                for binding in bindings {
                    for (offset, undefined) in [
                        (ENV_SLOT_TAG_OFFSET, ValueKind::Undefined.tag() as u64),
                        (ENV_SLOT_PAYLOAD_OFFSET, 0),
                    ] {
                        match &binding.value {
                            lila_ir::FunctionBodyBindingValueIr::Undefined => {
                                self.store_i64_const_at_offset(
                                    self.current_env_local,
                                    Self::env_slot_offset(binding.slot, offset),
                                    undefined,
                                    function,
                                );
                            }
                            lila_ir::FunctionBodyBindingValueIr::Parameter { slot } => {
                                self.load_i64_to_local_from_offset(
                                    parent_env_local,
                                    Self::env_slot_offset(*slot, offset),
                                    self.scratch_local,
                                    function,
                                );
                                self.store_i64_local_at_offset(
                                    self.current_env_local,
                                    Self::env_slot_offset(binding.slot, offset),
                                    self.scratch_local,
                                    function,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn emit_prepare_resumable_function_body_environment(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(mut environment) = self.body.statements.iter().find_map(|statement| {
            let StatementIr::Block(body) = statement else {
                return None;
            };
            body.lexical_environment
                .as_ref()
                .filter(|environment| {
                    matches!(
                        environment.initialization,
                        lila_ir::LexicalEnvironmentInitializationIr::FunctionBody { .. }
                    )
                })
                .cloned()
        }) else {
            return Ok(());
        };
        environment.initialization = lila_ir::LexicalEnvironmentInitializationIr::Uninitialized;
        let parameter_environment = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parameter_environment));
        self.emit_allocate_lexical_environment_record(&environment, function)?;
        self.store_i64_local_at_offset(
            parameter_environment,
            ENV_FUNCTION_BODY_OFFSET,
            self.current_env_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parameter_environment));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.release_temp_local(parameter_environment);
        Ok(())
    }

    pub(crate) fn emit_enter_resumable_block_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        resume_state_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(
            environment.initialization,
            lila_ir::LexicalEnvironmentInitializationIr::Uninitialized
        ) {
            return self.emit_enter_lexical_environment(environment, function);
        }
        let parameter_environment = self.reserve_temp_local();
        let body_environment = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parameter_environment));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            parameter_environment,
            ENV_FUNCTION_BODY_OFFSET,
            body_environment,
            function,
        );
        function.instruction(&Instruction::LocalGet(body_environment));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            parameter_environment,
            ENV_PARENT_OFFSET,
            parameter_environment,
            function,
        );
        function.instruction(&Instruction::LocalGet(parameter_environment));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(body_environment));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        let activation = self
            .new_target_payload_local()
            .expect("resumable body uses an activation");
        self.load_i64_to_local_from_offset(
            activation,
            resume_state_offset,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(entry_state as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_initialize_function_body_bindings(environment, parameter_environment, function);
        function.instruction(&Instruction::End);
        self.begin_existing_lexical_environment_scope(environment);
        self.release_temp_local(body_environment);
        self.release_temp_local(parameter_environment);
        Ok(())
    }
}
