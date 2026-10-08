use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_async_generator_method_builtin(
        &mut self,
        kind: AsyncGeneratorRequestCompletionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let promise_value = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        let settled = schema.reserve_completion(function);
        let state = schema.reserve_i32_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        // Intrinsic capability allocation precedes receiver validation.
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(function);
        let capability = self
            .emit_new_current_function_realm_intrinsic_promise_capability(constructor, function)?;
        self.emit_read_promise_capability_promise(&capability, &promise_value, function);
        let promise = schema.reserve_gc_local(function).initialize(
            promise_value.cast_reference::<PromiseObject>(schema, function),
            function,
        );
        output.set_normal(&promise_value, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<AsyncGeneratorObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ASYNCGENERATOR_METHOD_CALLED_ON_INCOMPATIBLE_RECEIVER,
            &pending,
            function,
        )?;
        self.emit_call_promise_capability(
            &capability,
            PromiseSettlement::Reject,
            pending.value(),
            &settled,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let generator = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<AsyncGeneratorObject>(schema, function),
            function,
        );
        let activation = schema.reserve_gc_local(function).initialize(
            schema
                .field(AsyncGeneratorObjectSchema::ACTIVATION)
                .read(&generator, schema, function)
                .reference(),
            function,
        );
        generator.clear(function);
        let row = schema.struct_type::<AsyncGeneratorActivation>();
        row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
            .read(&activation, schema, function)
            .store(state, function);
        match kind {
            AsyncGeneratorRequestCompletionKind::Normal => {
                state.load(function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    AsyncGeneratorExecutionState::Completed,
                )));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                argument.set_undefined(function);
                self.emit_iterator_result_object_from_locals(&argument, true, &pending, function)?;
                self.emit_call_promise_capability(
                    &capability,
                    PromiseSettlement::Fulfill,
                    pending.value(),
                    &settled,
                    function,
                )?;
                self.emit_branch_to_target(exit, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            AsyncGeneratorRequestCompletionKind::Throw => {
                state.load(function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    AsyncGeneratorExecutionState::SuspendedStart,
                )));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
                    .write(
                        &activation,
                        GcOperand::constant(AsyncGeneratorExecutionState::Completed),
                        schema,
                        function,
                    );
                row.field(AsyncGeneratorActivationSchema::DELEGATE).write(
                    &activation,
                    GcOperand::null(schema),
                    schema,
                    function,
                );
                row.field(AsyncGeneratorActivationSchema::RESUME_POINT)
                    .write(&activation, GcOperand::i32(0), schema, function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    AsyncGeneratorExecutionState::Completed,
                )));
                state.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                state.load(function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    AsyncGeneratorExecutionState::Completed,
                )));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_call_promise_capability(
                    &capability,
                    PromiseSettlement::Reject,
                    &argument,
                    &settled,
                    function,
                )?;
                self.emit_branch_to_target(exit, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            AsyncGeneratorRequestCompletionKind::Return => {}
        }
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&argument, function),
            function,
        );
        let request = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<AsyncGeneratorRequest>().construct(
                (
                    GcOperand::constant(kind),
                    GcOperand::reference(&stored, schema),
                    GcOperand::reference(&capability, schema),
                    GcOperand::reference(&promise, schema),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        let tail = schema.reserve_gc_local(function).initialize(
            row.field(AsyncGeneratorActivationSchema::REQUEST_TAIL)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        tail.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        row.field(AsyncGeneratorActivationSchema::REQUEST_HEAD)
            .write(
                &activation,
                GcOperand::nullable_reference(&request, schema),
                schema,
                function,
            );
        function.instruction(&Instruction::Else);
        let previous = schema.reserve_gc_local(function).initialize(
            tail.load(schema, function).require_non_null(function),
            function,
        );
        schema.field(AsyncGeneratorRequestSchema::NEXT).write(
            &previous,
            GcOperand::nullable_reference(&request, schema),
            schema,
            function,
        );
        previous.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        row.field(AsyncGeneratorActivationSchema::REQUEST_TAIL)
            .write(
                &activation,
                GcOperand::nullable_reference(&request, schema),
                schema,
                function,
            );
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorExecutionState::SuspendedYield,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        row.field(AsyncGeneratorActivationSchema::ACTIVE_REQUEST)
            .write(
                &activation,
                GcOperand::nullable_reference(&request, schema),
                schema,
                function,
            );
        row.field(AsyncGeneratorActivationSchema::RESUME_VALUE)
            .write(
                &activation,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        let resume_kind = match kind {
            AsyncGeneratorRequestCompletionKind::Normal => AsyncGeneratorResumeKind::Normal,
            AsyncGeneratorRequestCompletionKind::Return => AsyncGeneratorResumeKind::Return,
            AsyncGeneratorRequestCompletionKind::Throw => AsyncGeneratorResumeKind::Throw,
        };
        row.field(AsyncGeneratorActivationSchema::RESUME_KIND)
            .write(
                &activation,
                GcOperand::constant(resume_kind),
                schema,
                function,
            );
        match kind {
            AsyncGeneratorRequestCompletionKind::Return => {
                row.field(AsyncGeneratorActivationSchema::BODY_STATUS)
                    .write(
                        &activation,
                        GcOperand::constant(AsyncGeneratorBodyStatus::Await),
                        schema,
                        function,
                    );
                row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
                    .write(
                        &activation,
                        GcOperand::constant(AsyncGeneratorExecutionState::Executing),
                        schema,
                        function,
                    );
                self.emit_async_generator_yield_return_reactions(&activation, &argument, function)?;
                pending.copy_from(self.completion(), function);
                pending.kind().load(function);
                function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                let rejected = schema.reserve_i32_local(function);
                function.instruction(&Instruction::I32Const(1));
                rejected.store(function);
                self.emit_async_generator_resume_value(
                    &activation,
                    pending.value(),
                    rejected,
                    AsyncGeneratorResumeKind::Return,
                    AsyncGeneratorResumeKind::Throw,
                    function,
                );
                self.completion().initialize(function);
                self.emit_start_async_generator_body(&activation, function)?;
                schema.release_i32_local(rejected, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            AsyncGeneratorRequestCompletionKind::Normal
            | AsyncGeneratorRequestCompletionKind::Throw => {
                self.emit_start_async_generator_body(&activation, function)?;
            }
        }
        function.instruction(&Instruction::Else);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorExecutionState::SuspendedStart,
        )));
        function.instruction(&Instruction::I32Eq);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorExecutionState::Completed,
        )));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        match kind {
            AsyncGeneratorRequestCompletionKind::Return => {
                row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
                    .write(
                        &activation,
                        GcOperand::constant(AsyncGeneratorExecutionState::DrainingQueue),
                        schema,
                        function,
                    );
                row.field(AsyncGeneratorActivationSchema::DELEGATE).write(
                    &activation,
                    GcOperand::null(schema),
                    schema,
                    function,
                );
                row.field(AsyncGeneratorActivationSchema::RESUME_POINT)
                    .write(&activation, GcOperand::i32(0), schema, function);
                self.emit_drain_async_generator_queue(&activation, function)?;
            }
            AsyncGeneratorRequestCompletionKind::Normal => {
                row.field(AsyncGeneratorActivationSchema::ACTIVE_REQUEST)
                    .write(
                        &activation,
                        GcOperand::nullable_reference(&request, schema),
                        schema,
                        function,
                    );
                row.field(AsyncGeneratorActivationSchema::RESUME_VALUE)
                    .write(
                        &activation,
                        GcOperand::reference(&stored, schema),
                        schema,
                        function,
                    );
                row.field(AsyncGeneratorActivationSchema::RESUME_KIND)
                    .write(
                        &activation,
                        GcOperand::constant(AsyncGeneratorResumeKind::Normal),
                        schema,
                        function,
                    );
                self.emit_start_async_generator_body(&activation, function)?;
            }
            AsyncGeneratorRequestCompletionKind::Throw => {
                function.instruction(&Instruction::Unreachable);
            }
        }
        function.instruction(&Instruction::Else);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorExecutionState::Executing,
        )));
        function.instruction(&Instruction::I32Eq);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorExecutionState::DrainingQueue,
        )));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        tail.clear(function);
        request.clear(function);
        stored.clear(function);
        activation.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        schema.release_i32_local(state, function);
        promise.clear(function);
        capability.clear(function);
        settled.clear(function);
        pending.clear(function);
        output.clear(function);
        promise_value.clear(function);
        argument.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
