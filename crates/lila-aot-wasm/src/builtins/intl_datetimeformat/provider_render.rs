use super::provider_wire::{DtfResponseReader, DtfWireField, DtfWireWord};
use super::*;
use lila_intl::{DateTimePartKind, DateTimeRangeSource, IntlHostOp};

pub(super) enum DtfInputRecords {
    Single(u32),
    Range { start: u32, end: u32 },
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_result_object(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype = self.reserve_temp_local();
        let realm = self.reserve_temp_local();
        let intrinsics = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm,
            function,
        );
        for (source, offset, destination) in [
            (realm, HEAP_REALM_INTRINSICS_OFFSET, intrinsics),
            (
                intrinsics,
                HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
                prototype,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(source, offset, destination, function);
        }
        function.instruction(&Instruction::LocalGet(prototype));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype), None, function)?;
        for local in [intrinsics, realm, prototype] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_dtf_output_array_entry(
        &mut self,
        array: u32,
        index: u32,
        payload: u32,
        kind: ValueKind,
        function: &mut Function,
    ) {
        let entry = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array, HEAP_PTR_OFFSET, entry, function);
        function.instruction(&Instruction::LocalGet(entry));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry));
        self.store_i64_const_at_offset(entry, HEAP_ARRAY_TAG_OFFSET, kind.tag() as u64, function);
        self.store_i64_local_at_offset(entry, HEAP_ARRAY_PAYLOAD_OFFSET, payload, function);
        self.store_i64_const_at_offset(
            entry,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        self.release_temp_local(entry);
    }

    fn emit_dtf_provider_format(
        &mut self,
        record: u32,
        inputs: DtfInputRecords,
        mode: DtfFormatMode,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let plan = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let part_type = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let source = self.reserve_temp_local();
        let object = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        let string_tag = self.reserve_temp_local();
        let (operation, mut fields, range) = match inputs {
            DtfInputRecords::Single(input) => (
                IntlHostOp::FormatDateTimeParts,
                vec![DtfWireField::InputRecord(input)],
                false,
            ),
            DtfInputRecords::Range { start, end } => (
                IntlHostOp::FormatDateTimeRangeParts,
                vec![
                    DtfWireField::InputRecord(start),
                    DtfWireField::InputRecord(end),
                ],
                true,
            ),
        };
        self.load_i64_to_local_from_offset(record, HEAP_INTL_DTF_PLAN_OFFSET, plan, function);
        fields.push(DtfWireField::Bytes(plan));
        self.emit_dtf_provider_request(operation, &fields, request, function)?;
        self.emit_dtf_provider_call(operation, request, response, function)?;
        let reader = DtfResponseReader::new(self, response, function);
        reader.word(self, count, function);
        reader.require_records(count, if range { 24 } else { 16 }, function);
        match mode {
            DtfFormatMode::String => self.emit_dtf_set_string(destination, "", function),
            DtfFormatMode::Parts => self
                .emit_alloc_array_payload_with_length_in_current_function_realm(
                    count,
                    destination,
                    function,
                )?,
        }
        self.emit_dtf_set_const(string_tag, ValueKind::String.tag() as i64, function);
        self.emit_dtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.word(self, kind, function);
        self.emit_dtf_set_const(part_type, 0, function);
        for part in DateTimePartKind::ALL {
            self.emit_dtf_if_code_eq(kind, part.wire_code() as i64, function);
            self.emit_dtf_set_string(part_type, part.as_str(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(part_type));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        if range {
            reader.word(self, kind, function);
            self.emit_dtf_set_const(source, 0, function);
            for attribution in DateTimeRangeSource::ALL {
                self.emit_dtf_if_code_eq(kind, attribution.wire_code() as i64, function);
                self.emit_dtf_set_string(source, attribution.as_str(), function);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        reader.bytes(self, value, function);
        match mode {
            DtfFormatMode::String => {
                self.emit_concat_string_payloads_local(destination, value, function)?;
                function.instruction(&Instruction::LocalSet(destination));
            }
            DtfFormatMode::Parts => {
                self.emit_dtf_result_object(function)?;
                function.instruction(&Instruction::LocalSet(object));
                for (name, payload) in [("type", part_type), ("value", value)]
                    .into_iter()
                    .chain(range.then_some(("source", source)))
                {
                    self.emit_dtf_set_string(key, name, function);
                    self.emit_object_append_data_property_with_flags(
                        object, key, payload, string_tag, true, true, true, function,
                    )?;
                }
                self.emit_dtf_output_array_entry(
                    destination,
                    index,
                    object,
                    ValueKind::Object,
                    function,
                );
            }
        }
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        for local in [
            string_tag, key, object, source, value, part_type, kind, index, count, response,
            request, plan,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_date_time_format_bound_format(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let input = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
            object,
            function,
        );
        self.load_i64_to_local_from_offset(
            object,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        self.emit_dtf_single_input(record, input, function)?;
        self.emit_dtf_provider_format(
            record,
            DtfInputRecords::Single(input),
            DtfFormatMode::String,
            output,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_dtf_set_const(
            self.result_tag_local,
            ValueKind::String.tag() as i64,
            function,
        );
        for local in [output, input, record, object] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_date_time_format_format_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let input = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        self.emit_intl_dtf_record_from_receiver(
            record,
            &IntlDateTimeFormatReceiverOperation::FormatToParts,
            function,
        )?;
        self.emit_dtf_single_input(record, input, function)?;
        self.emit_dtf_provider_format(
            record,
            DtfInputRecords::Single(input),
            DtfFormatMode::Parts,
            output,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_dtf_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );
        for local in [output, input, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_format_range(
        &mut self,
        mode: DtfFormatMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let start = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let (receiver_operation, result_kind) = match mode {
            DtfFormatMode::String => (
                IntlDateTimeFormatReceiverOperation::FormatRange,
                ValueKind::String,
            ),
            DtfFormatMode::Parts => (
                IntlDateTimeFormatReceiverOperation::FormatRangeToParts,
                ValueKind::Array,
            ),
        };
        self.emit_intl_dtf_record_from_receiver(record, &receiver_operation, function)?;
        self.emit_dtf_range_inputs(record, start, end, function)?;
        self.emit_dtf_provider_format(
            record,
            DtfInputRecords::Range { start, end },
            mode,
            output,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_dtf_set_const(self.result_tag_local, result_kind.tag() as i64, function);
        for local in [output, end, start, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_date_time_format_format_range(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_format_range(DtfFormatMode::String, function)
    }
    pub(crate) fn emit_intl_date_time_format_format_range_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_format_range(DtfFormatMode::Parts, function)
    }

    pub(crate) fn emit_intl_date_time_format_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let locales = self.reserve_temp_local();
        let matcher = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, payload, tag, function);
        self.emit_dtf_requested_locales(payload, tag, locales, function)?;
        self.emit_builtin_arg_to_locals(1, payload, tag, function);
        self.emit_dtf_set_const(matcher, 2, function);
        function.instruction(&Instruction::LocalGet(tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            payload, tag, payload, tag, function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_dtf_matcher_option(
            payload,
            tag,
            "localeMatcher",
            lila_intl::DateTimeLocaleMatcher::OPTIONS,
            matcher,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_dtf_provider_request(
            IntlHostOp::SupportedDateTimeLocales,
            &[
                DtfWireField::Word(DtfWireWord::Local(matcher)),
                DtfWireField::CanonicalLocales(locales),
            ],
            request,
            function,
        )?;
        self.emit_dtf_provider_call(
            IntlHostOp::SupportedDateTimeLocales,
            request,
            response,
            function,
        )?;
        let reader = DtfResponseReader::new(self, response, function);
        reader.word(self, count, function);
        reader.require_records(count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_dtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.bytes(self, payload, function);
        self.emit_dtf_output_array_entry(output, index, payload, ValueKind::String, function);
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
        self.emit_dtf_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );
        for local in [
            output, index, count, response, request, matcher, locales, tag, payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
