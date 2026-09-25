//! `Temporal.Duration.prototype.round`, `total` and `Temporal.Duration.compare`
//! relative to the record `GetTemporalRelativeToOption` produced.
//!
//! A plain relative-to is an ISO date; a zoned one is an exact time in a time
//! zone with a fixed UTC offset, the only kind this backend resolves. With a
//! fixed offset, `GetISODateTimeFor` is the exact time plus the offset,
//! `GetEpochNanosecondsFor` is the wall-clock time minus it, and every
//! difference of wall-clock times equals the difference of the exact times, so
//! both kinds run through the same ISO date-time difference machinery; the
//! zoned kind adds the instant range checks the spec performs on the way.

use super::super::temporal::{TemporalRelativeTo, TemporalRelativeToKind};
use super::super::temporal_difference::{TemporalDifferenceContext, TemporalEqualEndpoints};
use super::super::temporal_options::{TemporalOverflow, TemporalTimeUnit};
use super::super::temporal_plain_date_time_methods::{
    ResolvedTemporalDateTimeDifferenceSettings, TemporalPlainDifferenceOperation,
};
use super::super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::*;

/// `nsMaxInstant / 10^9`: the exact-time range is +-10^8 days.
const EPOCH_SECONDS_LIMIT: i64 = 8_640_000_000_000;
/// `ISODateTimeWithinLimits` admits wall-clock times up to one day beyond the
/// exact-time range, exclusive at both ends.
const ISO_DATE_TIME_SECONDS_LIMIT: i64 = EPOCH_SECONDS_LIMIT + 86_400;
/// `maxTimeDuration` is `2^53 * 10^9 - 1` nanoseconds: whole seconds must stay
/// below `2^53`.
const TIME_DURATION_SECONDS_LIMIT: i64 = 1 << 53;

impl<'a> FunctionBuilder<'a> {
    /// `high:low = a * b + addend` over unsigned 64-bit operands, through
    /// 32-bit limbs.
    pub(crate) fn emit_temporal_u64_product_plus(
        &mut self,
        a_local: u32,
        b_local: u32,
        addend_local: u32,
        high_local: u32,
        low_local: u32,
        function: &mut Function,
    ) {
        let low_product = self.reserve_temp_local();
        let cross_one = self.reserve_temp_local();
        let cross_two = self.reserve_temp_local();
        let middle = self.reserve_temp_local();
        const LOW_HALF: i64 = 0xffff_ffff;
        for (destination, a_high, b_high) in [
            (low_product, false, false),
            (cross_one, false, true),
            (cross_two, true, false),
            (high_local, true, true),
        ] {
            for (operand, take_high) in [(a_local, a_high), (b_local, b_high)] {
                function.instruction(&Instruction::LocalGet(operand));
                if take_high {
                    function.instruction(&Instruction::I64Const(32));
                    function.instruction(&Instruction::I64ShrU);
                } else {
                    function.instruction(&Instruction::I64Const(LOW_HALF));
                    function.instruction(&Instruction::I64And);
                }
            }
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(low_product));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        for cross in [cross_one, cross_two] {
            function.instruction(&Instruction::LocalGet(cross));
            function.instruction(&Instruction::I64Const(LOW_HALF));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(middle));
        function.instruction(&Instruction::LocalGet(low_product));
        function.instruction(&Instruction::I64Const(LOW_HALF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(low_local));
        function.instruction(&Instruction::LocalGet(high_local));
        for carry in [cross_one, cross_two, middle] {
            function.instruction(&Instruction::LocalGet(carry));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(high_local));
        function.instruction(&Instruction::LocalGet(low_local));
        function.instruction(&Instruction::LocalGet(addend_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_local));
        function.instruction(&Instruction::LocalGet(low_local));
        function.instruction(&Instruction::LocalGet(addend_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(high_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_local));
        function.instruction(&Instruction::End);
        for local in [middle, cross_two, cross_one, low_product] {
            self.release_temp_local(local);
        }
    }

    /// `TotalTimeDuration(timeDuration, unit)` for a time duration held as a
    /// same-signed (seconds, subsecond) pair and a unit from `day` to
    /// `nanosecond`. The quotient is rounded once, from the exact 128-bit
    /// nanosecond count; the f64 bits land in `output_bits_local`.
    pub(crate) fn emit_temporal_total_time_duration(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        unit_local: u32,
        output_bits_local: u32,
        function: &mut Function,
    ) {
        let negative_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let scale_local = self.reserve_temp_local();
        let divisor_local = self.reserve_temp_local();
        let high_local = self.reserve_temp_local();
        let low_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, remainder_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::LocalSet(divisor_local));
        for unit in TemporalTimeUnit::ALL {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            function.instruction(&Instruction::LocalSet(divisor_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::LocalSet(scale_local));
        self.emit_temporal_u64_product_plus(
            magnitude_local,
            scale_local,
            remainder_local,
            high_local,
            low_local,
            function,
        );
        self.emit_temporal_exact_quotient_bits(
            high_local,
            low_local,
            TemporalExactDivisor::Local(divisor_local),
            output_bits_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
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
        for local in [
            low_local,
            high_local,
            divisor_local,
            scale_local,
            remainder_local,
            magnitude_local,
            negative_local,
        ] {
            self.release_temp_local(local);
        }
    }

    /// `GetISODateTimeFor` for a fixed offset: the wall-clock fields of the
    /// exact time `seconds + subsecond` (subsecond in `[0, 10^9)`).
    fn emit_temporal_wall_fields_from_epoch(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        offset_seconds_local: u32,
        fields: &[u32; 9],
        function: &mut Function,
    ) {
        let wall_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_civil_from_days(days_local, fields[0], fields[1], fields[2], function);
        for (local, source, divisor, modulus) in [
            (fields[3], wall_local, 3_600, None),
            (fields[4], wall_local, 60, Some(60)),
            (fields[5], wall_local, 1, Some(60)),
            (fields[6], subsecond_local, 1_000_000, None),
            (fields[7], subsecond_local, 1_000, Some(1_000)),
            (fields[8], subsecond_local, 1, Some(1_000)),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            if let Some(modulus) = modulus {
                function.instruction(&Instruction::I64Const(modulus));
                function.instruction(&Instruction::I64RemU);
            }
            function.instruction(&Instruction::LocalSet(local));
        }
        self.release_temp_local(days_local);
        self.release_temp_local(wall_local);
    }

    /// The exact time of wall-clock `fields` at a fixed offset (none for
    /// `GetUTCEpochNanoseconds`): whole seconds and a subsecond part in
    /// `[0, 10^9)`.
    fn emit_temporal_epoch_from_wall_fields(
        &mut self,
        fields: &[u32; 9],
        offset_seconds_local: Option<u32>,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_plain_date_epoch_days(
            fields[0],
            fields[1],
            fields[2],
            seconds_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        for (field, scale) in [(fields[3], 3_600), (fields[4], 60), (fields[5], 1)] {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        if let Some(offset_seconds_local) = offset_seconds_local {
            function.instruction(&Instruction::LocalGet(offset_seconds_local));
            function.instruction(&Instruction::I64Sub);
        }
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        for (field, scale) in [(fields[6], 1_000_000), (fields[7], 1_000), (fields[8], 1)] {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(subsecond_local));
    }

    /// `ISODateTimeWithinLimits`, as a RangeError.
    fn emit_temporal_require_iso_date_time_within_limits(
        &mut self,
        fields: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.emit_temporal_epoch_from_wall_fields(
            fields,
            None,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(-ISO_DATE_TIME_SECONDS_LIMIT));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(-ISO_DATE_TIME_SECONDS_LIMIT));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(ISO_DATE_TIME_SECONDS_LIMIT));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDateTime is outside the supported date range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        Ok(())
    }

    /// `IsValidEpochNanoseconds` of `seconds + subsecond` (subsecond in
    /// `[0, 10^9)`), as a RangeError.
    fn emit_temporal_require_valid_epoch(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(-EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Instant epoch nanoseconds are outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `AddZonedDateTime(relativeTo, timeZone, calendar,
    /// ToInternalDurationRecord(duration), constrain)` for a fixed offset:
    /// the date part is calendar arithmetic on the wall-clock date
    /// (`start_wall`, already filled), the time part is exact.
    fn emit_temporal_add_zoned_date_time(
        &mut self,
        relative: &TemporalRelativeTo,
        duration: &TemporalDurationFields,
        start_wall: &[u32; 9],
        target_seconds_local: u32,
        target_subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_seconds_local = self.reserve_temp_local();
        let time_subsecond_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let intermediate = self.reserve_temporal_plain_date_time_field_locals();
        let date = self.reserve_temporal_duration_date_field_locals(duration, function);
        self.emit_temporal_duration_normalize_seconds(
            duration,
            TemporalUnit::Hour,
            time_seconds_local,
            time_subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(date[0]));
        for local in &date[1..] {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `AddInstant(epochNanoseconds, duration.[[Time]])`.
        function.instruction(&Instruction::LocalGet(relative.epoch_seconds_local));
        function.instruction(&Instruction::LocalSet(target_seconds_local));
        function.instruction(&Instruction::LocalGet(relative.epoch_subsecond_local));
        function.instruction(&Instruction::LocalSet(target_subsecond_local));
        function.instruction(&Instruction::Else);
        for (source, destination) in start_wall.iter().zip(intermediate.iter()) {
            function.instruction(&Instruction::LocalGet(*source));
            function.instruction(&Instruction::LocalSet(*destination));
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        self.emit_temporal_add_iso_date(
            intermediate[0],
            intermediate[1],
            intermediate[2],
            date[0],
            date[1],
            date[2],
            date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_require_iso_date_time_within_limits(&intermediate, function)?;
        // `GetEpochNanosecondsFor(timeZone, intermediateDateTime, compatible)`:
        // for a fixed offset, `GetPossibleEpochNanoseconds` rejects an exact
        // time outside the valid range even when the time part would bring it
        // back.
        self.emit_temporal_epoch_from_wall_fields(
            &intermediate,
            Some(relative.offset_seconds_local),
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        self.emit_temporal_require_valid_epoch(
            target_seconds_local,
            target_subsecond_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_seconds_local));
        function.instruction(&Instruction::LocalGet(time_seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_seconds_local));
        function.instruction(&Instruction::LocalGet(target_subsecond_local));
        function.instruction(&Instruction::LocalGet(time_subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_subsecond_local));
        self.emit_temporal_normalize_seconds_and_subseconds(
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        self.emit_temporal_require_valid_epoch(
            target_seconds_local,
            target_subsecond_local,
            function,
        )?;
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        self.release_temporal_plain_date_time_field_locals(intermediate);
        for local in [overflow_local, time_subsecond_local, time_seconds_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The two ends of the difference a plain relative-to reduces `round` and
    /// `total` to: the relative date at midnight, and
    /// `CalendarDateAdd(date, (years, months, weeks, targetTime.[[Days]]))`
    /// combined with `targetTime = AddTime(midnight, 24-hour-day time)`.
    fn emit_temporal_duration_plain_endpoints(
        &mut self,
        relative: &TemporalRelativeTo,
        duration: &TemporalDurationFields,
        start: &[u32; 9],
        target: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let time_of_day_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let date = self.reserve_temporal_duration_date_field_locals(duration, function);
        for (index, local) in start.iter().enumerate() {
            if index < 3 {
                function.instruction(&Instruction::LocalGet(relative.date_locals[index]));
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::LocalSet(*local));
        }
        self.emit_temporal_duration_normalize_seconds(
            duration,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(time_of_day_local));
        function.instruction(&Instruction::LocalGet(time_of_day_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(time_of_day_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(time_of_day_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        for index in 0..3 {
            function.instruction(&Instruction::LocalGet(relative.date_locals[index]));
            function.instruction(&Instruction::LocalSet(target[index]));
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        self.emit_temporal_add_iso_date(
            target[0],
            target[1],
            target[2],
            date[0],
            date[1],
            date[2],
            days_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_time_from_nanoseconds(
            time_of_day_local,
            &Self::temporal_plain_date_time_time_locals(target),
            function,
        );
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [
            overflow_local,
            time_of_day_local,
            days_local,
            subsecond_local,
            seconds_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Leaves an `i32` on the stack: whether two ISO date-times are equal.
    fn emit_temporal_iso_date_time_equal_i32(
        &mut self,
        left: &[u32; 9],
        right: &[u32; 9],
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(1));
        for (left, right) in left.iter().zip(right.iter()) {
            function.instruction(&Instruction::LocalGet(*left));
            function.instruction(&Instruction::LocalGet(*right));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
        }
    }

    /// `Temporal.Duration.prototype.round` steps 27 and 28: with a zoned or a
    /// plain `relativeTo`, the rounded duration is a rounded difference, and
    /// this emits it and returns. An undefined `relativeTo` falls through to
    /// the caller's time-only rounding.
    pub(in crate::builtins) fn emit_temporal_duration_round_relative(
        &mut self,
        duration: &TemporalDurationFields,
        relative: &TemporalRelativeTo,
        settings: &ResolvedTemporalDateTimeDifferenceSettings,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start = self.reserve_temporal_plain_date_time_field_locals();
        let target = self.reserve_temporal_plain_date_time_field_locals();
        let target_seconds_local = self.reserve_temp_local();
        let target_subsecond_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_wall_fields_from_epoch(
            relative.epoch_seconds_local,
            relative.epoch_subsecond_local,
            relative.offset_seconds_local,
            &start,
            function,
        );
        self.emit_temporal_add_zoned_date_time(
            relative,
            duration,
            &start,
            target_seconds_local,
            target_subsecond_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(settings.largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `DifferenceInstant`, then `TemporalDurationFromInternal` with the
        // time largest unit.
        let balanced = self.reserve_temporal_duration_field_locals();
        for (target_local, origin_local) in [
            (target_seconds_local, relative.epoch_seconds_local),
            (target_subsecond_local, relative.epoch_subsecond_local),
        ] {
            function.instruction(&Instruction::LocalGet(target_local));
            function.instruction(&Instruction::LocalGet(origin_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(target_local));
        }
        self.emit_temporal_duration_renormalize(
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        self.emit_temporal_round_difference_time(
            target_seconds_local,
            target_subsecond_local,
            settings.smallest_unit_local,
            settings.increment_local,
            settings.mode_local,
            function,
        );
        self.emit_temporal_duration_balance(
            target_seconds_local,
            target_subsecond_local,
            settings.largest_unit_local,
            &balanced,
            function,
        )?;
        self.emit_create_temporal_duration(&balanced, function)?;
        self.release_temporal_duration_field_locals(balanced);
        function.instruction(&Instruction::Else);
        self.emit_temporal_wall_fields_from_epoch(
            target_seconds_local,
            target_subsecond_local,
            relative.offset_seconds_local,
            &target,
            function,
        );
        self.emit_temporal_difference_date_time(
            &start,
            &target,
            settings,
            TemporalPlainDifferenceOperation::Until,
            TemporalDifferenceContext::Zoned {
                offset_seconds_local: relative.offset_seconds_local,
            },
            TemporalEqualEndpoints::Round,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_plain_endpoints(relative, duration, &start, &target, function)?;
        // `DifferencePlainDateTimeWithRounding` step 2 runs after the
        // equal-endpoints shortcut of step 1, which the difference itself
        // takes.
        self.emit_temporal_iso_date_time_equal_i32(&start, &target, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_require_iso_date_time_within_limits(&start, function)?;
        self.emit_temporal_require_iso_date_time_within_limits(&target, function)?;
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_date_time(
            &start,
            &target,
            settings,
            TemporalPlainDifferenceOperation::Until,
            TemporalDifferenceContext::Plain,
            TemporalEqualEndpoints::ReturnZero,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(target_subsecond_local);
        self.release_temp_local(target_seconds_local);
        self.release_temporal_plain_date_time_field_locals(target);
        self.release_temporal_plain_date_time_field_locals(start);
        Ok(())
    }

    /// `Temporal.Duration.prototype.total` steps 11 and 12: with a zoned or a
    /// plain `relativeTo`, the total of the difference, returned as a Number.
    /// An undefined `relativeTo` falls through to the caller.
    pub(in crate::builtins) fn emit_temporal_duration_total_relative(
        &mut self,
        duration: &TemporalDurationFields,
        relative: &TemporalRelativeTo,
        unit_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start = self.reserve_temporal_plain_date_time_field_locals();
        let target = self.reserve_temporal_plain_date_time_field_locals();
        let target_seconds_local = self.reserve_temp_local();
        let target_subsecond_local = self.reserve_temp_local();
        let total_bits_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_wall_fields_from_epoch(
            relative.epoch_seconds_local,
            relative.epoch_subsecond_local,
            relative.offset_seconds_local,
            &start,
            function,
        );
        self.emit_temporal_add_zoned_date_time(
            relative,
            duration,
            &start,
            target_seconds_local,
            target_subsecond_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `DifferenceZonedDateTimeWithTotal` step 1: a time unit totals the
        // exact difference.
        for (target_local, origin_local) in [
            (target_seconds_local, relative.epoch_seconds_local),
            (target_subsecond_local, relative.epoch_subsecond_local),
        ] {
            function.instruction(&Instruction::LocalGet(target_local));
            function.instruction(&Instruction::LocalGet(origin_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(target_local));
        }
        self.emit_temporal_duration_renormalize(
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        self.emit_temporal_total_time_duration(
            target_seconds_local,
            target_subsecond_local,
            unit_local,
            total_bits_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_wall_fields_from_epoch(
            target_seconds_local,
            target_subsecond_local,
            relative.offset_seconds_local,
            &target,
            function,
        );
        self.emit_temporal_difference_total(
            &start,
            &target,
            unit_local,
            TemporalDifferenceContext::Zoned {
                offset_seconds_local: relative.offset_seconds_local,
            },
            total_bits_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_number_result(total_bits_local, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_plain_endpoints(relative, duration, &start, &target, function)?;
        // `DifferencePlainDateTimeWithTotal` steps 1 and 2.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(total_bits_local));
        self.emit_temporal_iso_date_time_equal_i32(&start, &target, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_require_iso_date_time_within_limits(&start, function)?;
        self.emit_temporal_require_iso_date_time_within_limits(&target, function)?;
        self.emit_temporal_difference_total(
            &start,
            &target,
            unit_local,
            TemporalDifferenceContext::Plain,
            total_bits_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_number_result(total_bits_local, function);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(total_bits_local);
        self.release_temp_local(target_subsecond_local);
        self.release_temp_local(target_seconds_local);
        self.release_temporal_plain_date_time_field_locals(target);
        self.release_temporal_plain_date_time_field_locals(start);
        Ok(())
    }

    /// `Temporal.Duration.compare` from step 6 on, after the field-for-field
    /// equality shortcut: the durations' exact time spans relative to
    /// `relativeTo`, compared. Sets the Number result.
    pub(in crate::builtins) fn emit_temporal_duration_compare_relative(
        &mut self,
        one: &TemporalDurationFields,
        two: &TemporalDurationFields,
        relative: &TemporalRelativeTo,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let largest_one_local = self.reserve_temp_local();
        let largest_two_local = self.reserve_temp_local();
        let one_seconds_local = self.reserve_temp_local();
        let one_subsecond_local = self.reserve_temp_local();
        let two_seconds_local = self.reserve_temp_local();
        let two_subsecond_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let start = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_duration_default_largest_unit(one, largest_one_local, function);
        self.emit_temporal_duration_default_largest_unit(two, largest_two_local, function);

        // Steps 12-13: a zoned relativeTo and a date unit on either side
        // compares the two `AddZonedDateTime` results.
        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_one_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::LocalGet(largest_two_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_wall_fields_from_epoch(
            relative.epoch_seconds_local,
            relative.epoch_subsecond_local,
            relative.offset_seconds_local,
            &start,
            function,
        );
        for (duration, seconds_local, subsecond_local) in [
            (one, one_seconds_local, one_subsecond_local),
            (two, two_seconds_local, two_subsecond_local),
        ] {
            self.emit_temporal_add_zoned_date_time(
                relative,
                duration,
                &start,
                seconds_local,
                subsecond_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        // Steps 14-16: with a calendar unit on either side, both date parts
        // become days relative to the plain relativeTo; otherwise days are
        // 24 hours.
        function.instruction(&Instruction::LocalGet(largest_one_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(largest_two_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration operation requires relativeTo for calendar units",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        for (duration, seconds_local, subsecond_local) in [
            (one, one_seconds_local, one_subsecond_local),
            (two, two_seconds_local, two_subsecond_local),
        ] {
            self.emit_temporal_duration_relative_time_span(
                duration,
                Some(relative),
                seconds_local,
                subsecond_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        for (duration, seconds_local, subsecond_local) in [
            (one, one_seconds_local, one_subsecond_local),
            (two, two_seconds_local, two_subsecond_local),
        ] {
            self.emit_temporal_duration_relative_time_span(
                duration,
                None,
                seconds_local,
                subsecond_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Step 17: `CompareTimeDuration`. Both pairs share their sign between
        // the seconds and the subsecond part, so the comparison is
        // lexicographic.
        function.instruction(&Instruction::LocalGet(one_seconds_local));
        function.instruction(&Instruction::LocalGet(two_seconds_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(one_seconds_local));
        function.instruction(&Instruction::LocalGet(two_seconds_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(one_seconds_local));
        function.instruction(&Instruction::LocalGet(two_seconds_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(one_subsecond_local));
        function.instruction(&Instruction::LocalGet(two_subsecond_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(one_subsecond_local));
        function.instruction(&Instruction::LocalGet(two_subsecond_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_temporal_duration_number_result(result_local, function);

        self.release_temporal_plain_date_time_field_locals(start);
        for local in [
            result_local,
            two_subsecond_local,
            two_seconds_local,
            one_subsecond_local,
            one_seconds_local,
            largest_two_local,
            largest_one_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Add24HourDaysToTimeDuration(duration.[[Time]], days)` where `days` is
    /// `DateDurationDays(duration.[[Date]], plainRelativeTo)` when a plain
    /// relative-to is given and the duration's own days otherwise. The pair
    /// comes back sign-consistent; exceeding `maxTimeDuration` is a
    /// RangeError.
    fn emit_temporal_duration_relative_time_span(
        &mut self,
        duration: &TemporalDurationFields,
        relative: Option<&TemporalRelativeTo>,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(relative) = relative else {
            self.emit_temporal_duration_normalize_seconds(
                duration,
                TemporalUnit::Day,
                seconds_local,
                subsecond_local,
                function,
            );
            return Ok(());
        };
        let days_local = self.reserve_temp_local();
        let epoch_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let later = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let date = self.reserve_temporal_duration_date_field_locals(duration, function);
        self.emit_temporal_duration_normalize_seconds(
            duration,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        // `DateDurationDays`: years, months and weeks become the days between
        // the relative date and that date moved by them.
        function.instruction(&Instruction::LocalGet(date[3]));
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(date[0]));
        function.instruction(&Instruction::LocalGet(date[1]));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(date[2]));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (source, destination) in relative.date_locals.into_iter().zip(later) {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(date[3]));
        self.emit_temporal_add_iso_date(
            later[0],
            later[1],
            later[2],
            date[0],
            date[1],
            date[2],
            date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(later[0], later[1], later[2], epoch_local, function);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(days_local));
        self.emit_temporal_plain_date_epoch_days(
            relative.date_locals[0],
            relative.date_locals[1],
            relative.date_locals[2],
            epoch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(TIME_DURATION_SECONDS_LIMIT));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(-TIME_DURATION_SECONDS_LIMIT));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.Duration: fields must not exceed the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in later.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [overflow_local, epoch_local, days_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Set the completion to the Number whose bits are in `bits_local`.
    pub(in crate::builtins) fn emit_temporal_duration_number_result(
        &mut self,
        bits_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(bits_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }
}
