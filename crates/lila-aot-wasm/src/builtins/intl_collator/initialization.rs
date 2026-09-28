use super::provider_wire::{
    CollatorOperation, CollatorResponseReader, CollatorWireField, CollatorWireWord,
};
use super::*;

impl FunctionBuilder<'_> {
    fn emit_collator_record_from_response(
        &mut self,
        reader: &CollatorResponseReader,
        record: u32,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [
            HEAP_INTL_COL_LOCALE_OFFSET,
            HEAP_INTL_COL_DATA_LOCALE_OFFSET,
        ] {
            reader.bytes(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        reader.word(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_COL_USAGE_OFFSET, value, function);
        reader.bytes(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_COL_COLLATION_OFFSET, value, function);
        for offset in [
            HEAP_INTL_COL_NUMERIC_OFFSET,
            HEAP_INTL_COL_CASE_FIRST_OFFSET,
            HEAP_INTL_COL_SENSITIVITY_OFFSET,
            HEAP_INTL_COL_IGNORE_PUNCTUATION_OFFSET,
        ] {
            reader.word(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        self.release_temp_local(value);
    }

    fn emit_collator_locale_request(
        &mut self,
        locales: u32,
        matcher: u32,
        usage: u32,
        collation: u32,
        numeric: u32,
        case_first: u32,
        sensitivity: u32,
        ignore_punctuation: u32,
        request: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let matcher_wire = self.reserve_temp_local();
        let usage_wire = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(matcher));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(matcher_wire));
        function.instruction(&Instruction::LocalGet(usage));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(usage_wire));
        let fields = [
            CollatorWireField::Word(CollatorWireWord::Constant(0)),
            CollatorWireField::Word(CollatorWireWord::Local(matcher_wire)),
            CollatorWireField::CanonicalLocales(locales),
            CollatorWireField::Word(CollatorWireWord::Local(usage_wire)),
            CollatorWireField::Bytes(collation),
            CollatorWireField::Word(CollatorWireWord::Local(numeric)),
            CollatorWireField::Word(CollatorWireWord::Local(case_first)),
            CollatorWireField::Word(CollatorWireWord::Local(sensitivity)),
            CollatorWireField::Word(CollatorWireWord::Local(ignore_punctuation)),
        ];
        let result = self.emit_col_provider_request(
            CollatorOperation::ResolveLocale,
            &fields,
            request,
            function,
        );
        self.release_temp_local(usage_wire);
        self.release_temp_local(matcher_wire);
        result
    }

    pub(crate) fn emit_intl_collator_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reserved = self.emit_reserve_collator_object(function)?;
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let usage = self.reserve_temp_local();
        let matcher = self.reserve_temp_local();
        let collation = self.reserve_temp_local();
        let numeric = self.reserve_temp_local();
        let case_first = self.reserve_temp_local();
        let sensitivity = self.reserve_temp_local();
        let ignore_punctuation = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let result_kind = self.reserve_temp_local();
        let record = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_col_canonical_locales(locales, requested.payload, requested.tag, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_col_options_object(options, function)?;
        self.emit_col_choice_option(
            options,
            "usage",
            &[("sort", 1), ("search", 2)],
            1,
            usage,
            function,
        )?;
        self.emit_col_choice_option(
            options,
            "localeMatcher",
            &[("best fit", 1), ("lookup", 2)],
            1,
            matcher,
            function,
        )?;
        self.emit_col_set_string(collation, "", function);
        self.emit_col_string_option(options, "collation", value, function)?;
        self.emit_col_if_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::LocalSet(collation));
        self.emit_intl_validate_unicode_type_string(collation, function)?;
        function.instruction(&Instruction::End);
        self.emit_col_optional_boolean_option(options, "numeric", numeric, function)?;
        self.emit_col_choice_option(
            options,
            "caseFirst",
            &[("false", 1), ("upper", 2), ("lower", 3)],
            0,
            case_first,
            function,
        )?;
        self.emit_col_choice_option(
            options,
            "sensitivity",
            &[("base", 1), ("accent", 2), ("case", 3), ("variant", 4)],
            0,
            sensitivity,
            function,
        )?;
        self.emit_col_optional_boolean_option(
            options,
            "ignorePunctuation",
            ignore_punctuation,
            function,
        )?;
        // A missing caseFirst/sensitivity is encoded as zero; public locale
        // matching can then preserve a matching Unicode extension key.
        self.emit_collator_locale_request(
            requested.payload,
            matcher,
            usage,
            collation,
            numeric,
            case_first,
            sensitivity,
            ignore_punctuation,
            request,
            function,
        )?;
        self.emit_col_provider_call(
            CollatorOperation::ResolveLocale,
            request,
            response,
            function,
        )?;
        let reader = CollatorResponseReader::new(
            self,
            response,
            CollatorOperation::ResolveLocale,
            function,
        )?;
        reader.word(self, result_kind, function);
        self.emit_col_if_eq(result_kind, 0, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(HEAP_INTL_COLLATOR_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        self.emit_collator_record_from_response(&reader, record, function);
        reader.finish(self, function);
        let initialized = self.emit_initialize_collator_object(reserved, record, function);
        for local in [
            record,
            result_kind,
            response,
            request,
            ignore_punctuation,
            sensitivity,
            case_first,
            numeric,
            collation,
            matcher,
            usage,
            value.tag,
            value.payload,
            requested.tag,
            requested.payload,
            options.tag,
            options.payload,
            locales.tag,
            locales.payload,
        ] {
            self.release_temp_local(local);
        }
        self.emit_publish_collator_object(initialized, function);
        Ok(())
    }

    pub(crate) fn emit_intl_collator_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let matcher = self.reserve_temp_local();
        let matcher_wire = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_col_canonical_locales(locales, requested.payload, requested.tag, function)?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_col_options_object(options, function)?;
        self.emit_col_choice_option(
            options,
            "localeMatcher",
            &[("best fit", 1), ("lookup", 2)],
            1,
            matcher,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(matcher));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(matcher_wire));
        let fields = [
            CollatorWireField::Word(CollatorWireWord::Constant(1)),
            CollatorWireField::Word(CollatorWireWord::Local(matcher_wire)),
            CollatorWireField::CanonicalLocales(requested.payload),
        ];
        self.emit_col_provider_request(
            CollatorOperation::SupportedLocales,
            &fields,
            request,
            function,
        )?;
        self.emit_col_provider_call(
            CollatorOperation::SupportedLocales,
            request,
            response,
            function,
        )?;
        let reader = CollatorResponseReader::new(
            self,
            response,
            CollatorOperation::SupportedLocales,
            function,
        )?;
        reader.word(self, kind, function);
        self.emit_col_if_eq(kind, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        reader.word(self, count, function);
        reader.require_records(self, count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_col_set_const(index, 0, function);
        self.emit_col_set_const(value.tag, ValueKind::String.tag() as i64, function);
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
        self.emit_col_set_const(
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
            kind,
            response,
            request,
            matcher_wire,
            matcher,
            requested.tag,
            requested.payload,
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
