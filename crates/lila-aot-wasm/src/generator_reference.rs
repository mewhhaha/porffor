use super::*;
use lila_ir::FunctionExecutionKind;

impl<'a> FunctionBuilder<'a> {
    /// Select the activation layout from its actual execution owner, never from
    /// a caller-supplied offset. Both generator forms use the same Reference
    /// lifecycle; only their activation layouts differ.
    fn suspended_property_reference_offsets(&self) -> Result<[u64; 4], EmitError> {
        let kind = self
            .current_function_meta()
            .map(|meta| meta.protocol.execution_kind());
        match kind {
            Some(FunctionExecutionKind::Generator) => Ok([
                HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
                HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
                HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
                HEAP_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET,
            ]),
            Some(FunctionExecutionKind::AsyncGenerator) => Ok([
                HEAP_ASYNC_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
                HEAP_ASYNC_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
                HEAP_ASYNC_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
                HEAP_ASYNC_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET,
            ]),
            Some(FunctionExecutionKind::Ordinary | FunctionExecutionKind::Async) | None => {
                Err(EmitError::unsupported(
                    "suspended property Reference requires a generator activation",
                ))
            }
        }
    }

    pub(crate) fn clear_suspended_property_reference(
        &self,
        activation_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (offset, value) in self
            .suspended_property_reference_offsets()?
            .into_iter()
            .zip([
                0,
                ValueKind::Undefined.tag() as u64,
                0,
                ValueKind::Undefined.tag() as u64,
            ])
        {
            self.store_i64_const_at_offset(activation_local, offset, value, function);
        }
        Ok(())
    }

    /// Evaluate the base and raw computed key before the RHS suspends. In
    /// `base[key] = yield value`, ToPropertyKey belongs to the eventual
    /// PutValue, not to Reference evaluation (ECMA-262 13.3.3).
    pub(crate) fn prepare_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        activation_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offsets = self.suspended_property_reference_offsets()?;
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver,
                key,
                strictness: _,
            } => {
                let locals = std::array::from_fn::<_, 4, _>(|_| self.reserve_temp_local());
                let [base_payload, base_tag, key_payload, key_tag] = locals;
                self.compile_expr_to_locals(base_and_receiver, base_payload, base_tag, function)?;
                self.emit_propagate_throw_from_locals_if_needed(base_payload, base_tag, function)?;
                match key {
                    PropertyKeyIr::StaticString(name) => {
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_payload));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(key_tag));
                    }
                    PropertyKeyIr::ArrayLength => {
                        function
                            .instruction(&Instruction::I64Const(self.strings.payload("length")));
                        function.instruction(&Instruction::LocalSet(key_payload));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(key_tag));
                    }
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        self.compile_expr_to_locals(expr, key_payload, key_tag, function)?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            key_payload,
                            key_tag,
                            function,
                        )?;
                    }
                }
                for (offset, local) in offsets.into_iter().zip(locals) {
                    self.store_i64_local_at_offset(activation_local, offset, local, function);
                }
                for local in locals.into_iter().rev() {
                    self.release_temp_local(local);
                }
                Ok(())
            }
        }
    }

    /// Consume the saved Reference only on normal RHS completion. A throwing
    /// key conversion or setter routes through the active catch/finally just
    /// like any other PutValue. User code cannot overwrite the saved RHS via
    /// the emitter's result registers while converting the key.
    pub(crate) fn write_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offsets = self.suspended_property_reference_offsets()?;
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver: _,
                key: _,
                strictness,
            } => {
                let locals = std::array::from_fn::<_, 6, _>(|_| self.reserve_temp_local());
                let [base_payload, base_tag, key_payload, key_tag, value_payload, value_tag] =
                    locals;
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalSet(value_payload));
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::LocalSet(value_tag));
                for (offset, local) in offsets.into_iter().zip(locals) {
                    self.load_i64_to_local_from_offset(activation_local, offset, local, function);
                }
                self.clear_suspended_property_reference(activation_local, function)?;

                // PutValue's ToObject precedes ToPropertyKey: a nullish base
                // throws after the RHS, but without calling key coercion hooks.
                self.compile_nullish_tagged_i32(base_tag, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert undefined or null to object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_current_completion_if_throw(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_property_key_locals(key_payload, key_tag, function)?;
                self.with_reference_strictness(strictness, function, |emitter, function| {
                    emitter.emit_object_write(
                        base_payload,
                        base_tag,
                        key_payload,
                        value_payload,
                        value_tag,
                        function,
                    )
                })?;
                self.emit_propagate_current_completion_if_throw(function);
                for local in locals.into_iter().rev() {
                    self.release_temp_local(local);
                }
                Ok(())
            }
        }
    }
}
