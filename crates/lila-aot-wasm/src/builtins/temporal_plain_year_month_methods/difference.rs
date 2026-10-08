//! Existing calendar difference and rounding owner, consumed by builtin dispatch.

use super::*;

impl FunctionBuilder<'_> {
    /// A calendar-month step from an actual first-of-calendar-month ISO
    /// carrier, used for both rounding brackets and the year bubble boundary.
    fn emit_temporal_year_month_calendar_anchor(
        &mut self,
        calendar: &crate::builtins::temporal_zone_provider::TemporalCalendarSlotLocals,
        origin: [I64Local; 3],
        years: I64Local,
        months: I64Local,
        output: [I64Local; 3],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let zero = self.runtime_schema().reserve_i64_local(function);
        let overflow = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (zero).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow).store(function);
        for (source, destination) in origin.into_iter().zip(output) {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar, output[0], output[1], output[2], years, months, zero, zero, overflow,
            function,
        )?;
        self.runtime_schema().release_i64_local(overflow, function);
        self.runtime_schema().release_i64_local(zero, function);
        Ok(())
    }

    /// `DifferenceTemporalPlainYearMonth`. Only `year` and `month` are legal
    /// units. Keep the calendar year/month pair through both actual rounding
    /// anchors; a variable-year calendar cannot flatten years to one count.
    pub(in crate::builtins) fn emit_temporal_plain_year_month_until_or_since(
        &mut self,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year_local = self.runtime_schema().reserve_i64_local(function);
        let month_local = self.runtime_schema().reserve_i64_local(function);
        let day_local = self.runtime_schema().reserve_i64_local(function);
        let other_year_local = self.runtime_schema().reserve_i64_local(function);
        let other_month_local = self.runtime_schema().reserve_i64_local(function);
        let other_day_local = self.runtime_schema().reserve_i64_local(function);
        let largest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let smallest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);
        let mode_local = self.runtime_schema().reserve_i64_local(function);
        let original_mode_local = self.runtime_schema().reserve_i64_local(function);
        let years_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let anchor_local = self.runtime_schema().reserve_i64_local(function);
        let months_local = self.runtime_schema().reserve_i64_local(function);
        let quotient_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let remainder_local = self.runtime_schema().reserve_i64_local(function);
        let start_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_days_local = self.runtime_schema().reserve_i64_local(function);
        let dest_days_local = self.runtime_schema().reserve_i64_local(function);
        let scratch_year_local = self.runtime_schema().reserve_i64_local(function);
        let scratch_month_local = self.runtime_schema().reserve_i64_local(function);
        let scratch_day_local = self.runtime_schema().reserve_i64_local(function);
        let month_step_local = self.runtime_schema().reserve_i64_local(function);
        let year_step_local = self.runtime_schema().reserve_i64_local(function);
        let retained_years_local = self.runtime_schema().reserve_i64_local(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);

        self.emit_temporal_year_month_receiver_fields(
            &[year_local, month_local, day_local],
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_temporal_to_temporal_year_month(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            other_year_local,
            other_month_local,
            other_day_local,
            &other_calendar_value,
            function,
        )?;
        let calendar = self.emit_temporal_year_month_calendar_slot(&calendar_value, function)?;
        let other_calendar =
            self.emit_temporal_year_month_calendar_slot(&other_calendar_value, function)?;
        // CalendarEquals runs before every options Get.
        self.emit_temporal_require_same_calendar(
            calendar.calendar_id(),
            other_calendar.calendar_id(),
            TemporalDifferenceGuard::PlainYearMonthSameCalendar,
            function,
        )?;
        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit - the order is observable.
        self.emit_temporal_duration_options_object(&options, function)?;
        self.emit_temporal_duration_unit_option(
            &options,
            TemporalUnitOptionProperty::LargestUnit,
            largest_unit_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(&options, increment_local, function)?;
        self.emit_temporal_duration_rounding_mode_option(
            &options,
            TemporalRoundingMode::Trunc,
            mode_local,
            function,
        )?;
        match operation {
            TemporalPlainDifferenceOperation::Until => {}
            TemporalPlainDifferenceOperation::Since => {
                (mode_local).load(function);
                (original_mode_local).store(function);
                for mode in TemporalRoundingMode::ALL {
                    if mode.negated() == mode {
                        continue;
                    }
                    (original_mode_local).load(function);
                    function.instruction(&Instruction::I64Const(mode.code()));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    function.instruction(&Instruction::I64Const(mode.negated().code()));
                    (mode_local).store(function);
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
            }
        }
        self.emit_temporal_duration_unit_option(
            &options,
            TemporalUnitOptionProperty::SmallestUnit,
            smallest_unit_local,
            function,
        )?;
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        (smallest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Only `year` and `month` survive; every smaller unit, and `week`, is a
        // RangeError for this type.
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Month,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINYEARMONTH_SMALLESTUNIT,
            function,
        )?;
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        (largest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Month,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINYEARMONTH_LARGESTUNIT,
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;

        self.emit_temporal_duration_zero_fields(&duration_locals, function);

        // CompareISODate precedes conversion to first-of-month dates. In
        // particular, the minimum year-month can differ from itself by zero
        // even though its first day is outside the PlainDate range.
        for (index, (left, right)) in [
            (year_local, other_year_local),
            (month_local, other_month_local),
            (day_local, other_day_local),
        ]
        .into_iter()
        .enumerate()
        {
            (left).load(function);
            (right).load(function);
            function.instruction(&Instruction::I64Eq);
            if index != 0 {
                function.instruction(&Instruction::I32And);
            }
        }
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            [year_local, month_local, day_local],
            function,
        )?;
        for (source, destination) in
            reference
                .fields()
                .into_iter()
                .zip([year_local, month_local, day_local])
        {
            (source).load(function);
            (destination).store(function);
        }
        reference.release(self, function);
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainYearMonth,
            [other_year_local, other_month_local, other_day_local],
            function,
        )?;
        for (source, destination) in reference.fields().into_iter().zip([
            other_year_local,
            other_month_local,
            other_day_local,
        ]) {
            (source).load(function);
            (destination).store(function);
        }
        reference.release(self, function);
        self.emit_temporal_reject_iso_date(
            other_year_local,
            other_month_local,
            other_day_local,
            function,
        )?;
        self.emit_temporal_difference_calendar_date(
            &calendar,
            [year_local, month_local, day_local],
            [other_year_local, other_month_local, other_day_local],
            largest_unit_local,
            years_local,
            months_local,
            remainder_local,
            quantum_local,
            function,
        );

        // RoundRelativeDuration is omitted only for unrounded month precision.
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);

        self.emit_temporal_compare_iso_date(
            [other_year_local, other_month_local, other_day_local],
            [year_local, month_local, day_local],
            sign_local,
            function,
        );
        (years_local).load(function);
        (retained_years_local).store(function);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (years_local).load(function);
        function.instruction(&Instruction::Else);
        (months_local).load(function);
        function.instruction(&Instruction::End);
        (increment_local).load(function);
        function.instruction(&Instruction::I64DivS);
        (quotient_local).store(function);
        (quotient_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (anchor_local).store(function);
        (increment_local).load(function);
        (quantum_local).store(function);

        // NudgeToCalendarUnit constructs and validates both bracketing dates,
        // even when the destination is exactly the first one or rounds down.
        // Distances are measured in days because calendar months vary in length.
        for (expand, days_local) in [(false, start_days_local), (true, end_days_local)] {
            (anchor_local).load(function);
            if expand {
                (quantum_local).load(function);
                (sign_local).load(function);
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
            }
            (month_step_local).store(function);
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (month_step_local).load(function);
            (year_step_local).store(function);
            function.instruction(&Instruction::I64Const(0));
            (month_step_local).store(function);
            function.instruction(&Instruction::Else);
            (retained_years_local).load(function);
            (year_step_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.emit_temporal_year_month_calendar_anchor(
                &calendar,
                [year_local, month_local, day_local],
                year_step_local,
                month_step_local,
                [scratch_year_local, scratch_month_local, scratch_day_local],
                function,
            )?;
            self.emit_temporal_plain_date_epoch_days(
                scratch_year_local,
                scratch_month_local,
                scratch_day_local,
                days_local,
                function,
            );
        }
        self.emit_temporal_plain_date_epoch_days(
            other_year_local,
            other_month_local,
            other_day_local,
            dest_days_local,
            function,
        );
        (dest_days_local).load(function);
        (start_days_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (remainder_local).store(function);
        (end_days_local).load(function);
        (start_days_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (end_days_local).store(function);
        for local in [remainder_local, end_days_local] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(0));
            (local).load(function);
            function.instruction(&Instruction::I64Sub);
            (local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            end_days_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        (month_step_local).store(function);
        (month_step_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (quantum_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (anchor_local).load(function);
        function.instruction(&Instruction::I64Add);
        (anchor_local).store(function);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (anchor_local).load(function);
        (years_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (months_local).store(function);
        function.instruction(&Instruction::Else);
        (retained_years_local).load(function);
        (years_local).store(function);
        (anchor_local).load(function);
        (months_local).store(function);
        // Bubble only after an expanded nudge. The shared owner validates the
        // next actual year anchor and normalizes the virtual calendar pair.
        (month_step_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_bubble_calendar_difference(
            &calendar,
            [year_local, month_local, day_local],
            years_local,
            months_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (unit, local) in [
            (TemporalUnit::Year, years_local),
            (TemporalUnit::Month, months_local),
        ] {
            self.emit_temporal_duration_set_integer_field(&duration_locals, unit, local, function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        match operation {
            TemporalPlainDifferenceOperation::Until => {}
            TemporalPlainDifferenceOperation::Since => {
                self.emit_temporal_duration_negate_fields(&duration_locals, function);
            }
        }
        self.emit_create_temporal_duration(&duration_locals, function)?;

        other_calendar.release(self, function);
        calendar.release(self, function);
        self.release_temporal_duration_field_locals(duration_locals, function);
        for local in [
            retained_years_local,
            year_step_local,
            month_step_local,
            scratch_day_local,
            scratch_month_local,
            scratch_year_local,
            dest_days_local,
            end_days_local,
            start_days_local,
            remainder_local,
            sign_local,
            quotient_local,
            months_local,
            anchor_local,
            quantum_local,
            years_local,
            original_mode_local,
            mode_local,
            increment_local,
            smallest_unit_local,
            largest_unit_local,
            other_day_local,
            other_month_local,
            other_year_local,
            day_local,
            month_local,
            year_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }
}
