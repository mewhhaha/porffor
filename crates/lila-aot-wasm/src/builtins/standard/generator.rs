use super::*;

impl<'a> FunctionBuilder<'a> {
    fn emit_generator_resume_call(
        &mut self,
        activation: &GcLocal<GeneratorActivation>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let row = schema.struct_type::<GeneratorActivation>();
        let pending = schema.reserve_completion(function);
        let value = schema.reserve_value_local(function);
        let state = schema.reserve_i32_local(function);
        let done = schema.reserve_i32_local(function);
        let frame = schema.reserve_gc_local(function).initialize(
            row.field(GeneratorActivationSchema::FRAME)
                .read(activation, schema, function)
                .reference(),
            function,
        );
        let callable = schema.reserve_gc_local(function).initialize(
            schema
                .field(InvocationFrameSchema::FUNCTION)
                .read(&frame, schema, function)
                .reference(),
            function,
        );
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .field(FunctionObjectSchema::CONTEXT)
                .read(&callable, schema, function)
                .reference(),
            function,
        );
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .field(FunctionContextSchema::REALM)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        context.clear(function);
        callable.clear(function);
        frame.clear(function);
        row.field(GeneratorActivationSchema::STATUS).write(
            activation,
            GcOperand::constant(GeneratorState::Executing),
            schema,
            function,
        );
        self.emit_saved_generator_body_call(activation, &pending, function);
        row.field(GeneratorActivationSchema::STATUS)
            .read(activation, schema, function)
            .store(state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorState::SuspendedYield,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // A live delegate owns the already-created IteratorResult identity.
        row.field(GeneratorActivationSchema::DELEGATE)
            .read(activation, schema, function)
            .reference()
            .is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        done.store(function);
        let result =
            self.emit_iterator_result_object_in_realm(&realm, pending.value(), done, function)?;
        value.set_reference(&result, schema, function);
        output.set_normal(&value, function);
        result.clear(function);
        function.instruction(&Instruction::Else);
        output.set_normal(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        row.field(GeneratorActivationSchema::STATUS).write(
            activation,
            GcOperand::constant(GeneratorState::Completed),
            schema,
            function,
        );
        row.field(GeneratorActivationSchema::DELEGATE).write(
            activation,
            GcOperand::null(schema),
            schema,
            function,
        );
        row.field(GeneratorActivationSchema::RESUME_POINT).write(
            activation,
            GcOperand::i32(0),
            schema,
            function,
        );
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(&pending, function);
        function.instruction(&Instruction::Else);
        value.set_undefined(function);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Return.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        value.copy_from(pending.value(), function);
        function.instruction(&Instruction::Else);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // GeneratorStart restores the caller context before terminal result
        // allocation; plain Yield allocates while the body context is active.
        self.emit_iterator_result_object_from_locals(&value, true, output, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        realm.clear(function);
        schema.release_i32_local(done, function);
        schema.release_i32_local(state, function);
        value.clear(function);
        pending.clear(function);
        Ok(())
    }

    pub(super) fn emit_generator_method_builtin(
        &mut self,
        kind: GeneratorResumeKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let state = schema.reserve_i32_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        output.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<GeneratorObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::GENERATOR_METHOD_CALLED_ON_INCOMPATIBLE_RECEIVER,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let generator = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<GeneratorObject>(schema, function),
            function,
        );
        let activation = schema.reserve_gc_local(function).initialize(
            schema
                .field(GeneratorObjectSchema::ACTIVATION)
                .read(&generator, schema, function)
                .reference(),
            function,
        );
        generator.clear(function);
        let row = schema.struct_type::<GeneratorActivation>();
        row.field(GeneratorActivationSchema::STATUS)
            .read(&activation, schema, function)
            .store(state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorState::Executing,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::GENERATOR_IS_ALREADY_RUNNING,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Abrupt entry to suspended-start never executes the user body.
        match kind {
            GeneratorResumeKind::Normal => {}
            GeneratorResumeKind::Return | GeneratorResumeKind::Throw => {
                state.load(function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    GeneratorState::SuspendedStart,
                )));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                row.field(GeneratorActivationSchema::STATUS).write(
                    &activation,
                    GcOperand::constant(GeneratorState::Completed),
                    schema,
                    function,
                );
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    GeneratorState::Completed,
                )));
                state.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorState::Completed,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        match kind {
            GeneratorResumeKind::Normal => {
                argument.set_undefined(function);
                self.emit_iterator_result_object_from_locals(&argument, true, &output, function)?;
            }
            GeneratorResumeKind::Return => {
                self.emit_iterator_result_object_from_locals(&argument, true, &output, function)?;
            }
            GeneratorResumeKind::Throw => output.set_throw(&argument, function),
        }
        function.instruction(&Instruction::Else);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorState::SuspendedStart,
        )));
        function.instruction(&Instruction::I32Eq);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorState::SuspendedYield,
        )));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&argument, function),
            function,
        );
        row.field(GeneratorActivationSchema::RESUME_VALUE).write(
            &activation,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        row.field(GeneratorActivationSchema::RESUME_KIND).write(
            &activation,
            GcOperand::constant(kind),
            schema,
            function,
        );
        stored.clear(function);
        self.emit_generator_resume_call(&activation, &output, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        activation.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        schema.release_i32_local(state, function);
        output.clear(function);
        argument.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
