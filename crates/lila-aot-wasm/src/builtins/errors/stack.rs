//! Stage-three Error stack accessors use the native ErrorData brand and ordinary
//! receiver descriptors. Captured text is internal, never a constructor property.
use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;

impl FunctionBuilder<'_> {
    fn emit_stack_type_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        result: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, result, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    pub(in crate::builtins) fn emit_error_stack_getter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let result = s.reserve_completion(f);
        result.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_stack_type_error_if(
            RuntimeErrorMessage::ERROR_STACK_ACCESSOR_REQUIRES_OBJECT_RECEIVER,
            &result,
            exit,
            f,
        )?;
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<NativeErrorObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        let error = s
            .reserve_gc_local(f)
            .initialize(receiver.cast_reference::<NativeErrorObject>(s, f), f);
        let stack = s.reserve_gc_local(f).initialize(
            s.field(NativeErrorObjectSchema::STACK)
                .read(&error, s, f)
                .reference(),
            f,
        );
        result.value().set_reference(&stack, s, f);
        stack.clear(f);
        error.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&result, f);
        result.clear(f);
        receiver.clear(f);
        Ok(())
    }

    pub(in crate::builtins) fn emit_error_stack_setter_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let home = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let result = s.reserve_completion(f);
        result.initialize(f);
        self.compile_this_to_locals(&receiver, f)?;
        self.emit_builtin_arg_to_value(0, &value, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_stack_type_error_if(
            RuntimeErrorMessage::ERROR_STACK_ACCESSOR_REQUIRES_OBJECT_RECEIVER,
            &result,
            exit,
            f,
        )?;
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        f.instruction(&Instruction::I32Ne);
        self.emit_stack_type_error_if(
            RuntimeErrorMessage::ERROR_STACK_SETTER_REQUIRES_STRING,
            &result,
            exit,
            f,
        )?;
        let realm = s
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ErrorPrototype,
            &home,
            f,
        );
        realm.clear(f);
        receiver.reference().load(f);
        home.reference().load(f);
        f.instruction(&Instruction::RefEq);
        self.emit_stack_type_error_if(
            RuntimeErrorMessage::ERROR_STACK_SETTER_CANNOT_WRITE_HOME_PROTOTYPE,
            &result,
            exit,
            f,
        )?;
        let key = self.emit_error_property_key("stack", f)?;
        let descriptor = self.emit_proxy_target_own_descriptor(&receiver, &key, f)?;
        pending.copy_from(self.completion(), f);
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        result.copy_from(&pending, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        descriptor.emit_found_i32(s, f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_object_write_strict(&receiver, &key, &value, &pending, f)?;
        f.instruction(&Instruction::Else);
        self.emit_create_data_property_or_throw(&receiver, &key, &value, &pending, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        descriptor.clear(f);
        key.clear(f);
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        result.copy_from(&pending, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&result, f);
        result.clear(f);
        pending.clear(f);
        home.clear(f);
        value.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
