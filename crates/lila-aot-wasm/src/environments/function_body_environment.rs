use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_initialize_function_body_bindings<N: GcFieldNullability>(
        &mut self,
        environment: &LexicalEnvironmentIr,
        record: &GcLocal<Environment, N>,
        parent: &GcLocal<Environment, Nullable>,
        function: &mut Function,
    ) {
        match &environment.initialization {
            lila_ir::LexicalEnvironmentInitializationIr::Uninitialized => {}
            lila_ir::LexicalEnvironmentInitializationIr::FunctionBody { bindings } => {
                let schema = self.runtime_schema();
                let value = schema.reserve_value_local(function);
                for binding in bindings {
                    match &binding.value {
                        lila_ir::FunctionBodyBindingValueIr::Undefined => {
                            value.set_undefined(function)
                        }
                        lila_ir::FunctionBodyBindingValueIr::Parameter { slot } => {
                            let source = self.emit_environment_cell_local(parent, *slot, function);
                            let initialized =
                                self.emit_read_environment_cell(&source, &value, function);
                            schema.release_i32_local(initialized, function);
                            source.clear(function);
                        }
                    }
                    let target = self.emit_environment_cell_local(record, binding.slot, function);
                    self.emit_initialize_environment_cell(&target, &value, function);
                    target.clear(function);
                }
                value.clear(function);
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
        let schema = self.runtime_schema();
        let parameter = self.resolve_env_handle_local(0, function);
        self.emit_allocate_lexical_environment_record(&environment, function)?;
        schema
            .struct_type::<Environment>()
            .field(EnvironmentSchema::FUNCTION_BODY)
            .write(
                &parameter,
                GcOperand::reference(self.current_environment(), schema),
                schema,
                function,
            );
        self.replace_current_environment(parameter.load(schema, function), function);
        parameter.clear(function);
        Ok(())
    }

    pub(crate) fn emit_enter_resumable_block_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(
            environment.initialization,
            lila_ir::LexicalEnvironmentInitializationIr::Uninitialized
        ) {
            if self.current_function_meta().is_some_and(|meta| {
                matches!(
                    meta.protocol().execution_kind(),
                    FunctionExecutionKind::Async | FunctionExecutionKind::Generator
                ) || (meta.protocol().execution_kind() == FunctionExecutionKind::AsyncGenerator
                    && self.checked_async_generator_environment_owner.is_some())
            }) {
                return self.emit_enter_resumable_lexical_environment(
                    environment,
                    entry_state,
                    function,
                );
            }
            return self.emit_enter_lexical_environment(environment, function);
        }
        let schema = self.runtime_schema();
        let parameter = self.resolve_env_handle_local(0, function);
        let body = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize_null(schema, function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        body.replace(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::FUNCTION_BODY)
                .read(&parameter, schema, function)
                .reference(),
            function,
        );
        body.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        parameter.replace(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::PARENT)
                .read(&parameter, schema, function)
                .reference(),
            function,
        );
        parameter.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.replace_current_environment(body.load(schema, function), function);
        self.body_entry_locals()
            .expect("resumable body owns an entry")
            .resume_point()
            .expect("resumable body owns a resume point")
            .load(function);
        function.instruction(&Instruction::I32Const(entry_state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_initialize_function_body_bindings(environment, &body, &parameter, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.begin_existing_lexical_environment_scope(environment);
        body.clear(function);
        parameter.clear(function);
        Ok(())
    }
}
