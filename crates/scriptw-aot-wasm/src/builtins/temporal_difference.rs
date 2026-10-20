//! Exact time-duration rounding and relative calendar rounding for differences.

use super::super::*;
use super::temporal_options::{TemporalOverflow, TemporalUnit};
use super::temporal_plain_date_time_methods::{
    ResolvedTemporalDateTimeDifferenceSettings, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use crate::gc_types::*;

impl<'a> FunctionBuilder<'a> {
    /// The input pair has one sign and a subsecond magnitude below 10^9.
    /// Quantums are bounded by their next time unit; no whole duration is
    /// multiplied into nanoseconds.
    pub(super) fn emit_temporal_round_difference_time(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        smallest_unit_local: I64Local,
        increment_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) {
        let quantum_local = self.runtime_schema().reserve_i64_local(function);

        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        (increment_local).load(function);
        (quantum_local).store(function);
        for (unit, scale) in [
            (TemporalUnit::Day, 86_400),
            (TemporalUnit::Hour, 3_600),
            (TemporalUnit::Minute, 60),
        ] {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (increment_local).load(function);
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            (quantum_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_duration_round_seconds(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_rounding_quantum(
            smallest_unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_duration_round_subsecond(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(quantum_local, function);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_difference_date_time(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        field_locals: &[I64Local; 9],
        other_locals: &[I64Local; 9],
        settings: &ResolvedTemporalDateTimeDifferenceSettings,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expanded_local = self.runtime_schema().reserve_i64_local(function);
        let total_local = self.runtime_schema().reserve_i64_local(function);
        let other_total_local = self.runtime_schema().reserve_i64_local(function);
        let date_sign_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let years_local = self.runtime_schema().reserve_i64_local(function);
        let months_local = self.runtime_schema().reserve_i64_local(function);
        let weeks_local = self.runtime_schema().reserve_i64_local(function);
        let days_local = self.runtime_schema().reserve_i64_local(function);
        let adjusted_year_local = self.runtime_schema().reserve_i64_local(function);
        let adjusted_month_local = self.runtime_schema().reserve_i64_local(function);
        let adjusted_day_local = self.runtime_schema().reserve_i64_local(function);
        let epoch_local = self.runtime_schema().reserve_i64_local(function);
        let time_largest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);
        let largest_unit_local = settings.largest_unit();
        let smallest_unit_local = settings.smallest_unit();
        let increment_local = settings.rounding_increment();
        let mode_local = settings.rounding_mode();
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        let other_time_locals = Self::temporal_plain_date_time_time_locals(&other_locals);
        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        self.emit_temporal_plain_time_total_nanoseconds(
            &other_time_locals,
            other_total_local,
            function,
        );

        for local in [
            years_local,
            months_local,
            weeks_local,
            days_local,
            seconds_local,
            expanded_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }

        self.emit_temporal_compare_iso_date(
            [field_locals[0], field_locals[1], field_locals[2]],
            [other_locals[0], other_locals[1], other_locals[2]],
            date_sign_local,
            function,
        );
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (total_local).load(function);
        (other_total_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_zero_fields(&duration_locals, function);
        self.emit_create_temporal_duration(&duration_locals, function)?;
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        // Keep whole seconds separate: valid dates can differ by more than
        // i64::MAX nanoseconds, while their epoch-second difference is exact.
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            other_locals[0],
            other_locals[1],
            other_locals[2],
            adjusted_day_local,
            function,
        );
        (adjusted_day_local).load(function);
        (epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        (seconds_local).store(function);
        (other_total_local).load(function);
        (total_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (total_local).store(function);
        (largest_unit_local).load(function);
        (time_largest_unit_local).store(function);
        function.instruction(&Instruction::Else);
        // Day and above: borrow a day when the time-of-day difference runs
        // against the date difference, then take a calendar difference.
        self.emit_temporal_compare_iso_date(
            [other_locals[0], other_locals[1], other_locals[2]],
            [field_locals[0], field_locals[1], field_locals[2]],
            date_sign_local,
            function,
        );
        for (source, destination) in [
            (other_locals[0], adjusted_year_local),
            (other_locals[1], adjusted_month_local),
            (other_locals[2], adjusted_day_local),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        (other_total_local).load(function);
        (total_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (total_local).store(function);
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_date_epoch_days(
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            epoch_local,
            function,
        );
        (epoch_local).load(function);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (epoch_local).store(function);
        self.emit_temporal_civil_from_days(
            epoch_local,
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            function,
        );
        (total_local).load(function);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (total_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_calendar_date(
            calendar,
            [field_locals[0], field_locals[1], field_locals[2]],
            [
                adjusted_year_local,
                adjusted_month_local,
                adjusted_day_local,
            ],
            largest_unit_local,
            years_local,
            months_local,
            weeks_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        (time_largest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        // NudgeToDayOrTime includes days in the quotient parity.
        (seconds_local).load(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);

        (total_local).load(function);
        (subsecond_local).store(function);
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        self.emit_temporal_round_difference_time(
            seconds_local,
            subsecond_local,
            smallest_unit_local,
            increment_local,
            mode_local,
            function,
        );
        (subsecond_local).load(function);
        (total_local).store(function);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        (days_local).load(function);
        (epoch_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (date_sign_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (days_local).store(function);
        (days_local).load(function);
        (epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        (expanded_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (total_local).load(function);
        function.instruction(&Instruction::I64Add);
        (total_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (seconds_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (days_local).load(function);
        (epoch_local).store(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        (seconds_local).store(function);
        (total_local).load(function);
        (subsecond_local).store(function);
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (date_sign_local).store(function);
        self.emit_temporal_round_difference_time(
            seconds_local,
            subsecond_local,
            smallest_unit_local,
            increment_local,
            mode_local,
            function,
        );
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (days_local).store(function);
        (days_local).load(function);
        (epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        (expanded_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (seconds_local).store(function);
        function.instruction(&Instruction::Else);

        self.emit_temporal_nudge_difference_calendar(
            calendar,
            field_locals,
            other_locals,
            [years_local, months_local, weeks_local, days_local],
            smallest_unit_local,
            increment_local,
            mode_local,
            expanded_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        (total_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_bubble_difference(
            calendar,
            field_locals,
            [years_local, months_local, weeks_local, days_local],
            total_local,
            largest_unit_local,
            smallest_unit_local,
            expanded_local,
            function,
        )?;

        match operation {
            TemporalPlainDifferenceOperation::Until => {}
            TemporalPlainDifferenceOperation::Since => {
                for local in [
                    years_local,
                    months_local,
                    weeks_local,
                    days_local,
                    total_local,
                    seconds_local,
                ] {
                    function.instruction(&Instruction::I64Const(0));
                    (local).load(function);
                    function.instruction(&Instruction::I64Sub);
                    (local).store(function);
                }
            }
        }
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond_local).store(function);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            time_largest_unit_local,
            &duration_locals,
            function,
        )?;
        for (source, unit) in [
            (years_local, TemporalUnit::Year),
            (months_local, TemporalUnit::Month),
            (weeks_local, TemporalUnit::Week),
        ] {
            self.emit_temporal_duration_set_integer_field(&duration_locals, unit, source, function);
        }
        duration_locals
            .number_bits(TemporalUnit::Day)
            .load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        (days_local).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        duration_locals
            .number_bits(TemporalUnit::Day)
            .store(function);
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals, function);
        self.runtime_schema()
            .release_i64_local(time_largest_unit_local, function);
        self.runtime_schema()
            .release_i64_local(epoch_local, function);
        self.runtime_schema()
            .release_i64_local(adjusted_day_local, function);
        self.runtime_schema()
            .release_i64_local(adjusted_month_local, function);
        self.runtime_schema()
            .release_i64_local(adjusted_year_local, function);
        self.runtime_schema()
            .release_i64_local(days_local, function);
        self.runtime_schema()
            .release_i64_local(weeks_local, function);
        self.runtime_schema()
            .release_i64_local(months_local, function);
        self.runtime_schema()
            .release_i64_local(years_local, function);
        self.runtime_schema()
            .release_i64_local(subsecond_local, function);
        self.runtime_schema()
            .release_i64_local(seconds_local, function);
        self.runtime_schema()
            .release_i64_local(date_sign_local, function);
        self.runtime_schema()
            .release_i64_local(other_total_local, function);
        self.runtime_schema()
            .release_i64_local(total_local, function);
        self.runtime_schema()
            .release_i64_local(expanded_local, function);
        Ok(())
    }
    /// NudgeToCalendarUnit for a PlainDateTime. CalendarDateAdd checks both
    /// bracket dates before their non-throwing plain epoch projection.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_nudge_difference_calendar(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        origin: &[I64Local; 9],
        destination: &[I64Local; 9],
        duration: [I64Local; 4],
        smallest_unit_local: I64Local,
        increment_local: I64Local,
        mode_local: I64Local,
        expanded_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let destination_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let origin_time_local = self.runtime_schema().reserve_i64_local(function);
        let destination_time_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let step_local = self.runtime_schema().reserve_i64_local(function);
        let quotient_local = self.runtime_schema().reserve_i64_local(function);
        let start_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let end_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let distance_local = self.runtime_schema().reserve_i64_local(function);
        let tail_local = self.runtime_schema().reserve_i64_local(function);
        let width_local = self.runtime_schema().reserve_i64_local(function);
        let twice_local = self.runtime_schema().reserve_i64_local(function);
        let encoded_local = self.runtime_schema().reserve_i64_local(function);
        let four_local = self.runtime_schema().reserve_i64_local(function);
        let take_end_local = self.runtime_schema().reserve_i64_local(function);
        let shifted_local = self.runtime_schema().reserve_i64_local(function);
        let start_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let end_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let end_duration = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let origin_date = [origin[0], origin[1], origin[2]];
        let destination_date = [destination[0], destination[1], destination[2]];
        let origin_clock = Self::temporal_plain_date_time_time_locals(origin);
        let destination_clock = Self::temporal_plain_date_time_time_locals(destination);
        self.emit_temporal_plain_time_total_nanoseconds(&origin_clock, origin_time_local, function);
        self.emit_temporal_plain_time_total_nanoseconds(
            &destination_clock,
            destination_time_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            origin_date[0],
            origin_date[1],
            origin_date[2],
            origin_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            destination_date[0],
            destination_date[1],
            destination_date[2],
            destination_epoch_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        (destination_epoch_local).load(function);
        (origin_epoch_local).load(function);
        function.instruction(&Instruction::I64LtS);
        (destination_epoch_local).load(function);
        (origin_epoch_local).load(function);
        function.instruction(&Instruction::I64Eq);
        (destination_time_local).load(function);
        (origin_time_local).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (sign_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (shifted_local).store(function);
        function.instruction(&Instruction::I64Const(4));
        (four_local).store(function);
        (increment_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (step_local).store(function);
        for (index, unit) in [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ]
        .into_iter()
        .enumerate()
        {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            if unit == TemporalUnit::Week {
                (duration[2]).load(function);
                (duration[3]).load(function);
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::I64DivS);
                function.instruction(&Instruction::I64Add);
                (duration[2]).store(function);
            }
            (duration[index]).load(function);
            (increment_local).load(function);
            function.instruction(&Instruction::I64DivS);
            (quotient_local).store(function);
            (quotient_local).load(function);
            (increment_local).load(function);
            function.instruction(&Instruction::I64Mul);
            (duration[index]).store(function);
            for local in duration.iter().skip(index + 1) {
                function.instruction(&Instruction::I64Const(0));
                (*local).store(function);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        for index in 0..4 {
            (duration[index]).load(function);
            (end_duration[index]).store(function);
        }
        for (index, unit) in [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ]
        .into_iter()
        .enumerate()
        {
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (end_duration[index]).load(function);
            (step_local).load(function);
            function.instruction(&Instruction::I64Add);
            (end_duration[index]).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for date in [start_date, end_date] {
            for index in 0..3 {
                (origin[index]).load(function);
                (date[index]).store(function);
            }
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            start_date[0],
            start_date[1],
            start_date[2],
            duration[0],
            duration[1],
            duration[2],
            duration[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_add_calendar_date(
            calendar,
            end_date[0],
            end_date[1],
            end_date[2],
            end_duration[0],
            end_duration[1],
            end_duration[2],
            end_duration[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            start_date[0],
            start_date[1],
            start_date[2],
            start_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            end_date[0],
            end_date[1],
            end_date[2],
            end_epoch_local,
            function,
        );
        (destination_epoch_local).load(function);
        (end_epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        (destination_epoch_local).load(function);
        (end_epoch_local).load(function);
        function.instruction(&Instruction::I64Eq);
        (destination_time_local).load(function);
        (origin_time_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        (shifted_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shifted_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (expanded_local).store(function);
        for index in 0..4 {
            (end_duration[index]).load(function);
            (duration[index]).store(function);
        }
        (quotient_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Add);
        (quotient_local).store(function);
        function.instruction(&Instruction::Br(1));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (destination_epoch_local).load(function);
        (start_epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (distance_local).store(function);
        (destination_time_local).load(function);
        (origin_time_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (tail_local).store(function);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (distance_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (distance_local).store(function);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        (tail_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (end_epoch_local).load(function);
        (start_epoch_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (width_local).store(function);
        (distance_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (width_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (twice_local).store(function);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (tail_local).store(function);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Sub);
        (tail_local).store(function);
        (twice_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (twice_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        (encoded_local).store(function);
        (twice_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(3));
        (encoded_local).store(function);
        (twice_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (tail_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(2));
        (encoded_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (destination_epoch_local).load(function);
        (start_epoch_local).load(function);
        function.instruction(&Instruction::I64Eq);
        (destination_time_local).load(function);
        (origin_time_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (encoded_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_round_up_i32(
            encoded_local,
            four_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        (destination_epoch_local).load(function);
        (end_epoch_local).load(function);
        function.instruction(&Instruction::I64Eq);
        (destination_time_local).load(function);
        (origin_time_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (take_end_local).store(function);
        (take_end_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (expanded_local).store(function);
        for index in 0..4 {
            (end_duration[index]).load(function);
            (duration[index]).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in end_duration
            .into_iter()
            .rev()
            .chain(end_date.into_iter().rev())
            .chain(start_date.into_iter().rev())
        {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.runtime_schema()
            .release_i64_local(shifted_local, function);
        self.runtime_schema()
            .release_i64_local(take_end_local, function);
        self.runtime_schema()
            .release_i64_local(four_local, function);
        self.runtime_schema()
            .release_i64_local(encoded_local, function);
        self.runtime_schema()
            .release_i64_local(twice_local, function);
        self.runtime_schema()
            .release_i64_local(width_local, function);
        self.runtime_schema()
            .release_i64_local(tail_local, function);
        self.runtime_schema()
            .release_i64_local(distance_local, function);
        self.runtime_schema()
            .release_i64_local(end_epoch_local, function);
        self.runtime_schema()
            .release_i64_local(start_epoch_local, function);
        self.runtime_schema()
            .release_i64_local(quotient_local, function);
        self.runtime_schema()
            .release_i64_local(step_local, function);
        self.runtime_schema()
            .release_i64_local(overflow_local, function);
        self.runtime_schema()
            .release_i64_local(sign_local, function);
        self.runtime_schema()
            .release_i64_local(destination_time_local, function);
        self.runtime_schema()
            .release_i64_local(origin_time_local, function);
        self.runtime_schema()
            .release_i64_local(destination_epoch_local, function);
        self.runtime_schema()
            .release_i64_local(origin_epoch_local, function);
        Ok(())
    }
    /// Bubble only after expansion, checking each next calendar boundary even
    /// when it is not selected. Rounding can replace a bottom-heavy duration
    /// with a whole larger unit; arithmetic division of its fields is not equivalent.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_bubble_difference(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        origin: &[I64Local; 9],
        duration: [I64Local; 4],
        time_local: I64Local,
        largest_unit_local: I64Local,
        smallest_unit_local: I64Local,
        expanded_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_time_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_time_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let candidate_epoch_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let done_local = self.runtime_schema().reserve_i64_local(function);
        let zero_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let candidate_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let candidate = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        (expanded_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (done_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (zero_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        for local in duration.into_iter().chain([time_local]) {
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(-1));
            (sign_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        let clock = Self::temporal_plain_date_time_time_locals(origin);
        self.emit_temporal_plain_time_total_nanoseconds(&clock, origin_time_local, function);
        for index in 0..3 {
            (origin[index]).load(function);
            (nudged_date[index]).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            nudged_date[0],
            nudged_date[1],
            nudged_date[2],
            duration[0],
            duration[1],
            duration[2],
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            nudged_date[0],
            nudged_date[1],
            nudged_date[2],
            nudged_epoch_local,
            function,
        );
        // AddTimeDurationToEpochNanoseconds permits an out-of-range nudged
        // instant. Only the larger calendar candidates below are validated.
        (nudged_epoch_local).load(function);
        (duration[3]).load(function);
        function.instruction(&Instruction::I64Add);
        (nudged_epoch_local).store(function);
        (origin_time_local).load(function);
        (time_local).load(function);
        function.instruction(&Instruction::I64Add);
        (nudged_time_local).store(function);
        (nudged_time_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (nudged_time_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        (nudged_time_local).store(function);
        (nudged_epoch_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (nudged_epoch_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (nudged_time_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (nudged_time_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Sub);
        (nudged_time_local).store(function);
        (nudged_epoch_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (nudged_epoch_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (index, unit) in [
            (2, TemporalUnit::Week),
            (1, TemporalUnit::Month),
            (0, TemporalUnit::Year),
        ] {
            (done_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            (smallest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            if unit == TemporalUnit::Week {
                (largest_unit_local).load(function);
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            self.open_frame(ControlFrameKind::If, function);
            for slot in 0..4 {
                if slot <= index {
                    (duration[slot]).load(function);
                } else {
                    function.instruction(&Instruction::I64Const(0));
                }
                if slot == index {
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                }
                (candidate[slot]).store(function);
            }
            for slot in 0..3 {
                (origin[slot]).load(function);
                (candidate_date[slot]).store(function);
            }
            self.emit_temporal_add_calendar_date(
                calendar,
                candidate_date[0],
                candidate_date[1],
                candidate_date[2],
                candidate[0],
                candidate[1],
                candidate[2],
                candidate[3],
                overflow_local,
                function,
            )?;
            self.emit_temporal_plain_date_epoch_days(
                candidate_date[0],
                candidate_date[1],
                candidate_date[2],
                candidate_epoch_local,
                function,
            );
            (nudged_epoch_local).load(function);
            (candidate_epoch_local).load(function);
            function.instruction(&Instruction::I64Sub);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            (nudged_epoch_local).load(function);
            (candidate_epoch_local).load(function);
            function.instruction(&Instruction::I64Eq);
            (nudged_time_local).load(function);
            (origin_time_local).load(function);
            function.instruction(&Instruction::I64Sub);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            for slot in 0..4 {
                (candidate[slot]).load(function);
                (duration[slot]).store(function);
            }
            function.instruction(&Instruction::I64Const(0));
            (time_local).store(function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            (done_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in candidate
            .into_iter()
            .rev()
            .chain(candidate_date.into_iter().rev())
            .chain(nudged_date.into_iter().rev())
        {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.runtime_schema()
            .release_i64_local(zero_local, function);
        self.runtime_schema()
            .release_i64_local(done_local, function);
        self.runtime_schema()
            .release_i64_local(overflow_local, function);
        self.runtime_schema()
            .release_i64_local(sign_local, function);
        self.runtime_schema()
            .release_i64_local(candidate_epoch_local, function);
        self.runtime_schema()
            .release_i64_local(nudged_epoch_local, function);
        self.runtime_schema()
            .release_i64_local(nudged_time_local, function);
        self.runtime_schema()
            .release_i64_local(origin_time_local, function);
        Ok(())
    }
}
