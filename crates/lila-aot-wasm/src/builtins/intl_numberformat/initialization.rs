use super::*;
use crate::builtins::intl_provider_wire::IntlNumberProviderRequest;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_number_format_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reserved = self.emit_reserve_intl_number_format_object(function)?;
        let schema = self.runtime_schema();
        let locales = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &locales, function);
        let requested = self.emit_intl_canonical_locale_list(&locales, function)?;
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_intl_number_options_object(&options, function)?;
        let matcher = GcI32DomainLocal::new(schema, LocaleMatcher::BestFit, function);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::LocaleMatcher,
            LocaleMatcher::ALL.iter().map(|v| (v.name(), *v)),
            LocaleMatcher::BestFit,
            &matcher,
            function,
        )?;
        let numbering = self.emit_nf_numbering_option(&options, function)?;
        let response = self.emit_intl_number_provider_call(
            IntlNumberProviderRequest::Resolve {
                locales: &requested,
                matcher: &matcher,
                numbering_system: &numbering,
            },
            function,
        )?;
        let selected = NfOptionsLocals::new(self, function)?;
        let currency = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let unit = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::Style,
            StyleOption::ALL.iter().map(|v| (v.name(), *v)),
            StyleOption::Decimal,
            &selected.style,
            function,
        )?;
        self.emit_intl_number_string_value(&options, "currency", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        emit_domain_is(&selected.style, StyleOption::Currency, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(NF_CURRENCY_REQUIRED, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let upper = self.emit_nf_currency_code(&text, function)?;
        currency.replace(upper.load(schema, function), function);
        upper.clear(function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::CurrencyDisplay,
            CurrencyDisplay::ALL.iter().map(|v| (v.name(), Some(*v))),
            Some(CurrencyDisplay::Symbol),
            &selected.currency_display,
            function,
        )?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::CurrencySign,
            CurrencySign::ALL.iter().map(|v| (v.name(), Some(*v))),
            Some(CurrencySign::Standard),
            &selected.currency_sign,
            function,
        )?;
        self.emit_intl_number_string_value(&options, "unit", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        emit_domain_is(&selected.style, StyleOption::Unit, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(NF_UNIT_REQUIRED, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_nf_unit_identifier(&text, function)?;
        unit.replace(text.load(schema, function), function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::UnitDisplay,
            UnitDisplay::ALL.iter().map(|v| (v.name(), Some(*v))),
            Some(UnitDisplay::Short),
            &selected.unit_display,
            function,
        )?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::Notation,
            NotationOption::ALL.iter().map(|v| (v.name(), *v)),
            NotationOption::Standard,
            &selected.notation,
            function,
        )?;
        let rounding = self.emit_intl_number_digit_options(
            &options,
            &selected.notation,
            IntlDigitDefaults::NumberFormat {
                style: &selected.style,
                currency: &currency,
            },
            function,
        )?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::CompactDisplay,
            CompactDisplay::ALL.iter().map(|v| (v.name(), Some(*v))),
            Some(CompactDisplay::Short),
            &selected.compact_display,
            function,
        )?;
        self.emit_nf_grouping_option(&options, &selected.notation, &selected.grouping, function)?;
        self.emit_intl_number_choice_option(
            &options,
            IntlErrorOption::SignDisplay,
            SignDisplay::ALL.iter().map(|v| (v.name(), *v)),
            SignDisplay::Auto,
            &selected.sign,
            function,
        )?;
        // Inactive options were still observed and validated. Their absence is
        // canonicalized only after the complete source observation phase.
        emit_domain_is(&selected.style, StyleOption::Currency, function);
        self.open_frame(ControlFrameKind::If, function);
        selected
            .style_text
            .replace(currency.load(schema, function), function);
        function.instruction(&Instruction::Else);
        selected.currency_display.set_constant(None, function);
        selected.currency_sign.set_constant(None, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        emit_domain_is(&selected.style, StyleOption::Unit, function);
        self.open_frame(ControlFrameKind::If, function);
        selected
            .style_text
            .replace(unit.load(schema, function), function);
        function.instruction(&Instruction::Else);
        selected.unit_display.set_constant(None, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        emit_domain_is(&selected.notation, NotationOption::Compact, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        selected.compact_display.set_constant(None, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let reader = response.reader(schema, function);
        let locale = reader.read_utf8(schema, function);
        let data_locale = reader.read_utf8(schema, function);
        let numbering_system = reader.read_utf8(schema, function);
        reader.finish(schema, function);
        let completed = self.emit_initialize_intl_number_format_object(
            reserved,
            &locale,
            &data_locale,
            &numbering_system,
            &selected,
            &rounding,
            function,
        );
        self.emit_publish_intl_number_format_object(completed, function);
        numbering_system.clear(function);
        data_locale.clear(function);
        locale.clear(function);
        rounding.clear(function);
        value.clear(function);
        unit.clear(function);
        currency.clear(function);
        selected.clear(schema, function);
        response.clear(function);
        numbering.clear(function);
        matcher.clear(schema, function);
        options.clear(function);
        requested.clear(function);
        locales.clear(function);
        Ok(())
    }
}
