//! `Temporal.PlainMonthDay` codegen.
//!
//! Temporal proposal 10: a calendar month and day with a *reference* ISO year
//! that is not observable through any accessor. The record is the
//! `Temporal.PlainDate` one under a different brand, so every layout constant
//! and every ISO helper is shared rather than duplicated; only the field set,
//! the reference year and the string form differ.
//!
//! The reference year is 1972 - a leap year, so `--02-29` is representable.

use super::super::*;
use super::temporal_options::{ShowCalendarName, TemporalOverflow};
use super::temporal_plain_year_month::{TemporalPartialDatePrototype, TemporalPartialDateType};

/// `ISO_REFERENCE_YEAR` from the proposal.
const TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR: i64 = 1972;

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_month_day_overflow_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options_payload_local,
            options_tag_local,
            overflow_local,
            "Temporal.PlainMonthDay options must be an object or undefined",
            "Invalid Temporal.PlainMonthDay overflow option",
            function,
        )
    }

    /// Temporal proposal 10.1.1:
    /// `Temporal.PlainMonthDay(isoMonth, isoDay [, calendar [, referenceISOYear]])`.
    pub(crate) fn emit_temporal_plain_month_day_constructor(
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
            "Temporal.PlainMonthDay constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, output_local, message) in [
            (
                0,
                month_local,
                "Temporal.PlainMonthDay month must be an integer",
            ),
            (
                1,
                day_local,
                "Temporal.PlainMonthDay day must be an integer",
            ),
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
        self.emit_builtin_arg_to_locals(2, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        self.emit_builtin_arg_to_locals(3, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_to_integer_with_truncation(
            argument_payload_local,
            argument_tag_local,
            year_local,
            "Temporal.PlainMonthDay reference year must be an integer",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::FromNewTarget,
            function,
        )?;

        for local in [
            new_target_tag_local,
            new_target_payload_local,
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

    /// The `[[InitializedTemporalMonthDay]]` brand check.
    pub(crate) fn emit_temporal_plain_month_day_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_branded_record_from_receiver(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            "Temporal.PlainMonthDay receiver does not have [[InitializedTemporalMonthDay]]",
            record_local,
            function,
        )
    }

    /// The three `Temporal.PlainMonthDay.prototype` accessors.
    pub(crate) fn emit_temporal_plain_month_day_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter => {
                self.emit_temporal_month_code_payload(month_local, function);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                function.instruction(&Instruction::LocalGet(day_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
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

    /// `RegulateISODate` without the `ISODateWithinLimits` tail. A month-day
    /// stores the reference year 1972, so a caller-supplied `year` only ever
    /// decides how 29 February constrains - Test262's
    /// `from/iso-year-used-only-for-overflow.js` pins that an out-of-range year
    /// must not throw.
    fn emit_temporal_month_day_regulate(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(overflow_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay is not a valid ISO date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    /// `CalendarMonthDayToISOReferenceDate`. The supplied year, if any, decides
    /// how 29 February is constrained; the stored year is always 1972.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_month_day_resolve_fields(
        &mut self,
        year_local: u32,
        year_present_local: u32,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        day_present_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let month_from_code_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay fields require day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay fields require month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // The ISO reference year stands in when the bag carries no `year`. A
        // supplied year is only ever used to pick how 29 February constrains -
        // it is deliberately not range-checked, and never stored.
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_from_code_local));
        for month in 1_i64..=12 {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(expected_payload_local));
            self.emit_string_payload_equality_i32(
                month_code_payload_local,
                expected_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::LocalSet(month_from_code_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainMonthDay monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay month and monthCode must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay month and day must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));

        self.release_temp_local(expected_payload_local);
        self.release_temp_local(month_from_code_local);
        Ok(())
    }

    /// `ToTemporalMonthDay`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_to_temporal_month_day(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        read_options: bool,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));

        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        if read_options {
            self.emit_temporal_month_day_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            year_local,
            year_present_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            true,
            true,
            function,
        )?;
        if read_options {
            self.emit_temporal_month_day_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_month_day_resolve_fields(
            year_local,
            year_present_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay expects a string, a property bag, or a Temporal.PlainMonthDay",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_month_day_string(
            argument_payload_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        if read_options {
            self.emit_temporal_month_day_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::End);

        for local in [
            handled_local,
            day_present_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            year_present_local,
            record_local,
            brand_local,
            overflow_local,
            calendar_tag_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 10.2.2 `Temporal.PlainMonthDay.from`.
    pub(crate) fn emit_temporal_plain_month_day_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_temporal_to_temporal_month_day(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            true,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ParseTemporalMonthDayString`, via the shared bare-form rewrite.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_parse_month_day_string(
        &mut self,
        string_payload_local: u32,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rewritten_local = self.reserve_temp_local();

        self.emit_temporal_month_day_rewrite_string(
            string_payload_local,
            rewritten_local,
            function,
        )?;
        self.emit_temporal_parse_plain_date_string(
            rewritten_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;

        self.release_temp_local(rewritten_local);
        Ok(())
    }

    /// Temporal proposal 10.3.x `equals`.
    pub(crate) fn emit_temporal_plain_month_day_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let other_year_local = self.reserve_temp_local();
        let other_month_local = self.reserve_temp_local();
        let other_day_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_temporal_to_temporal_month_day(
            argument_payload_local,
            argument_tag_local,
            undefined_local,
            undefined_tag_local,
            false,
            other_year_local,
            other_month_local,
            other_day_local,
            other_calendar_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(other_year_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(other_month_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(other_day_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(
            calendar_payload_local,
            other_calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            other_calendar_payload_local,
            other_day_local,
            other_month_local,
            other_year_local,
            undefined_tag_local,
            undefined_local,
            argument_tag_local,
            argument_payload_local,
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

    /// Temporal proposal 10.3.x `with`.
    pub(crate) fn emit_temporal_plain_month_day_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let new_year_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let new_month_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let new_day_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.with requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `IsPartialTemporalObject` step 2 runs before the two `Get`s below.
        self.emit_temporal_reject_branded_partial_object(
            argument_payload_local,
            argument_tag_local,
            "Temporal.PlainMonthDay.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject` reads both keys with `Get`, not with a
        // `HasProperty` probe, and Test262's `with/order-of-operations.js`
        // observes the two reads.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                key_local,
                present_local,
                calendar_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(calendar_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.PlainMonthDay.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            new_year_local,
            year_present_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.with requires at least one field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalSet(new_day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_month_code_payload(month_local, function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_resolve_fields(
            new_year_local,
            year_present_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            new_year_local,
            new_month_local,
            new_day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
            day_present_local,
            new_day_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            new_month_local,
            year_present_local,
            new_year_local,
            present_local,
            key_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_tag_local,
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

    /// `TemporalMonthDayToString`. The reference year is prefixed only when the
    /// calendar annotation is shown, which is the only way a round-trip could
    /// otherwise lose it.
    pub(crate) fn emit_temporal_plain_month_day_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let show_calendar_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::LocalSet(show_calendar_local));
        if matches!(
            builtin,
            StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_string_valued_option::<ShowCalendarName>(
                options_payload_local,
                options_tag_local,
                show_calendar_local,
                "Temporal.PlainMonthDay options must be an object or undefined",
                "Invalid Temporal.PlainMonthDay calendarName option",
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Always.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_pad_iso_year(
            year_local,
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        self.emit_temporal_append_separated_two_digits(
            month_local,
            "-",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_temporal_append_separated_two_digits(
            month_local,
            "",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_append_separated_two_digits(
            day_local,
            "-",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        self.emit_temporal_append_calendar_annotation(
            show_calendar_local,
            calendar_payload_local,
            output_payload_local,
            piece_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            show_calendar_local,
            options_tag_local,
            options_payload_local,
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

    /// Temporal proposal 10.3.x `toPlainDate ( item )`: the receiver's month
    /// and day plus a `year` read from `item`.
    pub(crate) fn emit_temporal_plain_month_day_to_plain_date(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.toPlainDate requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            "Temporal.PlainMonthDay year must be finite",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.toPlainDate requires a year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        self.emit_temporal_plain_date_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            overflow_local,
            present_local,
            value_tag_local,
            value_payload_local,
            key_local,
            argument_tag_local,
            argument_payload_local,
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
}
