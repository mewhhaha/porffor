//! Exact time-duration rounding and relative calendar rounding for differences.

use super::super::*;
use super::temporal_options::{TemporalOverflow, TemporalUnit};
use super::temporal_plain_date_time_methods::{
    ResolvedTemporalDateTimeDifferenceSettings, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;

#[derive(Clone, Copy)]
pub(super) enum TemporalDifferenceContext {
    Plain,
    Zoned { offset_seconds_local: u32 },
}

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
        context: TemporalDifferenceContext,
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
        match context {
            TemporalDifferenceContext::Plain => {
                // NudgeToDayOrTime includes days in the quotient parity.
                function.instruction(&Instruction::LocalGet(seconds_local));
                function.instruction(&Instruction::LocalGet(days_local));
                function.instruction(&Instruction::I64Const(86_400));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(seconds_local));
            }
            TemporalDifferenceContext::Zoned { .. } => {
                self.emit_temporal_zoned_time_nudge_range(
                    field_locals,
                    [years_local, months_local, weeks_local, days_local],
                    total_local,
                    smallest_unit_local,
                    increment_local,
                    context,
                    function,
                )?;
            }
        }
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
        if matches!(context, TemporalDifferenceContext::Zoned { .. }) {
            function.instruction(&Instruction::LocalGet(days_local));
            function.instruction(&Instruction::I64Add);
        }
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
        if matches!(context, TemporalDifferenceContext::Plain) {
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
            context,
            function,
        )?;
        if matches!(context, TemporalDifferenceContext::Plain) {
            function.instruction(&Instruction::End);
        }
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
            context,
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
        for (source, index) in [(years_local, 0_usize), (months_local, 1), (weeks_local, 2)] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(duration_locals[index]));
        }
        function.instruction(&Instruction::LocalGet(duration_locals[3]));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duration_locals[3]));
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
    /// Validate a calendar candidate in the range owned by its consumer.
    fn emit_temporal_difference_candidate_range(
        &mut self,
        date: [u32; 3],
        time_local: u32,
        context: TemporalDifferenceContext,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match context {
            // CalendarDateAdd already checked ISODateWithinLimits. Its plain
            // epoch projection accepts the boundary midnight and cannot throw.
            TemporalDifferenceContext::Plain => {}
            TemporalDifferenceContext::Zoned {
                offset_seconds_local,
            } => {
                let epoch_local = self.reserve_temp_local();
                let seconds_local = self.reserve_temp_local();
                let subsecond_local = self.reserve_temp_local();
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.emit_temporal_plain_date_epoch_days(
                    date[0],
                    date[1],
                    date[2],
                    epoch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(epoch_local));
                function.instruction(&Instruction::I64Const(86_400));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::LocalGet(time_local));
                function.instruction(&Instruction::I64Const(1_000_000_000));
                function.instruction(&Instruction::I64DivS);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalGet(offset_seconds_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(seconds_local));
                function.instruction(&Instruction::LocalGet(time_local));
                function.instruction(&Instruction::I64Const(1_000_000_000));
                function.instruction(&Instruction::I64RemS);
                function.instruction(&Instruction::LocalSet(subsecond_local));
                self.emit_temporal_epoch_nanoseconds_bigint(
                    seconds_local,
                    subsecond_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_temporal_instant_validate_range(payload_local, tag_local, function)?;
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                self.release_temp_local(subsecond_local);
                self.release_temp_local(seconds_local);
                self.release_temp_local(epoch_local);
            }
        }
        Ok(())
    }

    /// NudgeToCalendarUnit for the ISO calendar and currently supported fixed
    /// offsets. The two bracket dates are checked before selecting a result.
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
        context: TemporalDifferenceContext,
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
        self.emit_temporal_difference_candidate_range(
            start_date,
            origin_time_local,
            context,
            function,
        )?;
        self.emit_temporal_difference_candidate_range(
            end_date,
            origin_time_local,
            context,
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
        context: TemporalDifferenceContext,
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
            self.emit_temporal_difference_candidate_range(
                candidate_date,
                origin_time_local,
                context,
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
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_zoned_time_nudge_range(
        &mut self,
        origin: &[u32; 9],
        duration: [u32; 4],
        time_local: u32,
        smallest_unit_local: u32,
        increment_local: u32,
        context: TemporalDifferenceContext,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let origin_time_local = self.reserve_temp_local();
        let date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        // DifferenceZonedDateTimeWithRounding has a nanosecond/1 shortcut.
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
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
        for index in 0..3 {
            function.instruction(&Instruction::LocalGet(origin[index]));
            function.instruction(&Instruction::LocalSet(date[index]));
        }
        let clock = Self::temporal_plain_date_time_time_locals(origin);
        self.emit_temporal_plain_time_total_nanoseconds(&clock, origin_time_local, function);
        self.emit_temporal_add_iso_date(
            date[0],
            date[1],
            date[2],
            duration[0],
            duration[1],
            duration[2],
            duration[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_difference_candidate_range(date, origin_time_local, context, function)?;
        self.emit_temporal_add_iso_date(
            date[0],
            date[1],
            date[2],
            zero_local,
            zero_local,
            zero_local,
            sign_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_difference_candidate_range(date, origin_time_local, context, function)?;
        function.instruction(&Instruction::End);
        for local in date.into_iter().rev().chain([
            origin_time_local,
            overflow_local,
            zero_local,
            sign_local,
        ]) {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
