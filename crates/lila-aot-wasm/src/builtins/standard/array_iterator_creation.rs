use super::*;

pub(super) enum ArrayIteratorReceiverPolicy {
    GenericArrayLike,
    TypedArray,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_array_iterator_method_builtin(
        &mut self,
        kind: ArrayIterationKind,
        receiver_policy: ArrayIteratorReceiverPolicy,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        self.compile_this_to_locals(&receiver, function)?;
        output.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        match receiver_policy {
            ArrayIteratorReceiverPolicy::GenericArrayLike => {
                self.emit_value_to_current_function_realm_object_locals(
                    &receiver, &pending, function,
                )?;
                pending.kind().load(function);
                function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                output.copy_from(&pending, function);
                self.emit_branch_to_target(exit, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                // Borrowed Array methods retain the converted object in the
                // generic iterator. TypedArray admission belongs to next().
                let iterator = schema.reserve_gc_local(function).initialize(
                    self.emit_array_iterator_create_from_locals(pending.value(), kind, function)?,
                    function,
                );
                output.value().set_reference(&iterator, schema, function);
                iterator.clear(function);
            }
            ArrayIteratorReceiverPolicy::TypedArray => {
                receiver.reference().load(function);
                function.instruction(&Instruction::RefTestNonNull(
                    schema
                        .reference_type::<TypedArrayObject>(GcNullability::NonNullable)
                        .heap_type,
                ));
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_throw_current_function_realm_type_error(
                    RuntimeErrorMessage::TYPEDARRAY_ITERATOR_METHOD_REQUIRES_A_TYPEDARRAY,
                    &output,
                    function,
                )?;
                self.emit_branch_to_target(exit, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                let array = schema.reserve_gc_local(function).initialize(
                    receiver.cast_reference::<TypedArrayObject>(schema, function),
                    function,
                );
                let length = schema.reserve_i64_local(function);
                self.emit_validate_typed_array_view(&array, length, &pending, function)?;
                pending.kind().load(function);
                function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                output.copy_from(&pending, function);
                self.emit_branch_to_target(exit, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                let iterator = schema.reserve_gc_local(function).initialize(
                    self.emit_typed_array_iterator_create_from_locals(&array, kind, function)?,
                    function,
                );
                output.value().set_reference(&iterator, schema, function);
                iterator.clear(function);
                schema.release_i64_local(length, function);
                array.clear(function);
            }
        }
        output.set_kind(CompletionKind::Normal, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        pending.clear(function);
        output.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
