use super::*;

impl FunctionBuilder<'_> {
    /// The thrown object's observable constructor must be the expected value.
    /// Prototype ancestry never substitutes for this identity comparison.
    pub(crate) fn compile_host_assert_throws_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let expected = schema.reserve_value_local(f);
        let callback = schema.reserve_value_local(f);
        let undefined = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let call = schema.reserve_completion(f);
        let constructor = schema.reserve_completion(f);
        let actual_type = schema.reserve_value_local(f);
        let object_type = schema.reserve_value_local(f);
        let object_name = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("object", f)?, f);
        object_type.set_reference(&object_name, schema, f);
        object_name.clear(f);
        self.emit_builtin_arg_to_value(0, &expected, f);
        self.emit_builtin_arg_to_value(1, &callback, f);
        undefined.set_undefined(f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_callable_i32(&callback, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::ASSERT_THROWS_REQUIRES_A_FUNCTION_CALLBACK,
            &output,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], f);
        self.emit_function_or_proxy_call_with_argv(&callback, &undefined, &arguments, &call, f)?;
        arguments.clear(f);
        call.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_runtime_error(
            NativeErrorKind::Error,
            RuntimeErrorMessage::ASSERT_THROWS_EXPECTED_A_THROW,
            &output,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_typeof_value(call.value(), &actual_type, f)?;
        self.emit_tagged_payload_equality_i32(&actual_type, &object_type, f)?;
        call.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null as i32));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_runtime_error(
            NativeErrorKind::Error,
            RuntimeErrorMessage::ASSERT_THROWS_EXPECTED_AN_ERROR_OBJECT,
            &output,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let name = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference("constructor", f)?, f);
        let key = PropertyKeyLocals::from_string(schema, &name, f);
        self.emit_object_read(call.value(), call.value(), &key, &constructor, f)?;
        key.clear(f);
        name.clear(f);
        self.emit_host_abrupt_exit(&constructor, &output, exit, f);
        self.emit_tagged_payload_equality_i32(constructor.value(), &expected, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_runtime_error(
            NativeErrorKind::Error,
            RuntimeErrorMessage::ASSERT_THROWS_RECEIVED_THE_WRONG_ERROR_CONSTRUCTOR,
            &output,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        object_type.clear(f);
        actual_type.clear(f);
        constructor.clear(f);
        call.clear(f);
        output.clear(f);
        undefined.clear(f);
        callback.clear(f);
        expected.clear(f);
        Ok(())
    }
}
