use super::*;
use crate::gc_types::*;

impl FunctionBuilder<'_> {
    fn suspended_property_reference_frame(
        &self,
        function: &mut Function,
    ) -> Result<GcLocal<InvocationFrame>, EmitError> {
        let schema = self.runtime_schema();
        let entry = self.body_entry_locals().ok_or_else(|| {
            EmitError::unsupported("compiler invariant: suspended Reference has no body entry")
        })?;
        match entry.resume_activation() {
            Some(
                crate::function_entry::ResumableEntryLocals::Generator(_)
                | crate::function_entry::ResumableEntryLocals::AsyncGenerator(_),
            ) => {}
            Some(crate::function_entry::ResumableEntryLocals::Async(_)) | None => {
                return Err(EmitError::unsupported(
                    "compiler invariant: suspended Reference requires a generator activation",
                ));
            }
        }
        let frame = entry.resume_frame().ok_or_else(|| {
            EmitError::unsupported(
                "compiler invariant: suspended Reference has no invocation frame",
            )
        })?;
        Ok(schema
            .reserve_gc_local(function)
            .initialize(frame.load(schema, function), function))
    }

    pub(crate) fn clear_suspended_property_reference(
        &self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let frame = self.suspended_property_reference_frame(function)?;
        let empty = schema.reserve_value_local(function);
        empty.set_scalar(ScalarValue::Undefined, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&empty, function),
            function,
        );
        let row = schema.struct_type::<InvocationFrame>();
        row.field(InvocationFrameSchema::ASSIGNMENT_TARGET).write(
            &frame,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        row.field(InvocationFrameSchema::ASSIGNMENT_KEY).write(
            &frame,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        stored.clear(function);
        empty.clear(function);
        frame.clear(function);
        Ok(())
    }

    /// Reference evaluation retains the raw key before the RHS suspends.
    /// ToPropertyKey belongs to the eventual PutValue after resumption.
    pub(crate) fn prepare_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let frame = self.suspended_property_reference_frame(function)?;
        let base = schema.reserve_value_local(function);
        let raw_key = schema.reserve_value_local(function);
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver,
                key,
                strictness: _,
            } => {
                self.compile_expr_to_value(base_and_receiver, &base, function)?;
                self.emit_propagate_current_throw_if_needed(function);
                match key {
                    PropertyKeyIr::StaticString(name) => {
                        let key = schema.reserve_gc_local(function).initialize(
                            self.emit_interned_string_reference(name, function)?,
                            function,
                        );
                        raw_key.set_reference(&key, schema, function);
                        key.clear(function);
                    }
                    PropertyKeyIr::ArrayLength => {
                        let key = schema.reserve_gc_local(function).initialize(
                            self.emit_interned_string_reference("length", function)?,
                            function,
                        );
                        raw_key.set_reference(&key, schema, function);
                        key.clear(function);
                    }
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        self.compile_expr_to_value(expr, &raw_key, function)?;
                        self.emit_propagate_current_throw_if_needed(function);
                    }
                }
            }
        }
        let stored_base = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&base, function),
            function,
        );
        let stored_key = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&raw_key, function),
            function,
        );
        let row = schema.struct_type::<InvocationFrame>();
        row.field(InvocationFrameSchema::ASSIGNMENT_TARGET).write(
            &frame,
            GcOperand::reference(&stored_base, schema),
            schema,
            function,
        );
        row.field(InvocationFrameSchema::ASSIGNMENT_KEY).write(
            &frame,
            GcOperand::reference(&stored_key, schema),
            schema,
            function,
        );
        stored_key.clear(function);
        stored_base.clear(function);
        raw_key.clear(function);
        base.clear(function);
        frame.clear(function);
        Ok(())
    }

    pub(crate) fn write_suspended_property_reference(
        &mut self,
        reference: &SuspendedPropertyReferenceIr,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let frame = self.suspended_property_reference_frame(function)?;
        let base = schema.reserve_value_local(function);
        let raw_key = schema.reserve_value_local(function);
        let rhs = schema.reserve_value_local(function);
        rhs.copy_from(value, function);
        let row = schema.struct_type::<InvocationFrame>();
        let stored_base = schema.reserve_gc_local(function).initialize(
            row.field(InvocationFrameSchema::ASSIGNMENT_TARGET)
                .read(&frame, schema, function)
                .reference(),
            function,
        );
        let stored_key = schema.reserve_gc_local(function).initialize(
            row.field(InvocationFrameSchema::ASSIGNMENT_KEY)
                .read(&frame, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_base, &base, schema, function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_key, &raw_key, schema, function);
        stored_key.clear(function);
        stored_base.clear(function);
        self.clear_suspended_property_reference(function)?;
        // PutValue rejects a nullish base before converting the retained key.
        self.compile_nullish_tagged_i32(base.tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let rejected = schema.reserve_completion(function);
        self.emit_throw_runtime_error(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::CANNOT_CONVERT_UNDEFINED_OR_NULL_TO_OBJECT,
            &rejected,
            function,
        )?;
        self.completion().copy_from(&rejected, function);
        rejected.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let key = self.emit_value_to_property_key_locals(&raw_key, function)?;
        let strict = schema.reserve_i32_local(function);
        let SuspendedPropertyReferenceUse::Ordinary { strictness, .. } = reference.use_view();
        function.instruction(&Instruction::I32Const(i32::from(
            strictness.throws_on_failed_set(),
        )));
        strict.store(function);
        let result = schema.reserve_completion(function);
        self.emit_object_write(&base, &key, &rhs, strict, &result, function)?;
        self.completion().copy_from(&result, function);
        result.clear(function);
        schema.release_i32_local(strict, function);
        key.clear(function);
        rhs.clear(function);
        raw_key.clear(function);
        base.clear(function);
        frame.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
}
