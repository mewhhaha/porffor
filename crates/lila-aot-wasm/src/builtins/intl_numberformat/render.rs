use super::numeric_input::NfNumericLocals;
use super::provider_wire::{NfOperation, NfResponseReader, NfWireField};
use super::*;

enum NfInputs<'a> {
    Scalar(&'a NfNumericLocals),
    Range {
        start: &'a NfNumericLocals,
        end: &'a NfNumericLocals,
    },
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_nf_result_object(
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

    pub(super) fn emit_nf_output_array_entry(
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

    fn emit_nf_provider_format(
        &mut self,
        record: u32,
        inputs: NfInputs<'_>,
        mode: NfFormatMode,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
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
        let approximately = self.reserve_temp_local();
        let (operation, fields, range) = match inputs {
            NfInputs::Scalar(input) => (
                NfOperation::ScalarParts,
                vec![
                    NfWireField::Configuration(record),
                    NfWireField::Numeric(input),
                ],
                false,
            ),
            NfInputs::Range { start, end } => (
                NfOperation::RangeParts,
                vec![
                    NfWireField::Configuration(record),
                    NfWireField::Numeric(start),
                    NfWireField::Numeric(end),
                ],
                true,
            ),
        };
        self.emit_nf_provider_request(operation, &fields, request, function)?;
        self.emit_nf_provider_call(operation, request, response, function)?;
        let reader = NfResponseReader::new(self, response, function);
        reader.word(self, count, function);
        reader.require_records(count, if range { 24 } else { 16 }, function);
        match mode {
            NfFormatMode::String => self.emit_nf_set_string(destination, "", function),
            NfFormatMode::Parts => self
                .emit_alloc_array_payload_with_length_in_current_function_realm(
                    count,
                    destination,
                    function,
                )?,
        }
        self.emit_nf_set_const(string_tag, ValueKind::String.tag() as i64, function);
        self.emit_nf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.word(self, kind, function);
        self.emit_nf_set_const(part_type, 0, function);
        self.emit_nf_set_const(approximately, 0, function);
        if range {
            self.emit_nf_if_eq(kind, lila_intl::NUMBER_APPROXIMATELY_SIGN_CODE, function);
            self.emit_nf_set_string(part_type, "approximatelySign", function);
            self.emit_nf_set_const(approximately, 1, function);
            function.instruction(&Instruction::End);
        }
        for part in NumberPartKind::ALL {
            self.emit_nf_if_eq(kind, part.wire_code(), function);
            self.emit_nf_set_string(part_type, part.name(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(part_type));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        if range {
            reader.word(self, kind, function);
            self.emit_nf_if_nonzero(approximately, function);
            function.instruction(&Instruction::LocalGet(kind));
            function.instruction(&Instruction::I64Const(
                RangePartSource::Shared.wire_code() as i64
            ));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.emit_nf_set_const(source, 0, function);
            for attribution in RangePartSource::ALL {
                self.emit_nf_if_eq(kind, attribution.wire_code(), function);
                self.emit_nf_set_string(source, attribution.name(), function);
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
            NfFormatMode::String => {
                self.emit_concat_string_payloads_local(destination, value, function)?;
                function.instruction(&Instruction::LocalSet(destination));
            }
            NfFormatMode::Parts => {
                self.emit_nf_result_object(function)?;
                function.instruction(&Instruction::LocalSet(object));
                for (name, payload) in [("type", part_type), ("value", value)]
                    .into_iter()
                    .chain(range.then_some(("source", source)))
                {
                    self.emit_nf_set_string(key, name, function);
                    self.emit_object_append_data_property_with_flags(
                        object, key, payload, string_tag, true, true, true, function,
                    )?;
                }
                self.emit_nf_output_array_entry(
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
            approximately,
            string_tag,
            key,
            object,
            source,
            value,
            part_type,
            kind,
            index,
            count,
            response,
            request,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_number_format_bound_format(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let input = NfNumericLocals::reserve(self);
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
        self.emit_builtin_arg_to_locals(0, value.payload, value.tag, function);
        self.emit_nf_observe_numeric(value, &input, function)?;
        self.emit_nf_provider_format(
            record,
            NfInputs::Scalar(&input),
            NfFormatMode::String,
            self.result_local,
            function,
        )?;
        self.emit_nf_set_const(
            self.result_tag_local,
            ValueKind::String.tag() as i64,
            function,
        );
        input.release(self);
        for local in [value.tag, value.payload, record, object] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(crate) fn emit_intl_number_format_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let input = NfNumericLocals::reserve(self);
        self.emit_nf_record_from_receiver(record, function)?;
        self.emit_builtin_arg_to_locals(0, value.payload, value.tag, function);
        self.emit_nf_observe_numeric(value, &input, function)?;
        self.emit_nf_provider_format(
            record,
            NfInputs::Scalar(&input),
            NfFormatMode::Parts,
            self.result_local,
            function,
        )?;
        self.emit_nf_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );
        input.release(self);
        for local in [value.tag, value.payload, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(crate) fn emit_intl_number_format_range(
        &mut self,
        mode: NfFormatMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let left = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let right = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let start = NfNumericLocals::reserve(self);
        let end = NfNumericLocals::reserve(self);
        self.emit_nf_record_from_receiver(record, function)?;
        self.emit_builtin_arg_to_locals(0, left.payload, left.tag, function);
        self.emit_builtin_arg_to_locals(1, right.payload, right.tag, function);
        for tag in [left.tag, right.tag] {
            self.emit_nf_if_eq(tag, ValueKind::Undefined.tag() as u64, function);
            self.emit_nf_type_error(NF_RANGE_UNDEFINED, function)?;
            function.instruction(&Instruction::End);
        }
        self.emit_nf_observe_numeric(left, &start, function)?;
        self.emit_nf_observe_numeric(right, &end, function)?;
        self.emit_nf_provider_format(
            record,
            NfInputs::Range {
                start: &start,
                end: &end,
            },
            mode,
            self.result_local,
            function,
        )?;
        self.emit_nf_set_const(
            self.result_tag_local,
            match mode {
                NfFormatMode::String => ValueKind::String.tag(),
                NfFormatMode::Parts => ValueKind::Array.tag(),
            } as i64,
            function,
        );
        end.release(self);
        start.release(self);
        for local in [right.tag, right.payload, left.tag, left.payload, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }
    pub(crate) fn emit_intl_number_format_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let bound = self.reserve_temp_local();
        self.emit_nf_record_from_receiver(record, function)?;
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_NF_BOUND_FORMAT_OFFSET,
            bound,
            function,
        );
        function.instruction(&Instruction::LocalGet(bound));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlNumberFormatBoundFormat.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing NumberFormat bound format dependency")
            })?;
        self.emit_current_builtin_realm_closure_value(
            &meta,
            self.this_payload_local
                .expect("checked NumberFormat receiver"),
            bound,
            function,
        )?;
        self.store_i64_local_at_offset(record, HEAP_INTL_NF_BOUND_FORMAT_OFFSET, bound, function);
        function.instruction(&Instruction::End);
        self.emit_nf_copy(bound, self.result_local, function);
        self.emit_nf_set_const(
            self.result_tag_local,
            ValueKind::Function.tag() as i64,
            function,
        );
        self.release_temp_local(bound);
        self.release_temp_local(record);
        Ok(())
    }
}
