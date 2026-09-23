use super::provider_wire::{DtfResponseReader, DtfWireField, DtfWireWord};
use super::*;
use crate::builtins::date::DateLocaleFormat;
use lila_intl::{DateTimeCalendar, DateTimeDefaults, DateTimeRequired, IntlHostOp};

pub(crate) enum IntlDateTimeFormatPurpose {
    Constructor,
    DateLocale(DateLocaleFormat),
    Temporal(DtfTemporalKind),
}

enum RejectedDateTimeStyle {
    Date,
    Time,
}

impl IntlDateTimeFormatPurpose {
    fn required(&self) -> DateTimeRequired {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::DateAndTime) => {
                DateTimeRequired::Any
            }
            Self::DateLocale(DateLocaleFormat::Date) => DateTimeRequired::Date,
            Self::DateLocale(DateLocaleFormat::Time) => DateTimeRequired::Time,
            Self::Temporal(kind) => match kind {
                DtfTemporalKind::PlainDate
                | DtfTemporalKind::PlainYearMonth
                | DtfTemporalKind::PlainMonthDay => DateTimeRequired::Date,
                DtfTemporalKind::PlainTime => DateTimeRequired::Time,
                DtfTemporalKind::PlainDateTime | DtfTemporalKind::Instant => DateTimeRequired::Any,
            },
        }
    }
    fn defaults(&self) -> DateTimeDefaults {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::Date) => DateTimeDefaults::Date,
            Self::DateLocale(DateLocaleFormat::Time) => DateTimeDefaults::Time,
            Self::DateLocale(DateLocaleFormat::DateAndTime) => DateTimeDefaults::All,
            Self::Temporal(kind) => match kind {
                DtfTemporalKind::PlainDate
                | DtfTemporalKind::PlainYearMonth
                | DtfTemporalKind::PlainMonthDay => DateTimeDefaults::Date,
                DtfTemporalKind::PlainTime => DateTimeDefaults::Time,
                DtfTemporalKind::PlainDateTime | DtfTemporalKind::Instant => DateTimeDefaults::All,
            },
        }
    }
    fn rejected_style(&self) -> Option<(RejectedDateTimeStyle, String)> {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::DateAndTime) => None,
            Self::DateLocale(DateLocaleFormat::Date) => Some((
                RejectedDateTimeStyle::Time,
                "Date.prototype.toLocaleDateString does not support the timeStyle option".into(),
            )),
            Self::DateLocale(DateLocaleFormat::Time) => Some((
                RejectedDateTimeStyle::Date,
                "Date.prototype.toLocaleTimeString does not support the dateStyle option".into(),
            )),
            Self::Temporal(kind) => kind.rejected_style().map(|(property, offset)| {
                (
                    if offset == HEAP_INTL_DTF_DATE_STYLE_OFFSET {
                        RejectedDateTimeStyle::Date
                    } else {
                        RejectedDateTimeStyle::Time
                    },
                    intl_dtf_temporal_style_message(kind.type_name(), property),
                )
            }),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_create_date_time_format(
        &mut self,
        purpose: IntlDateTimeFormatPurpose,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales_payload = self.reserve_temp_local();
        let locales_tag = self.reserve_temp_local();
        let requested_locales = self.reserve_temp_local();
        let options_payload = self.reserve_temp_local();
        let options_tag = self.reserve_temp_local();
        let locale_matcher = self.reserve_temp_local();
        let calendar = self.reserve_temp_local();
        let numbering_system = self.reserve_temp_local();
        let hour12 = self.reserve_temp_local();
        let hour_cycle = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let resolved_locale = self.reserve_temp_local();
        let time_zone = DtfCanonicalTimeZone::reserve(self);
        let explicit = self.reserve_temp_local();
        let present = self.reserve_temp_local();
        let format_matcher = self.reserve_temp_local();
        let date_style = self.reserve_temp_local();
        let time_style = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let plan = self.reserve_temp_local();
        let components: Vec<u32> = INTL_DTF_COMPONENT_OPTIONS
            .iter()
            .map(|_| self.reserve_temp_local())
            .collect();
        let fractional = self.reserve_temp_local();

        // Reserve before observing locales/options, and publish only the fully
        // selected record. A failed initialization cannot expose its object.
        let reserved_object = self.emit_reserve_intl_date_time_format_object(function)?;
        self.emit_builtin_arg_to_locals(0, locales_payload, locales_tag, function);
        self.emit_dtf_requested_locales(locales_payload, locales_tag, requested_locales, function)?;
        self.emit_builtin_arg_to_locals(1, options_payload, options_tag, function);
        function.instruction(&Instruction::LocalGet(options_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options_payload));
        self.emit_dtf_set_const(options_tag, ValueKind::Object.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(
            options_payload,
            options_tag,
            options_payload,
            options_tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.emit_dtf_matcher_option(
            options_payload,
            options_tag,
            "localeMatcher",
            lila_intl::DateTimeLocaleMatcher::OPTIONS,
            locale_matcher,
            function,
        )?;
        self.emit_dtf_keyword_option(options_payload, options_tag, "calendar", calendar, function)?;
        self.emit_dtf_keyword_option(
            options_payload,
            options_tag,
            "numberingSystem",
            numbering_system,
            function,
        )?;
        self.emit_intl_dtf_hour12_option(options_payload, options_tag, hour12, function)?;
        self.emit_intl_dtf_string_option(
            options_payload,
            options_tag,
            &INTL_DTF_HOUR_CYCLE_OPTION,
            hour_cycle,
            None,
            function,
        )?;
        // A present hour12 suppresses the requested hc extension even though
        // hourCycle's getter and validation still occur before ResolveLocale.
        self.emit_dtf_if_code_eq(hour12, 1, function);
        self.emit_dtf_set_const(hour_cycle, 6, function);
        function.instruction(&Instruction::End);
        self.emit_dtf_if_code_eq(hour12, 2, function);
        self.emit_dtf_set_const(hour_cycle, 5, function);
        function.instruction(&Instruction::End);
        self.emit_dtf_provider_request(
            IntlHostOp::ResolveDateTimeLocale,
            &[
                DtfWireField::Word(DtfWireWord::Local(locale_matcher)),
                DtfWireField::Word(DtfWireWord::Local(hour_cycle)),
                DtfWireField::Bytes(calendar),
                DtfWireField::Bytes(numbering_system),
                DtfWireField::CanonicalLocales(requested_locales),
            ],
            request,
            function,
        )?;
        self.emit_dtf_provider_call(
            IntlHostOp::ResolveDateTimeLocale,
            request,
            resolved_locale,
            function,
        )?;

        let time_zone =
            self.emit_intl_dtf_time_zone_option(options_payload, options_tag, time_zone, function)?;
        self.emit_dtf_set_const(explicit, 0, function);
        for (option, destination) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&components) {
            self.emit_intl_dtf_string_option(
                options_payload,
                options_tag,
                option,
                *destination,
                Some(present),
                function,
            )?;
            function.instruction(&Instruction::LocalGet(explicit));
            function.instruction(&Instruction::LocalGet(present));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(explicit));
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                self.emit_intl_dtf_fractional_second_digits_option(
                    options_payload,
                    options_tag,
                    fractional,
                    present,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(explicit));
                function.instruction(&Instruction::LocalGet(present));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(explicit));
            }
        }
        self.emit_dtf_matcher_option(
            options_payload,
            options_tag,
            "formatMatcher",
            lila_intl::DateTimeFormatMatcher::OPTIONS,
            format_matcher,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload,
            options_tag,
            &INTL_DTF_DATE_STYLE_OPTION,
            date_style,
            None,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload,
            options_tag,
            &INTL_DTF_TIME_STYLE_OPTION,
            time_style,
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(date_style));
        function.instruction(&Instruction::LocalGet(time_style));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(explicit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "dateStyle and timeStyle may not be used with explicit date-time components",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        if let Some((style, message)) = purpose.rejected_style() {
            self.emit_dtf_if_nonzero(
                match style {
                    RejectedDateTimeStyle::Date => date_style,
                    RejectedDateTimeStyle::Time => time_style,
                },
                function,
            );
            self.emit_throw_current_function_realm_type_error(
                &message,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        // The provider sees the original component selection. Defaulting and
        // per-Temporal-kind pattern selection share the same retained recipe.
        let common = || {
            vec![
                DtfWireField::RecordBody(resolved_locale),
                DtfWireField::TimeZone {
                    identifier: time_zone.0.identifier_local,
                    kind: time_zone.0.kind_local,
                    fixed_seconds: time_zone.0.fixed_seconds_local,
                },
                DtfWireField::Word(DtfWireWord::Local(format_matcher)),
                DtfWireField::Word(DtfWireWord::Constant(purpose.required().wire_code())),
                DtfWireField::Word(DtfWireWord::Constant(purpose.defaults().wire_code())),
            ]
        };
        function.instruction(&Instruction::LocalGet(date_style));
        function.instruction(&Instruction::LocalGet(time_style));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let mut fields = common();
        fields.push(DtfWireField::Word(DtfWireWord::Constant(1)));
        for (option, local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&components) {
            fields.push(DtfWireField::Word(DtfWireWord::Local(*local)));
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                fields.push(DtfWireField::Word(DtfWireWord::Local(fractional)));
            }
        }
        self.emit_dtf_provider_request(
            IntlHostOp::SelectDateTimeFormat,
            &fields,
            request,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let mut fields = common();
        fields.extend([
            DtfWireField::Word(DtfWireWord::Constant(2)),
            DtfWireField::Word(DtfWireWord::Local(date_style)),
            DtfWireField::Word(DtfWireWord::Local(time_style)),
        ]);
        self.emit_dtf_provider_request(
            IntlHostOp::SelectDateTimeFormat,
            &fields,
            request,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_dtf_provider_call(IntlHostOp::SelectDateTimeFormat, request, plan, function)?;
        self.emit_heap_alloc_const(HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        time_zone.store(self, record, function);
        self.emit_dtf_store_selected_plan(plan, record, function)?;
        self.store_i64_const_at_offset(record, HEAP_INTL_DTF_BOUND_FORMAT_OFFSET, 0, function);
        let initialized_object =
            self.emit_initialize_intl_date_time_format_object(reserved_object, record, function);
        self.emit_publish_intl_date_time_format_object(initialized_object, function);

        self.release_temp_local(fractional);
        for local in components.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [
            plan,
            record,
            time_style,
            date_style,
            format_matcher,
            present,
            explicit,
        ] {
            self.release_temp_local(local);
        }
        time_zone.release(self);
        for local in [
            resolved_locale,
            request,
            hour_cycle,
            hour12,
            numbering_system,
            calendar,
            locale_matcher,
            options_tag,
            options_payload,
            requested_locales,
            locales_tag,
            locales_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_dtf_requested_locales(
        &mut self,
        payload: u32,
        tag: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result_tag = self.reserve_temp_local();
        let canonicalize = self
            .functions
            .get(&StandardBuiltinId::IntlGetCanonicalLocales.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Intl.getCanonicalLocales builtin"))?;
        self.emit_direct_js_call(
            &canonicalize,
            None,
            &[(payload, tag)],
            destination,
            result_tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.release_temp_local(result_tag);
        Ok(())
    }

    pub(super) fn emit_dtf_matcher_option(
        &mut self,
        payload: u32,
        tag: u32,
        property: &'static str,
        codes: &'static [(&'static str, i64)],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_string_option(
            payload,
            tag,
            &IntlDtfOption {
                property,
                slot_offset: 0,
                codes,
            },
            destination,
            None,
            function,
        )?;
        self.emit_dtf_if_code_eq(destination, 0, function);
        self.emit_dtf_set_const(destination, 2, function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_dtf_keyword_option(
        &mut self,
        payload: u32,
        tag: u32,
        property: &'static str,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let value_tag = self.reserve_temp_local();
        let valid = self.reserve_temp_local();
        let lowered = self.reserve_temp_local();
        self.emit_dtf_set_string(destination, "", function);
        self.emit_dtf_set_string(key, property, function);
        self.emit_object_read(payload, tag, payload, tag, key, value, value_tag, function)?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value, value_tag, function)?;
        function.instruction(&Instruction::LocalSet(value));
        self.emit_return_current_completion_if_throw(function);
        let checked = self.emit_intl_dtf_type_nonterminal_guard(
            value,
            valid,
            lowered,
            &format!("Invalid {property} option"),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(checked.lowered_local));
        function.instruction(&Instruction::LocalSet(destination));
        function.instruction(&Instruction::End);
        for local in [lowered, valid, value_tag, value, key] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_dtf_store_selected_plan(
        &mut self,
        response: u32,
        record: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = self.reserve_temp_local();
        let calendar = self.reserve_temp_local();
        let reader = DtfResponseReader::new(self, response, function);
        reader.bytes(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_LOCALE_OFFSET, value, function);
        reader.bytes(self, value, function); // The plan retains dataLocale; resolvedOptions omits it.
        reader.word(self, calendar, function);
        self.emit_dtf_set_const(value, 0, function);
        for kind in DateTimeCalendar::ALL {
            self.emit_dtf_if_code_eq(calendar, kind.wire_code() as i64, function);
            self.emit_dtf_set_string(value, kind.as_str(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_CALENDAR_OFFSET, value, function);
        reader.bytes(self, value, function);
        self.store_i64_local_at_offset(
            record,
            HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
            value,
            function,
        );
        reader.word(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_HOUR_CYCLE_OFFSET, value, function);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(value));
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_HOUR12_OFFSET, value, function);
        reader.word(self, value, function);
        self.store_i64_local_at_offset(
            record,
            HEAP_INTL_DTF_TIME_ZONE_KIND_OFFSET,
            value,
            function,
        );
        self.emit_dtf_if_code_eq(value, TimeZoneKind::Named.code(), function);
        reader.bytes(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_TIME_ZONE_OFFSET, value, function);
        function.instruction(&Instruction::Else);
        reader.word(self, value, function);
        self.store_i64_local_at_offset(
            record,
            HEAP_INTL_DTF_TIME_ZONE_FIXED_SECONDS_OFFSET,
            value,
            function,
        );
        function.instruction(&Instruction::End);
        for offset in INTL_DTF_FORMAT_COMPONENT_SLOTS.into_iter().chain([
            HEAP_INTL_DTF_DATE_STYLE_OFFSET,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            HEAP_INTL_DTF_AVAILABLE_FORMATS_OFFSET,
        ]) {
            reader.word(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        reader.bytes(self, value, function);
        self.store_i64_local_at_offset(record, HEAP_INTL_DTF_PLAN_OFFSET, value, function);
        reader.finish(self, function);
        self.release_temp_local(calendar);
        self.release_temp_local(value);
        Ok(())
    }
}
