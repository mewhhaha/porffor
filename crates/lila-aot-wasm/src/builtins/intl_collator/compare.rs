use super::*;
/// Both ordered ToString conversions have completed before comparison can emit.
pub(super) struct CompletedCollatorCompareStringsLocals {
    left: GcLocal<StringValue>,
    right: GcLocal<StringValue>,
}
impl CompletedCollatorCompareStringsLocals {
    pub(super) fn left(&self) -> &GcLocal<StringValue> {
        &self.left
    }
    pub(super) fn right(&self) -> &GcLocal<StringValue> {
        &self.right
    }
    fn clear(self, function: &mut Function) {
        self.right.clear(function);
        self.left.clear(function);
    }
}
impl FunctionBuilder<'_> {
    fn emit_completed_collator_strings(
        &mut self,
        left: &ValueLocals,
        right: &ValueLocals,
        function: &mut Function,
    ) -> Result<CompletedCollatorCompareStringsLocals, EmitError> {
        let left = self.emit_intl_number_to_string(left, function)?;
        let right = self.emit_intl_number_to_string(right, function)?;
        Ok(CompletedCollatorCompareStringsLocals { left, right })
    }
    fn emit_collator_compare_strings(
        &mut self,
        record: &GcLocal<IntlCollatorObject>,
        input: &CompletedCollatorCompareStringsLocals,
        out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let response = self.emit_collator_provider_call(
            CollatorProviderRequest::Compare { record, input },
            function,
        )?;
        let schema = self.runtime_schema();
        let reader = response.reader(schema, function);
        let ordering = schema.reserve_i64_local(function);
        let recognized = schema.reserve_i32_local(function);
        reader.read_u64(ordering, schema, function);
        set_i32(recognized, 0, function);
        for variant in CollatorOrdering::ALL {
            ordering.load(function);
            function.instruction(&Instruction::I64Const(variant.wire_code() as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            let value: f64 = match variant {
                CollatorOrdering::Equal => 0.0,
                CollatorOrdering::Less => -1.0,
                CollatorOrdering::Greater => 1.0,
            };
            out.set_scalar(ScalarValue::NumberBits(value.to_bits() as i64), function);
            set_i32(recognized, 1, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        reader.finish(schema, function);
        schema.release_i32_local(recognized, function);
        schema.release_i64_local(ordering, function);
        response.clear(function);
        Ok(())
    }
    pub(crate) fn emit_intl_collator_compare_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_collator_record_from_receiver(function)?;
        let schema = self.runtime_schema();
        let bound = schema
            .reserve_gc_local::<FunctionObject, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<IntlCollatorObject>()
                    .field(IntlCollatorObjectSchema::BOUND_COMPARE)
                    .read(&record, schema, function)
                    .reference(),
                function,
            );
        bound.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlCollatorBoundCompare.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Collator bound-compare dependency"))?;
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<BuiltinClosureCapture>()
                    .publish(
                        BuiltinClosurePayload::IntlCollator(&record),
                        schema,
                        function,
                    )
                    .nullable(),
                function,
            );
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_current_builtin_realm_closure_value(&meta, &capture, function)?,
            function,
        );
        schema
            .struct_type::<IntlCollatorObject>()
            .field(IntlCollatorObjectSchema::BOUND_COMPARE)
            .write(
                &record,
                GcOperand::nullable_reference(&callable, schema),
                schema,
                function,
            );
        bound.replace(callable.load(schema, function).nullable(), function);
        callable.clear(function);
        capture.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let callable = schema.reserve_gc_local(function).initialize(
            bound.load(schema, function).require_non_null(function),
            function,
        );
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&callable, schema, function);
        callable.clear(function);
        bound.clear(function);
        record.clear(function);
        Ok(())
    }
    pub(crate) fn emit_intl_collator_bound_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let context = self
            .body_entry_locals()
            .and_then(|entry| entry.function_context())
            .ok_or_else(|| {
                EmitError::unsupported("Collator closure lacks actual function context")
            })?;
        let capture = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(context, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::COLLATOR)
                .read(&capture, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let left = schema.reserve_value_local(function);
        let right = schema.reserve_value_local(function);
        let out = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &left, function);
        self.emit_builtin_arg_to_value(1, &right, function);
        let strings = self.emit_completed_collator_strings(&left, &right, function)?;
        self.emit_collator_compare_strings(&record, &strings, &out, function)?;
        self.completion().initialize(function);
        self.completion().value().copy_from(&out, function);
        strings.clear(function);
        out.clear(function);
        right.clear(function);
        left.clear(function);
        record.clear(function);
        capture.clear(function);
        Ok(())
    }
    /// String owns RequireObjectCoercible and the first ToString. This producer
    /// retains that String while the comparison operand and Intl options run.
    pub(crate) fn emit_intrinsic_string_locale_compare(
        &mut self,
        receiver: &GcLocal<StringValue>,
        comparison: &ValueLocals,
        locales: &ValueLocals,
        options: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let left = schema
            .reserve_gc_local(function)
            .initialize(receiver.load(schema, function), function);
        let finish = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_string_payload(comparison, result, function)?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(finish, function);
        let right = schema.reserve_gc_local(function).initialize(
            result
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        let strings = CompletedCollatorCompareStringsLocals { left, right };
        let constructor = self
            .functions
            .get(&StandardBuiltinId::IntlCollatorConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing immutable Collator constructor dependency")
            })?;
        self.emit_direct_js_call(&constructor, None, &[locales, options], result, function)?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(finish, function);
        let record = schema.reserve_gc_local(function).initialize(
            result
                .value()
                .cast_reference::<IntlCollatorObject>(schema, function),
            function,
        );
        self.emit_collator_compare_strings(&record, &strings, result.value(), function)?;
        result.set_kind(CompletionKind::Normal, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        record.clear(function);
        strings.clear(function);
        Ok(())
    }
}
