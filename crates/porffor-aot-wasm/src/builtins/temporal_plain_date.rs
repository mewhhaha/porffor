//! `Temporal.PlainDate` codegen.
//!
//! Temporal proposal 3: a calendar date with no time and no time zone. The
//! record is three plain `i64` ISO fields plus an interned calendar payload —
//! `RejectISODate` bounds every field, so nothing here needs the BigInt
//! machinery the epoch-nanosecond types carry.
//!
//! Only the ISO 8601 calendar exists in this backend (see
//! `emit_temporal_plain_date_calendar`), so `era`/`eraYear` are always
//! `undefined` and `monthsInYear` is always 12.

use super::super::*;

/// `ISODateToEpochDays` bounds from `ISODateWithinLimits`: noon on the day must
/// stay inside `nsMinInstant - nsPerDay` .. `nsMaxInstant + nsPerDay`, which
/// works out to one more day below the epoch-day limit than above it.
/// `-271821-04-19` and `+275760-09-13` are the exact endpoints Test262's
/// `PlainDate/limits.js` pins.
const TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY: i64 = -100_000_001;
const TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY: i64 = 100_000_000;

impl<'a> FunctionBuilder<'a> {
    /// `ToIntegerWithTruncation`: `ToNumber`, reject NaN and the infinities with
    /// a RangeError, then truncate toward zero.
    pub(crate) fn emit_temporal_to_integer_with_truncation(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        output_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(output_local));
        Ok(())
    }

    /// `ToTemporalCalendarIdentifier`. `undefined` defaults to `iso8601`; a
    /// non-string throws a TypeError; anything but a case-insensitive `iso8601`
    /// throws a RangeError, because this backend ships no other calendar.
    pub(crate) fn emit_temporal_plain_date_calendar(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
        function.instruction(&Instruction::LocalSet(expected_payload_local));
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            calendar_payload_local,
            expected_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainDate calendar",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    /// Leaves an `i32` on the stack: 1 when the ISO year is a leap year.
    pub(crate) fn emit_temporal_iso_year_is_leap_i32(
        &mut self,
        year_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
    }

    /// `ISODaysInMonth` into `output_local`.
    pub(crate) fn emit_temporal_iso_days_in_month(
        &mut self,
        year_local: u32,
        month_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(output_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_iso_year_is_leap_i32(year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
    }

    /// `ISODateToEpochDays` into `days_local`, reserving and releasing the
    /// scratch locals `emit_temporal_days_from_civil` needs.
    pub(crate) fn emit_temporal_plain_date_epoch_days(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        let adjusted_year_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        self.emit_temporal_days_from_civil(
            year_local,
            month_local,
            day_local,
            adjusted_year_local,
            era_local,
            month_index_local,
            days_local,
            function,
        );
        self.release_temp_local(month_index_local);
        self.release_temp_local(era_local);
        self.release_temp_local(adjusted_year_local);
    }

    /// `RejectISODate` followed by the `ISODateWithinLimits` check that
    /// `CreateTemporalDate` performs. Both failures are RangeErrors, so the
    /// two are fused into one guard.
    pub(crate) fn emit_temporal_reject_iso_date(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();

        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDate is not a valid ISO date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDate is outside the supported date range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(days_local);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_temporal_plain_date(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(record_local, offset, local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// The `[[InitializedTemporalDate]]` brand check. On failure it throws and
    /// returns, so callers may assume `record_local` is a live record after it.
    pub(crate) fn emit_temporal_plain_date_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate receiver does not have [[InitializedTemporalDate]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate receiver does not have [[InitializedTemporalDate]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_temporal_plain_date_load_record(
        &mut self,
        record_local: u32,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
    }

    /// Temporal proposal 3.1: `Temporal.PlainDate(isoYear, isoMonth, isoDay [, calendar])`.
    pub(crate) fn emit_temporal_plain_date_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, output_local, message) in [
            (0, year_local, "Temporal.PlainDate year must be an integer"),
            (
                1,
                month_local,
                "Temporal.PlainDate month must be an integer",
            ),
            (2, day_local, "Temporal.PlainDate day must be an integer"),
        ] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_temporal_to_integer_with_truncation(
                argument_payload_local,
                argument_tag_local,
                output_local,
                message,
                function,
            )?;
        }
        self.emit_builtin_arg_to_locals(3, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            new_target_tag_local,
            new_target_payload_local,
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Every `Temporal.PlainDate.prototype` accessor. They all start from the
    /// same three ISO fields, so one emitter serves the family the way
    /// `emit_temporal_zoned_date_time_iso_field` does for ZonedDateTime.
    pub(crate) fn emit_temporal_plain_date_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        self.emit_temporal_plain_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter => {
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 2_i64..=12 {
                    function.instruction(&Instruction::LocalGet(month_local));
                    function.instruction(&Instruction::I64Const(month));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(&format!("M{month:02}")),
                    ));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            // The ISO 8601 calendar has no eras, so both slots are `undefined`
            // rather than a fabricated "ce"/"bce" pair.
            StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                let value_local = self.reserve_temp_local();
                self.emit_temporal_plain_date_numeric_field(
                    builtin,
                    year_local,
                    month_local,
                    day_local,
                    value_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(value_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(value_local);
            }
        }

        for local in [
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The purely-numeric accessors. Split out of `emit_temporal_plain_date_field`
    /// so the calendar-arithmetic locals are only reserved when a caller needs
    /// them.
    pub(crate) fn emit_temporal_plain_date_numeric_field(
        &mut self,
        builtin: StandardBuiltinId,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeYearGetter => {
                function.instruction(&Instruction::LocalGet(year_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter => {
                function.instruction(&Instruction::LocalGet(month_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayGetter => {
                function.instruction(&Instruction::LocalGet(day_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter => {
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter => {
                function.instruction(&Instruction::I64Const(12));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter => {
                self.emit_temporal_iso_days_in_month(
                    year_local,
                    month_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(366));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(365));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter => {
                self.emit_temporal_plain_date_day_of_week(
                    year_local,
                    month_local,
                    day_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter => {
                self.emit_temporal_plain_date_day_of_year(
                    year_local,
                    month_local,
                    day_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter => {
                let week_local = self.reserve_temp_local();
                let year_of_week_local = self.reserve_temp_local();
                self.emit_temporal_plain_date_iso_week(
                    year_local,
                    month_local,
                    day_local,
                    week_local,
                    year_of_week_local,
                    function,
                );
                let selected = if matches!(
                    builtin,
                    StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
                ) {
                    week_local
                } else {
                    year_of_week_local
                };
                function.instruction(&Instruction::LocalGet(selected));
                function.instruction(&Instruction::LocalSet(output_local));
                self.release_temp_local(year_of_week_local);
                self.release_temp_local(week_local);
            }
            _ => unreachable!("non-numeric Temporal.PlainDate accessor"),
        }
    }

    /// ISO weekday, 1 = Monday .. 7 = Sunday. Epoch day 0 is a Thursday, so the
    /// `+3` shift lands Monday on 0 before the floor-mod.
    fn emit_temporal_plain_date_day_of_week(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let days_local = self.reserve_temp_local();
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(days_local));
        // `I64RemS` truncates toward zero, so a pre-epoch date would yield a
        // negative remainder; the `+7 % 7` restores floor semantics.
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(days_local);
    }

    fn emit_temporal_plain_date_day_of_year(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let days_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            one_local,
            one_local,
            start_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(one_local);
        self.release_temp_local(start_local);
        self.release_temp_local(days_local);
    }

    /// Leaves an `i32` on the stack: 1 when ISO year `year_local` has 53 weeks.
    /// A year is long when 1 January is a Thursday, or when it is a leap year
    /// starting on a Wednesday.
    fn emit_temporal_plain_date_year_is_long_i32(
        &mut self,
        year_local: u32,
        function: &mut Function,
    ) {
        let january_first_weekday_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        self.emit_temporal_plain_date_day_of_week(
            year_local,
            one_local,
            one_local,
            january_first_weekday_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(january_first_weekday_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(january_first_weekday_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        self.emit_temporal_iso_year_is_leap_i32(year_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.release_temp_local(one_local);
        self.release_temp_local(january_first_weekday_local);
    }

    /// ISO 8601 week-of-year and its week-numbering year.
    fn emit_temporal_plain_date_iso_week(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        week_local: u32,
        year_of_week_local: u32,
        function: &mut Function,
    ) {
        let day_of_year_local = self.reserve_temp_local();
        let day_of_week_local = self.reserve_temp_local();
        let adjacent_year_local = self.reserve_temp_local();
        let weeks_in_year_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_day_of_year(
            year_local,
            month_local,
            day_local,
            day_of_year_local,
            function,
        );
        self.emit_temporal_plain_date_day_of_week(
            year_local,
            month_local,
            day_local,
            day_of_week_local,
            function,
        );
        // `dayOfYear >= 1` and `dayOfWeek <= 7`, so the dividend is at least 4
        // and the truncating `I64DivS` already floors.
        function.instruction(&Instruction::LocalGet(day_of_year_local));
        function.instruction(&Instruction::LocalGet(day_of_week_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalSet(year_of_week_local));

        function.instruction(&Instruction::LocalGet(week_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(adjacent_year_local));
        function.instruction(&Instruction::LocalGet(adjacent_year_local));
        function.instruction(&Instruction::LocalSet(year_of_week_local));
        self.emit_temporal_plain_date_year_is_long_i32(adjacent_year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(53));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_date_year_is_long_i32(year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(53));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(weeks_in_year_local));
        function.instruction(&Instruction::LocalGet(week_local));
        function.instruction(&Instruction::LocalGet(weeks_in_year_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_of_week_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(weeks_in_year_local);
        self.release_temp_local(adjacent_year_local);
        self.release_temp_local(day_of_week_local);
        self.release_temp_local(day_of_year_local);
    }
}
