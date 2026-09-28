//! AOT calls into the pinned Temporal calendar kernel.
//!
//! All observable reads and conversions happen in generated Wasm first. This
//! module only serializes already-read primitive locals, invokes host op 17,
//! and turns a calendar RangeError answer into the current realm's error.

use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, TemporalCalendar, TemporalCalendarEra,
    TemporalCalendarQueryKind, TemporalCalendarRangeError, TEMPORAL_CALENDAR_FROM_FIELDS_DAY,
    TEMPORAL_CALENDAR_FROM_FIELDS_ERA, TEMPORAL_CALENDAR_FROM_FIELDS_ERA_YEAR,
    TEMPORAL_CALENDAR_FROM_FIELDS_MONTH, TEMPORAL_CALENDAR_FROM_FIELDS_MONTH_CODE,
    TEMPORAL_CALENDAR_FROM_FIELDS_YEAR, TEMPORAL_CALENDAR_REQUEST_BYTES,
    TEMPORAL_CALENDAR_REQUEST_CALENDAR_OFFSET, TEMPORAL_CALENDAR_RESPONSE_BYTES,
    TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_MONTH_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_YEAR_OFFSET, TEMPORAL_CALENDAR_RESPONSE_DAY_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_DAY_OF_YEAR_OFFSET, TEMPORAL_CALENDAR_RESPONSE_ERA_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_ERA_YEAR_OFFSET, TEMPORAL_CALENDAR_RESPONSE_IN_LEAP_YEAR_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_ISO_DAY_OFFSET, TEMPORAL_CALENDAR_RESPONSE_ISO_MONTH_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_ISO_YEAR_OFFSET, TEMPORAL_CALENDAR_RESPONSE_MONTHS_IN_YEAR_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_LEAP_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_OFFSET, TEMPORAL_CALENDAR_RESPONSE_MONTH_OFFSET,
    TEMPORAL_CALENDAR_RESPONSE_STATUS_OFFSET, TEMPORAL_CALENDAR_RESPONSE_YEAR_OFFSET,
    TEMPORAL_CALENDAR_STATUS_DATE,
};

use super::super::*;

#[derive(Clone, Copy)]
pub(crate) enum CalendarWord {
    Local(u32),
    Constant(i64),
}

impl CalendarWord {
    const ZERO: Self = Self::Constant(0);
}

/// Every calendar name and month-code spelling the emitters compare against,
/// together with the RangeErrors that may come back from the kernel.
pub(crate) fn temporal_calendar_pool_strings() -> Vec<&'static str> {
    let mut strings = Vec::new();
    for calendar in TemporalCalendar::ALL {
        strings.push(calendar.identifier());
        strings.extend(calendar_aliases(calendar));
    }
    for era in TemporalCalendarEra::ALL {
        strings.push(era.identifier());
    }
    strings.extend([
        "ad",
        "bc",
        "year",
        "era",
        "eraYear",
        "month",
        "monthCode",
        "day",
        "M01",
        "M02",
        "M03",
        "M04",
        "M05",
        "M06",
        "M07",
        "M08",
        "M09",
        "M10",
        "M11",
        "M12",
        "M13",
        "M01L",
        "M02L",
        "M03L",
        "M04L",
        "M05L",
        "M06L",
        "M07L",
        "M08L",
        "M09L",
        "M10L",
        "M11L",
        "M12L",
        "M13L",
    ]);
    strings.extend(
        TemporalCalendarRangeError::ALL
            .into_iter()
            .map(TemporalCalendarRangeError::message),
    );
    strings.extend([
        "Temporal.PlainDate fields require year",
        "Temporal.PlainDate fields require day",
        "Temporal.PlainDate fields require month or monthCode",
        "Temporal.PlainDate month and day must be positive",
        "Invalid Temporal.PlainDate monthCode",
        "Temporal era and eraYear must be provided together",
        "Invalid Temporal era for this calendar",
        "Temporal era and year must agree",
    ]);
    strings
}

pub(crate) const fn calendar_aliases(calendar: TemporalCalendar) -> &'static [&'static str] {
    match calendar {
        TemporalCalendar::IslamicCivil => &["islamicc"],
        TemporalCalendar::Ethioaa => &["ethiopic-amete-alem"],
        _ => &[],
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_store_word(
        &mut self,
        request_local: u32,
        offset: u64,
        word: CalendarWord,
        function: &mut Function,
    ) {
        match word {
            CalendarWord::Local(local) => {
                self.store_i64_local_at_offset(request_local, offset, local, function)
            }
            CalendarWord::Constant(value) => {
                self.store_i64_const_at_offset(request_local, offset, value as u64, function)
            }
        }
    }

    /// Emits one fixed-size Temporal calendar query. `words` is the complete
    /// wire request (including zeroed unused fields); response slots are
    /// loaded only after a successful status.
    fn emit_temporal_calendar_query(
        &mut self,
        calendar_payload_local: u32,
        words: [CalendarWord; 17],
        response_fields: &[(u64, u32)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let import = self.intl_call_import_function_index()?;
        let request_length_local = self.reserve_temp_local();
        let request_local = self.reserve_temp_local();
        let response_length_local = self.reserve_temp_local();
        let response_local = self.reserve_temp_local();
        let status_local = self.reserve_temp_local();
        let wire_calendar_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(wire_calendar_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        for calendar in TemporalCalendar::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(calendar.identifier()),
            ));
            function.instruction(&Instruction::LocalSet(request_length_local));
            self.emit_string_payload_equality_i32(
                calendar_payload_local,
                request_length_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(calendar.wire()));
            function.instruction(&Instruction::LocalSet(wire_calendar_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_local));
            function.instruction(&Instruction::End);
        }
        // Gregorian uses wire 0, so a separate found bit is required. The
        // calendar payload is canonical and this loop is exhaustive; an
        // unknown slot is therefore an internal compiler/record fault.
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            TEMPORAL_CALENDAR_REQUEST_BYTES as i64,
        ));
        function.instruction(&Instruction::LocalSet(request_length_local));
        self.emit_heap_alloc_from_local(request_length_local, function)?;
        function.instruction(&Instruction::LocalSet(request_local));
        self.emit_store_word(
            request_local,
            TEMPORAL_CALENDAR_REQUEST_CALENDAR_OFFSET,
            CalendarWord::Local(wire_calendar_local),
            function,
        );
        for (index, word) in words.into_iter().enumerate() {
            if index == 1 {
                continue;
            }
            self.emit_store_word(request_local, (index * 8) as u64, word, function);
        }

        function.instruction(&Instruction::I64Const(
            TEMPORAL_CALENDAR_RESPONSE_BYTES as i64,
        ));
        function.instruction(&Instruction::LocalSet(response_length_local));
        self.emit_heap_alloc_from_local(response_length_local, function)?;
        function.instruction(&Instruction::LocalSet(response_local));
        function.instruction(&Instruction::I64Const(
            IntlHostOp::QueryTemporalCalendar.wire(),
        ));
        self.emit_pack_string_payload(request_local, request_length_local, function);
        self.emit_pack_string_payload(response_local, response_length_local, function);
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::Written(TEMPORAL_CALENDAR_RESPONSE_BYTES as u32).wire(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            response_local,
            TEMPORAL_CALENDAR_RESPONSE_STATUS_OFFSET,
            status_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(status_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_CALENDAR_STATUS_DATE));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for error in TemporalCalendarRangeError::ALL {
            function.instruction(&Instruction::LocalGet(status_local));
            function.instruction(&Instruction::I64Const(error.status()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                error.message(),
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        for (offset, local) in response_fields {
            self.load_i64_to_local_from_offset(response_local, *offset, *local, function);
        }

        for local in [
            found_local,
            wire_calendar_local,
            status_local,
            response_local,
            response_length_local,
            request_local,
            request_length_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_calendar_fields_query(
        &mut self,
        calendar_payload_local: u32,
        iso_date: [u32; 3],
        response_fields: &[(u64, u32)],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mut words = [CalendarWord::ZERO; 17];
        words[0] = CalendarWord::Constant(TemporalCalendarQueryKind::Fields.wire());
        for (index, local) in iso_date.into_iter().enumerate() {
            words[2 + index] = CalendarWord::Local(local);
        }
        self.emit_temporal_calendar_query(calendar_payload_local, words, response_fields, function)
    }

    /// Resolves fields already read by `PrepareCalendarFields`. The caller
    /// releases the four era locals after its other scratch locals, preserving
    /// the strict LIFO local allocator.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_calendar_from_fields_query(
        &mut self,
        calendar_payload_local: u32,
        year_local: u32,
        year_present_local: u32,
        era_locals: [u32; 4],
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        day_present_local: u32,
        overflow_local: u32,
        iso_result: [u32; 3],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [era_payload_local, era_present_local, era_year_local, era_year_present_local] =
            era_locals;
        let flags_local = self.reserve_temp_local();
        let month_wire_local = self.reserve_temp_local();
        let month_code_wire_local = self.reserve_temp_local();
        let month_code_found_local = self.reserve_temp_local();
        let day_wire_local = self.reserve_temp_local();
        let year_wire_local = self.reserve_temp_local();
        let era_year_wire_local = self.reserve_temp_local();
        let era_wire_local = self.reserve_temp_local();
        let era_found_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let era_pair_present_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(era_present_local));
        function.instruction(&Instruction::LocalGet(era_year_present_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(era_pair_present_local));

        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::LocalGet(era_pair_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate fields require year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(era_present_local));
        function.instruction(&Instruction::LocalGet(era_year_present_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal era and eraYear must be provided together",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate fields require day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate fields require month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Calendar-specific era validation happens after the option read and
        // before month-code/date regulation. The host receives a closed era id
        // only after this calendar-scoped spelling match succeeds.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(era_wire_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(era_found_local));
        for calendar in TemporalCalendar::ALL {
            for era in TemporalCalendarEra::ALL {
                if !era.allowed_in(calendar) {
                    continue;
                }
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(calendar.identifier()),
                ));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    calendar_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(era.identifier()),
                ));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    era_payload_local,
                    expected_payload_local,
                    function,
                );
                if matches!(
                    era,
                    TemporalCalendarEra::Ce | TemporalCalendarEra::JapaneseCe
                ) {
                    function.instruction(&Instruction::I64Const(self.strings.payload("ad")));
                    function.instruction(&Instruction::LocalSet(expected_payload_local));
                    self.emit_string_payload_equality_i32(
                        era_payload_local,
                        expected_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::I32Or);
                } else if matches!(
                    era,
                    TemporalCalendarEra::Bce | TemporalCalendarEra::JapaneseBce
                ) {
                    function.instruction(&Instruction::I64Const(self.strings.payload("bc")));
                    function.instruction(&Instruction::LocalSet(expected_payload_local));
                    self.emit_string_payload_equality_i32(
                        era_payload_local,
                        expected_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::I32Or);
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(era_found_local));
                function.instruction(&Instruction::I64Const(era.wire()));
                function.instruction(&Instruction::LocalSet(era_wire_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(era_pair_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(era_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal era for this calendar",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Parse the already-ToString'd Temporal monthCode using the full M01…
        // M13 and leap suffix domain. Calendar/year suitability is determined
        // by the pinned kernel after this syntax check.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_code_wire_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_code_found_local));
        for month in 1_i64..=13 {
            for leap in [false, true] {
                let spelling = if leap {
                    format!("M{month:02}L")
                } else {
                    format!("M{month:02}")
                };
                function.instruction(&Instruction::I64Const(self.strings.payload(&spelling)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    month_code_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(
                    month | if leap { 1 << 8 } else { 0 },
                ));
                function.instruction(&Instruction::LocalSet(month_code_wire_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(month_code_found_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(month_code_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainDate monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // ToPositiveInteger errors before the host call. Values above the
        // largest possible calendar month/day are saturated at 255 so the
        // fixed i32-compatible wire remains valid and constrain/reject still
        // receives an out-of-domain value.
        for (input, output, present) in [
            (month_local, month_wire_local, month_present_local),
            (day_local, day_wire_local, day_present_local),
        ] {
            function.instruction(&Instruction::LocalGet(present));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(output));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.PlainDate month and day must be positive",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::I64Const(255));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(255));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(output));
            function.instruction(&Instruction::End);
        }

        for (input, output) in [
            (year_local, year_wire_local),
            (era_year_local, era_year_wire_local),
        ] {
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::I64Const(i32::MAX as i64));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(i32::MAX as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::I64Const(i32::MIN as i64));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(i32::MIN as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(output));
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(flags_local));
        for (present, bit) in [
            (year_present_local, TEMPORAL_CALENDAR_FROM_FIELDS_YEAR),
            (month_present_local, TEMPORAL_CALENDAR_FROM_FIELDS_MONTH),
            (
                month_code_present_local,
                TEMPORAL_CALENDAR_FROM_FIELDS_MONTH_CODE,
            ),
            (day_present_local, TEMPORAL_CALENDAR_FROM_FIELDS_DAY),
            (era_present_local, TEMPORAL_CALENDAR_FROM_FIELDS_ERA),
            (
                era_year_present_local,
                TEMPORAL_CALENDAR_FROM_FIELDS_ERA_YEAR,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::LocalGet(present));
            function.instruction(&Instruction::I64Const(bit));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(flags_local));
        }
        let mut words = [CalendarWord::ZERO; 17];
        words[0] = CalendarWord::Constant(TemporalCalendarQueryKind::FromFields.wire());
        words[5] = CalendarWord::Local(year_wire_local);
        words[6] = CalendarWord::Local(month_wire_local);
        words[7] = CalendarWord::Local(month_code_wire_local);
        words[8] = CalendarWord::Local(day_wire_local);
        words[9] = CalendarWord::Local(era_wire_local);
        words[10] = CalendarWord::Local(era_year_wire_local);
        words[11] = CalendarWord::Local(overflow_local);
        words[16] = CalendarWord::Local(flags_local);
        self.emit_temporal_calendar_query(
            calendar_payload_local,
            words,
            &[
                (TEMPORAL_CALENDAR_RESPONSE_ISO_YEAR_OFFSET, iso_result[0]),
                (TEMPORAL_CALENDAR_RESPONSE_ISO_MONTH_OFFSET, iso_result[1]),
                (TEMPORAL_CALENDAR_RESPONSE_ISO_DAY_OFFSET, iso_result[2]),
            ],
            function,
        )?;

        for local in [
            era_pair_present_local,
            expected_payload_local,
            era_found_local,
            era_wire_local,
            era_year_wire_local,
            year_wire_local,
            day_wire_local,
            month_code_found_local,
            month_code_wire_local,
            month_wire_local,
            flags_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_calendar_date_add_query(
        &mut self,
        calendar_payload_local: u32,
        iso_date: [u32; 3],
        date_duration: [u32; 4],
        overflow_local: u32,
        iso_result: [u32; 3],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mut words = [CalendarWord::ZERO; 17];
        words[0] = CalendarWord::Constant(TemporalCalendarQueryKind::DateAdd.wire());
        for (index, local) in iso_date.into_iter().enumerate() {
            words[2 + index] = CalendarWord::Local(local);
        }
        words[11] = CalendarWord::Local(overflow_local);
        for (index, local) in date_duration.into_iter().enumerate() {
            words[12 + index] = CalendarWord::Local(local);
        }
        self.emit_temporal_calendar_query(
            calendar_payload_local,
            words,
            &[
                (TEMPORAL_CALENDAR_RESPONSE_ISO_YEAR_OFFSET, iso_result[0]),
                (TEMPORAL_CALENDAR_RESPONSE_ISO_MONTH_OFFSET, iso_result[1]),
                (TEMPORAL_CALENDAR_RESPONSE_ISO_DAY_OFFSET, iso_result[2]),
            ],
            function,
        )
    }

    /// Reads the calendar-dependent PlainDate accessors from one projection
    /// of the complete ISO date. `field` selects which response word is used.
    pub(crate) fn emit_temporal_plain_date_calendar_field(
        &mut self,
        builtin: StandardBuiltinId,
        calendar_payload_local: u32,
        iso_date: [u32; 3],
        function: &mut Function,
    ) -> Result<bool, EmitError> {
        let field_offset = match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeEraGetter => {
                TEMPORAL_CALENDAR_RESPONSE_ERA_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_YEAR_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_ERA_YEAR_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter => {
                TEMPORAL_CALENDAR_RESPONSE_MONTH_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter => {
                TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayGetter => {
                TEMPORAL_CALENDAR_RESPONSE_DAY_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_DAY_OF_YEAR_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter => {
                TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_MONTH_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_YEAR_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_MONTHS_IN_YEAR_OFFSET
            }
            StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter => {
                TEMPORAL_CALENDAR_RESPONSE_IN_LEAP_YEAR_OFFSET
            }
            _ => return Ok(false),
        };
        let field_local = self.reserve_temp_local();
        let mut outputs = vec![(field_offset, field_local)];
        let second_local = match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => {
                let local = self.reserve_temp_local();
                outputs.push((TEMPORAL_CALENDAR_RESPONSE_ERA_OFFSET, local));
                Some(local)
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter => {
                let local = self.reserve_temp_local();
                outputs.push((TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_LEAP_OFFSET, local));
                Some(local)
            }
            _ => None,
        };
        self.emit_temporal_calendar_fields_query(
            calendar_payload_local,
            iso_date,
            &outputs,
            function,
        )?;
        match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeEraGetter => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                for era in TemporalCalendarEra::ALL {
                    function.instruction(&Instruction::LocalGet(field_local));
                    function.instruction(&Instruction::I64Const(era.wire()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(era.identifier()),
                    ));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    function.instruction(&Instruction::End);
                }
            }
            StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => {
                let era_local = second_local.expect("eraYear projection has era output");
                function.instruction(&Instruction::LocalGet(era_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(field_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter => {
                let leap_local = second_local.expect("monthCode projection has leap output");
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 1_i64..=13 {
                    for leap in [false, true] {
                        function.instruction(&Instruction::LocalGet(field_local));
                        function.instruction(&Instruction::I64Const(month));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::LocalGet(leap_local));
                        function.instruction(&Instruction::I64Const(i64::from(leap)));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::I32And);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        let spelling = if leap {
                            format!("M{month:02}L")
                        } else {
                            format!("M{month:02}")
                        };
                        function
                            .instruction(&Instruction::I64Const(self.strings.payload(&spelling)));
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                }
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter => {
                function.instruction(&Instruction::LocalGet(field_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                function.instruction(&Instruction::LocalGet(field_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
        if let Some(local) = second_local {
            self.release_temp_local(local);
        }
        self.release_temp_local(field_local);
        Ok(true)
    }
}
