use super::provider_wire::{
    RelativeTimeLocaleQueryLocals, RelativeTimeOperation, RelativeTimeResponseReader,
};
use super::*;

const RTF_RESULT_RESOLVED: i64 = 1;
const RTF_RESULT_SUPPORTED: i64 = 2;

impl FunctionBuilder<'_> {
    fn emit_rtf_record_from_locale_response(
        &mut self,
        reader: &RelativeTimeResponseReader,
        record: u32,
        expected_style: u32,
        expected_numeric: u32,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [
            HEAP_INTL_RTF_LOCALE_OFFSET,
            HEAP_INTL_RTF_DATA_LOCALE_OFFSET,
            HEAP_INTL_RTF_NUMBERING_SYSTEM_OFFSET,
        ] {
            reader.bytes(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        for (offset, expected) in [
            (HEAP_INTL_RTF_STYLE_OFFSET, expected_style),
            (HEAP_INTL_RTF_NUMERIC_OFFSET, expected_numeric),
        ] {
            reader.word(self, value, function);
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::LocalGet(expected));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        self.release_temp_local(value);
    }

    pub(crate) fn emit_intl_relative_time_format_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reserved = self.emit_reserve_relative_time_format_object(function)?;
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let matcher = self.reserve_temp_local();
        let numbering_system = self.reserve_temp_local();
        let style = self.reserve_temp_local();
        let numeric = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let status = self.reserve_temp_local();
        let record = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_rtf_canonical_locales(locales, requested, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_rtf_options_object(options, function)?;

        self.emit_rtf_choice_option(
            options,
            "localeMatcher",
            &[("lookup", 1), ("best fit", 2)],
            2,
            matcher,
            function,
        )?;
        self.emit_rtf_set_string(numbering_system, "", function);
        self.emit_rtf_string_option(options, "numberingSystem", value, function)?;
        self.emit_rtf_if_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_rtf_copy(value.payload, numbering_system, function);
        self.emit_intl_validate_unicode_type_string(numbering_system, function)?;
        function.instruction(&Instruction::End);
        self.emit_rtf_choice_option(
            options,
            "style",
            &[("long", 1), ("short", 2), ("narrow", 3)],
            1,
            style,
            function,
        )?;
        self.emit_rtf_choice_option(
            options,
            "numeric",
            &[("always", 1), ("auto", 2)],
            1,
            numeric,
            function,
        )?;

        self.emit_rtf_locale_request(
            matcher,
            requested,
            RelativeTimeLocaleQueryLocals::Resolve {
                numbering_system,
                style,
                numeric,
            },
            request,
            function,
        )?;
        self.emit_rtf_provider_call(
            RelativeTimeOperation::ResolveLocale,
            request,
            response,
            function,
        )?;
        let reader = RelativeTimeResponseReader::new(
            self,
            response,
            RelativeTimeOperation::ResolveLocale,
            function,
        );
        reader.word(self, status, function);
        self.emit_rtf_if_eq(status, RTF_RESULT_RESOLVED, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(HEAP_INTL_RELATIVE_TIME_FORMAT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        self.emit_rtf_record_from_locale_response(&reader, record, style, numeric, function);
        reader.finish(self, function);
        let initialized =
            self.emit_initialize_relative_time_format_object(reserved, record, function);

        for local in [
            record,
            status,
            response,
            request,
            numeric,
            style,
            numbering_system,
            matcher,
            value.tag,
            value.payload,
            requested,
            options.tag,
            options.payload,
            locales.tag,
            locales.payload,
        ] {
            self.release_temp_local(local);
        }
        self.emit_publish_relative_time_format_object(initialized, function);
        Ok(())
    }

    pub(crate) fn emit_intl_relative_time_format_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = self.reserve_temp_local();
        let matcher = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let status = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_rtf_canonical_locales(locales, requested, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_rtf_options_object(options, function)?;
        self.emit_rtf_choice_option(
            options,
            "localeMatcher",
            &[("lookup", 1), ("best fit", 2)],
            2,
            matcher,
            function,
        )?;
        self.emit_rtf_locale_request(
            matcher,
            requested,
            RelativeTimeLocaleQueryLocals::Supported,
            request,
            function,
        )?;
        self.emit_rtf_provider_call(
            RelativeTimeOperation::ResolveLocale,
            request,
            response,
            function,
        )?;
        let reader = RelativeTimeResponseReader::new(
            self,
            response,
            RelativeTimeOperation::ResolveLocale,
            function,
        );
        reader.word(self, status, function);
        self.emit_rtf_if_eq(status, RTF_RESULT_SUPPORTED, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        reader.word(self, count, function);
        reader.require_records(count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_rtf_set_const(index, 0, function);
        self.emit_rtf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.bytes(self, value.payload, function);
        self.emit_array_write(output, index, value.payload, value.tag, function)?;
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
            ValueKind::Array.tag() as i64,
            function,
        );

        for local in [
            value.tag,
            value.payload,
            output,
            index,
            count,
            status,
            response,
            request,
            matcher,
            requested,
            options.tag,
            options.payload,
            locales.tag,
            locales.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
