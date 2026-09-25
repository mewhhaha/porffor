//! `Temporal.ZonedDateTime.prototype` arithmetic and calendar methods:
//! `add`, `subtract`, `until`, `since`, `withCalendar`.
//!
//! Split out of `temporal.rs` (record, constructor, accessors, `equals`,
//! `toInstant`, `withTimeZone`, `toPlainDateTime`) the same way
//! `temporal_plain_date_time_methods.rs` is split out of
//! `temporal_plain_date_time.rs`. Both halves are `impl FunctionBuilder`
//! blocks; the boundary is pinned by `scripts/check-module-boundaries.sh`.
//!
//! The arithmetic itself — `AddZonedDateTime`, `DifferenceZonedDateTime` and
//! its rounding — lives in `temporal_zoned_difference.rs`, on exact times and
//! through the time-zone kernel.

use super::super::*;
use super::temporal::TemporalEpochNanosecondsRecord;
use super::temporal_options::{TemporalOverflow, TemporalUnit};
use super::temporal_plain_date_time_methods::TemporalDateTimeDifferenceSettingsPlan;
use super::temporal_zoned_difference::TemporalInternalDuration;

/// Which of the two arithmetic members is being emitted.
///
/// This was a `bool` named `subtract`. The two call sites are adjacent arms of
/// one `match` in `builtins/standard.rs`, so a transposed `bool` compiles,
/// formats, passes every type check, and makes `zdt.add(d)` subtract. The
/// closed set gets a closed type.
enum ZonedDateTimeArithmetic {
    Add,
    Subtract,
}

/// Which of the two difference members is being emitted. Same reasoning as
/// [`ZonedDateTimeArithmetic`]: `until` and `since` differ only in operand
/// order, so a transposed `bool` produces a correctly-shaped `Temporal.Duration`
/// with the wrong sign.
enum ZonedDateTimeDifference {
    Until,
    Since,
}

impl<'a> FunctionBuilder<'a> {
    /// Temporal proposal 6.3.x `Temporal.ZonedDateTime.prototype.withCalendar`.
    ///
    /// Structurally the twin of
    /// [`Self::emit_temporal_zoned_date_time_with_time_zone`]: read the
    /// receiver's record, resolve the one argument, and allocate a fresh
    /// ZonedDateTime with that one slot replaced. The epoch nanoseconds are
    /// unchanged, which is the whole content of the operation — a calendar
    /// re-labels an instant, it does not move it.
    ///
    /// This is on the critical path for the gate, not a bonus: `since` and
    /// `until`'s `era-boundary-*.js` files call `one.withCalendar("iso8601")`
    /// to build the ISO oracle they compare the `weeks`/`days` answers against
    /// (`since/era-boundary-gregory.js:65`), so those 14 cases need five
    /// callables, not four.
    pub(crate) fn emit_temporal_zoned_date_time_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, calendar_payload_local, calendar_tag_local, function);
        // `ToTemporalCalendarIdentifier` treats `undefined` as `iso8601`, which
        // is right for a property bag with no `calendar` key and wrong here:
        // `zdt.withCalendar()` must throw rather than silently switch the
        // receiver to ISO. `emit_temporal_plain_date_time_with_calendar` guards
        // the same way, for the same reason.
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_to_temporal_calendar_identifier(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.ZonedDateTime calendar must be a string",
            function,
        )?;
        // The identifier helper leaves a canonical calendar string in
        // `calendar_payload_local` and a `String` tag in `calendar_tag_local`
        // on every path it returns from, so the record's calendar tag is
        // written from the local rather than re-asserted as a constant.
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_add_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_add_or_subtract(ZonedDateTimeArithmetic::Add, function)
    }

    pub(super) fn emit_temporal_zoned_date_time_subtract_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_add_or_subtract(
            ZonedDateTimeArithmetic::Subtract,
            function,
        )
    }

    pub(super) fn emit_temporal_zoned_date_time_until_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_until_or_since(ZonedDateTimeDifference::Until, function)
    }

    pub(super) fn emit_temporal_zoned_date_time_since_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_until_or_since(ZonedDateTimeDifference::Since, function)
    }

    /// Temporal proposal 6.3.x `add` and `subtract`:
    /// `AddDurationToZonedDateTime`. The duration and the `overflow` option are
    /// read first; `AddZonedDateTime` then moves the wall-clock date by the date
    /// part and the exact time by the time part.
    fn emit_temporal_zoned_date_time_add_or_subtract(
        &mut self,
        arithmetic: ZonedDateTimeArithmetic,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let duration_payload_local = self.reserve_temp_local();
        let duration_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        // Brand check first, and it is the observable one: a non-ZonedDateTime
        // receiver must throw before either argument is coerced.
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, duration_payload_local, duration_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_duration(
            duration_payload_local,
            duration_tag_local,
            &duration_locals,
            function,
        )?;
        match arithmetic {
            ZonedDateTimeArithmetic::Add => {}
            ZonedDateTimeArithmetic::Subtract => {
                self.emit_temporal_duration_negate_fields(&duration_locals, function);
            }
        }
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options_payload_local,
            options_tag_local,
            overflow_local,
            "Temporal.ZonedDateTime options must be an object or undefined",
            "Invalid Temporal.ZonedDateTime overflow option",
            function,
        )?;

        // `ToInternalDurationRecord`.
        let date = self.reserve_temporal_duration_date_field_locals(&duration_locals, function);
        let internal = TemporalInternalDuration {
            date,
            time_seconds: self.reserve_temp_local(),
            time_subsecond: self.reserve_temp_local(),
        };
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Hour,
            internal.time_seconds,
            internal.time_subsecond,
            function,
        );
        let start = self.reserve_temporal_exact_time();
        let target = self.reserve_temporal_exact_time();
        self.emit_temporal_epoch_value_seconds(
            epoch_payload_local,
            epoch_tag_local,
            start.seconds,
            start.subsecond,
            function,
        );
        self.emit_temporal_add_zoned_date_time(
            time_zone_payload_local,
            start,
            &internal,
            overflow_local,
            target,
            function,
        )?;
        self.emit_temporal_epoch_nanoseconds_bigint(
            target.seconds,
            target.subsecond,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temporal_exact_time(target);
        self.release_temporal_exact_time(start);
        self.release_temporal_internal_duration(internal);
        self.release_temporal_duration_field_locals(duration_locals);
        for local in [
            prototype_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            duration_tag_local,
            duration_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 6.3.x `until` and `since`, both through
    /// `DifferenceTemporalZonedDateTime`.
    ///
    /// Calendar equality precedes the single settings read, which uses the
    /// ZonedDateTime hour fallback. Time-unit differences round the exact
    /// difference; date-unit differences require equal zones and run
    /// `DifferenceZonedDateTimeWithRounding` on the two exact times.
    fn emit_temporal_zoned_date_time_until_or_since(
        &mut self,
        difference: ZonedDateTimeDifference,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let other_time_zone_payload_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let zones_equal_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        // `ToTemporalZonedDateTime(other)`. Same idiom
        // `emit_temporal_zoned_date_time_equals` uses, and for the same reason:
        // `from` is the only entry point that accepts every legal spelling of
        // `other` (an instance, a property bag, an ISO string).
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimeFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.from`",
                )
            })?;
        self.emit_direct_js_call(
            &from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_payload_local,
            other_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                other_time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                other_calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(other_record_local, offset, local, function);
        }
        // `CalendarEquals` — RangeError, and before any option is read.
        self.emit_temporal_require_same_calendar(
            calendar_payload_local,
            other_calendar_payload_local,
            TemporalDifferenceGuard::ZonedDateTimeSameCalendar,
            function,
        )?;
        let plan = match difference {
            ZonedDateTimeDifference::Until => TemporalDateTimeDifferenceSettingsPlan::ZonedUntil,
            ZonedDateTimeDifference::Since => TemporalDateTimeDifferenceSettingsPlan::ZonedSince,
        };
        let settings = self.emit_temporal_date_time_difference_settings(
            options_payload_local,
            options_tag_local,
            plan,
            function,
        )?;
        let origin = self.reserve_temporal_exact_time();
        let destination = self.reserve_temporal_exact_time();
        self.emit_temporal_epoch_nanoseconds_pair(
            record_local,
            TemporalEpochNanosecondsRecord::ZonedDateTime,
            origin.seconds,
            origin.subsecond,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_pair(
            other_record_local,
            TemporalEpochNanosecondsRecord::ZonedDateTime,
            destination.seconds,
            destination.subsecond,
            function,
        );

        function.instruction(&Instruction::LocalGet(settings.largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        {
            // A time unit: `DifferenceInstant` of the truncated pairs.
            let seconds_local = self.reserve_temp_local();
            let subsecond_local = self.reserve_temp_local();
            let duration_fields = self.reserve_temporal_duration_field_locals();
            for (receiver, other, output) in [
                (origin.seconds, destination.seconds, seconds_local),
                (origin.subsecond, destination.subsecond, subsecond_local),
            ] {
                function.instruction(&Instruction::LocalGet(other));
                function.instruction(&Instruction::LocalGet(receiver));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(output));
            }
            self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
            self.emit_temporal_round_difference_time(
                seconds_local,
                subsecond_local,
                settings.smallest_unit_local,
                settings.increment_local,
                settings.mode_local,
                function,
            );
            // `since` negates the result of the receiver-to-other difference.
            match difference {
                ZonedDateTimeDifference::Until => {}
                ZonedDateTimeDifference::Since => {
                    for local in [seconds_local, subsecond_local] {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalGet(local));
                        function.instruction(&Instruction::I64Sub);
                        function.instruction(&Instruction::LocalSet(local));
                    }
                }
            }
            self.emit_temporal_duration_balance(
                seconds_local,
                subsecond_local,
                settings.largest_unit_local,
                &duration_fields,
                function,
            )?;
            self.emit_create_temporal_duration(&duration_fields, function)?;
            self.emit_return_current_completion(function);
            self.release_temporal_duration_field_locals(duration_fields);
            self.release_temp_local(subsecond_local);
            self.release_temp_local(seconds_local);
        }
        function.instruction(&Instruction::End);

        // A date unit: the zones must agree (`TimeZoneEquals`, primary
        // identifiers for named zones).
        self.emit_temporal_time_zone_equals(
            time_zone_payload_local,
            other_time_zone_payload_local,
            zones_equal_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(zones_equal_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            TemporalDifferenceGuard::ZonedDateTimeSameTimeZone.message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        for pair in [origin, destination] {
            self.emit_temporal_normalize_seconds_and_subseconds(
                pair.seconds,
                pair.subsecond,
                function,
            );
        }

        let internal = self.reserve_temporal_internal_duration();
        let start_wall = self.reserve_temporal_plain_date_time_field_locals();
        // Step 10: equal instants are the zero duration, before any rounding.
        function.instruction(&Instruction::LocalGet(origin.seconds));
        function.instruction(&Instruction::LocalGet(destination.seconds));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(origin.subsecond));
        function.instruction(&Instruction::LocalGet(destination.subsecond));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        for local in internal
            .date
            .iter()
            .copied()
            .chain([internal.time_seconds, internal.time_subsecond])
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_iso_date_time_for(
            time_zone_payload_local,
            origin.seconds,
            origin.subsecond,
            &start_wall,
            function,
        )?;
        self.emit_temporal_difference_zoned_date_time(
            time_zone_payload_local,
            origin,
            destination,
            &start_wall,
            settings.largest_unit_local,
            &internal,
            function,
        )?;
        // `DifferenceZonedDateTimeWithRounding` step 3: no rounding at
        // nanosecond/1.
        function.instruction(&Instruction::LocalGet(settings.smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(settings.increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_round_relative_duration_zoned(
            time_zone_payload_local,
            &internal,
            origin,
            destination,
            &start_wall,
            settings.largest_unit_local,
            settings.increment_local,
            settings.smallest_unit_local,
            settings.mode_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        match difference {
            ZonedDateTimeDifference::Until => {}
            ZonedDateTimeDifference::Since => {
                for local in internal
                    .date
                    .iter()
                    .copied()
                    .chain([internal.time_seconds, internal.time_subsecond])
                {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalGet(local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(local));
                }
            }
        }
        self.emit_temporal_duration_from_internal_hours(&internal, function)?;

        self.release_temporal_plain_date_time_field_locals(start_wall);
        self.release_temporal_internal_duration(internal);
        self.release_temporal_exact_time(destination);
        self.release_temporal_exact_time(origin);
        for local in [
            settings.mode_local,
            settings.increment_local,
            settings.smallest_unit_local,
            settings.largest_unit_local,
        ] {
            self.release_temp_local(local);
        }
        for local in [
            zones_equal_local,
            other_calendar_payload_local,
            other_time_zone_payload_local,
            other_record_local,
            other_tag_local,
            other_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            time_zone_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
