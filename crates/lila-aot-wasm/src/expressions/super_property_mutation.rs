use super::*;

#[derive(Debug)]
#[must_use = "a raw Super Property Reference must be consumed by GetValue"]
struct EvaluatedRawSuperPropertyReferenceLocals {
    base_payload: u32,
    base_tag: u32,
    receiver_payload: u32,
    receiver_tag: u32,
    referenced_name_payload: u32,
    referenced_name_tag: u32,
}

#[derive(Debug)]
#[must_use = "a coerced Super Property Reference must be consumed by PutValue"]
struct CoercedSuperPropertyReferenceLocals {
    base_payload: u32,
    base_tag: u32,
    receiver_payload: u32,
    receiver_tag: u32,
    property_key_payload: u32,
    property_key_tag: u32,
}

impl<'a> FunctionBuilder<'a> {
    fn evaluate_raw_super_property_reference(
        &mut self,
        receiver: &TypedExpr,
        referenced_name: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<EvaluatedRawSuperPropertyReferenceLocals, EmitError> {
        let base_payload = self.reserve_temp_local();
        let base_tag = self.reserve_temp_local();
        let receiver_payload = self.reserve_temp_local();
        let receiver_tag = self.reserve_temp_local();
        let referenced_name_payload = self.reserve_temp_local();
        let referenced_name_tag = self.reserve_temp_local();

        self.compile_expr_to_locals(receiver, receiver_payload, receiver_tag, function)?;
        self.compile_raw_property_key_expression_to_locals(
            referenced_name,
            referenced_name_payload,
            referenced_name_tag,
            function,
        )?;
        self.emit_load_super_base(base_payload, base_tag, function)?;
        self.emit_throw_if_null_super_base(base_payload, base_tag, function)?;

        Ok(EvaluatedRawSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            referenced_name_payload,
            referenced_name_tag,
        })
    }

    fn emit_get_value_from_raw_super_property_reference(
        &mut self,
        reference: EvaluatedRawSuperPropertyReferenceLocals,
        value_payload: u32,
        value_tag: u32,
        function: &mut Function,
    ) -> Result<CoercedSuperPropertyReferenceLocals, EmitError> {
        let EvaluatedRawSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            referenced_name_payload: property_key_payload,
            referenced_name_tag: property_key_tag,
        } = reference;

        self.emit_value_to_property_key_locals(property_key_payload, property_key_tag, function)?;
        self.emit_object_read_with_key_tag(
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            Some(property_key_tag),
            value_payload,
            value_tag,
            function,
        )?;

        Ok(CoercedSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
        })
    }

    fn emit_put_value_from_coerced_super_property_reference(
        &mut self,
        reference: CoercedSuperPropertyReferenceLocals,
        value_payload: u32,
        value_tag: u32,
        set_result: u32,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CoercedSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
        } = reference;

        self.emit_ordinary_set_result_via_helper(
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
            value_payload,
            value_tag,
            set_result,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.with_reference_strictness(strictness, function, |emitter, function| {
            emitter.emit_object_write_set_failure_else("Cannot assign to super property", function)
        })?;
        function.instruction(&Instruction::End);

        self.release_temp_local(property_key_tag);
        self.release_temp_local(property_key_payload);
        self.release_temp_local(receiver_tag);
        self.release_temp_local(receiver_payload);
        self.release_temp_local(base_tag);
        self.release_temp_local(base_payload);
        Ok(())
    }

    pub(super) fn compile_super_property_mutation_to_locals(
        &mut self,
        mutation: &SuperPropertyMutationIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // These result locals sit below the reference carrier so PutValue can
        // consume and release the carrier while retaining both numeric values
        // until a successful Set selects the prefix/postfix result.
        let old_value_payload = self.reserve_temp_local();
        let old_value_tag = self.reserve_temp_local();
        let new_value_payload = self.reserve_temp_local();
        let new_value_tag = self.reserve_temp_local();
        let set_result = self.reserve_temp_local();

        let raw_reference = self.evaluate_raw_super_property_reference(
            mutation.receiver(),
            mutation.referenced_name(),
            function,
        )?;
        let coerced_reference = self.emit_get_value_from_raw_super_property_reference(
            raw_reference,
            old_value_payload,
            old_value_tag,
            function,
        )?;

        match mutation.operation() {
            SuperPropertyMutationOperationIr::NumericUpdate {
                op,
                return_mode,
                value_kind,
            } => {
                match value_kind {
                    NumericUpdateValueKind::Dynamic => self.emit_value_to_numeric_locals(
                        old_value_payload,
                        old_value_tag,
                        function,
                    )?,
                    NumericUpdateValueKind::Number => {
                        self.emit_value_to_number_payload(
                            old_value_tag,
                            old_value_payload,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalSet(old_value_payload));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(old_value_tag));
                        self.emit_return_current_completion_if_throw(function);
                    }
                    NumericUpdateValueKind::BigInt => {}
                }
                self.emit_update_delta_from_locals(
                    *op,
                    *value_kind,
                    old_value_payload,
                    old_value_tag,
                    function,
                );
                function.instruction(&Instruction::LocalSet(new_value_payload));
                function.instruction(&Instruction::LocalGet(old_value_tag));
                function.instruction(&Instruction::LocalSet(new_value_tag));

                self.emit_put_value_from_coerced_super_property_reference(
                    coerced_reference,
                    new_value_payload,
                    new_value_tag,
                    set_result,
                    mutation.strictness(),
                    function,
                )?;

                let (result_payload, result_tag) = match return_mode {
                    UpdateReturnMode::Prefix => (new_value_payload, new_value_tag),
                    UpdateReturnMode::Postfix => (old_value_payload, old_value_tag),
                };
                function.instruction(&Instruction::LocalGet(result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            SuperPropertyMutationOperationIr::EagerCompound {
                old_value_binding,
                result,
            } => {
                self.push_scope();
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(
                        old_value_binding.clone(),
                        BindingStorage::Dynamic {
                            tag_local: old_value_tag,
                            payload_local: old_value_payload,
                        },
                    );
                let compile_result =
                    self.compile_expr_to_locals(result, new_value_payload, new_value_tag, function);
                self.pop_scope();
                compile_result?;

                self.emit_put_value_from_coerced_super_property_reference(
                    coerced_reference,
                    new_value_payload,
                    new_value_tag,
                    set_result,
                    mutation.strictness(),
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(new_value_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(new_value_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
        }

        self.release_temp_local(set_result);
        self.release_temp_local(new_value_tag);
        self.release_temp_local(new_value_payload);
        self.release_temp_local(old_value_tag);
        self.release_temp_local(old_value_payload);
        Ok(())
    }
}
