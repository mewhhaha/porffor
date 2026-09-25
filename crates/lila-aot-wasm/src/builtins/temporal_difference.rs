//! Exact time-duration rounding and relative calendar rounding for the
//! differences of plain date-times, where every day is 24 hours. Zoned
//! differences run on exact times in `temporal_zoned_difference.rs`.

use super::super::*;
use super::temporal_duration::TemporalExactDivisor;
use super::temporal_options::{TemporalOverflow, TemporalUnit};
use super::temporal_plain_date_time_methods::{
    ResolvedTemporalDateTimeDifferenceSettings, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;

impl<'a> FunctionBuilder<'a> {
    /// The input pair has one sign and a subsecond magnitude below 10^9.
    /// Quantums are bounded by their next time unit; no whole duration is
    /// multiplied into nanoseconds.
    pub(super) fn emit_temporal_round_difference_time(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        smallest_unit_local: u32,
        increment_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) {
        let quantum_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::LocalSet(quantum_local));
        for (unit, scale) in [
            (TemporalUnit::Day, 86_400),
            (TemporalUnit::Hour, 3_600),
            (TemporalUnit::Minute, 60),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(quantum_local));
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
        function.instruction(&Instruction::End);
        self.release_temp_local(quantum_local);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_difference_date_time(
        &mut self,
        field_locals: &[u32; 9],
        other_locals: &[u32; 9],
        settings: &ResolvedTemporalDateTimeDifferenceSettings,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expanded_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let other_total_local = self.reserve_temp_local();
        let date_sign_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let years_local = self.reserve_temp_local();
        let months_local = self.reserve_temp_local();
        let weeks_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let adjusted_year_local = self.reserve_temp_local();
        let adjusted_month_local = self.reserve_temp_local();
        let adjusted_day_local = self.reserve_temp_local();
        let epoch_local = self.reserve_temp_local();
        let time_largest_unit_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();
        let largest_unit_local = settings.largest_unit_local;
        let smallest_unit_local = settings.smallest_unit_local;
        let increment_local = settings.increment_local;
        let mode_local = settings.mode_local;
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
            function.instruction(&Instruction::LocalSet(local));
        }

        self.emit_temporal_compare_iso_date(
            [field_locals[0], field_locals[1], field_locals[2]],
            [other_locals[0], other_locals[1], other_locals[2]],
            date_sign_local,
            function,
        );
        // `DifferencePlainDateTimeWithRounding` step 1: equal endpoints are
        // the zero duration, before any rounding.
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_zero_fields(&duration_locals, function);
        self.emit_create_temporal_duration(&duration_locals, function)?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::LocalGet(adjusted_day_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::LocalSet(time_largest_unit_local));
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
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_epoch_days(
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            epoch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(epoch_local));
        self.emit_temporal_civil_from_days(
            epoch_local,
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_iso_date(
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
        function.instruction(&Instruction::LocalSet(time_largest_unit_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        // NudgeToDayOrTime includes days in the quotient parity.
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        self.emit_temporal_round_difference_time(
            seconds_local,
            subsecond_local,
            smallest_unit_local,
            increment_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalSet(epoch_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(date_sign_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(expanded_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::LocalSet(epoch_local));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::I64Const(86_400));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(seconds_local));
            function.instruction(&Instruction::LocalGet(total_local));
            function.instruction(&Instruction::LocalSet(subsecond_local));
            self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
            function.instruction(&Instruction::LocalGet(seconds_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(subsecond_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(date_sign_local));
            self.emit_temporal_round_difference_time(
                seconds_local,
                subsecond_local,
                smallest_unit_local,
                increment_local,
                mode_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(seconds_local));
            function.instruction(&Instruction::I64Const(86_400));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(days_local));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::LocalGet(epoch_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalGet(date_sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(expanded_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(seconds_local));
            function.instruction(&Instruction::Else);
        }
        self.emit_temporal_nudge_difference_calendar(
            field_locals,
            other_locals,
            [years_local, months_local, weeks_local, days_local],
            smallest_unit_local,
            increment_local,
            mode_local,
            expanded_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_bubble_difference(
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
                    function.instruction(&Instruction::LocalGet(local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(local));
                }
            }
        }
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
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
        function.instruction(&Instruction::LocalGet(
            duration_locals.number_bits(TemporalUnit::Day),
        ));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(
            duration_locals.number_bits(TemporalUnit::Day),
        ));
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        self.release_temp_local(time_largest_unit_local);
        self.release_temp_local(epoch_local);
        self.release_temp_local(adjusted_day_local);
        self.release_temp_local(adjusted_month_local);
        self.release_temp_local(adjusted_year_local);
        self.release_temp_local(days_local);
        self.release_temp_local(weeks_local);
        self.release_temp_local(months_local);
        self.release_temp_local(years_local);
        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        self.release_temp_local(date_sign_local);
        self.release_temp_local(other_total_local);
        self.release_temp_local(total_local);
        self.release_temp_local(expanded_local);
        Ok(())
    }

    /// NudgeToCalendarUnit for the ISO calendar without a time zone. The two
    /// bracket dates are checked before selecting a result.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_nudge_difference_calendar(
        &mut self,
        origin: &[u32; 9],
        destination: &[u32; 9],
        duration: [u32; 4],
        smallest_unit_local: u32,
        increment_local: u32,
        mode_local: u32,
        expanded_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_epoch_local = self.reserve_temp_local();
        let destination_epoch_local = self.reserve_temp_local();
        let origin_time_local = self.reserve_temp_local();
        let destination_time_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let step_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let start_epoch_local = self.reserve_temp_local();
        let end_epoch_local = self.reserve_temp_local();
        let distance_local = self.reserve_temp_local();
        let tail_local = self.reserve_temp_local();
        let width_local = self.reserve_temp_local();
        let twice_local = self.reserve_temp_local();
        let encoded_local = self.reserve_temp_local();
        let four_local = self.reserve_temp_local();
        let take_end_local = self.reserve_temp_local();
        let shifted_local = self.reserve_temp_local();
        let start_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let end_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let end_duration = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
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
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(origin_epoch_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(origin_epoch_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(shifted_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(four_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(step_local));
        for (index, unit) in [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ]
        .into_iter()
        .enumerate()
        {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            if unit == TemporalUnit::Week {
                function.instruction(&Instruction::LocalGet(duration[2]));
                function.instruction(&Instruction::LocalGet(duration[3]));
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::I64DivS);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(duration[2]));
            }
            function.instruction(&Instruction::LocalGet(duration[index]));
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalTee(quotient_local));
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(duration[index]));
            for local in duration.iter().skip(index + 1) {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(*local));
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        for index in 0..4 {
            function.instruction(&Instruction::LocalGet(duration[index]));
            function.instruction(&Instruction::LocalSet(end_duration[index]));
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
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(end_duration[index]));
            function.instruction(&Instruction::LocalGet(step_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(end_duration[index]));
            function.instruction(&Instruction::End);
        }
        for date in [start_date, end_date] {
            for index in 0..3 {
                function.instruction(&Instruction::LocalGet(origin[index]));
                function.instruction(&Instruction::LocalSet(date[index]));
            }
        }
        self.emit_temporal_add_iso_date(
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
        self.emit_temporal_add_iso_date(
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
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(shifted_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expanded_local));
        for index in 0..4 {
            function.instruction(&Instruction::LocalGet(end_duration[index]));
            function.instruction(&Instruction::LocalSet(duration[index]));
        }
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(distance_local));
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(tail_local));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(distance_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(distance_local));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(tail_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(width_local));
        function.instruction(&Instruction::LocalGet(distance_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(width_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(twice_local));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(tail_local));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(tail_local));
        function.instruction(&Instruction::LocalGet(twice_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(twice_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::LocalGet(twice_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::LocalGet(twice_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(tail_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_round_up_i32(
            encoded_local,
            four_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(destination_epoch_local));
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(take_end_local));
        function.instruction(&Instruction::LocalGet(take_end_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expanded_local));
        for index in 0..4 {
            function.instruction(&Instruction::LocalGet(end_duration[index]));
            function.instruction(&Instruction::LocalSet(duration[index]));
        }
        function.instruction(&Instruction::End);
        for local in end_duration
            .into_iter()
            .rev()
            .chain(end_date.into_iter().rev())
            .chain(start_date.into_iter().rev())
        {
            self.release_temp_local(local);
        }
        self.release_temp_local(shifted_local);
        self.release_temp_local(take_end_local);
        self.release_temp_local(four_local);
        self.release_temp_local(encoded_local);
        self.release_temp_local(twice_local);
        self.release_temp_local(width_local);
        self.release_temp_local(tail_local);
        self.release_temp_local(distance_local);
        self.release_temp_local(end_epoch_local);
        self.release_temp_local(start_epoch_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(step_local);
        self.release_temp_local(overflow_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(destination_time_local);
        self.release_temp_local(origin_time_local);
        self.release_temp_local(destination_epoch_local);
        self.release_temp_local(origin_epoch_local);
        Ok(())
    }
    /// Bubble only after expansion, checking each next calendar boundary even
    /// when it is not selected. Rounding can replace a bottom-heavy duration
    /// with a whole larger unit; arithmetic division of its fields is not equivalent.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_bubble_difference(
        &mut self,
        origin: &[u32; 9],
        duration: [u32; 4],
        time_local: u32,
        largest_unit_local: u32,
        smallest_unit_local: u32,
        expanded_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_time_local = self.reserve_temp_local();
        let nudged_time_local = self.reserve_temp_local();
        let nudged_epoch_local = self.reserve_temp_local();
        let candidate_epoch_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let nudged_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let candidate_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let candidate = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        function.instruction(&Instruction::LocalGet(expanded_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        for local in duration.into_iter().chain([time_local]) {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(sign_local));
            function.instruction(&Instruction::End);
        }
        let clock = Self::temporal_plain_date_time_time_locals(origin);
        self.emit_temporal_plain_time_total_nanoseconds(&clock, origin_time_local, function);
        for index in 0..3 {
            function.instruction(&Instruction::LocalGet(origin[index]));
            function.instruction(&Instruction::LocalSet(nudged_date[index]));
        }
        self.emit_temporal_add_iso_date(
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
        function.instruction(&Instruction::LocalGet(nudged_epoch_local));
        function.instruction(&Instruction::LocalGet(duration[3]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(nudged_epoch_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::LocalGet(time_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(nudged_time_local));
        function.instruction(&Instruction::LocalGet(nudged_time_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nudged_time_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(nudged_time_local));
        function.instruction(&Instruction::LocalGet(nudged_epoch_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(nudged_epoch_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(nudged_time_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nudged_time_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(nudged_time_local));
        function.instruction(&Instruction::LocalGet(nudged_epoch_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(nudged_epoch_local));
        function.instruction(&Instruction::End);
        for (index, unit) in [
            (2, TemporalUnit::Week),
            (1, TemporalUnit::Month),
            (0, TemporalUnit::Year),
        ] {
            function.instruction(&Instruction::LocalGet(done_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(largest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            if unit == TemporalUnit::Week {
                function.instruction(&Instruction::LocalGet(largest_unit_local));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            for slot in 0..4 {
                if slot <= index {
                    function.instruction(&Instruction::LocalGet(duration[slot]));
                } else {
                    function.instruction(&Instruction::I64Const(0));
                }
                if slot == index {
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::I64Add);
                }
                function.instruction(&Instruction::LocalSet(candidate[slot]));
            }
            for slot in 0..3 {
                function.instruction(&Instruction::LocalGet(origin[slot]));
                function.instruction(&Instruction::LocalSet(candidate_date[slot]));
            }
            self.emit_temporal_add_iso_date(
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
            function.instruction(&Instruction::LocalGet(nudged_epoch_local));
            function.instruction(&Instruction::LocalGet(candidate_epoch_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::LocalGet(nudged_epoch_local));
            function.instruction(&Instruction::LocalGet(candidate_epoch_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(nudged_time_local));
            function.instruction(&Instruction::LocalGet(origin_time_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            for slot in 0..4 {
                function.instruction(&Instruction::LocalGet(candidate[slot]));
                function.instruction(&Instruction::LocalSet(duration[slot]));
            }
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(time_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(done_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        for local in candidate
            .into_iter()
            .rev()
            .chain(candidate_date.into_iter().rev())
            .chain(nudged_date.into_iter().rev())
        {
            self.release_temp_local(local);
        }
        self.release_temp_local(zero_local);
        self.release_temp_local(done_local);
        self.release_temp_local(overflow_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(candidate_epoch_local);
        self.release_temp_local(nudged_epoch_local);
        self.release_temp_local(nudged_time_local);
        self.release_temp_local(origin_time_local);
        Ok(())
    }

    /// `DifferencePlainDateTimeWithTotal` steps 3-6 for the ISO calendar, from
    /// wall-clock `origin` to `destination`. The f64 bits of the total land in
    /// `output_bits_local`. The caller has already answered the
    /// equal-endpoints and `ISODateTimeWithinLimits` questions.
    ///
    /// A time unit — and `day`, which has no time zone to make it irregular —
    /// totals the exact difference. A calendar unit runs `NudgeToCalendarUnit`
    /// with increment 1 and `trunc`, and returns its `total`:
    /// `r1 + sign * (dest - start) / (end - start)`, computed as one exact
    /// quotient.
    pub(super) fn emit_temporal_difference_total(
        &mut self,
        origin: &[u32; 9],
        destination: &[u32; 9],
        unit_local: u32,
        output_bits_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_time_local = self.reserve_temp_local();
        let destination_time_local = self.reserve_temp_local();
        let origin_days_local = self.reserve_temp_local();
        let destination_days_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let origin_date = [origin[0], origin[1], origin[2]];
        let destination_date = [destination[0], destination[1], destination[2]];
        self.emit_temporal_plain_time_total_nanoseconds(
            &Self::temporal_plain_date_time_time_locals(origin),
            origin_time_local,
            function,
        );
        self.emit_temporal_plain_time_total_nanoseconds(
            &Self::temporal_plain_date_time_time_locals(destination),
            destination_time_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            origin_date[0],
            origin_date[1],
            origin_date[2],
            origin_days_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            destination_date[0],
            destination_date[1],
            destination_date[2],
            destination_days_local,
            function,
        );

        // Time units, and `day` when there is no time zone to make it
        // irregular: `TotalTimeDuration` of the exact difference.
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(destination_days_local));
        function.instruction(&Instruction::LocalGet(origin_days_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        self.emit_temporal_total_time_duration(
            seconds_local,
            subsecond_local,
            unit_local,
            output_bits_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_difference_calendar_total(
            origin,
            origin_time_local,
            destination_date,
            destination_time_local,
            destination_days_local,
            unit_local,
            output_bits_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            subsecond_local,
            seconds_local,
            destination_days_local,
            origin_days_local,
            destination_time_local,
            origin_time_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The calendar-unit half of [`Self::emit_temporal_difference_total`]:
    /// `DifferenceISODateTime(origin, destination, unit)`, then
    /// `NudgeToCalendarUnit`'s window (`ComputeNudgeWindow`, retried once with
    /// `additionalShift` when the destination falls outside it) and its total.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_difference_calendar_total(
        &mut self,
        origin: &[u32; 9],
        origin_time_local: u32,
        destination_date: [u32; 3],
        destination_time_local: u32,
        destination_days_local: u32,
        unit_local: u32,
        output_bits_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_local = self.reserve_temp_local();
        let time_sign_local = self.reserve_temp_local();
        let date_sign_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let r1_local = self.reserve_temp_local();
        let step_local = self.reserve_temp_local();
        let shifted_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let start_days_local = self.reserve_temp_local();
        let end_days_local = self.reserve_temp_local();
        let inside_local = self.reserve_temp_local();
        let numerator_local = self.reserve_temp_local();
        let denominator_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let high_local = self.reserve_temp_local();
        let low_local = self.reserve_temp_local();
        let adjusted = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let difference = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let window_duration = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let start_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let end_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let origin_date = [origin[0], origin[1], origin[2]];
        const UNITS: [TemporalUnit; 4] = [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ];

        // `DifferenceISODateTime`: borrow a day when the time of day runs
        // against the date, then `CalendarDateUntil` with `unit` as the
        // largest unit.
        function.instruction(&Instruction::LocalGet(destination_time_local));
        function.instruction(&Instruction::LocalGet(origin_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(time_local));
        function.instruction(&Instruction::LocalGet(time_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(time_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(time_sign_local));
        self.emit_temporal_compare_iso_date(
            origin_date,
            destination_date,
            date_sign_local,
            function,
        );
        for (source, destination) in destination_date.into_iter().zip(adjusted) {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(destination_days_local));
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_days_local));
        self.emit_temporal_civil_from_days(
            start_days_local,
            adjusted[0],
            adjusted[1],
            adjusted[2],
            function,
        );
        function.instruction(&Instruction::LocalGet(time_local));
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(time_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_iso_date(
            origin_date,
            adjusted,
            unit_local,
            difference[0],
            difference[1],
            difference[2],
            difference[3],
            function,
        );

        // `InternalDurationSign`, and the truncated count of `unit`: with
        // `unit` as the largest unit, every larger field is zero and the week
        // remainder is below seven days.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        for local in difference.into_iter().chain([time_local]) {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(sign_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(r1_local));
        for (unit, local) in UNITS.into_iter().zip(difference) {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::LocalSet(r1_local));
            function.instruction(&Instruction::End);
        }

        // `ComputeNudgeWindow`: the origin plus `r1` and `r1 + sign` units, at
        // the origin's time of day. A destination outside the window takes the
        // one `additionalShift` retry.
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(shifted_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        for (window, date, days_local, step) in [
            (window_duration, start_date, start_days_local, false),
            (window_duration, end_date, end_days_local, true),
        ] {
            function.instruction(&Instruction::LocalGet(r1_local));
            if step {
                function.instruction(&Instruction::LocalGet(sign_local));
                function.instruction(&Instruction::I64Add);
            }
            function.instruction(&Instruction::LocalSet(step_local));
            for (unit, local) in UNITS.into_iter().zip(window) {
                function.instruction(&Instruction::LocalGet(unit_local));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(step_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(local));
            }
            for (source, destination) in origin_date.into_iter().zip(date) {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }
            self.emit_temporal_add_iso_date(
                date[0],
                date[1],
                date[2],
                window[0],
                window[1],
                window[2],
                window[3],
                overflow_local,
                function,
            )?;
            self.emit_temporal_plain_date_epoch_days(date[0], date[1], date[2], days_local, function);
        }
        // Inside when `sign * (dest - start) >= 0` and `sign * (dest - end) <= 0`,
        // comparing (epoch day, time of day) pairs; both window ends share the
        // origin's time of day.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(inside_local));
        for (days_local, rejects_positive) in [(start_days_local, false), (end_days_local, true)] {
            // Leaves the (dest - bound) comparison, -1/0/1, times sign.
            function.instruction(&Instruction::LocalGet(destination_days_local));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(destination_days_local));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(destination_days_local));
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(destination_time_local));
            function.instruction(&Instruction::LocalGet(origin_time_local));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(destination_time_local));
            function.instruction(&Instruction::LocalGet(origin_time_local));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(if rejects_positive {
                &Instruction::I64GtS
            } else {
                &Instruction::I64LtS
            });
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(inside_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(inside_local));
        function.instruction(&Instruction::LocalGet(shifted_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(r1_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(r1_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(shifted_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // total = r1 + sign * (dest - start) / (end - start). Inside the
        // window both spans are at most one calendar unit plus a day, so they
        // fit i64 nanoseconds; the magnitude |r1| * den + num does not, and is
        // divided as a 128-bit integer with a single rounding.
        for (local, days_local, include_time) in [
            (numerator_local, destination_days_local, true),
            (denominator_local, end_days_local, false),
        ] {
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::LocalGet(start_days_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
            function.instruction(&Instruction::I64Mul);
            if include_time {
                function.instruction(&Instruction::LocalGet(destination_time_local));
                function.instruction(&Instruction::LocalGet(origin_time_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Add);
            }
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(r1_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(magnitude_local));
        self.emit_temporal_u64_product_plus(
            magnitude_local,
            denominator_local,
            numerator_local,
            high_local,
            low_local,
            function,
        );
        self.emit_temporal_exact_quotient_bits(
            high_local,
            low_local,
            TemporalExactDivisor::Local(denominator_local),
            output_bits_local,
            function,
        );
        // A zero total is +0 whatever the direction.
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::LocalSet(output_bits_local));
        function.instruction(&Instruction::End);

        for local in end_date
            .into_iter()
            .rev()
            .chain(start_date.into_iter().rev())
            .chain(window_duration.into_iter().rev())
            .chain(difference.into_iter().rev())
            .chain(adjusted.into_iter().rev())
        {
            self.release_temp_local(local);
        }
        for local in [
            low_local,
            high_local,
            magnitude_local,
            denominator_local,
            numerator_local,
            inside_local,
            end_days_local,
            start_days_local,
            overflow_local,
            shifted_local,
            step_local,
            r1_local,
            sign_local,
            date_sign_local,
            time_sign_local,
            time_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
