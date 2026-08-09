//! `Temporal.PlainYearMonth` statics and prototype methods.
//!
//! Split from `temporal_plain_year_month.rs` (constructor, record, accessors)
//! the way `Temporal.PlainDate` is split; both halves are
//! `impl FunctionBuilder` blocks.

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, TemporalOverflow, TemporalRoundingMode, TemporalUnit, TemporalUnitSlot,
};
use super::temporal_plain_date::{TemporalEraLocals, TemporalResolvedYear};
use super::temporal_plain_year_month::{TemporalPartialDatePrototype, TemporalPartialDateType};

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_year_month_overflow_option(
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
            "Temporal.PlainYearMonth options must be an object or undefined",
            "Invalid Temporal.PlainYearMonth overflow option",
            function,
        )
    }

    /// `PrepareCalendarFields` for the `« year, month, month-code »` key set
    /// plus the era pair, in the alphabetical order the spec pins: `calendar`,
    /// `era`, `eraYear`, `month`, `monthCode`, `year`. There is deliberately no
    /// `day` read — a `Temporal.PlainYearMonth` property bag never has one, and
    /// `intl402/Temporal/PlainYearMonth/from/argument-object.js` hands in a bag
    /// whose `day` getter throws to prove it.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_year_month_read_fields(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        year_local: u32,
        year_present_local: u32,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        read_calendar: bool,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let era_slots = self.reserve_temporal_era_slots();
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();

        if read_calendar {
            function.instruction(&Instruction::I64Const(self.strings.payload("calendar")));
            function.instruction(&Instruction::LocalSet(property_key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                property_key_local,
                calendar_payload_local,
                calendar_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            // Deliberately the PlainDate spelling: `PlainYearMonth` reuses the
            // PlainDate emitters wholesale and the string pool only seeds the
            // PlainDate message for this family. No Test262 case reads it.
            self.emit_temporal_to_temporal_calendar_identifier(
                calendar_payload_local,
                calendar_tag_local,
                "Temporal.PlainDate calendar must be a string",
                function,
            )?;
        }

        let era = self.emit_temporal_read_era_fields(
            era_slots,
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            function,
        )?;

        self.emit_temporal_property_bag_positive_integer(
            argument_payload_local,
            argument_tag_local,
            "month",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            month_local,
            0,
            "Temporal.PlainYearMonth fields must be finite",
            "Temporal.PlainYearMonth month must be positive",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::LocalSet(month_present_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("monthCode")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        self.emit_temporal_month_code_string(
            value_payload_local,
            value_tag_local,
            "Temporal.PlainYearMonth monthCode must be a string",
            "Invalid Temporal.PlainYearMonth monthCode",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            "Temporal.PlainYearMonth fields must be finite",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::LocalSet(year_present_local));

        for local in [
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(era)
    }

    /// `CalendarResolveFields` for the year-month field set, then
    /// `RegulateISODate` with day fixed at the reference day 1.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_year_month_resolve_fields(
        &mut self,
        resolved_year: &TemporalResolvedYear,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year_local = resolved_year.year_local();
        let year_present_local = resolved_year.year_present_local();
        let month_from_code_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainYearMonth fields require year",
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
            "Temporal.PlainYearMonth fields require month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
            "Invalid Temporal.PlainYearMonth monthCode",
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
            "Temporal.PlainYearMonth month and monthCode must agree",
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
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainYearMonth month must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_local));
        self.emit_temporal_year_month_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;

        self.release_temp_local(expected_payload_local);
        self.release_temp_local(month_from_code_local);
        Ok(())
    }

    /// `RegulateISODate` bounded by `ISOYearMonthWithinLimits` rather than
    /// `ISODateWithinLimits`: `-271821-04` is a representable year-month even
    /// though `-271821-04-01` is not a representable date.
    pub(crate) fn emit_temporal_year_month_regulate(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(overflow_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_reject_iso_year_month(year_local, month_local, day_local, function)?;
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
        self.emit_temporal_reject_iso_year_month(year_local, month_local, day_local, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    /// `ToTemporalYearMonth`. Accepts a branded `Temporal.PlainYearMonth`
    /// (cloned), any other object (read as a property bag), or an ISO string.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_to_temporal_year_month(
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
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH as i64,
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
            self.emit_temporal_year_month_overflow_option(
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
        let era = self.emit_temporal_year_month_read_fields(
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
            true,
            function,
        )?;
        if read_options {
            self.emit_temporal_year_month_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            year_local,
            year_present_local,
            function,
        )?;
        self.emit_temporal_year_month_resolve_fields(
            &resolved_year,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
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
            "Temporal.PlainYearMonth expects a string, a property bag, or a Temporal.PlainYearMonth",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_year_month_string(
            argument_payload_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        if read_options {
            self.emit_temporal_year_month_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        // A parsed string always names a real calendar day, so only the
        // year-month range check is left.
        self.emit_temporal_year_month_within_limits_check(year_local, month_local, function)?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);

        for local in [
            handled_local,
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

    /// `ParseTemporalYearMonthString`. `YYYY-MM` and `YYYYMM` are not
    /// `TemporalDateString`s, so the bare year-month spellings are rewritten
    /// with the reference day appended and handed to the one ISO parser; a
    /// string that already carries a day is passed through untouched.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_parse_year_month_string(
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

        self.emit_temporal_partial_date_rewrite_string(
            string_payload_local,
            true,
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

    /// The shared string rewrite behind `ParseTemporalYearMonthString` and
    /// `ParseTemporalMonthDayString`.
    ///
    /// `year_month` selects the goal. In both cases the head - everything
    /// before the first annotation bracket - is inspected: a bare `YYYY-MM` /
    /// `YYYYMM` gains the reference day, a bare `--MM-DD` / `MM-DD` / `MMDD`
    /// gains the reference year `1972`, and anything longer (a full date or
    /// date-time) is handed through unchanged. A UTC designator is a RangeError
    /// for both goals, so it is rejected here rather than inside the parser.
    pub(crate) fn emit_temporal_partial_date_rewrite_string(
        &mut self,
        string_payload_local: u32,
        year_month: bool,
        rewritten_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let head_end_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let head_local = self.reserve_temp_local();
        let tail_local = self.reserve_temp_local();
        let piece_local = self.reserve_temp_local();
        let bare_local = self.reserve_temp_local();
        let extended_local = self.reserve_temp_local();
        let signed_local = self.reserve_temp_local();
        let skip_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, length_local, function);

        // `head_end` is the first annotation bracket, or the whole string.
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalSet(head_end_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(head_end_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal partial-date strings must not carry a UTC designator",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // A head containing a date/time designator is never a bare
        // year-month or month-day.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(bare_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(head_end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(bare_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(extended_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(signed_local));

        if year_month {
            // `±YYYYYY-MM` (10) / `±YYYYYYMM` (9) / `YYYY-MM` (7) / `YYYYMM` (6).
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_string_byte(offset_local, skip_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'+' as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'-' as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(signed_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);

            // signed => 10 or 9, unsigned => 7 or 6.
            function.instruction(&Instruction::LocalGet(signed_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(9));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(7));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(6));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(bare_local));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(bare_local));

            // The extended spelling is the odd length (10 or 7).
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(7));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(extended_local));
        } else {
            // `--MM-DD` (7) / `--MMDD` (6) / `MM-DD` (5) / `MMDD` (4). The
            // optional `--` prefix is skipped before the length test.
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_string_byte(offset_local, skip_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'-' as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(cursor_local));
            self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'-' as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::LocalSet(skip_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::LocalGet(skip_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(5));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::LocalGet(skip_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(4));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(bare_local));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(bare_local));

            function.instruction(&Instruction::LocalGet(head_end_local));
            function.instruction(&Instruction::LocalGet(skip_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(5));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(extended_local));
        }

        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(rewritten_local));
        function.instruction(&Instruction::LocalGet(bare_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // head = string[skip .. head_end], tail = string[head_end ..].
        function.instruction(&Instruction::LocalGet(head_end_local));
        function.instruction(&Instruction::LocalGet(skip_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            skip_local,
            cursor_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(head_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalGet(head_end_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            head_end_local,
            cursor_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(tail_local));

        if year_month {
            function.instruction(&Instruction::LocalGet(head_local));
            function.instruction(&Instruction::LocalSet(rewritten_local));
        } else {
            function.instruction(&Instruction::LocalGet(extended_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.strings.payload("1972-")));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(self.strings.payload("1972")));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(rewritten_local));
            function.instruction(&Instruction::LocalGet(head_local));
            function.instruction(&Instruction::LocalSet(piece_local));
            self.emit_concat_string_payloads_local(rewritten_local, piece_local, function)?;
            function.instruction(&Instruction::LocalSet(rewritten_local));
        }

        if year_month {
            function.instruction(&Instruction::LocalGet(extended_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(self.strings.payload("-01")));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(self.strings.payload("01")));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(piece_local));
            self.emit_concat_string_payloads_local(rewritten_local, piece_local, function)?;
            function.instruction(&Instruction::LocalSet(rewritten_local));
        }

        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::LocalSet(piece_local));
        self.emit_concat_string_payloads_local(rewritten_local, piece_local, function)?;
        function.instruction(&Instruction::LocalSet(rewritten_local));
        function.instruction(&Instruction::End);

        for local in [
            skip_local,
            signed_local,
            extended_local,
            bare_local,
            piece_local,
            tail_local,
            head_local,
            byte_local,
            cursor_local,
            head_end_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ParseTemporalMonthDayString`'s half of the shared rewrite.
    pub(crate) fn emit_temporal_month_day_rewrite_string(
        &mut self,
        string_payload_local: u32,
        rewritten_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_partial_date_rewrite_string(
            string_payload_local,
            false,
            rewritten_local,
            function,
        )
    }

    /// Temporal proposal 9.2.2 `Temporal.PlainYearMonth.from`.
    pub(crate) fn emit_temporal_plain_year_month_from(
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
        self.emit_temporal_to_temporal_year_month(
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
            TemporalPartialDateType::PlainYearMonth,
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

    /// Temporal proposal 9.2.3 `Temporal.PlainYearMonth.compare`.
    pub(crate) fn emit_temporal_plain_year_month_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let left_year_local = self.reserve_temp_local();
        let left_month_local = self.reserve_temp_local();
        let left_day_local = self.reserve_temp_local();
        let right_year_local = self.reserve_temp_local();
        let right_month_local = self.reserve_temp_local();
        let right_day_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        for (index, year_local, month_local, day_local) in [
            (0, left_year_local, left_month_local, left_day_local),
            (1, right_year_local, right_month_local, right_day_local),
        ] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_temporal_to_temporal_year_month(
                argument_payload_local,
                argument_tag_local,
                undefined_local,
                undefined_tag_local,
                false,
                year_local,
                month_local,
                day_local,
                calendar_payload_local,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_local));
        for (left, right) in [
            (left_year_local, right_year_local),
            (left_month_local, right_month_local),
            (left_day_local, right_day_local),
        ] {
            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            value_local,
            right_day_local,
            right_month_local,
            right_year_local,
            left_day_local,
            left_month_local,
            left_year_local,
            calendar_payload_local,
            undefined_tag_local,
            undefined_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 9.3.x `equals`.
    pub(crate) fn emit_temporal_plain_year_month_equals(
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

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
        self.emit_temporal_to_temporal_year_month(
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

    /// Temporal proposal 9.3.x `with`.
    pub(crate) fn emit_temporal_plain_year_month_with(
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

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
            "Temporal.PlainYearMonth.prototype.with requires an object",
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
            "Temporal.PlainYearMonth.prototype.with does not accept a Temporal object",
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
                "Temporal.PlainYearMonth.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        let era = self.emit_temporal_year_month_read_fields(
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
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        for local in era.present_locals() {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainYearMonth.prototype.with requires at least one field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_year_month_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            new_year_local,
            year_present_local,
            function,
        )?;
        self.emit_temporal_resolved_year_default_to(&resolved_year, year_local, function);
        // `CalendarMergeFields` drops the receiver's `monthCode` as soon as the
        // argument supplies either `month` or `monthCode`.
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalSet(new_month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(month_present_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_year_month_resolve_fields(
            &resolved_year,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            overflow_local,
            function,
        )?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainYearMonth,
            new_year_local,
            new_month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
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

    /// `AddDurationToYearMonth`. The receiver is dated to the first of its
    /// month, or the last when the duration is negative, so that a shorter
    /// target month cannot pull the result back a month.
    pub(crate) fn emit_temporal_plain_year_month_add_or_subtract(
        &mut self,
        subtract: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let day_delta_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &duration_locals,
            function,
        )?;
        if subtract {
            for local in duration_locals.iter() {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(*local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(*local));
            }
        }
        self.emit_temporal_duration_sign(&duration_locals, sign_local, function);

        // The options are read before any algorithmic validation - Test262's
        // `add/options-read-before-algorithmic-validation.js` observes the
        // `overflow` read even on the paths that then throw.
        self.emit_temporal_year_month_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        // A year-month has no day, so a duration carrying weeks, days or any
        // time unit has no meaning here and is a RangeError rather than
        // something silently folded away.
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(duration_locals[2]));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainYearMonth arithmetic accepts only years and months",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(day_delta_local));

        // Date the receiver: first of the month, or its last day when the
        // duration runs backwards. `CalendarDateFromFields` range-checks that
        // intermediate date, and `-271821-04-01` is outside it even though
        // `-271821-04` is a representable year-month.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_iso_days_in_month(year_local, month_local, day_local, function);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;

        self.emit_temporal_add_iso_date(
            year_local,
            month_local,
            day_local,
            duration_locals[0],
            duration_locals[1],
            duration_locals[2],
            day_delta_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_local));
        self.emit_temporal_year_month_within_limits_check(year_local, month_local, function)?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainYearMonth,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        self.release_temporal_duration_field_locals(duration_locals);
        for local in [
            day_delta_local,
            subsecond_local,
            seconds_local,
            sign_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
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

    /// `DifferenceTemporalPlainYearMonth`. Only `year` and `month` are legal
    /// units, so the whole difference is a month count that is split, rounded
    /// and re-split - no nanosecond arithmetic is involved.
    pub(crate) fn emit_temporal_plain_year_month_until_or_since(
        &mut self,
        since: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let other_year_local = self.reserve_temp_local();
        let other_month_local = self.reserve_temp_local();
        let other_day_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let largest_unit_local = self.reserve_temp_local();
        let smallest_unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let original_mode_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let anchor_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let start_days_local = self.reserve_temp_local();
        let end_days_local = self.reserve_temp_local();
        let dest_days_local = self.reserve_temp_local();
        let scratch_year_local = self.reserve_temp_local();
        let scratch_month_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_temporal_to_temporal_year_month(
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
        // `DifferenceTemporalPlainYearMonth` step 2: `CalendarEquals` runs
        // between `ToTemporalYearMonth` and `GetOptionsObject`.
        self.emit_temporal_require_same_calendar(
            calendar_payload_local,
            other_calendar_payload_local,
            "Temporal.PlainYearMonth until and since require the same calendar",
            function,
        )?;

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit - the order is observable.
        self.emit_temporal_duration_options_object(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "largestUnit",
            true,
            largest_unit_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            options_payload_local,
            options_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            options_payload_local,
            options_tag_local,
            TemporalRoundingMode::Trunc,
            mode_local,
            function,
        )?;
        if since {
            function.instruction(&Instruction::LocalGet(mode_local));
            function.instruction(&Instruction::LocalSet(original_mode_local));
            for mode in TemporalRoundingMode::ALL {
                if mode.negated() == mode {
                    continue;
                }
                function.instruction(&Instruction::LocalGet(original_mode_local));
                function.instruction(&Instruction::I64Const(mode.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(mode.negated().code()));
                function.instruction(&Instruction::LocalSet(mode_local));
                function.instruction(&Instruction::End);
            }
        }
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "smallestUnit",
            false,
            smallest_unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        function.instruction(&Instruction::LocalSet(smallest_unit_local));
        function.instruction(&Instruction::End);
        // Only `year` and `month` survive; every smaller unit, and `week`, is a
        // RangeError for this type.
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Month,
            "Invalid Temporal.PlainYearMonth smallestUnit",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::LocalSet(largest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Month,
            "Invalid Temporal.PlainYearMonth largestUnit",
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;

        self.emit_temporal_duration_zero_fields(&duration_locals, function);

        // `CalendarDateUntil` on two first-of-month dates is a month count.
        function.instruction(&Instruction::LocalGet(other_year_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(other_month_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));

        // `quantum` is the rounding step measured in months.
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(quantum_local));

        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));

        // `anchor` is the truncated multiple of `quantum`; `remainder` is what
        // is left over, in months.
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(anchor_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(anchor_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The rounding boundary is measured in days between the two bracketing
        // months, not in months, because months are not all the same length.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        for (offset_local, days_local) in [
            (anchor_local, start_days_local),
            (quantum_local, end_days_local),
        ] {
            function.instruction(&Instruction::LocalGet(year_local));
            function.instruction(&Instruction::LocalSet(scratch_year_local));
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::LocalGet(anchor_local));
            function.instruction(&Instruction::I64Add);
            if offset_local == quantum_local {
                function.instruction(&Instruction::LocalGet(quantum_local));
                function.instruction(&Instruction::LocalGet(sign_local));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
            }
            function.instruction(&Instruction::LocalSet(scratch_month_local));
            self.emit_temporal_balance_iso_year_month(
                scratch_year_local,
                scratch_month_local,
                function,
            );
            self.emit_temporal_plain_date_epoch_days(
                scratch_year_local,
                scratch_month_local,
                one_local,
                days_local,
                function,
            );
        }
        self.emit_temporal_plain_date_epoch_days(
            other_year_local,
            other_month_local,
            one_local,
            dest_days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dest_days_local));
        function.instruction(&Instruction::LocalGet(start_days_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(end_days_local));
        function.instruction(&Instruction::LocalGet(start_days_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quantum_local));
        for local in [remainder_local, quantum_local] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalSet(dest_days_local));
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            quantum_local,
            anchor_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(anchor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(anchor_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(anchor_local));
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(duration_locals[0]));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(duration_locals[1]));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalSet(duration_locals[1]));
        function.instruction(&Instruction::End);

        if since {
            for index in [0_usize, 1] {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(duration_locals[index]));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(duration_locals[index]));
            }
        }
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        for local in [
            one_local,
            scratch_month_local,
            scratch_year_local,
            dest_days_local,
            end_days_local,
            start_days_local,
            remainder_local,
            sign_local,
            anchor_local,
            quantum_local,
            total_local,
            original_mode_local,
            mode_local,
            increment_local,
            smallest_unit_local,
            largest_unit_local,
            undefined_tag_local,
            undefined_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            other_calendar_payload_local,
            other_day_local,
            other_month_local,
            other_year_local,
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

    /// `Temporal.PlainYearMonth.prototype.toLocaleString`.
    ///
    /// `new Intl.DateTimeFormat(locales, options).format(this)`, with the
    /// reference day masked away by the year-month field set — the one thing
    /// that distinguishes it from `PlainDate`'s.
    pub(crate) fn emit_temporal_plain_year_month_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
        self.release_temp_local(record_local);
        self.emit_intl_dtf_temporal_to_locale_string(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
            function,
        )
    }

    /// `TemporalYearMonthToString`. The reference day is appended only when the
    /// calendar annotation is shown, which is the only way a round-trip could
    /// otherwise lose it.
    pub(crate) fn emit_temporal_plain_year_month_to_string(
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

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
            StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_string_valued_option::<ShowCalendarName>(
                options_payload_local,
                options_tag_local,
                show_calendar_local,
                "Temporal.PlainYearMonth options must be an object or undefined",
                "Invalid Temporal.PlainYearMonth calendarName option",
                function,
            )?;
        }

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
        // `TemporalYearMonthToString` step 4: the reference day is printed
        // under exactly the condition that prints the calendar annotation, so
        // `2026-01[u-ca=gregory]` is never emitted without its `-01`.
        self.emit_temporal_show_calendar_annotation_i32(
            show_calendar_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_append_separated_two_digits(
            day_local,
            "-",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
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

    /// `PadISOYear` into a fresh `output_payload_local`.
    pub(crate) fn emit_temporal_pad_iso_year(
        &mut self,
        year_local: u32,
        output_payload_local: u32,
        piece_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(9_999));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `separator` then a zero-padded two-digit field, appended in place.
    pub(crate) fn emit_temporal_append_separated_two_digits(
        &mut self,
        value_local: u32,
        separator: &str,
        output_payload_local: u32,
        piece_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !separator.is_empty() {
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            2,
            function,
        )?;
        Ok(())
    }

    /// Leaves an `i32` on the stack: 1 when the calendar has to appear in the
    /// output.
    ///
    /// `FormatCalendarAnnotation` steps 1-2: `never` never prints,
    /// `always`/`critical` always print, and `auto` prints exactly when the
    /// calendar is not `iso8601`. `TemporalYearMonthToString` step 4 (the
    /// reference day) and `TemporalMonthDayToString` step 2 (the reference
    /// year) are gated on the *same* condition, which is why this is one
    /// emitter and not three copies of an `or` that could drift.
    ///
    /// Before a second calendar existed the `auto` half was unreachable and the
    /// three sites each spelled out only the `always || critical` part.
    pub(crate) fn emit_temporal_show_calendar_annotation_i32(
        &mut self,
        show_calendar_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Always.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        self.emit_temporal_calendar_is_default_i32(calendar_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    /// `FormatCalendarAnnotation`. `auto` suppresses the annotation for
    /// `iso8601` and prints it for every other calendar.
    pub(crate) fn emit_temporal_append_calendar_annotation(
        &mut self,
        show_calendar_local: u32,
        calendar_payload_local: u32,
        output_payload_local: u32,
        piece_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_show_calendar_annotation_i32(
            show_calendar_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("[!u-ca=")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("[u-ca=")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(calendar_payload_local));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Temporal proposal 9.3.x `toPlainDate ( item )`: the receiver's year and
    /// month plus a `day` read from `item`.
    pub(crate) fn emit_temporal_plain_year_month_to_plain_date(
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

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
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
            "Temporal.PlainYearMonth.prototype.toPlainDate requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_property_bag_positive_integer(
            argument_payload_local,
            argument_tag_local,
            "day",
            key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            day_local,
            0,
            "Temporal.PlainYearMonth day must be finite",
            "Temporal.PlainYearMonth day must be positive",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainYearMonth.prototype.toPlainDate requires a day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainYearMonth day must be positive",
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
