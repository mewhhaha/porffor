use super::*;
use crate::builtins::date::DateLocaleFormat;

pub(crate) enum IntlDateTimeFormatPurpose {
    Constructor,
    DateLocale(DateLocaleFormat),
}

enum RejectedDateLocaleStyle {
    Date,
    Time,
}

impl IntlDateTimeFormatPurpose {
    fn required_components(&self) -> &'static [&'static str] {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::DateAndTime) => {
                &INTL_DTF_NEED_DEFAULTS_COMPONENTS
            }
            Self::DateLocale(DateLocaleFormat::Date) => &["weekday", "year", "month", "day"],
            Self::DateLocale(DateLocaleFormat::Time) => &[
                "dayPeriod",
                "hour",
                "minute",
                "second",
                "fractionalSecondDigits",
            ],
        }
    }

    fn default_components(&self) -> &'static [&'static str] {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::Date) => {
                &["year", "month", "day"]
            }
            Self::DateLocale(DateLocaleFormat::Time) => &["hour", "minute", "second"],
            Self::DateLocale(DateLocaleFormat::DateAndTime) => {
                &["year", "month", "day", "hour", "minute", "second"]
            }
        }
    }

    fn rejected_style(&self) -> Option<(RejectedDateLocaleStyle, &'static str)> {
        match self {
            Self::Constructor | Self::DateLocale(DateLocaleFormat::DateAndTime) => None,
            Self::DateLocale(DateLocaleFormat::Date) => Some((
                RejectedDateLocaleStyle::Time,
                "Date.prototype.toLocaleDateString does not support the timeStyle option",
            )),
            Self::DateLocale(DateLocaleFormat::Time) => Some((
                RejectedDateLocaleStyle::Date,
                "Date.prototype.toLocaleTimeString does not support the dateStyle option",
            )),
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `CreateDateTimeFormat` with the caller's required and default fields.
    ///
    /// Result reservation precedes every observable initialization step. The
    /// option reads then follow the exact order the specification prescribes;
    /// do not reorder them.
    pub(crate) fn emit_intl_create_date_time_format(
        &mut self,
        purpose: IntlDateTimeFormatPurpose,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales_payload_local = self.reserve_temp_local();
        let locales_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let locale_local = self.reserve_temp_local();
        let matched_tag_local = self.reserve_temp_local();
        let extension_value_row_local = self.reserve_temp_local();
        let extension_addition_row_local = self.reserve_temp_local();
        let extension_open_local = self.reserve_temp_local();
        let scratch_suffix_local = self.reserve_temp_local();
        let hour12_local = self.reserve_temp_local();
        let hour_cycle_local = self.reserve_temp_local();
        let hour_cycle_option_row_local = self.reserve_temp_local();
        let time_zone = DtfCanonicalTimeZone::reserve(self);
        let calendar_local = self.reserve_temp_local();
        let calendar_option_row_local = self.reserve_temp_local();
        let numbering_system_local = self.reserve_temp_local();
        let numbering_system_option_row_local = self.reserve_temp_local();
        let explicit_local = self.reserve_temp_local();
        // Required date/time fields clear defaults. `explicit_local` also
        // counts era and timeZoneName when checking style conflicts.
        let defaults_cleared_local = self.reserve_temp_local();
        let need_defaults_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let date_style_local = self.reserve_temp_local();
        let time_style_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let component_locals: Vec<u32> = INTL_DTF_COMPONENT_OPTIONS
            .iter()
            .map(|_| self.reserve_temp_local())
            .collect();
        let fractional_local = self.reserve_temp_local();

        // ECMA-402 11.1.1 steps 1-2. The shared prototype resolver preserves
        // the current plain-call fallback and observes an explicit
        // NewTarget.prototype exactly once. The reserved object remains
        // unreachable until its complete record and brand are installed.
        let reserved_object = self.emit_reserve_intl_date_time_format_object(function)?;

        // CreateDateTimeFormat step 2: CanonicalizeLocaleList. Every tag is
        // validated even though negotiation always lands on `en-US`, because
        // an invalid tag is a RangeError the caller can observe.
        self.emit_builtin_arg_to_locals(0, locales_payload_local, locales_tag_local, function);
        self.emit_intl_dtf_canonicalize_locale_list(
            locales_payload_local,
            locales_tag_local,
            locale_local,
            matched_tag_local,
            function,
        )?;

        // Step 3: CoerceOptionsToObject.
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(options_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        // Steps 5, 7, 10: localeMatcher, calendar, numberingSystem.
        self.emit_intl_dtf_validate_only_option(
            options_payload_local,
            options_tag_local,
            "localeMatcher",
            &["lookup", "best fit"],
            function,
        )?;
        self.emit_intl_dtf_relevant_extension_key_option(
            options_payload_local,
            options_tag_local,
            IntlDtfRelevantExtensionKey::Ca,
            calendar_option_row_local,
            function,
        )?;
        self.emit_intl_dtf_relevant_extension_key_option(
            options_payload_local,
            options_tag_local,
            IntlDtfRelevantExtensionKey::Nu,
            numbering_system_option_row_local,
            function,
        )?;

        // Steps 13-14: hour12 then hourCycle. Reading hour12 first is
        // observable; a present hour12 discards hourCycle entirely.
        self.emit_intl_dtf_hour12_option(
            options_payload_local,
            options_tag_local,
            hour12_local,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_HOUR_CYCLE_OPTION,
            hour_cycle_local,
            None,
            function,
        )?;
        // The `hc` options value, as a row of `Hc.accepted()`. Step 15: a
        // present `hour12` sets `hourCycle` to **null**, not to `undefined`.
        // The distinction is load-bearing rather than pedantic: `hc`'s
        // `[[LocaleData]]` list is « null, "h11", "h12", "h23", "h24" »
        // (11.2.3), so `null` is a value the list *contains*, and step 9(i)(iv)
        // therefore lets it discard a `-u-hc-` keyword and fall back to the
        // default. `undefined` — neither option present — leaves the keyword
        // standing. Both halves are asserted by
        // `resolvedOptions/resolved-locale-with-hc-unicode.js`.
        self.emit_dtf_set_const(hour_cycle_option_row_local, 0, function);
        for (index, (_, code)) in INTL_DTF_HOUR_CYCLE_OPTION.codes.iter().enumerate() {
            self.emit_dtf_if_code_eq(hour_cycle_local, *code, function);
            self.emit_dtf_set_const(
                hour_cycle_option_row_local,
                IntlDtfRelevantExtensionKey::Hc.canonical_row(index) as i64 + 1,
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(hour12_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(
            hour_cycle_option_row_local,
            INTL_DTF_EXTENSION_OPTION_NULL,
            function,
        );
        function.instruction(&Instruction::End);

        // The three relevant-extension-keys, each paired once with the local
        // its options value arrived in and the local `result.[[<key>]]` has to
        // land in. Built here and consumed by both the resolution loop below
        // and the record store further down, so a key cannot be read through
        // one slot and reflected through another.
        let extension_key_locals: [(IntlDtfRelevantExtensionKey, u32, u32); 3] =
            IntlDtfRelevantExtensionKey::ALL.map(|key| {
                let (option_row_local, dest_local) = match key {
                    IntlDtfRelevantExtensionKey::Ca => (calendar_option_row_local, calendar_local),
                    IntlDtfRelevantExtensionKey::Hc => {
                        (hour_cycle_option_row_local, hour_cycle_local)
                    }
                    IntlDtfRelevantExtensionKey::Nu => {
                        (numbering_system_option_row_local, numbering_system_local)
                    }
                };
                (key, option_row_local, dest_local)
            });

        // ECMA-402 9.2.7 `ResolveLocale` step 9, once per relevant-extension-key
        // in `ALL` order. Every options value has already been read in the
        // observable order the specification prescribes; the extension read
        // itself touches no user code, so running the three keys together here
        // is not a reordering a caller can see.
        self.emit_dtf_set_const(extension_open_local, 0, function);
        for (key, option_row_local, dest_local) in extension_key_locals.iter().copied() {
            self.emit_intl_dtf_resolve_extension_key(
                key,
                matched_tag_local,
                option_row_local,
                extension_value_row_local,
                extension_addition_row_local,
                function,
            );
            match key.resolution() {
                IntlDtfExtensionResolution::CanonicalString { default } => {
                    self.emit_dtf_set_string(dest_local, default, function);
                    for (index, (_, canonical)) in key.accepted().iter().enumerate() {
                        // Only canonical rows are reachable: both readers
                        // report `canonical_row`, so an alias row would be dead
                        // code that still cost emitted bytes.
                        if key.canonical_row(index) != index {
                            continue;
                        }
                        self.emit_dtf_if_code_eq(
                            extension_value_row_local,
                            index as i64 + 1,
                            function,
                        );
                        self.emit_dtf_set_string(dest_local, canonical, function);
                        function.instruction(&Instruction::End);
                    }
                }
                IntlDtfExtensionResolution::HourCycleCode => {
                    // The default is `null`, which the record spells 0; steps
                    // 32-35 below turn that into the `en` default.
                    self.emit_dtf_set_const(dest_local, 0, function);
                    for (index, (_, code)) in INTL_DTF_HOUR_CYCLE_OPTION.codes.iter().enumerate() {
                        self.emit_dtf_if_code_eq(
                            extension_value_row_local,
                            index as i64 + 1,
                            function,
                        );
                        self.emit_dtf_set_const(dest_local, *code, function);
                        function.instruction(&Instruction::End);
                    }
                }
            }
            // Step 10 with `InsertUnicodeExtensionAndCanonicalize`: the
            // keywords that were actually used are spelled back into
            // `[[Locale]]` under a single `-u-` singleton, in canonical key
            // order. This is deliberately outside the hourCycle-specific `if`
            // it used to live in — that placement is what made
            // `new Intl.DateTimeFormat("en-u-hc-h23", { hourCycle: "h23" })`
            // report `en`.
            self.emit_dtf_if_nonzero(extension_addition_row_local, function);
            function.instruction(&Instruction::LocalGet(extension_open_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_string(
                scratch_suffix_local,
                INTL_DTF_UNICODE_EXTENSION_SINGLETON,
                function,
            );
            self.emit_concat_string_payloads_local(locale_local, scratch_suffix_local, function)?;
            function.instruction(&Instruction::LocalSet(locale_local));
            self.emit_dtf_set_const(extension_open_local, 1, function);
            function.instruction(&Instruction::End);
            for (index, (_, canonical)) in key.accepted().iter().enumerate() {
                if key.canonical_row(index) != index {
                    continue;
                }
                self.emit_dtf_if_code_eq(extension_addition_row_local, index as i64 + 1, function);
                self.emit_dtf_set_string(
                    scratch_suffix_local,
                    &format!("-{}-{}", key.key(), canonical),
                    function,
                );
                self.emit_concat_string_payloads_local(
                    locale_local,
                    scratch_suffix_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(locale_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
        }

        // Steps 29-31: timeZone, as an identifier and an offset together. The
        // reserved triple is consumed here and comes back resolved; there is no
        // other way to obtain a value `store` will accept.
        let time_zone = self.emit_intl_dtf_time_zone_option(
            options_payload_local,
            options_tag_local,
            time_zone,
            function,
        )?;

        // Step 36: Table 7, in table order, with fractionalSecondDigits
        // spliced in after `second`.
        self.emit_dtf_set_const(explicit_local, 0, function);
        self.emit_dtf_set_const(defaults_cleared_local, 0, function);
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            self.emit_intl_dtf_string_option(
                options_payload_local,
                options_tag_local,
                option,
                *dest_local,
                Some(present_local),
                function,
            )?;
            self.emit_intl_dtf_note_component_present(
                &purpose,
                option.property,
                explicit_local,
                defaults_cleared_local,
                present_local,
                function,
            );
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                self.emit_intl_dtf_fractional_second_digits_option(
                    options_payload_local,
                    options_tag_local,
                    fractional_local,
                    present_local,
                    function,
                )?;
                self.emit_intl_dtf_note_component_present(
                    &purpose,
                    "fractionalSecondDigits",
                    explicit_local,
                    defaults_cleared_local,
                    present_local,
                    function,
                );
            }
        }

        // Step 37: formatMatcher, validated and discarded.
        self.emit_intl_dtf_validate_only_option(
            options_payload_local,
            options_tag_local,
            "formatMatcher",
            &["basic", "best fit"],
            function,
        )?;

        // Steps 38-40: dateStyle then timeStyle.
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_DATE_STYLE_OPTION,
            date_style_local,
            None,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_TIME_STYLE_OPTION,
            time_style_local,
            None,
            function,
        )?;

        // Step 42: a style and an explicit component cannot be combined.
        function.instruction(&Instruction::LocalGet(date_style_local));
        function.instruction(&Instruction::LocalGet(time_style_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(explicit_local));
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
            let style_local = match style {
                RejectedDateLocaleStyle::Date => date_style_local,
                RejectedDateLocaleStyle::Time => time_style_local,
            };
            self.emit_dtf_if_nonzero(style_local, function);
            self.emit_throw_current_function_realm_type_error(
                message,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        // Only fields required by this caller clear defaults. Temporal values
        // use the constructor's retained bit to select their own field defaults
        // at format time; Date methods format the snapshotted Number directly.
        function.instruction(&Instruction::LocalGet(date_style_local));
        function.instruction(&Instruction::LocalGet(time_style_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(defaults_cleared_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(need_defaults_local));
        self.emit_dtf_if_nonzero(need_defaults_local, function);
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            if purpose.default_components().contains(&option.property) {
                self.emit_dtf_set_const(*dest_local, 2, function);
            }
        }
        function.instruction(&Instruction::End);

        self.emit_intl_dtf_resolve_hour_cycle(hour12_local, hour_cycle_local, function);

        self.emit_heap_alloc_const(HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));

        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_LOCALE_OFFSET,
            locale_local,
            function,
        );
        // `[[Calendar]]`, `[[HourCycle]]` and `[[NumberingSystem]]` land in the
        // slots their own key names, from the same pairing the resolver used.
        for (key, _, dest_local) in extension_key_locals.iter().copied() {
            self.store_i64_local_at_offset(record_local, key.slot_offset(), dest_local, function);
        }
        time_zone.store(self, record_local, function);
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_HOUR12_OFFSET,
            hour12_local,
            function,
        );
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            self.store_i64_local_at_offset(record_local, option.slot_offset, *dest_local, function);
        }
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
            fractional_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_DATE_STYLE_OFFSET,
            date_style_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            time_style_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
            need_defaults_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
            0,
            function,
        );
        let initialized_object = self.emit_initialize_intl_date_time_format_object(
            reserved_object,
            record_local,
            function,
        );
        self.emit_publish_intl_date_time_format_object(initialized_object, function);

        self.release_temp_local(fractional_local);
        for local in component_locals.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [
            record_local,
            time_style_local,
            date_style_local,
            present_local,
            need_defaults_local,
            defaults_cleared_local,
            explicit_local,
            numbering_system_option_row_local,
            numbering_system_local,
            calendar_option_row_local,
            calendar_local,
        ] {
            self.release_temp_local(local);
        }
        time_zone.release(self);
        for local in [
            hour_cycle_option_row_local,
            hour_cycle_local,
            hour12_local,
            scratch_suffix_local,
            extension_open_local,
            extension_addition_row_local,
            extension_value_row_local,
            matched_tag_local,
            locale_local,
            options_tag_local,
            options_payload_local,
            locales_tag_local,
            locales_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Folds one component's presence into the constructor's two bits.
    ///
    /// `explicit_local` is `hasExplicitFormatComponents`, which every Table 7
    /// property feeds; `defaults_cleared_local` is the narrower step-40/41
    /// question, which `era` and `timeZoneName` do not answer. Routing both
    /// through one function is what stops a later reader from "simplifying"
    /// them back into a single OR.
    fn emit_intl_dtf_note_component_present(
        &self,
        purpose: &IntlDateTimeFormatPurpose,
        property: &str,
        explicit_local: u32,
        defaults_cleared_local: u32,
        present_local: u32,
        function: &mut Function,
    ) {
        for dest_local in [
            Some(explicit_local),
            purpose
                .required_components()
                .contains(&property)
                .then_some(defaults_cleared_local),
        ]
        .into_iter()
        .flatten()
        {
            function.instruction(&Instruction::LocalGet(dest_local));
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(dest_local));
        }
    }
}
