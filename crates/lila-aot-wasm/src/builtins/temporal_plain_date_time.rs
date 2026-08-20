//! `Temporal.PlainDateTime` codegen: record layout, validation, constructor and
//! the twenty-two accessors.
//!
//! Temporal proposal 5. A `PlainDateTime` is exactly a `PlainDate` glued to a
//! `PlainTime`, so this type owns almost no arithmetic of its own: the three
//! ISO date fields reuse `temporal_plain_date.rs` (`RejectISODate`,
//! `ISODaysInMonth`, the week/day-of-year accessors) and the six wall-clock
//! fields reuse `temporal_plain_time.rs` (`RejectTime`, `RegulateTime`, the
//! nanosecond-of-day scalar). The one genuinely new primitive here is
//! `emit_temporal_civil_from_days`, the inverse of `emit_temporal_days_from_civil`,
//! which date arithmetic needs to turn an epoch-day count back into a civil
//! date.
//!
//! The calendar is likewise `temporal_plain_date.rs`'s: `era`/`eraYear` are the
//! shared `emit_temporal_calendar_era_field`, so a `gregory` `PlainDateTime`
//! and a `gregory` `PlainDate` cannot disagree about the year-0 boundary.

use super::super::*;
use super::temporal_plain_date::TemporalEraField;

/// The first epoch day a `PlainDateTime` may name. Equal to
/// `TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY`: `PlainDate` may hold the whole day,
/// but `PlainDateTime` may not hold its midnight.
const TEMPORAL_PLAIN_DATE_TIME_MINIMUM_EPOCH_DAY: i64 = -100_000_001;

/// Field order: the constructor argument order, and the order the fields are
/// written into the record. Indices 0..3 are the date, 3..9 the time.
pub(crate) const TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS: [u64; 9] = [
    HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
    HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,
];

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn reserve_temporal_plain_date_time_field_locals(&mut self) -> [u32; 9] {
        let mut locals = [0_u32; 9];
        for slot in locals.iter_mut() {
            *slot = self.reserve_temp_local();
        }
        locals
    }

    pub(crate) fn release_temporal_plain_date_time_field_locals(&mut self, locals: [u32; 9]) {
        for local in locals.iter().rev() {
            self.release_temp_local(*local);
        }
    }

    /// The six time locals of a `PlainDateTime` field array, in the shape the
    /// `Temporal.PlainTime` helpers expect.
    pub(crate) fn temporal_plain_date_time_time_locals(field_locals: &[u32; 9]) -> [u32; 6] {
        [
            field_locals[3],
            field_locals[4],
            field_locals[5],
            field_locals[6],
            field_locals[7],
            field_locals[8],
        ]
    }

    /// Howard Hinnant's `civil_from_days`, the inverse of
    /// `emit_temporal_days_from_civil`. Every intermediate after `era` is
    /// non-negative, so the unsigned divisions are exact.
    pub(crate) fn emit_temporal_civil_from_days(
        &mut self,
        days_local: u32,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) {
        let shifted_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let day_of_era_local = self.reserve_temp_local();
        let year_of_era_local = self.reserve_temp_local();
        let day_of_year_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(719_468));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shifted_local));
        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::I64Const(146_096));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(146_097));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(era_local));

        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(146_097));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day_of_era_local));

        function.instruction(&Instruction::LocalGet(day_of_era_local));
        function.instruction(&Instruction::LocalGet(day_of_era_local));
        function.instruction(&Instruction::I64Const(1_460));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(day_of_era_local));
        function.instruction(&Instruction::I64Const(36_524));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(day_of_era_local));
        function.instruction(&Instruction::I64Const(146_096));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(year_of_era_local));

        function.instruction(&Instruction::LocalGet(year_of_era_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_local));

        function.instruction(&Instruction::LocalGet(day_of_era_local));
        function.instruction(&Instruction::LocalGet(year_of_era_local));
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(year_of_era_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(year_of_era_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day_of_year_local));

        function.instruction(&Instruction::LocalGet(day_of_year_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(153));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(month_index_local));

        function.instruction(&Instruction::LocalGet(day_of_year_local));
        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(153));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(day_local));

        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(month_local));

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_local));

        for local in [
            month_index_local,
            day_of_year_local,
            year_of_era_local,
            day_of_era_local,
            era_local,
            shifted_local,
        ] {
            self.release_temp_local(local);
        }
    }

    pub(crate) fn emit_alloc_temporal_plain_date_time(
        &mut self,
        field_locals: &[u32; 9],
        calendar_payload_local: u32,
        prototype_payload_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // `CreateTemporalDateTime` runs `ISODateTimeWithinLimits`. Every caller
        // reaches here with the final ISO fields, so validating once here keeps
        // `with`, `withPlainTime`, `round` and friends from minting the one
        // midnight the day-range check cannot reject.
        self.emit_temporal_reject_date_time_lower_bound(field_locals, function)?;
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let prototype_local = match prototype_payload_local {
            Some(local) => local,
            None => {
                let local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(
                    TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(local));
                local
            }
        };
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, local) in TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS
            .iter()
            .zip(field_locals.iter())
        {
            self.store_i64_local_at_offset(record_local, *offset, *local, function);
        }
        self.store_i64_local_at_offset(
            record_local,
            HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
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
        if prototype_payload_local.is_none() {
            self.release_temp_local(prototype_local);
        }
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// Leaves an `i32` on the stack: 1 when the value carries
    /// `[[InitializedTemporalDateTime]]`.
    pub(crate) fn emit_temporal_plain_date_time_brand_check_i32(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        brand_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(brand_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
    }

    pub(crate) fn emit_temporal_plain_date_time_load_record(
        &mut self,
        record_local: u32,
        field_locals: &[u32; 9],
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        for (offset, local) in TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS
            .iter()
            .zip(field_locals.iter())
        {
            self.load_i64_to_local_from_offset(record_local, *offset, *local, function);
        }
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );
    }

    /// The `[[InitializedTemporalDateTime]]` brand check on `this`, leaving the
    /// nine fields and the calendar loaded. On failure it throws and returns.
    pub(crate) fn emit_temporal_plain_date_time_fields_from_receiver(
        &mut self,
        field_locals: &[u32; 9],
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        self.emit_temporal_plain_date_time_brand_check_i32(
            receiver_payload_local,
            receiver_tag_local,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime receiver does not have [[InitializedTemporalDateTime]]",
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
        self.emit_temporal_plain_date_time_load_record(
            record_local,
            field_locals,
            calendar_payload_local,
            function,
        );

        for local in [
            record_local,
            receiver_brand_local,
            receiver_tag_local,
            receiver_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `RejectDateTime`: `RejectISODate` on the date half (which also runs the
    /// `ISODateTimeWithinLimits` day-range check) and `RejectTime` on the time
    /// half.
    pub(crate) fn emit_temporal_reject_date_time(
        &mut self,
        field_locals: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_reject_iso_date(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            function,
        )?;
        let time_locals = Self::temporal_plain_date_time_time_locals(field_locals);
        self.emit_temporal_reject_time(&time_locals, function)?;
        self.emit_temporal_reject_date_time_lower_bound(field_locals, function)
    }

    /// The nanosecond the day-range check cannot see. `ISODateTimeWithinLimits`
    /// rejects `ns <= nsMinInstant - nsPerDay`, and that bound lands exactly on
    /// `-271821-04-19T00:00:00`. The day itself is inside the `RejectISODate`
    /// range (`PlainDate` may hold it), so only the midnight instant on the
    /// first representable day is out of range; every later time that day is
    /// fine. The upper bound needs no companion check: `+275760-09-14` is
    /// already outside the day range, and `nsMaxInstant + nsPerDay` is that
    /// day's midnight.
    pub(crate) fn emit_temporal_reject_date_time_lower_bound(
        &mut self,
        field_locals: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let days_local = self.reserve_temp_local();
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_TIME_MINIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64Eq);
        for time_local in Self::temporal_plain_date_time_time_locals(field_locals) {
            function.instruction(&Instruction::LocalGet(time_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDateTime is outside the supported date range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(days_local);
        Ok(())
    }

    /// Temporal proposal 5.1: `Temporal.PlainDateTime(isoYear, isoMonth, isoDay
    /// [, hour [, minute [, second [, millisecond [, microsecond [, nanosecond
    /// [, calendar]]]]]]])`.
    pub(crate) fn emit_temporal_plain_date_time_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

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
            "Temporal.PlainDateTime constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for index in 0..9_usize {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            if index >= 3 {
                function.instruction(&Instruction::LocalGet(argument_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(field_locals[index]));
                function.instruction(&Instruction::Else);
            }
            self.emit_temporal_to_integer_with_truncation(
                argument_payload_local,
                argument_tag_local,
                field_locals[index],
                "Temporal.PlainDateTime field must be an integer",
                function,
            )?;
            if index >= 3 {
                function.instruction(&Instruction::End);
            }
        }
        self.emit_builtin_arg_to_locals(9, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_temporal_reject_date_time(&field_locals, function)?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            Some(prototype_payload_local),
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            new_target_tag_local,
            new_target_payload_local,
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Every `Temporal.PlainDateTime.prototype` accessor. The date-derived ones
    /// delegate to the `Temporal.PlainDate` emitters, so the calendar
    /// arithmetic lives in exactly one place.
    pub(crate) fn emit_temporal_plain_date_time_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;

        match builtin {
            StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter => {
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 2_i64..=12 {
                    function.instruction(&Instruction::LocalGet(field_locals[1]));
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
            StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    field_locals[0],
                    TemporalEraField::Era,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    field_locals[0],
                    TemporalEraField::EraYear,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(field_locals[0], function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                let value_local = self.reserve_temp_local();
                let time_index = match builtin {
                    StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter => Some(3),
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter => Some(4),
                    StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter => Some(5),
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter => Some(6),
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter => Some(7),
                    StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter => Some(8),
                    _ => None,
                };
                match time_index {
                    Some(index) => {
                        function.instruction(&Instruction::LocalGet(field_locals[index]));
                        function.instruction(&Instruction::LocalSet(value_local));
                    }
                    None => {
                        let date_builtin = Self::temporal_plain_date_time_date_accessor(builtin);
                        self.emit_temporal_plain_date_numeric_field(
                            date_builtin,
                            field_locals[0],
                            field_locals[1],
                            field_locals[2],
                            value_local,
                            function,
                        );
                    }
                }
                function.instruction(&Instruction::LocalGet(value_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(value_local);
            }
        }

        self.release_temporal_plain_date_time_field_locals(field_locals);
        self.release_temp_local(calendar_payload_local);
        Ok(())
    }

    /// The `Temporal.PlainDate` accessor that computes the same value from the
    /// same three ISO fields.
    fn temporal_plain_date_time_date_accessor(builtin: StandardBuiltinId) -> StandardBuiltinId {
        match builtin {
            StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
            }
            StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter => {
                StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter
            }
            _ => unreachable!("non-accessor Temporal.PlainDateTime builtin"),
        }
    }

    /// Temporal deliberately forbids implicit comparison, so `valueOf` always
    /// throws.
    pub(crate) fn emit_temporal_plain_date_time_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime does not support implicit conversion; use compare() or equals()",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
