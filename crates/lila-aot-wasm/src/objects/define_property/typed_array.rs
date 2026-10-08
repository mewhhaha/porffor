//! Integer-indexed exotic property methods; ordinary/Array descriptor owners stay separate.
use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::objects) fn emit_typed_array_set_if_handled(
        &mut self,
        target: &ValueLocals,
        receiver: &ValueLocals,
        key: &PropertyKeyLocals,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let handled = schema.reserve_i32_local(function);
        let valid = schema.reserve_i32_local(function);
        let boolean = schema.reserve_i32_local(function);
        self.emit_typed_array_canonical_numeric_index_i32(target, key, number, handled, function)?;
        handled.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let array = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<crate::gc_types::TypedArrayObject>(schema, function),
            function,
        );
        // Immutable rejection precedes receiver equality, index validity and
        // value conversion, including canonical numeric misses.
        self.emit_typed_array_immutable_buffer_i32(&array, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        boolean.store(function);
        self.emit_object_boolean_result(boolean, result, function);
        function.instruction(&Instruction::Else);
        self.emit_tagged_payload_same_value_i32(target, receiver, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_integer_index_or_invalid(number, index, function);
        self.emit_typed_array_element_write_from_locals(&array, index, value, result, function)?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(1));
        boolean.store(function);
        self.emit_object_boolean_result(boolean, result, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_typed_array_valid_integer_index_i32(&array, number, index, valid, function)?;
        valid.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(1));
        boolean.store(function);
        self.emit_object_boolean_result(boolean, result, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        array.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        schema.release_i32_local(boolean, function);
        schema.release_i32_local(valid, function);
        schema.release_i32_local(handled, function);
        schema.release_i64_local(index, function);
        schema.release_i64_local(number, function);
        Ok(())
    }

    pub(in crate::objects) fn emit_typed_array_own_descriptor(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &GcLocal<PropertyDescriptor, Nullable>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let handled = schema.reserve_i32_local(function);
        let valid = schema.reserve_i32_local(function);
        self.emit_typed_array_canonical_numeric_index_i32(target, key, number, handled, function)?;
        handled.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let array = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<crate::gc_types::TypedArrayObject>(schema, function),
            function,
        );
        self.emit_typed_array_valid_integer_index_i32(&array, number, index, valid, function)?;
        valid.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let value = schema.reserve_value_local(function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        self.emit_typed_array_element_read_from_locals(&array, index, &value, function)?;
        self.emit_typed_array_immutable_buffer_i32(&array, function);
        self.open_frame(ControlFrameKind::If, function);
        let descriptor = self.emit_alloc_property_descriptor(
            StoredPropertyAttributes::Data {
                writable: false,
                enumerable: true,
                configurable: false,
            },
            &value,
            &undefined,
            &undefined,
            function,
        );
        result.replace(descriptor.load(schema, function).nullable(), function);
        descriptor.clear(function);
        function.instruction(&Instruction::Else);
        let descriptor = self.emit_alloc_property_descriptor(
            StoredPropertyAttributes::Data {
                writable: true,
                enumerable: true,
                configurable: true,
            },
            &value,
            &undefined,
            &undefined,
            function,
        );
        result.replace(descriptor.load(schema, function).nullable(), function);
        descriptor.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        undefined.clear(function);
        value.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        array.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(valid, function);
        schema.release_i32_local(handled, function);
        schema.release_i64_local(index, function);
        schema.release_i64_local(number, function);
        Ok(())
    }

    pub(in crate::objects) fn emit_typed_array_define_own_property(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        incoming: &WasmDescriptor<'_>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let handled = schema.reserve_i32_local(function);
        let valid = schema.reserve_i32_local(function);
        self.emit_typed_array_canonical_numeric_index_i32(target, key, number, handled, function)?;
        handled.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let array = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<crate::gc_types::TypedArrayObject>(schema, function),
            function,
        );
        self.emit_typed_array_valid_integer_index_i32(&array, number, index, valid, function)?;
        valid.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_typed_array_immutable_buffer_i32(&array, function);
        self.open_frame(ControlFrameKind::If, function);
        // Existing descriptor compatibility uses SameValue on the uncoerced
        // incoming value; immutable elements are never rewritten.
        let current = schema
            .reserve_gc_local::<PropertyDescriptor, Nullable>(function)
            .initialize_null(schema, function);
        self.emit_typed_array_own_descriptor(target, key, &current, function)?;
        let record = schema.reserve_gc_local(function).initialize(
            current.load(schema, function).require_non_null(function),
            function,
        );
        self.emit_validate_stored_descriptor(&record, incoming, valid, function)?;
        self.emit_object_boolean_result(valid, result, function);
        record.clear(function);
        current.clear(function);
        function.instruction(&Instruction::Else);
        let descriptor = incoming.as_partial();
        emit_terms(classify(incoming).terms(DescriptorSide::Accessor), function);
        self.emit_descriptor_reject_if_true(valid, function);
        for presence in [
            &descriptor.configurable,
            &descriptor.enumerable,
            &descriptor.writable,
        ] {
            if let Some(flag) = presence.value() {
                emit_presence(presence, function);
                emit_flag(*flag, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32And);
                self.emit_descriptor_reject_if_true(valid, function);
            }
        }
        self.emit_object_boolean_result(valid, result, function);
        valid.load(function);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(value) = descriptor.value.value() {
            emit_presence(&descriptor.value, function);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_typed_array_element_write_from_locals(
                &array, index, value, result, function,
            )?;
            result.kind().load(function);
            function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_object_boolean_result(valid, result, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        // Invalid indices reject before inspecting immutable state or casting
        // the descriptor value to the element's numeric content type.
        self.emit_object_boolean_result(valid, result, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        array.clear(function);
        function.instruction(&Instruction::Else);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_object_header_projection(target, function),
            function,
        );
        self.emit_ordinary_define_own_property(&header, key, incoming, result, function)?;
        header.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(valid, function);
        schema.release_i32_local(handled, function);
        schema.release_i64_local(index, function);
        schema.release_i64_local(number, function);
        Ok(())
    }
}
