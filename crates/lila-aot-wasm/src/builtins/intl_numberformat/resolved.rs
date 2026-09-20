use super::provider_wire::{NfOperation, NfResponseReader, NfWireField, NfWireWord};
use super::*;

impl FunctionBuilder<'_> {
    fn emit_nf_resolved_property(
        &mut self,
        object: u32,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_nf_set_string(key, property, function);
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

    fn emit_nf_resolved_choice(
        &mut self,
        record: u32,
        object: u32,
        word: NfWord,
        property: &str,
        options: &[(&str, i64)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let code = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_nf_load_word(record, word, code, function);
        self.emit_nf_set_const(value.payload, 0, function);
        for &(name, expected) in options {
            self.emit_nf_if_eq(code, expected as u64, function);
            self.emit_nf_set_string(value.payload, name, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_nf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        self.emit_nf_resolved_property(object, property, value, function)?;
        for local in [value.tag, value.payload, code] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_nf_resolved_number(
        &mut self,
        record: u32,
        object: u32,
        word: NfWord,
        property: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_nf_load_word(record, word, value.payload, function);
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value.payload));
        self.emit_nf_set_const(value.tag, ValueKind::Number.tag() as i64, function);
        self.emit_nf_resolved_property(object, property, value, function)?;
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    pub(crate) fn emit_intl_number_format_resolved_options(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let object = self.reserve_temp_local();
        let code = self.reserve_temp_local();
        let precision = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_nf_record_from_receiver(record, function)?;
        self.emit_nf_result_object(function)?;
        function.instruction(&Instruction::LocalSet(object));
        self.emit_nf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        for (name, offset) in [
            ("locale", HEAP_INTL_NF_LOCALE_OFFSET),
            ("numberingSystem", HEAP_INTL_NF_NUMBERING_SYSTEM_OFFSET),
        ] {
            self.load_i64_to_local_from_offset(record, offset, value.payload, function);
            self.emit_nf_resolved_property(object, name, value, function)?;
        }
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::Style,
            "style",
            StyleOption::OPTIONS,
            function,
        )?;
        self.emit_nf_load_word(record, NfWord::Style, code, function);
        self.emit_nf_if_eq(code, StyleOption::Currency.wire_code(), function);
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_NF_STYLE_TEXT_OFFSET,
            value.payload,
            function,
        );
        self.emit_nf_resolved_property(object, "currency", value, function)?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::CurrencyDisplay,
            "currencyDisplay",
            CurrencyDisplay::OPTIONS,
            function,
        )?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::CurrencySign,
            "currencySign",
            CurrencySign::OPTIONS,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_nf_if_eq(code, StyleOption::Unit.wire_code(), function);
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_NF_STYLE_TEXT_OFFSET,
            value.payload,
            function,
        );
        self.emit_nf_resolved_property(object, "unit", value, function)?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::UnitDisplay,
            "unitDisplay",
            UnitDisplay::OPTIONS,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::MinimumInteger,
            "minimumIntegerDigits",
            function,
        )?;
        self.emit_nf_load_word(record, NfWord::Precision, precision, function);
        function.instruction(&Instruction::LocalGet(precision));
        function.instruction(&Instruction::I64Const(
            NumberPrecisionKind::Significant.wire_code() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::MinimumFraction,
            "minimumFractionDigits",
            function,
        )?;
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::MaximumFraction,
            "maximumFractionDigits",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(precision));
        function.instruction(&Instruction::I64Const(
            NumberPrecisionKind::Fraction.wire_code() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::MinimumSignificant,
            "minimumSignificantDigits",
            function,
        )?;
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::MaximumSignificant,
            "maximumSignificantDigits",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_nf_load_word(record, NfWord::Grouping, code, function);
        self.emit_nf_if_eq(code, Grouping::Never.wire_code(), function);
        self.emit_nf_set_const(value.payload, 0, function);
        self.emit_nf_set_const(value.tag, ValueKind::Boolean.tag() as i64, function);
        self.emit_nf_resolved_property(object, "useGrouping", value, function)?;
        function.instruction(&Instruction::Else);
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::Grouping,
            "useGrouping",
            &[
                ("auto", Grouping::Auto.wire_code() as i64),
                ("always", Grouping::Always.wire_code() as i64),
                ("min2", Grouping::MinTwo.wire_code() as i64),
            ],
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::Notation,
            "notation",
            NotationOption::OPTIONS,
            function,
        )?;
        self.emit_nf_load_word(record, NfWord::Notation, code, function);
        self.emit_nf_if_eq(code, NotationOption::Compact.wire_code(), function);
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::CompactDisplay,
            "compactDisplay",
            CompactDisplay::OPTIONS,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::SignDisplay,
            "signDisplay",
            SignDisplay::OPTIONS,
            function,
        )?;
        self.emit_nf_resolved_number(
            record,
            object,
            NfWord::RoundingIncrement,
            "roundingIncrement",
            function,
        )?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::RoundingMode,
            "roundingMode",
            RoundingMode::OPTIONS,
            function,
        )?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::Precision,
            "roundingPriority",
            &[
                (
                    RoundingPriority::Auto.name(),
                    NumberPrecisionKind::Fraction.wire_code() as i64,
                ),
                (
                    RoundingPriority::Auto.name(),
                    NumberPrecisionKind::Significant.wire_code() as i64,
                ),
                (
                    RoundingPriority::MorePrecision.name(),
                    NumberPrecisionKind::More.wire_code() as i64,
                ),
                (
                    RoundingPriority::LessPrecision.name(),
                    NumberPrecisionKind::Less.wire_code() as i64,
                ),
            ],
            function,
        )?;
        self.emit_nf_resolved_choice(
            record,
            object,
            NfWord::TrailingZero,
            "trailingZeroDisplay",
            TrailingZeroDisplay::OPTIONS,
            function,
        )?;
        self.emit_nf_copy(object, self.result_local, function);
        self.emit_nf_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        for local in [value.tag, value.payload, precision, code, object, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_number_format_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let locales = self.reserve_temp_local();
        let matcher = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, value.payload, value.tag, function);
        self.emit_nf_intrinsic_call(
            StandardBuiltinId::IntlGetCanonicalLocales,
            None,
            &[(value.payload, value.tag)],
            locales,
            value.tag,
            function,
        )?;
        self.emit_builtin_arg_to_locals(1, value.payload, value.tag, function);
        self.emit_nf_options_object(value, function)?;
        self.emit_nf_choice_option(
            value,
            "localeMatcher",
            LocaleMatcher::OPTIONS,
            LocaleMatcher::BestFit.wire_code(),
            matcher,
            function,
        )?;
        self.emit_nf_provider_request(
            NfOperation::SupportedLocales,
            &[
                NfWireField::Word(NfWireWord::Local(matcher)),
                NfWireField::CanonicalLocales(locales),
            ],
            request,
            function,
        )?;
        self.emit_nf_provider_call(NfOperation::SupportedLocales, request, response, function)?;
        let reader = NfResponseReader::new(self, response, function);
        reader.word(self, count, function);
        reader.require_records(count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count, output, function,
        )?;
        self.emit_nf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        reader.bytes(self, value.payload, function);
        self.emit_nf_output_array_entry(output, index, value.payload, ValueKind::String, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        reader.finish(self, function);
        self.emit_nf_copy(output, self.result_local, function);
        self.emit_nf_set_const(
            self.result_tag_local,
            ValueKind::Array.tag() as i64,
            function,
        );
        for local in [
            output,
            index,
            count,
            response,
            request,
            matcher,
            locales,
            value.tag,
            value.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
