//! `Temporal.PlainYearMonth` statics and prototype methods.
//!
//! Split from `temporal_plain_year_month.rs` (constructor, record, accessors)
//! the way `Temporal.PlainDate` is split; both halves are
//! `impl FunctionBuilder` blocks.

mod difference;

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, TemporalConversionOverflowOptions, TemporalOverflow, TemporalRoundingMode,
    TemporalUnit, TemporalUnitOptionProperty, TemporalUnitSlot,
};
use super::temporal_plain_date::{
    TemporalEraLocals, TemporalMonthFieldContext, TemporalResolvedCalendarYear,
};
use super::temporal_plain_date_time_methods::{
    TemporalPlainArithmeticOperation, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_year_month::TemporalPartialDateType;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

pub(super) enum TemporalPlainYearMonthStringMode {
    ToString,
    ToJson,
}

/// The partial-date parse goal and the missing source field its rewrite can
/// report. The two goals add different reference fields, so their output slots
/// remain named by the corresponding grammar field.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TemporalPartialDateRewrite {
    /// `ParseTemporalYearMonthString`: a bare `YYYY-MM` / `YYYYMM` gains the
    /// reference day `01`.
    YearMonth { day_empty_out: Option<I64Local> },
    /// `ParseTemporalMonthDayString`: a bare `--MM-DD` / `--MMDD` / `MM-DD` /
    /// `MMDD` gains the reference year `1972`.
    ///
    /// `year_empty_out`, when present, receives `1` exactly for those four bare
    /// spellings and `0` for every form that already carried a year — i.e.
    /// `ParseISODateTime`'s `result.[[Year]] is empty`, which
    /// `ToTemporalMonthDay` steps (g) and (k) both consult and which this
    /// rewrite is the only thing in the backend that can still tell apart.
    /// The calendar-identifier probe uses the same fact to enforce the
    /// partial-date grammar's ISO-only calendar annotation restriction.
    MonthDay { year_empty_out: Option<I64Local> },
}

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_year_month_overflow_option(
        &mut self,
        options: &ValueLocals,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options,
            overflow,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINYEARMONTH_OVERFLOW_OPTION,
            function,
        )
    }

    fn emit_temporal_year_month_calendar_slot(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<TemporalCalendarSlotLocals, EmitError> {
        let schema = self.runtime_schema();
        let identifier = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let calendar = self.emit_temporal_calendar_slot_from_identifier(&identifier, function)?;
        identifier.clear(function);
        Ok(calendar)
    }

    fn emit_temporal_year_month_receiver_fields(
        &mut self,
        fields: &[I64Local; 3],
        calendar: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_temporal_plain_year_month_record_from_receiver(function)?;
        self.emit_temporal_plain_year_month_load_record(&record, fields, calendar, function);
        record.clear(function);
        Ok(())
    }

    /// `PrepareCalendarFields` for the `« year, month, month-code »` key set
    /// plus the era pair. The caller acquires `calendar` first; this sweep then
    /// reads `era`, `eraYear`, `month`, `monthCode`, `year`. There is deliberately no
    /// `day` read — a `Temporal.PlainYearMonth` property bag never has one, and
    /// `intl402/Temporal/PlainYearMonth/from/argument-object.js` hands in a bag
    /// whose `day` getter throws to prove it.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_year_month_read_fields(
        &mut self,
        argument: &ValueLocals,
        calendar: &TemporalCalendarSlotLocals,
        year: I64Local,
        year_present: I64Local,
        month: I64Local,
        month_present: I64Local,
        month_code: &ValueLocals,
        month_code_present: I64Local,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let slots = self.reserve_temporal_era_slots(function);
        let era =
            self.emit_temporal_read_era_fields(slots, argument, calendar.calendar_id(), function)?;
        self.emit_temporal_property_bag_positive_integer(
            argument,
            "month",
            month_present,
            month,
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_FIELDS_MUST_BE_FINITE,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_MONTH_MUST_BE_POSITIVE,
            function,
        )?;
        self.emit_temporal_duration_option_get(argument, "monthCode", month_code, function)?;
        month_code.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        month_code_present.store(function);
        self.emit_temporal_month_code_string(
            month_code,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_MONTHCODE_MUST_BE_A_STRING,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINYEARMONTH_MONTHCODE,
            function,
        )?;
        self.emit_temporal_property_bag_integer(
            argument,
            "year",
            year_present,
            year,
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_FIELDS_MUST_BE_FINITE,
            function,
        )?;
        Ok(era)
    }

    /// `CalendarResolveFields` for the year-month field set, then
    /// regulate the complete calendar date with calendar day fixed at 1.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_year_month_resolve_fields(
        &mut self,
        resolved_year: TemporalResolvedCalendarYear,
        month: I64Local,
        month_present: I64Local,
        acquired_month_code: &ValueLocals,
        encoded_month_code: I64Local,
        month_code_present: I64Local,
        day: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year = resolved_year.year_local();
        resolved_year.year_present_local().load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_FIELDS_REQUIRE_YEAR,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        month_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_FIELDS_REQUIRE_MONTH_OR_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let resolved_month = self.emit_temporal_resolve_calendar_month(
            resolved_year,
            month,
            month_present,
            acquired_month_code,
            encoded_month_code,
            month_code_present,
            TemporalMonthFieldContext::PlainYearMonth,
            function,
        )?;
        month.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_MONTH_MUST_BE_POSITIVE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        day.store(function);
        self.emit_temporal_calendar_date_to_iso(resolved_month, day, overflow, function)?;
        self.emit_temporal_reject_iso_year_month(year, month, day, function)?;
        Ok(())
    }

    /// `RegulateISODate` bounded by `ISOYearMonthWithinLimits` rather than
    /// `ISODateWithinLimits`: `-271821-04` is a representable year-month even
    /// though `-271821-04-01` is not a representable date.
    /// `ToTemporalYearMonth`. Accepts a branded `Temporal.PlainYearMonth`
    /// (cloned), any other object (read as a property bag), or an ISO string.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_to_temporal_year_month(
        &mut self,
        argument: &ValueLocals,
        overflow_options: TemporalConversionOverflowOptions<'_>,
        year: I64Local,
        month: I64Local,
        day: I64Local,
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let overflow = schema.reserve_i64_local(function);
        let handled = schema.reserve_i64_local(function);
        let year_present = schema.reserve_i64_local(function);
        let month_present = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code = schema.reserve_value_local(function);
        function.instruction(&Instruction::I64Const(0));
        handled.store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        function.instruction(&Instruction::I64Const(0));
        encoded_month_code.store(function);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainYearMonthObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainYearMonthObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_year_month_load_record(
            &record,
            &[year, month, day],
            calendar_out,
            function,
        );
        record.clear(function);
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_year_month_overflow_option(options, overflow, function)?;
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_duration_option_get(argument, "calendar", &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar_value.clear(function);
        let era = self.emit_temporal_year_month_read_fields(
            argument,
            &calendar,
            year,
            year_present,
            month,
            month_present,
            &month_code,
            month_code_present,
            function,
        )?;
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_year_month_overflow_option(options, overflow, function)?;
        }
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            year,
            year_present,
            function,
        )?;
        self.emit_temporal_year_month_resolve_fields(
            resolved_year,
            month,
            month_present,
            &month_code,
            encoded_month_code,
            month_code_present,
            day,
            overflow,
            function,
        )?;
        calendar.release(self, function);
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_PLAINYEARMONTH, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_year_month_string(
            &string,
            year,
            month,
            day,
            calendar_out,
            function,
        )?;
        string.clear(function);
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_year_month_overflow_option(options, overflow, function)?;
        }
        self.emit_temporal_year_month_within_limits_check(year, month, function)?;
        let calendar = self.emit_temporal_year_month_calendar_slot(calendar_out, function)?;
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            [year, month, day],
            function,
        )?;
        for (source, destination) in reference.fields().into_iter().zip([year, month, day]) {
            source.load(function);
            destination.store(function);
        }
        reference.release(self, function);
        calendar.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        month_code.clear(function);
        for local in [
            encoded_month_code,
            month_code_present,
            month_present,
            year_present,
            handled,
            overflow,
        ] {
            schema.release_i64_local(local, function);
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
        string: &GcLocal<StringValue>,
        year: I64Local,
        month: I64Local,
        day: I64Local,
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let rewritten_value = schema.reserve_value_local(function);
        let day_empty = schema.reserve_i64_local(function);
        self.emit_temporal_partial_date_rewrite_string(
            string,
            TemporalPartialDateRewrite::YearMonth {
                day_empty_out: Some(day_empty),
            },
            &rewritten_value,
            function,
        )?;
        let rewritten = schema.reserve_gc_local(function).initialize(
            rewritten_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_plain_date_string(
            &rewritten,
            year,
            month,
            day,
            calendar_out,
            function,
        )?;
        rewritten.clear(function);
        rewritten_value.clear(function);
        let calendar = self.emit_temporal_year_month_calendar_slot(calendar_out, function)?;
        self.emit_temporal_calendar_is_default_i32(calendar.calendar_id(), function);
        function.instruction(&Instruction::I32Eqz);
        day_empty.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_YEAR_MONTH_STRING_WITH_A_NON_ISO_CALENDAR_REQUIRES_A_DAY, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        calendar.release(self, function);
        schema.release_i64_local(day_empty, function);
        Ok(())
    }

    /// The shared string rewrite behind `ParseTemporalYearMonthString` and
    /// `ParseTemporalMonthDayString`.
    ///
    /// [`TemporalPartialDateRewrite`] selects the goal. In both cases the head -
    /// everything before the first annotation bracket - is inspected: a bare
    /// `YYYY-MM` / `YYYYMM` gains the reference day, a bare `--MM-DD` / `MM-DD`
    /// / `MMDD` gains the reference year `1972`, and anything longer (a full
    /// date or date-time) is handed through unchanged. A UTC designator is a
    /// RangeError for both goals, so it is rejected here rather than inside the
    /// parser.
    ///
    /// `bare_local` below is the whole reason the month-day goal can answer
    /// `result.[[Year]] is empty`: the rewrite is the last point at which the
    /// four year-less spellings are distinguishable, because after it every one
    /// of them carries a literal `1972` that the ISO parser cannot tell from a
    /// year the source wrote itself. [`TemporalPartialDateRewrite::MonthDay`]'s
    /// `year_empty_out` is where that fact leaves this emitter instead of being
    /// released with the local.
    pub(crate) fn emit_temporal_partial_date_rewrite_string(
        &mut self,
        string: &GcLocal<StringValue>,
        goal: TemporalPartialDateRewrite,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length_local = self.runtime_schema().reserve_i64_local(function);
        let head_end_local = self.runtime_schema().reserve_i64_local(function);
        let cursor_local = self.runtime_schema().reserve_i64_local(function);
        let byte_local = self.runtime_schema().reserve_i64_local(function);
        let bare_local = self.runtime_schema().reserve_i64_local(function);
        let extended_local = self.runtime_schema().reserve_i64_local(function);
        let signed_local = self.runtime_schema().reserve_i64_local(function);
        let skip_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_temporal_string_length(string, length_local, function);

        // `head_end` is the first annotation bracket, or the whole string.
        (length_local).load(function);
        (head_end_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (cursor_local).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(string, cursor_local, byte_local, function);
        (byte_local).load(function);
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (cursor_local).load(function);
        (head_end_local).store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (byte_local).load(function);
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64Eq);
        (byte_local).load(function);
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PARTIAL_DATE_STRINGS_MUST_NOT_CARRY_A_UTC_DESIGNATOR,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        // A head containing a date/time designator is never a bare
        // year-month or month-day.
        function.instruction(&Instruction::I64Const(1));
        (bare_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (cursor_local).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor_local).load(function);
        (head_end_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(string, cursor_local, byte_local, function);
        (byte_local).load(function);
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        (byte_local).load(function);
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (bare_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        (skip_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (extended_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (signed_local).store(function);

        match goal {
            TemporalPartialDateRewrite::YearMonth { .. } => {
                // `±YYYYYY-MM` (10) / `±YYYYYYMM` (9) / `YYYY-MM` (7) / `YYYYMM` (6).
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64GtU);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_load_code_unit(string, skip_local, byte_local, function);
                (byte_local).load(function);
                function.instruction(&Instruction::I64Const(b'+' as i64));
                function.instruction(&Instruction::I64Eq);
                (byte_local).load(function);
                function.instruction(&Instruction::I64Const(b'-' as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(1));
                (signed_local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);

                // signed => 10 or 9, unsigned => 7 or 6.
                (signed_local).load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::I64Eq);
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(9));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::Else);
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::I64Eq);
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(6));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64ExtendI32U);
                (bare_local).load(function);
                function.instruction(&Instruction::I64And);
                (bare_local).store(function);

                // The extended spelling is the odd length (10 or 7).
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::I64Eq);
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I64ExtendI32U);
                (extended_local).store(function);
            }
            TemporalPartialDateRewrite::MonthDay { .. } => {
                // `--MM-DD` (7) / `--MMDD` (6) / `MM-DD` (5) / `MMDD` (4). The
                // optional `--` prefix is skipped before the length test.
                (head_end_local).load(function);
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64GeU);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_load_code_unit(string, skip_local, byte_local, function);
                (byte_local).load(function);
                function.instruction(&Instruction::I64Const(b'-' as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(1));
                (cursor_local).store(function);
                self.emit_temporal_load_code_unit(string, cursor_local, byte_local, function);
                (byte_local).load(function);
                function.instruction(&Instruction::I64Const(b'-' as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(2));
                (skip_local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);

                (head_end_local).load(function);
                (skip_local).load(function);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64Eq);
                (head_end_local).load(function);
                (skip_local).load(function);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(4));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I64ExtendI32U);
                (bare_local).load(function);
                function.instruction(&Instruction::I64And);
                (bare_local).store(function);

                (head_end_local).load(function);
                (skip_local).load(function);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                (extended_local).store(function);
            }
        }

        output.set_reference(string, self.runtime_schema(), function);
        bare_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        head_end_local.load(function);
        skip_local.load(function);
        function.instruction(&Instruction::I64Sub);
        cursor_local.store(function);
        let head = self.runtime_schema().reserve_gc_local(function).initialize(
            self.emit_temporal_string_slice(string, skip_local, cursor_local, function),
            function,
        );
        length_local.load(function);
        head_end_local.load(function);
        function.instruction(&Instruction::I64Sub);
        cursor_local.store(function);
        let tail = self.runtime_schema().reserve_gc_local(function).initialize(
            self.emit_temporal_string_slice(string, head_end_local, cursor_local, function),
            function,
        );
        let rewritten = self
            .runtime_schema()
            .reserve_gc_local(function)
            .initialize(head.load(self.runtime_schema(), function), function);
        match goal {
            TemporalPartialDateRewrite::YearMonth { .. } => {
                extended_local.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                let suffix = self.runtime_schema().reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("-01", function)?,
                    function,
                );
                rewritten.replace(
                    self.emit_concat_gc_strings(&head, &suffix, function),
                    function,
                );
                suffix.clear(function);
                function.instruction(&Instruction::Else);
                let suffix = self.runtime_schema().reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("01", function)?,
                    function,
                );
                rewritten.replace(
                    self.emit_concat_gc_strings(&head, &suffix, function),
                    function,
                );
                suffix.clear(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            TemporalPartialDateRewrite::MonthDay { .. } => {
                extended_local.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                let prefix = self.runtime_schema().reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("1972-", function)?,
                    function,
                );
                rewritten.replace(
                    self.emit_concat_gc_strings(&prefix, &head, function),
                    function,
                );
                prefix.clear(function);
                function.instruction(&Instruction::Else);
                let prefix = self.runtime_schema().reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("1972", function)?,
                    function,
                );
                rewritten.replace(
                    self.emit_concat_gc_strings(&prefix, &head, function),
                    function,
                );
                prefix.clear(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        rewritten.replace(
            self.emit_concat_gc_strings(&rewritten, &tail, function),
            function,
        );
        output.set_reference(&rewritten, self.runtime_schema(), function);
        rewritten.clear(function);
        tail.clear(function);
        head.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Preserve the grammar fact before the inserted reference field makes
        // an omitted field indistinguishable from an explicit one.
        let missing_field_out = match goal {
            TemporalPartialDateRewrite::YearMonth { day_empty_out } => day_empty_out,
            TemporalPartialDateRewrite::MonthDay { year_empty_out } => year_empty_out,
        };
        if let Some(missing_field_local) = missing_field_out {
            (bare_local).load(function);
            (missing_field_local).store(function);
        }

        for local in [
            skip_local,
            signed_local,
            extended_local,
            bare_local,
            byte_local,
            cursor_local,
            head_end_local,
            length_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// `ParseTemporalMonthDayString`'s half of the shared rewrite.
    ///
    /// `year_empty_out` is passed straight through to
    /// [`TemporalPartialDateRewrite::MonthDay`]. `ToTemporalMonthDay` supplies a
    /// slot because steps (g) and (k) need `result.[[Year]] is empty`. The
    /// calendar-identifier probe also requests that syntax fact, without
    /// applying MonthDay's constructor-specific reference-year checks.
    pub(crate) fn emit_temporal_month_day_rewrite_string(
        &mut self,
        input: &GcLocal<StringValue>,
        output: &ValueLocals,
        year_empty_out: Option<I64Local>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_partial_date_rewrite_string(
            input,
            TemporalPartialDateRewrite::MonthDay { year_empty_out },
            output,
            function,
        )
    }

    /// Temporal proposal 9.2.2 `Temporal.PlainYearMonth.from`.
    pub(crate) fn emit_temporal_plain_year_month_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_temporal_to_temporal_year_month(
            &argument,
            TemporalConversionOverflowOptions::Read(&options),
            fields[0],
            fields[1],
            fields[2],
            &calendar_value,
            function,
        )?;
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        let reference = self.emit_temporal_complete_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            fields,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 9.2.3 `Temporal.PlainYearMonth.compare`.
    pub(crate) fn emit_temporal_plain_year_month_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar = schema.reserve_value_local(function);
        let left: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let right: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let comparison = schema.reserve_i64_local(function);
        for (index, fields) in [(0_usize, left), (1, right)] {
            self.emit_builtin_arg_to_value(index, &argument, function);
            self.emit_temporal_to_temporal_year_month(
                &argument,
                TemporalConversionOverflowOptions::Omit,
                fields[0],
                fields[1],
                fields[2],
                &calendar,
                function,
            )?;
        }
        self.emit_temporal_compare_iso_date(left, right, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        comparison.store(function);
        self.completion().value().set_number(comparison, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        schema.release_i64_local(comparison, function);
        for local in right.into_iter().rev().chain(left.into_iter().rev()) {
            schema.release_i64_local(local, function);
        }
        calendar.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 9.3.x `equals`.
    pub(crate) fn emit_temporal_plain_year_month_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let other: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let equal = schema.reserve_i32_local(function);
        self.emit_temporal_year_month_receiver_fields(&fields, &calendar_value, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_temporal_to_temporal_year_month(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            other[0],
            other[1],
            other[2],
            &other_calendar_value,
            function,
        )?;
        for (index, (left, right)) in fields.into_iter().zip(other).enumerate() {
            left.load(function);
            right.load(function);
            function.instruction(&Instruction::I64Eq);
            if index != 0 {
                function.instruction(&Instruction::I32And);
            }
        }
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let calendar = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let other_calendar = schema.reserve_gc_local(function).initialize(
            other_calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_string_payload_equality_i32(&calendar, &other_calendar, function);
        other_calendar.clear(function);
        calendar.clear(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        equal.store(function);
        self.completion().value().set_boolean(equal, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        schema.release_i32_local(equal, function);
        for local in other.into_iter().rev().chain(fields.into_iter().rev()) {
            schema.release_i64_local(local, function);
        }
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 9.3.x `with`.
    pub(crate) fn emit_temporal_plain_year_month_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let probe = schema.reserve_value_local(function);
        let month_code = schema.reserve_value_local(function);
        let receiver_month_code = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let new_year = schema.reserve_i64_local(function);
        let year_present = schema.reserve_i64_local(function);
        let new_month = schema.reserve_i64_local(function);
        let month_present = schema.reserve_i64_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        self.emit_temporal_year_month_receiver_fields(&fields, &calendar_value, function)?;
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        let projected =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), fields, function);
        for (source, destination) in projected.fields().into_iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        self.emit_temporal_calendar_month_code_payload(&projected, &receiver_month_code, function)?;
        projected.release(self, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_branded_partial_object(&argument,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_OBJECT, function)?;
        for property in ["calendar", "timeZone"] {
            self.emit_temporal_duration_option_get(&argument, property, &probe, function)?;
            probe.tag().load(function);
            function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
                RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_WITH_DOES_NOT_ACCEPT_CALENDAR_OR_TIMEZONE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(0));
        encoded_month_code.store(function);
        let era = self.emit_temporal_year_month_read_fields(
            &argument,
            &calendar,
            new_year,
            year_present,
            new_month,
            month_present,
            &month_code,
            month_code_present,
            function,
        )?;
        year_present.load(function);
        month_present.load(function);
        function.instruction(&Instruction::I64Or);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Or);
        for local in era.present_locals() {
            local.load(function);
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_WITH_REQUIRES_AT_LEAST_ONE_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_year_month_overflow_option(&options, overflow, function)?;
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            new_year,
            year_present,
            function,
        )?;
        self.emit_temporal_resolved_year_default_to(&resolved_year, fields[0], function);
        month_present.load(function);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        month_code.copy_from(&receiver_month_code, function);
        function.instruction(&Instruction::I64Const(1));
        month_code_present.store(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_year_month_resolve_fields(
            resolved_year,
            new_month,
            month_present,
            &month_code,
            encoded_month_code,
            month_code_present,
            fields[2],
            overflow,
            function,
        )?;
        let reference = self.emit_temporal_complete_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            [new_year, new_month, fields[2]],
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in [
            overflow,
            month_code_present,
            encoded_month_code,
            month_present,
            new_month,
            year_present,
            new_year,
        ]
        .into_iter()
        .chain(fields.into_iter().rev())
        {
            schema.release_i64_local(local, function);
        }
        receiver_month_code.clear(function);
        month_code.clear(function);
        probe.clear(function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `AddDurationToYearMonth` starts at calendar day 1 for both duration
    /// signs, converts that date to ISO, then adds in the retained calendar.
    pub(super) fn emit_temporal_plain_year_month_add_or_subtract(
        &mut self,
        operation: TemporalPlainArithmeticOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let overflow = schema.reserve_i64_local(function);
        let seconds = schema.reserve_i64_local(function);
        let subsecond = schema.reserve_i64_local(function);
        let day_delta = schema.reserve_i64_local(function);
        let duration = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_year_month_receiver_fields(&fields, &calendar_value, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_duration(&argument, &duration, function)?;
        match operation {
            TemporalPlainArithmeticOperation::Add => {}
            TemporalPlainArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&duration, function)
            }
        }
        let date_fields = self.reserve_temporal_duration_date_field_locals(&duration, function);
        // Options are observed before algorithmic duration-category rejection.
        self.emit_temporal_year_month_overflow_option(&options, overflow, function)?;
        self.emit_temporal_duration_normalize_seconds(
            &duration,
            TemporalUnit::Day,
            seconds,
            subsecond,
            function,
        );
        seconds.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        subsecond.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        date_fields[2].load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_ARITHMETIC_ACCEPTS_ONLY_YEARS_AND_MONTHS,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        day_delta.store(function);
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            fields,
            function,
        )?;
        for (source, destination) in reference.fields().into_iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        reference.release(self, function);
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], function)?;
        self.emit_temporal_add_calendar_date(
            &calendar,
            fields[0],
            fields[1],
            fields[2],
            date_fields[0],
            date_fields[1],
            date_fields[2],
            day_delta,
            overflow,
            function,
        )?;
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            fields,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in date_fields
            .into_iter()
            .rev()
            .chain([day_delta, subsecond, seconds, overflow])
            .chain(fields.into_iter().rev())
        {
            schema.release_i64_local(local, function);
        }
        self.release_temporal_duration_field_locals(duration, function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
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
        let record = self.emit_temporal_plain_year_month_record_from_receiver(function)?;
        record.clear(function);
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::PlainYearMonth,
            function,
        )
    }

    /// `TemporalYearMonthToString`. A non-ISO calendar always retains the
    /// reference ISO day, including when its annotation is hidden.
    pub(crate) fn emit_temporal_plain_year_month_to_string(
        &mut self,
        mode: TemporalPlainYearMonthStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let calendar_value = schema.reserve_value_local(function);
        let output = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let show = schema.reserve_i64_local(function);
        self.emit_temporal_year_month_receiver_fields(&fields, &calendar_value, function)?;
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        show.store(function);
        match mode {
            TemporalPlainYearMonthStringMode::ToString => {
                self.emit_builtin_arg_to_value(0, &options, function);
                self.emit_temporal_string_valued_option::<ShowCalendarName>(&options, show,
                    RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINYEARMONTH_CALENDARNAME_OPTION, function)?;
            }
            TemporalPlainYearMonthStringMode::ToJson => {}
        }
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        self.emit_temporal_pad_iso_year(fields[0], &output, function)?;
        self.emit_temporal_append_separated_two_digits(fields[1], "-", &output, function)?;
        self.emit_temporal_show_partial_reference_i32(show, calendar.calendar_id(), function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_separated_two_digits(fields[2], "-", &output, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_append_calendar_annotation(&calendar, show, &output, function)?;
        self.completion().set_normal(&output, function);
        calendar.release(self, function);
        schema.release_i64_local(show, function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        options.clear(function);
        output.clear(function);
        calendar_value.clear(function);
        Ok(())
    }

    /// `PadISOYear` publishes a fresh rooted UTF-16 String into `output`.
    pub(crate) fn emit_temporal_pad_iso_year(
        &mut self,
        year: I64Local,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number_bits = schema.reserve_i64_local(function);
        let text = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        year.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "-", function)?;
        function.instruction(&Instruction::I64Const(0));
        year.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits.store(function);
        self.emit_date_append_padded_decimal(&text, number_bits, 6, function)?;
        function.instruction(&Instruction::Else);
        year.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits.store(function);
        year.load(function);
        function.instruction(&Instruction::I64Const(9_999));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "+", function)?;
        self.emit_date_append_padded_decimal(&text, number_bits, 6, function)?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(&text, number_bits, 4, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output.set_reference(&text, schema, function);
        text.clear(function);
        schema.release_i64_local(number_bits, function);
        Ok(())
    }

    /// `separator` then a zero-padded two-digit field, appended in place.
    pub(crate) fn emit_temporal_append_separated_two_digits(
        &mut self,
        value: I64Local,
        separator: &str,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number_bits = schema.reserve_i64_local(function);
        let text = schema.reserve_gc_local(function).initialize(
            output.cast_reference::<StringValue>(schema, function),
            function,
        );
        if !separator.is_empty() {
            self.emit_temporal_append_gc_literal(&text, separator, function)?;
        }
        value.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits.store(function);
        self.emit_date_append_padded_decimal(&text, number_bits, 2, function)?;
        output.set_reference(&text, schema, function);
        text.clear(function);
        schema.release_i64_local(number_bits, function);
        Ok(())
    }

    /// `TemporalYearMonthToString` and `TemporalMonthDayToString` retain the
    /// reference ISO field for every non-ISO calendar. For ISO, only `always`
    /// and `critical` expose it. Annotation suppression is a separate rule.
    pub(super) fn emit_temporal_show_partial_reference_i32(
        &self,
        show: I64Local,
        calendar: I64Local,
        function: &mut Function,
    ) {
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Always.code()));
        function.instruction(&Instruction::I64Eq);
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_calendar_is_default_i32(calendar, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
    }

    /// `FormatCalendarAnnotation`: `never` suppresses the annotation,
    /// `always`/`critical` include it, and `auto` includes non-ISO calendars.
    /// Keep this private to the annotation consumer so partial-date fields
    /// cannot accidentally inherit the `never` suppression again.
    fn emit_temporal_show_calendar_annotation_i32(
        &self,
        show: I64Local,
        calendar: I64Local,
        function: &mut Function,
    ) {
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Always.code()));
        function.instruction(&Instruction::I64Eq);
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        self.emit_temporal_calendar_is_default_i32(calendar, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    /// `FormatCalendarAnnotation`. `auto` suppresses the annotation for
    /// `iso8601` and prints it for every other calendar.
    pub(crate) fn emit_temporal_append_calendar_annotation(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        show: I64Local,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_show_calendar_annotation_i32(show, calendar.calendar_id(), function);
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let text = schema.reserve_gc_local(function).initialize(
            output.cast_reference::<StringValue>(schema, function),
            function,
        );
        show.load(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "[!u-ca=", function)?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_append_gc_literal(&text, "[u-ca=", function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        text.replace(
            self.emit_concat_gc_strings(&text, calendar.identifier(), function),
            function,
        );
        self.emit_temporal_append_gc_literal(&text, "]", function)?;
        output.set_reference(&text, schema, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Temporal proposal 9.3.x `toPlainDate ( item )`: the receiver's year and
    /// month plus a `day` read from `item`.
    pub(crate) fn emit_temporal_plain_year_month_to_plain_date(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let present = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        self.emit_temporal_year_month_receiver_fields(&fields, &calendar_value, function)?;
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        let projected =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), fields, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_TOPLAINDATE_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_property_bag_positive_integer(
            &argument,
            "day",
            present,
            fields[2],
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_DAY_MUST_BE_FINITE,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_PROTOTYPE_TOPLAINDATE_REQUIRES_A_DAY,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        fields[2].load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        self.emit_temporal_calendar_date_from_projection(
            &projected, fields[2], overflow, fields, function,
        )?;
        projected.release(self, function);
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], function)?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        schema.release_i64_local(overflow, function);
        schema.release_i64_local(present, function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }
}
