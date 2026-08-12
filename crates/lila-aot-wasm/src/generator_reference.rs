use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Evaluates an ordinary property Reference before its generator suspends
    /// and persists the normalized record operands in the activation.
    pub(crate) fn prepare_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        activation_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver,
                key,
                strictness: _,
            } => {
                let base_payload_local = self.reserve_temp_local();
                let base_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();

                self.compile_expr_to_locals(
                    base_and_receiver,
                    base_payload_local,
                    base_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    base_payload_local,
                    base_tag_local,
                    function,
                )?;
                self.compile_object_key_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.store_i64_local_at_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
                    base_payload_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
                    base_tag_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
                    key_payload_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_KEY_TAG_OFFSET,
                    key_tag_local,
                    function,
                );

                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(base_tag_local);
                self.release_temp_local(base_payload_local);
                Ok(())
            }
        }
    }

    /// Consumes the property Reference after a normal resume. The base/key
    /// come from the activation; `[[Strict]]` comes from the same typed record
    /// and selects PutValue 3.d through the shared Reference guard.
    pub(crate) fn write_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver: _,
                key: _,
                strictness,
            } => {
                let base_payload_local = self.reserve_temp_local();
                let base_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_TARGET_PAYLOAD_OFFSET,
                    base_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_TARGET_TAG_OFFSET,
                    base_tag_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    activation_local,
                    HEAP_GENERATOR_ASSIGNMENT_KEY_PAYLOAD_OFFSET,
                    key_payload_local,
                    function,
                );
                self.with_reference_strictness(strictness, function, |emitter, function| {
                    emitter.emit_object_write(
                        base_payload_local,
                        base_tag_local,
                        key_payload_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )
                })?;

                // PutValue is part of resuming the suspended assignment, not a
                // detached side effect. The outlined object-write helper reports
                // a failed strict [[Set]] as a Throw completion; route it before
                // the generator dispatcher can emit the following statement (or
                // replace it with the yield* statement result).
                self.emit_propagate_current_completion_if_throw(function);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(base_tag_local);
                self.release_temp_local(base_payload_local);
                Ok(())
            }
        }
    }
}
