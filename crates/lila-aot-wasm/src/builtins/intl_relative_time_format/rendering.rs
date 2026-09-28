use super::provider_wire::{RelativeTimeOperation, RelativeTimeResponseReader};
use super::*;

const RELATIVE_TIME_NUMBER_NEGATIVE_ZERO: i64 =
    RelativeTimeNumberKind::NegativeZero.wire_code() as i64;
const RELATIVE_TIME_NUMBER_SHORTEST_DECIMAL: i64 =
    RelativeTimeNumberKind::ShortestDecimal.wire_code() as i64;
const RELATIVE_TIME_PART_LITERAL: i64 = RelativeTimePartKind::Literal.wire_code() as i64;

pub(super) struct RelativeTimeCallLocals {
    pub(super) record: u32,
    pub(super) response: u32,
}

impl FunctionBuilder<'_> {
    fn emit_rtf_number_to_string(
        &mut self,
        number: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_rtf_if_eq(number, (-0.0f64).to_bits() as i64, function);
        self.emit_rtf_set_string(destination, "-0", function);
        function.instruction(&Instruction::Else);
        self.emit_number_to_string_payload(number, function)?;
        function.instruction(&Instruction::LocalSet(destination));
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_rtf_validate_unit(
        &mut self,
        text: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected = self.reserve_temp_local();
        self.emit_rtf_set_const(destination, 0, function);
        for unit in RelativeTimeUnit::ALL {
            let plural = format!("{}s", unit.name());
            for spelling in [unit.name(), plural.as_str()] {
                self.emit_rtf_set_string(expected, spelling, function);
                self.emit_string_payload_equality_i32(text, expected, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_rtf_set_const(destination, unit.wire_code() as i64, function);
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_rtf_range_error(RTF_INVALID_UNIT, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(expected);
        Ok(())
    }

    fn emit_rtf_prepare_format_call(
        &mut self,
        function: &mut Function,
    ) -> Result<RelativeTimeCallLocals, EmitError> {
        let record = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let receiver = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let primitive = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let number = self.reserve_temp_local();
        let unit_input = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let unit_text = self.reserve_temp_local();
        let unit_code = self.reserve_temp_local();
        let number_kind = self.reserve_temp_local();
        let number_text = self.reserve_temp_local();
        let request = self.reserve_temp_local();

        self.emit_rtf_record_from_receiver(receiver, record, function)?;
        self.emit_builtin_arg_to_locals(0, value.payload, value.tag, function);

        // ToNumber(value) happens before ToString(unit), preserving the
        // observable order of user conversion hooks and exceptions.
        let observed = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::Number,
            value.payload,
            value.tag,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_tagged_locals(
            observed,
            primitive.payload,
            primitive.tag,
            function,
        );
        self.emit_primitive_to_number_payload(primitive.tag, primitive.payload, function)?;
        function.instruction(&Instruction::LocalSet(primitive.payload));
        self.emit_return_current_completion_if_throw(function);
        self.emit_rtf_copy(primitive.payload, number, function);

        self.emit_builtin_arg_to_locals(1, unit_input.payload, unit_input.tag, function);
        self.emit_rtf_to_string(unit_input, unit_text, function)?;

        // PartitionRelativeTimePattern rejects a non-finite number after unit
        // coercion, before validating and singularizing the unit string.
        function.instruction(&Instruction::LocalGet(number));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_rtf_range_error(RTF_NON_FINITE, function)?;
        function.instruction(&Instruction::End);
        self.emit_rtf_validate_unit(unit_text, unit_code, function)?;

        self.emit_rtf_number_to_string(number, number_text, function)?;
        self.emit_rtf_set_const(number_kind, RELATIVE_TIME_NUMBER_SHORTEST_DECIMAL, function);
        self.emit_rtf_if_eq(number, (-0.0f64).to_bits() as i64, function);
        self.emit_rtf_set_const(number_kind, RELATIVE_TIME_NUMBER_NEGATIVE_ZERO, function);
        self.emit_rtf_set_string(number_text, "", function);
        function.instruction(&Instruction::End);

        self.emit_rtf_format_request(
            record,
            unit_code,
            number_kind,
            number_text,
            request,
            function,
        )?;
        self.emit_rtf_provider_call(RelativeTimeOperation::Format, request, response, function)?;

        for local in [
            request,
            number_text,
            number_kind,
            unit_code,
            unit_text,
            unit_input.tag,
            unit_input.payload,
            number,
            primitive.tag,
            primitive.payload,
            value.tag,
            value.payload,
            receiver.tag,
            receiver.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(RelativeTimeCallLocals { record, response })
    }

    fn emit_rtf_part_type(&mut self, part_kind: u32, destination: u32, function: &mut Function) {
        self.emit_rtf_set_const(destination, 0, function);
        for kind in RelativeTimePartKind::ALL {
            self.emit_rtf_if_eq(part_kind, kind.wire_code() as i64, function);
            self.emit_rtf_set_string(destination, kind.name(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_rtf_unit_name(&mut self, unit: u32, destination: u32, function: &mut Function) {
        self.emit_rtf_set_const(destination, 0, function);
        for unit_kind in RelativeTimeUnit::ALL {
            self.emit_rtf_if_eq(unit, unit_kind.wire_code() as i64, function);
            self.emit_rtf_set_string(destination, unit_kind.name(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_rtf_part_property(
        &mut self,
        object: u32,
        name: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_rtf_set_string(key, name, function);
        self.emit_object_append_data_property_with_flags(
            object,
            key,
            value.payload,
            value.tag,
            true,
            true,
            true,
            function,
        )?;
        self.release_temp_local(key);
        Ok(())
    }

    pub(crate) fn emit_intl_relative_time_format_format(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let call = self.emit_rtf_prepare_format_call(function)?;
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let part_kind = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let unit = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let reader = RelativeTimeResponseReader::new(
            self,
            call.response,
            RelativeTimeOperation::Format,
            function,
        );
        reader.word(self, count, function);
        reader.require_records(count, 24, function);
        self.emit_rtf_set_string(output, "", function);
        self.emit_rtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.word(self, part_kind, function);
        reader.bytes(self, value, function);
        reader.word(self, unit, function);
        self.emit_concat_string_payloads_local(output, value, function)?;
        function.instruction(&Instruction::LocalSet(output));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_rtf_set_const(
            self.result_tag_local,
            ValueKind::String.tag() as i64,
            function,
        );
        for local in [
            output,
            unit,
            value,
            part_kind,
            index,
            count,
            call.response,
            call.record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_relative_time_format_format_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let call = self.emit_rtf_prepare_format_call(function)?;
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let part_kind = self.reserve_temp_local();
        let part_value = self.reserve_temp_local();
        let part_unit = self.reserve_temp_local();
        let array = self.reserve_temp_local();
        let object = self.reserve_temp_local();
        let object_tag = self.reserve_temp_local();
        let type_value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let unit = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let reader = RelativeTimeResponseReader::new(
            self,
            call.response,
            RelativeTimeOperation::Format,
            function,
        );

        reader.word(self, count, function);
        reader.require_records(count, 24, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, array, function,
        )?;
        self.emit_rtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.word(self, part_kind, function);
        reader.bytes(self, part_value, function);
        reader.word(self, part_unit, function);
        let part_object = self.emit_rtf_result_object(function)?;
        function.instruction(&Instruction::LocalGet(part_object));
        function.instruction(&Instruction::LocalSet(object));
        self.release_temp_local(part_object);

        self.emit_rtf_part_type(part_kind, type_value.payload, function);
        self.emit_rtf_set_const(type_value.tag, ValueKind::String.tag() as i64, function);
        self.emit_rtf_part_property(object, "type", type_value, function)?;
        self.emit_rtf_copy(part_value, value.payload, function);
        self.emit_rtf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_rtf_part_property(object, "value", value, function)?;
        self.emit_rtf_if_eq(part_kind, RELATIVE_TIME_PART_LITERAL, function);
        function.instruction(&Instruction::Else);
        self.emit_rtf_unit_name(part_unit, unit.payload, function);
        self.emit_rtf_set_const(unit.tag, ValueKind::String.tag() as i64, function);
        self.emit_rtf_part_property(object, "unit", unit, function)?;
        function.instruction(&Instruction::End);
        self.emit_rtf_set_const(object_tag, ValueKind::Object.tag() as i64, function);
        self.emit_array_write(array, index, object, object_tag, function)?;
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        function.instruction(&Instruction::LocalGet(array));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_rtf_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );
        for local in [
            unit.tag,
            unit.payload,
            value.tag,
            value.payload,
            type_value.tag,
            type_value.payload,
            object_tag,
            object,
            array,
            part_unit,
            part_value,
            part_kind,
            index,
            count,
            call.response,
            call.record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
