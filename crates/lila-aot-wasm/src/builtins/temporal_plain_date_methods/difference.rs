//! Existing calendar difference and rounding owner, consumed by builtin dispatch.

use super::*;

impl FunctionBuilder<'_> {
    /// `NudgeToCalendarUnit` for a date-only receiver: `smallestUnit` is
    /// `year`, `month` or `week`, none of which has a fixed length, so the
    /// difference cannot be rounded by dividing. The proposal instead dates
    /// both candidates — the already-truncated `r1` in `years/months/weeks`,
    /// and `r2` one `increment` further in the direction of travel — and asks
    /// where `other` falls between them.
    ///
    /// Deliberately *not* `emit_temporal_plain_date_time_nudge_calendar_unit`
    /// with zeroed times: that helper measures the bracket in nanoseconds, and
    /// a `PlainDate` bracket can be 200,000,001 days wide, which is 1.7e22
    /// nanoseconds and overflows `i64`. With no time-of-day to account for, the
    /// same comparison is exact in the day domain.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_plain_date_nudge_calendar_unit(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        date: [I64Local; 3],
        other: [I64Local; 3],
        smallest_unit_local: I64Local,
        increment_local: I64Local,
        mode_local: I64Local,
        years_local: I64Local,
        months_local: I64Local,
        weeks_local: I64Local,
        expanded_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(0));
        (expanded_local).store(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let zero_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let step_local = self.runtime_schema().reserve_i64_local(function);
        let receiver_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let other_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let start_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let end_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let numerator_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let start_year_local = self.runtime_schema().reserve_i64_local(function);
        let start_month_local = self.runtime_schema().reserve_i64_local(function);
        let start_day_local = self.runtime_schema().reserve_i64_local(function);
        let end_year_local = self.runtime_schema().reserve_i64_local(function);
        let end_month_local = self.runtime_schema().reserve_i64_local(function);
        let end_day_local = self.runtime_schema().reserve_i64_local(function);
        let nudge_years_local = self.runtime_schema().reserve_i64_local(function);
        let nudge_months_local = self.runtime_schema().reserve_i64_local(function);
        let nudge_weeks_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (zero_local).store(function);
        self.emit_temporal_plain_date_epoch_days(
            date[0],
            date[1],
            date[2],
            receiver_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            other[0],
            other[1],
            other[2],
            other_epoch_local,
            function,
        );

        // `DurationSign` of the untruncated difference. With no time-of-day the
        // date comparison is the whole answer.
        function.instruction(&Instruction::I64Const(0));
        (sign_local).store(function);
        (other_epoch_local).load(function);
        (receiver_epoch_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (other_epoch_local).load(function);
        (receiver_epoch_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (sign_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        (sign_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);

        (sign_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (step_local).store(function);
        for (source, destination) in [
            (years_local, nudge_years_local),
            (months_local, nudge_months_local),
            (weeks_local, nudge_weeks_local),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        for (unit, local) in [
            (TemporalUnit::Year, nudge_years_local),
            (TemporalUnit::Month, nudge_months_local),
            (TemporalUnit::Week, nudge_weeks_local),
        ] {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (local).load(function);
            (step_local).load(function);
            function.instruction(&Instruction::I64Add);
            (local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        for (year, month, day) in [
            (start_year_local, start_month_local, start_day_local),
            (end_year_local, end_month_local, end_day_local),
        ] {
            for (source, destination) in [(date[0], year), (date[1], month), (date[2], day)] {
                (source).load(function);
                (destination).store(function);
            }
        }
        // Both candidates go through `AddISODate`, so a rounding increment that
        // walks either of them off the representable range throws here — which
        // is what `throws-if-rounded-date-outside-valid-iso-range.js` asserts.
        self.emit_temporal_add_calendar_date(
            calendar,
            start_year_local,
            start_month_local,
            start_day_local,
            years_local,
            months_local,
            weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_add_calendar_date(
            calendar,
            end_year_local,
            end_month_local,
            end_day_local,
            nudge_years_local,
            nudge_months_local,
            nudge_weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            start_year_local,
            start_month_local,
            start_day_local,
            start_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            end_year_local,
            end_month_local,
            end_day_local,
            end_epoch_local,
            function,
        );

        // `numerator` is `other - r1`, signed with the direction of travel;
        // `quantum` is the bracket width, always positive.
        (other_epoch_local).load(function);
        (start_epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (numerator_local).store(function);
        (end_epoch_local).load(function);
        (start_epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (quantum_local).store(function);

        (quantum_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_time_round_nanoseconds(
            numerator_local,
            quantum_local,
            mode_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // `other` lies inside the bracket, so the rounded value is either zero
        // (keep `r1`) or the whole bracket (take `r2`).
        (numerator_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (expanded_local).store(function);
        for (source, destination) in [
            (nudge_years_local, years_local),
            (nudge_months_local, months_local),
            (nudge_weeks_local, weeks_local),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            nudge_weeks_local,
            nudge_months_local,
            nudge_years_local,
            end_day_local,
            end_month_local,
            end_year_local,
            start_day_local,
            start_month_local,
            start_year_local,
            quantum_local,
            numerator_local,
            end_epoch_local,
            start_epoch_local,
            other_epoch_local,
            receiver_epoch_local,
            step_local,
            sign_local,
            zero_local,
            overflow_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `until` and `since`, both through
    /// `DifferenceTemporalPlainDate`.
    ///
    /// Three things differ from the `PlainDateTime` shape beyond the missing
    /// time half:
    ///
    /// * both unit options are confined to `year..day`, so
    ///   `until/throws-with-time-units.js` gets its RangeError and
    ///   `ValidateTemporalRoundingIncrement` never applies (a date unit has no
    ///   maximum increment);
    /// * a `day` smallestUnit rounds the *day count*, not a nanosecond count.
    ///   `add/argument-duration-max-plus-min-date.js` reaches a 200,000,001-day
    ///   span, and that many nanoseconds does not fit in `i64`;
    /// * `NudgeToCalendarUnit` is fed `r1` already truncated to a multiple of
    ///   `roundingIncrement`, and `BubbleRelativeDuration` folds a calendar
    ///   year's months back into a year afterwards. `until/roundingincrement.js`
    ///   and `until/round-cross-unit-boundary.js` need those two respectively.
    pub(in crate::builtins) fn emit_temporal_plain_date_until_or_since(
        &mut self,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let year_local = schema.reserve_i64_local(function);
        let month_local = schema.reserve_i64_local(function);
        let day_local = schema.reserve_i64_local(function);
        let other_year_local = schema.reserve_i64_local(function);
        let other_month_local = schema.reserve_i64_local(function);
        let other_day_local = schema.reserve_i64_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let largest_unit_local = schema.reserve_i64_local(function);
        let smallest_unit_local = schema.reserve_i64_local(function);
        let increment_local = schema.reserve_i64_local(function);
        let mode_local = schema.reserve_i64_local(function);
        let original_mode_local = schema.reserve_i64_local(function);
        let years_local = schema.reserve_i64_local(function);
        let months_local = schema.reserve_i64_local(function);
        let weeks_local = schema.reserve_i64_local(function);
        let days_local = schema.reserve_i64_local(function);
        let carry_local = schema.reserve_i64_local(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_plain_date_receiver_fields(
            &[year_local, month_local, day_local],
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        // Conversion omits overflow. Calendar equality precedes all options.
        self.emit_temporal_to_temporal_date(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &[other_year_local, other_month_local, other_day_local],
            &other_calendar_value,
            function,
        )?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        let other_calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &other_calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_temporal_require_same_calendar(
            calendar.calendar_id(),
            other_calendar.calendar_id(),
            TemporalDifferenceGuard::PlainDateSameCalendar,
            function,
        )?;
        other_calendar.release(self, function);
        // Largest unit, increment, mode and smallest unit are acquired before
        // the algorithm's range/default validation, in that observable order.
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
                // `NegateRoundingMode`: ceil and floor swap, as do halfCeil and
                // halfFloor; the sign-symmetric modes are unchanged.
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
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        (smallest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Day,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_UNIT_OPTION,
            function,
        )?;
        // An unset or `"auto"` largestUnit falls back to the larger of day and
        // the smallest unit: `day` by default, `year` for
        // `{smallestUnit: "years"}`.
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        (largest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Day,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_UNIT_OPTION,
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;

        for local in [years_local, months_local, weeks_local, days_local] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.emit_temporal_difference_calendar_date(
            &calendar,
            [year_local, month_local, day_local],
            [other_year_local, other_month_local, other_day_local],
            largest_unit_local,
            years_local,
            months_local,
            weeks_local,
            days_local,
            function,
        );

        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        // `emit_temporal_plain_time_round_nanoseconds` is unit-agnostic: it
        // rounds a signed count to a multiple of the increment under the mode.
        // Applying it to the day count directly avoids the nanosecond scaling
        // that would overflow on a full-range span.
        self.emit_temporal_plain_time_round_nanoseconds(
            days_local,
            increment_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        // `smallestUnit` is year, month or week — the range check above leaves
        // no other option below `day`.
        //
        // Everything under the smallest unit drops, which is both the `trunc`
        // answer and the lower of the two candidates every other mode picks
        // between.
        for (limit, local) in [
            (TemporalUnit::Week.code(), days_local),
            (TemporalUnit::Month.code(), weeks_local),
            (TemporalUnit::Year.code(), months_local),
        ] {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(limit));
            function.instruction(&Instruction::I64LeS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // `NudgeToCalendarUnit` defines `r1` as
        // `RoundTowardZero(value / increment) * increment`, so the smallest
        // unit is truncated to a multiple of the increment *before* the two
        // candidates are dated. `I64DivS` truncates toward zero, which is the
        // spec's rounding for both signs.
        for (unit, local) in [
            (TemporalUnit::Year, years_local),
            (TemporalUnit::Month, months_local),
            (TemporalUnit::Week, weeks_local),
        ] {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (local).load(function);
            (increment_local).load(function);
            function.instruction(&Instruction::I64DivS);
            (increment_local).load(function);
            function.instruction(&Instruction::I64Mul);
            (local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_plain_date_nudge_calendar_unit(
            &calendar,
            [year_local, month_local, day_local],
            [other_year_local, other_month_local, other_day_local],
            smallest_unit_local,
            increment_local,
            mode_local,
            years_local,
            months_local,
            weeks_local,
            carry_local,
            function,
        )?;
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        (carry_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
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
        calendar.release(self, function);

        match operation {
            TemporalPlainDifferenceOperation::Until => {}
            TemporalPlainDifferenceOperation::Since => {
                for local in [years_local, months_local, weeks_local, days_local] {
                    function.instruction(&Instruction::I64Const(0));
                    (local).load(function);
                    function.instruction(&Instruction::I64Sub);
                    (local).store(function);
                }
            }
        }
        // The time tail is identically zero, so there is nothing for
        // `BalanceTimeDuration` to do.
        self.emit_temporal_duration_zero_fields(&duration_locals, function);
        for (source, unit) in [
            (years_local, TemporalUnit::Year),
            (months_local, TemporalUnit::Month),
            (weeks_local, TemporalUnit::Week),
            (days_local, TemporalUnit::Day),
        ] {
            self.emit_temporal_duration_set_integer_field(&duration_locals, unit, source, function);
        }
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals, function);
        for local in [
            carry_local,
            days_local,
            weeks_local,
            months_local,
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
        options.clear(function);
        argument.clear(function);
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        Ok(())
    }
}
