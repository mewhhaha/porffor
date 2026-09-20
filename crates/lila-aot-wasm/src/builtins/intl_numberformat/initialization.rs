use super::digits::ObservedNumberDigitOptions;
use super::provider_wire::{NfOperation, NfResponseReader, NfWireField, NfWireWord};
use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_number_format_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let options = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let requested = self.reserve_temp_local();
        let matcher = self.reserve_temp_local();
        let numbering_system = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let currency = self.reserve_temp_local();
        let unit = self.reserve_temp_local();
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let fallback = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let selected = NfOptionsLocals::reserve(self);
        let reserved = self.emit_reserve_intl_number_format_object(function)?;

        self.emit_builtin_arg_to_locals(0, locales.payload, locales.tag, function);
        self.emit_nf_intrinsic_call(
            StandardBuiltinId::IntlGetCanonicalLocales,
            None,
            &[(locales.payload, locales.tag)],
            requested,
            value.tag,
            function,
        )?;
        self.emit_builtin_arg_to_locals(1, options.payload, options.tag, function);
        self.emit_nf_options_object(options, function)?;
        self.emit_nf_choice_option(
            options,
            "localeMatcher",
            LocaleMatcher::OPTIONS,
            LocaleMatcher::BestFit.wire_code(),
            matcher,
            function,
        )?;
        self.emit_nf_numbering_option(options, numbering_system, function)?;
        self.emit_nf_provider_request(
            NfOperation::ResolveLocale,
            &[
                NfWireField::Word(NfWireWord::Local(matcher)),
                NfWireField::Bytes(numbering_system),
                NfWireField::CanonicalLocales(requested),
            ],
            request,
            function,
        )?;
        self.emit_nf_provider_call(NfOperation::ResolveLocale, request, response, function)?;
        for word in NfWord::ALL {
            self.emit_nf_set_const(selected.word(word), 0, function);
        }
        self.emit_nf_set_string(selected.style_text, "", function);
        self.emit_nf_choice_option(
            options,
            "style",
            StyleOption::OPTIONS,
            StyleOption::Decimal.wire_code(),
            selected.word(NfWord::Style),
            function,
        )?;

        self.emit_nf_set_string(currency, "", function);
        self.emit_nf_string_value(options, "currency", value, function)?;
        self.emit_nf_if_eq(value.tag, ValueKind::Undefined.tag() as u64, function);
        self.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Currency.wire_code(),
            function,
        );
        self.emit_nf_type_error(NF_CURRENCY_REQUIRED, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_nf_currency_code(value.payload, currency, function)?;
        function.instruction(&Instruction::End);
        self.emit_nf_choice_option(
            options,
            "currencyDisplay",
            CurrencyDisplay::OPTIONS,
            CurrencyDisplay::Symbol.wire_code(),
            selected.word(NfWord::CurrencyDisplay),
            function,
        )?;
        self.emit_nf_choice_option(
            options,
            "currencySign",
            CurrencySign::OPTIONS,
            CurrencySign::Standard.wire_code(),
            selected.word(NfWord::CurrencySign),
            function,
        )?;
        self.emit_nf_set_string(unit, "", function);
        self.emit_nf_string_value(options, "unit", value, function)?;
        self.emit_nf_if_eq(value.tag, ValueKind::Undefined.tag() as u64, function);
        self.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Unit.wire_code(),
            function,
        );
        self.emit_nf_type_error(NF_UNIT_REQUIRED, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_nf_unit_identifier(value.payload, function)?;
        self.emit_nf_copy(value.payload, unit, function);
        function.instruction(&Instruction::End);
        self.emit_nf_choice_option(
            options,
            "unitDisplay",
            UnitDisplay::OPTIONS,
            UnitDisplay::Short.wire_code(),
            selected.word(NfWord::UnitDisplay),
            function,
        )?;
        self.emit_nf_choice_option(
            options,
            "notation",
            NotationOption::OPTIONS,
            NotationOption::Standard.wire_code(),
            selected.word(NfWord::Notation),
            function,
        )?;
        self.emit_nf_set_const(fallback, 1, function);
        self.emit_nf_number_option(
            options,
            "minimumIntegerDigits",
            1,
            21,
            fallback,
            selected.word(NfWord::MinimumInteger),
            function,
        )?;
        ObservedNumberDigitOptions::read(self, options, function)?
            .observe_rounding(self, options, &selected, function)?
            .finish(self, &selected, currency, function)?;
        self.emit_nf_choice_option(
            options,
            "compactDisplay",
            CompactDisplay::OPTIONS,
            CompactDisplay::Short.wire_code(),
            selected.word(NfWord::CompactDisplay),
            function,
        )?;
        self.emit_nf_grouping_option(
            options,
            selected.word(NfWord::Notation),
            selected.word(NfWord::Grouping),
            function,
        )?;
        self.emit_nf_choice_option(
            options,
            "signDisplay",
            SignDisplay::OPTIONS,
            SignDisplay::Auto.wire_code(),
            selected.word(NfWord::SignDisplay),
            function,
        )?;

        // All source options were observed, even inactive ones. Only the active
        // closed configuration crosses the private record/provider boundary.
        self.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Currency.wire_code(),
            function,
        );
        self.emit_nf_copy(currency, selected.style_text, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_set_const(selected.word(NfWord::CurrencyDisplay), 0, function);
        self.emit_nf_set_const(selected.word(NfWord::CurrencySign), 0, function);
        function.instruction(&Instruction::End);
        self.emit_nf_if_eq(
            selected.word(NfWord::Style),
            StyleOption::Unit.wire_code(),
            function,
        );
        self.emit_nf_copy(unit, selected.style_text, function);
        function.instruction(&Instruction::Else);
        self.emit_nf_set_const(selected.word(NfWord::UnitDisplay), 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(selected.word(NfWord::Notation)));
        function.instruction(&Instruction::I64Const(
            NotationOption::Compact.wire_code() as i64
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(selected.word(NfWord::CompactDisplay), 0, function);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(HEAP_INTL_NUMBER_FORMAT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        let reader = NfResponseReader::new(self, response, function);
        for offset in [
            HEAP_INTL_NF_LOCALE_OFFSET,
            HEAP_INTL_NF_DATA_LOCALE_OFFSET,
            HEAP_INTL_NF_NUMBERING_SYSTEM_OFFSET,
        ] {
            reader.bytes(self, value.payload, function);
            self.store_i64_local_at_offset(record, offset, value.payload, function);
        }
        reader.finish(self, function);
        self.store_i64_local_at_offset(
            record,
            HEAP_INTL_NF_STYLE_TEXT_OFFSET,
            selected.style_text,
            function,
        );
        self.store_i64_const_at_offset(record, HEAP_INTL_NF_BOUND_FORMAT_OFFSET, 0, function);
        for word in NfWord::ALL {
            self.store_i64_local_at_offset(
                record,
                HEAP_INTL_NF_WORDS_OFFSET + word.offset(),
                selected.word(word),
                function,
            );
        }
        let initialized =
            self.emit_initialize_intl_number_format_object(reserved, record, function);
        self.emit_publish_intl_number_format_object(initialized, function);
        selected.release(self);
        for local in [
            record,
            fallback,
            value.tag,
            value.payload,
            unit,
            currency,
            response,
            request,
            numbering_system,
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
