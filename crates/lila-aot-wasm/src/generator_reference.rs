use super::*;
use lila_ir::FunctionExecutionKind;

impl<'a> FunctionBuilder<'a> {
    /// Select the activation layout from its actual execution owner, never from
    /// a caller-supplied offset. Both generator forms use the same Reference
    /// lifecycle; only their activation layouts differ.
    pub(crate) fn suspended_property_reference_offsets(&self) -> Result<[u64; 4], EmitError> {
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
                let locals: [u32; crate::planning::ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS] =
                    std::array::from_fn(|_| self.reserve_temp_local());
                let [base_payload, base_tag, key_payload, key_tag] = locals;
                self.compile_expr_to_locals(base_and_receiver, base_payload, base_tag, function)?;
                self.emit_propagate_throw_from_locals_if_needed(base_payload, base_tag, function)?;
                self.compile_raw_property_key_expression_to_locals(
                    key,
                    key_payload,
                    key_tag,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(key_payload, key_tag, function)?;
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
}
