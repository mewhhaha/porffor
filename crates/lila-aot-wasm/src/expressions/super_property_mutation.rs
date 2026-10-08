use super::*;

#[must_use = "raw Super Reference operands must enter GetValue or PutValue"]
struct EvaluatedRawSuperPropertyReferenceLocals {
    base: ValueLocals,
    receiver: ValueLocals,
    referenced_name: ValueLocals,
}

#[must_use = "a canonical Super Reference must be consumed by PutValue"]
struct CoercedSuperPropertyReferenceLocals {
    base: ValueLocals,
    receiver: ValueLocals,
    property_key: PropertyKeyLocals,
}
impl CoercedSuperPropertyReferenceLocals {
    fn clear(self, function: &mut Function) {
        self.property_key.clear(function);
        self.receiver.clear(function);
        self.base.clear(function);
    }
}

impl FunctionBuilder<'_> {
    fn evaluate_raw_super_property_reference(
        &mut self,
        receiver: &TypedExpr,
        referenced_name: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<EvaluatedRawSuperPropertyReferenceLocals, EmitError> {
        let schema = self.runtime_schema();
        let base = schema.reserve_value_local(function);
        let receiver_value = schema.reserve_value_local(function);
        let name = schema.reserve_value_local(function);
        self.compile_expr_to_value(receiver, &receiver_value, function)?;
        self.compile_raw_property_key_expression_to_value(referenced_name, &name, function)?;
        self.emit_load_super_base(&base, function)?;
        Ok(EvaluatedRawSuperPropertyReferenceLocals {
            base,
            receiver: receiver_value,
            referenced_name: name,
        })
    }

    fn canonicalize_super_property_reference(
        &mut self,
        raw: EvaluatedRawSuperPropertyReferenceLocals,
        function: &mut Function,
    ) -> Result<CoercedSuperPropertyReferenceLocals, EmitError> {
        let EvaluatedRawSuperPropertyReferenceLocals {
            base,
            receiver,
            referenced_name,
        } = raw;
        self.emit_throw_if_null_super_base(&base, function)?;
        let property_key = self.emit_value_to_property_key_locals(&referenced_name, function)?;
        referenced_name.clear(function);
        Ok(CoercedSuperPropertyReferenceLocals {
            base,
            receiver,
            property_key,
        })
    }

    fn emit_get_value_from_raw_super_property_reference(
        &mut self,
        raw: EvaluatedRawSuperPropertyReferenceLocals,
        old: &ValueLocals,
        function: &mut Function,
    ) -> Result<CoercedSuperPropertyReferenceLocals, EmitError> {
        let reference = self.canonicalize_super_property_reference(raw, function)?;
        let schema = self.runtime_schema();
        schema
            .call_helper(
                crate::runtime_helpers::ObjectReadArguments::new(
                    &reference.base,
                    &reference.receiver,
                    &reference.property_key,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        old.copy_from(self.completion().value(), function);
        Ok(reference)
    }

    fn emit_put_value_from_coerced_super_property_reference(
        &mut self,
        reference: CoercedSuperPropertyReferenceLocals,
        value: &ValueLocals,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        schema
            .call_helper(
                crate::runtime_helpers::OrdinarySetArguments::new(
                    &reference.base,
                    &reference.receiver,
                    &reference.property_key,
                    value,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        if strictness.throws_on_failed_set() {
            self.completion().value().scalar().load(function);
            function.instruction(&Instruction::I64Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_expression_native_error(
                NativeErrorKind::TypeError,
                RuntimeErrorMessage::CANNOT_ASSIGN_TO_SUPER_PROPERTY,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        reference.clear(function);
        Ok(())
    }

    pub(super) fn compile_super_property_read_to_value(
        &mut self,
        key: &PropertyKeyIr,
        receiver: &TypedExpr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let raw = self.evaluate_raw_super_property_reference(receiver, key, function)?;
        let reference =
            self.emit_get_value_from_raw_super_property_reference(raw, output, function)?;
        reference.clear(function);
        Ok(())
    }

    pub(super) fn compile_super_property_write_to_value(
        &mut self,
        key: &PropertyKeyIr,
        receiver: &TypedExpr,
        value: &TypedExpr,
        strictness: Strictness,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let raw = self.evaluate_raw_super_property_reference(receiver, key, function)?;
        let rhs = self.runtime_schema().reserve_value_local(function);
        self.compile_expr_to_value(value, &rhs, function)?;
        // A null Super base is permitted while evaluating the Reference. Plain
        // assignment completes the RHS before PutValue rejects that base.
        let reference = self.canonicalize_super_property_reference(raw, function)?;
        self.emit_put_value_from_coerced_super_property_reference(
            reference, &rhs, strictness, function,
        )?;
        output.copy_from(&rhs, function);
        rhs.clear(function);
        Ok(())
    }

    pub(super) fn compile_super_property_mutation_to_value(
        &mut self,
        mutation: &SuperPropertyMutationIr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match mutation.operation() {
            SuperPropertyMutationOperationIr::Capture(capture) => {
                return self.compile_super_property_capture(mutation, capture, output, function);
            }
            SuperPropertyMutationOperationIr::PutCaptured { capture, value } => {
                return self.compile_captured_super_property_put(
                    capture,
                    value,
                    mutation.strictness(),
                    output,
                    function,
                );
            }
            SuperPropertyMutationOperationIr::NumericUpdate { .. }
            | SuperPropertyMutationOperationIr::EagerCompound { .. } => {}
        }
        let schema = self.runtime_schema();
        let old = schema.reserve_value_local(function);
        let new = schema.reserve_value_local(function);
        let raw = self.evaluate_raw_super_property_reference(
            mutation.receiver(),
            mutation.referenced_name(),
            function,
        )?;
        let reference =
            self.emit_get_value_from_raw_super_property_reference(raw, &old, function)?;
        match mutation.operation() {
            SuperPropertyMutationOperationIr::Capture(_)
            | SuperPropertyMutationOperationIr::PutCaptured { .. } => {
                unreachable!("captured Super operations have already consumed their owner")
            }
            SuperPropertyMutationOperationIr::NumericUpdate {
                op,
                return_mode,
                value_kind,
            } => {
                self.emit_numeric_reference_old_value(*value_kind, &old, function)?;
                self.emit_numeric_update_to_locals(*op, *value_kind, &old, &new, function)?;
                self.emit_put_value_from_coerced_super_property_reference(
                    reference,
                    &new,
                    mutation.strictness(),
                    function,
                )?;
                output.copy_from(
                    match return_mode {
                        UpdateReturnMode::Prefix => &new,
                        UpdateReturnMode::Postfix => &old,
                    },
                    function,
                );
            }
            SuperPropertyMutationOperationIr::EagerCompound {
                old_value_binding,
                result,
            } => {
                self.push_scope();
                let (_, id) = self.retain_expression_operand(old_value_binding, &old, function);
                let compiled = self.compile_expr_to_value(result, &new, function);
                self.pop_scope();
                self.release_local_binding(id, function);
                compiled?;
                self.emit_put_value_from_coerced_super_property_reference(
                    reference,
                    &new,
                    mutation.strictness(),
                    function,
                )?;
                output.copy_from(&new, function);
            }
        }
        new.clear(function);
        old.clear(function);
        Ok(())
    }

    fn compile_super_property_capture(
        &mut self,
        mutation: &SuperPropertyMutationIr,
        capture: &lila_ir::SuperPropertyReferenceCaptureIr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let raw = self.evaluate_raw_super_property_reference(
            mutation.receiver(),
            mutation.referenced_name(),
            function,
        )?;
        match capture.mode() {
            lila_ir::SuperPropertyCaptureMode::ReadBeforeRhs => {
                let reference =
                    self.emit_get_value_from_raw_super_property_reference(raw, output, function)?;
                for (name, value) in [
                    (capture.receiver_storage_name(), &reference.receiver),
                    (capture.base_storage_name(), &reference.base),
                    (
                        capture.referenced_name_storage_name(),
                        reference.property_key.value(),
                    ),
                ] {
                    let storage = self
                        .lookup_binding(name)
                        .expect("Super Get capture owns its activation cell");
                    self.write_binding_from_locals(storage, value, function);
                }
                reference.clear(function);
            }
            lila_ir::SuperPropertyCaptureMode::WriteOnly => {
                for (name, value) in [
                    (capture.receiver_storage_name(), &raw.receiver),
                    (capture.base_storage_name(), &raw.base),
                    (capture.referenced_name_storage_name(), &raw.referenced_name),
                ] {
                    let storage = self
                        .lookup_binding(name)
                        .expect("raw Super capture owns its activation cell");
                    self.write_binding_from_locals(storage, value, function);
                }
                raw.receiver.clear(function);
                raw.base.clear(function);
                raw.referenced_name.clear(function);
                output.set_undefined(function);
            }
        }
        Ok(())
    }

    fn compile_captured_super_property_put(
        &mut self,
        capture: &lila_ir::SuperPropertyReferenceCaptureIr,
        value: &TypedExpr,
        strictness: Strictness,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let base = schema.reserve_value_local(function);
        let receiver = schema.reserve_value_local(function);
        let referenced_name = schema.reserve_value_local(function);
        for (name, value) in [
            (capture.receiver_storage_name(), &receiver),
            (capture.base_storage_name(), &base),
            (capture.referenced_name_storage_name(), &referenced_name),
        ] {
            let storage = self
                .lookup_binding(name)
                .expect("captured Super Put owns its activation cell");
            self.read_binding_to_locals(storage, value, function)?;
        }
        let rhs = schema.reserve_value_local(function);
        self.compile_expr_to_value(value, &rhs, function)?;
        // Plain assignment first finishes its RHS. A preceding Get has already
        // stored a String/Symbol name, so this fast path cannot re-run coercion.
        let reference = self.canonicalize_super_property_reference(
            EvaluatedRawSuperPropertyReferenceLocals {
                base,
                receiver,
                referenced_name,
            },
            function,
        )?;
        self.emit_put_value_from_coerced_super_property_reference(
            reference, &rhs, strictness, function,
        )?;
        output.copy_from(&rhs, function);
        rhs.clear(function);
        Ok(())
    }
}
