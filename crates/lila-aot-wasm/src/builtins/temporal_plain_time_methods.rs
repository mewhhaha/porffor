//! `Temporal.PlainTime` statics and prototype methods.
//!
//! Split from `temporal_plain_time.rs` (record, constructor, accessors) so the
//! two halves stay readable; both are `impl FunctionBuilder` blocks.
//!
//! Everything arithmetic here funnels through the nanosecond-of-day scalar:
//! `add`/`subtract` add a signed offset and wrap, `until`/`since` subtract two
//! scalars and hand the difference to the Duration balancer, and `round` and
//! `toString` share one rounding step. Reusing the `Temporal.Duration` option
//! plumbing and its rounding-mode decision table is deliberate — the two types
//! have to agree on what `halfEven` means.

use super::super::*;
use super::temporal::TemporalTimeCalendarUse;
use super::temporal_options::{
    TemporalConversionOverflowOptions, TemporalOverflow, TemporalRoundingMode, TemporalTimeUnit,
    TemporalUnit, TemporalUnitOptionProperty, TemporalUnitSlot,
};
use super::temporal_plain_date_time_methods::{
    TemporalPlainArithmeticOperation, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_time::{
    NANOSECONDS_PER_TEMPORAL_DAY, TEMPORAL_PLAIN_TIME_ALPHABETICAL_FIELDS,
};
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

/// The two formatting entry points have different observable option reads.
pub(super) enum TemporalPlainTimeStringMode {
    ToString,
    ToJson,
}

/// `ToSecondsStringPrecisionRecord` precision codes. Non-negative values are a
/// literal digit count.
pub(crate) const TEMPORAL_PRECISION_AUTO: i64 = -1;
pub(crate) const TEMPORAL_PRECISION_MINUTE: i64 = -2;

impl<'a> FunctionBuilder<'a> {
    /// `GetTemporalOverflowOption`. Unlike the unit and rounding-mode options,
    /// the two accepted spellings are matched case-sensitively, because the
    /// proposal compares them with `SameValue`.
    fn emit_temporal_plain_time_overflow_option(
        &mut self,
        options: &ValueLocals,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options,
            overflow,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_OVERFLOW_OPTION,
            function,
        )
    }

    /// `ToTemporalTimeRecord`. Reads the six properties in alphabetical order —
    /// the reads are observable — leaving each value in `fields` and its
    /// presence flag in `present_fields`. Absent fields keep the caller's
    /// initial coordinates, so the same emitter serves both the complete form
    /// (`from`, defaults zero) and the partial form (`with`, defaults the
    /// receiver).
    fn emit_temporal_plain_time_read_fields(
        &mut self,
        argument: &ValueLocals,
        fields: &[I64Local; 6],
        present_fields: &[I64Local; 6],
        any_present: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let present = schema.reserve_i64_local(function);
        let parsed = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        any_present.store(function);
        for (property, index) in TEMPORAL_PLAIN_TIME_ALPHABETICAL_FIELDS {
            self.emit_temporal_property_bag_integer(
                argument,
                property,
                present,
                parsed,
                0,
                RuntimeErrorMessage::TEMPORAL_PLAINTIME_FIELD_MUST_BE_FINITE,
                function,
            )?;
            present.load(function);
            present_fields[index].store(function);
            present.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            parsed.load(function);
            fields[index].store(function);
            function.instruction(&Instruction::I64Const(1));
            any_present.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        any_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_REQUIRES_AT_LEAST_ONE_TIME_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(parsed, function);
        schema.release_i64_local(present, function);
        Ok(())
    }

    /// `ToTemporalTime`. Accepts a branded `Temporal.PlainTime` (cloned), any
    /// other object (read as a property bag), or an ISO time string.
    ///
    /// `compare`, `equals`, `until` and `since` omit overflow options because
    /// they pass no options through to the conversion.
    pub(super) fn emit_to_temporal_time(
        &mut self,
        argument: &ValueLocals,
        overflow_options: TemporalConversionOverflowOptions<'_>,
        fields: &[I64Local; 6],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let overflow = schema.reserve_i64_local(function);
        let handled = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let nanoseconds = schema.reserve_i64_local(function);
        let present_fields = self.reserve_temporal_plain_time_field_locals(function);
        function.instruction(&Instruction::I64Const(0));
        handled.store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        for field in fields {
            function.instruction(&Instruction::I64Const(0));
            field.store(function);
        }
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let time = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainTimeObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_time_load_record(&time, fields, function);
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_plain_time_overflow_option(options, overflow, function)?;
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        time.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Branded date-time inputs read internal slots without observable Gets.
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let date_time = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainDateTimeObject>(schema, function),
            function,
        );
        let record = schema.struct_type::<TemporalPlainDateTimeObject>();
        let component = schema.reserve_i32_local(function);
        for (destination, field) in fields.iter().zip([
            TemporalPlainDateTimeObjectSchema::HOUR,
            TemporalPlainDateTimeObjectSchema::MINUTE,
            TemporalPlainDateTimeObjectSchema::SECOND,
            TemporalPlainDateTimeObjectSchema::MILLISECOND,
            TemporalPlainDateTimeObjectSchema::MICROSECOND,
            TemporalPlainDateTimeObjectSchema::NANOSECOND,
        ]) {
            record
                .field(field)
                .read(&date_time, schema, function)
                .store(component, function);
            component.load(function);
            function.instruction(&Instruction::I64ExtendI32S);
            destination.store(function);
        }
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_plain_time_overflow_option(options, overflow, function)?;
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        schema.release_i32_local(component, function);
        date_time.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let zoned = self.emit_temporal_branded_zoned_record_from_value(argument, function)?;
        let epoch = self.emit_temporal_normalized_instant_from_zoned_record(&zoned, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&zoned, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &epoch, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        for (source, destination) in iso.fields()[3..].iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        iso.release(self, function);
        snapshot.release(self, function);
        zone.release(self, function);
        epoch.release(self, function);
        zoned.release(function);
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_plain_time_overflow_option(options, overflow, function)?;
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_time_read_fields(
            argument,
            fields,
            &present_fields,
            any_present,
            function,
        )?;
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_plain_time_overflow_option(options, overflow, function)?;
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        self.emit_temporal_regulate_time(fields, overflow, function)?;
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_PLAINTIME,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_plain_time_string(
            &string,
            fields[0],
            fields[1],
            fields[2],
            nanoseconds,
            TemporalTimeCalendarUse::Ignore,
            function,
        )?;
        string.clear(function);
        // The parser returns the whole fractional nanosecond count.
        for (index, divisor) in [(5_usize, 1_000_i64), (4, 1_000), (3, 1_000)] {
            nanoseconds.load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemS);
            fields[index].store(function);
            nanoseconds.load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivS);
            nanoseconds.store(function);
        }
        // Overflow is observed only after successful parsing.
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_plain_time_overflow_option(options, overflow, function)?;
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temporal_plain_time_field_locals(present_fields, function);
        for local in [nanoseconds, any_present, handled, overflow] {
            schema.release_i64_local(local, function);
        }
        Ok(())
    }

    /// Temporal proposal 4.2.2 `Temporal.PlainTime.from`.
    pub(crate) fn emit_temporal_plain_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_time(
            &argument,
            TemporalConversionOverflowOptions::Read(&options),
            &fields,
            function,
        )?;
        self.emit_alloc_temporal_plain_time(&fields, TemporalPrototypeSource::Intrinsic, function)?;
        self.release_temporal_plain_time_field_locals(fields, function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Leaves an `i64` in `comparison_local`: -1, 0 or 1 for the two times.
    fn emit_temporal_plain_time_compare_fields(
        &mut self,
        left: &[I64Local; 6],
        right: &[I64Local; 6],
        comparison: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let left_total = schema.reserve_i64_local(function);
        let right_total = schema.reserve_i64_local(function);
        self.emit_temporal_plain_time_total_nanoseconds(left, left_total, function);
        self.emit_temporal_plain_time_total_nanoseconds(right, right_total, function);
        function.instruction(&Instruction::I64Const(0));
        comparison.store(function);
        left_total.load(function);
        right_total.load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        comparison.store(function);
        function.instruction(&Instruction::Else);
        left_total.load(function);
        right_total.load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        comparison.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(right_total, function);
        schema.release_i64_local(left_total, function);
    }

    /// Temporal proposal 4.2.3 `Temporal.PlainTime.compare`.
    pub(crate) fn emit_temporal_plain_time_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let comparison = schema.reserve_i64_local(function);
        let left = self.reserve_temporal_plain_time_field_locals(function);
        let right = self.reserve_temporal_plain_time_field_locals(function);
        for (index, fields) in [(0_usize, &left), (1, &right)] {
            self.emit_builtin_arg_to_value(index, &argument, function);
            self.emit_to_temporal_time(
                &argument,
                TemporalConversionOverflowOptions::Omit,
                fields,
                function,
            )?;
        }
        self.emit_temporal_plain_time_compare_fields(&left, &right, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        comparison.store(function);
        let result = schema.reserve_value_local(function);
        result.set_number(comparison, function);
        self.completion().set_normal(&result, function);
        result.clear(function);
        self.release_temporal_plain_time_field_locals(right, function);
        self.release_temporal_plain_time_field_locals(left, function);
        schema.release_i64_local(comparison, function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 4.3.x `Temporal.PlainTime.prototype.equals`.
    pub(crate) fn emit_temporal_plain_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let comparison = schema.reserve_i64_local(function);
        let equal = schema.reserve_i32_local(function);
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        let other = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_time_fields_from_receiver(&fields, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_temporal_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other,
            function,
        )?;
        self.emit_temporal_plain_time_compare_fields(&fields, &other, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::I64Eqz);
        equal.store(function);
        let result = schema.reserve_value_local(function);
        result.set_boolean(equal, function);
        self.completion().set_normal(&result, function);
        result.clear(function);
        self.release_temporal_plain_time_field_locals(other, function);
        self.release_temporal_plain_time_field_locals(fields, function);
        schema.release_i32_local(equal, function);
        schema.release_i64_local(comparison, function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 4.3.x `with`. A partial time record replaces only the
    /// fields it names; the rest come from the receiver.
    pub(crate) fn emit_temporal_plain_time_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let probe = schema.reserve_value_local(function);
        let overflow = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        let present = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_time_fields_from_receiver(&fields, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_branded_partial_object(
            &argument,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_OBJECT,
            function,
        )?;
        // These Gets precede all time-field Gets, and undefined is accepted.
        for property in ["calendar", "timeZone"] {
            self.emit_temporal_duration_option_get(&argument, property, &probe, function)?;
            probe.tag().load(function);
            function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::TypeError,
                RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_WITH_DOES_NOT_ACCEPT_CALENDAR_OR_TIMEZONE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_plain_time_read_fields(
            &argument,
            &fields,
            &present,
            any_present,
            function,
        )?;
        self.emit_temporal_plain_time_overflow_option(&options, overflow, function)?;
        self.emit_temporal_regulate_time(&fields, overflow, function)?;
        self.emit_alloc_temporal_plain_time(&fields, TemporalPrototypeSource::Intrinsic, function)?;
        self.release_temporal_plain_time_field_locals(present, function);
        self.release_temporal_plain_time_field_locals(fields, function);
        schema.release_i64_local(any_present, function);
        schema.release_i64_local(overflow, function);
        probe.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 4.3.x `add` and `subtract`. The duration's calendar
    /// fields are ignored outright — a `PlainTime` has no date for a year or a
    /// month to land on — and the result wraps around midnight.
    pub(super) fn emit_temporal_plain_time_add_or_subtract(
        &mut self,
        operation: TemporalPlainArithmeticOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let total_local = self.runtime_schema().reserve_i64_local(function);
        let field_locals = self.reserve_temporal_plain_time_field_locals(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_temporal_duration(&argument, &duration_locals, function)?;
        match operation {
            TemporalPlainArithmeticOperation::Add => {}
            TemporalPlainArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&duration_locals, function);
            }
        }
        // Hours and below only: `AddDurationToTime` never consults the date
        // fields.
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        // A duration may hold billions of seconds, so the day-modulo has to
        // happen before the conversion to nanoseconds or the multiply would
        // overflow the `i64`.
        seconds_local.load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        seconds_local.store(function);
        seconds_local.load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        subsecond_local.load(function);
        function.instruction(&Instruction::I64Add);
        subsecond_local.store(function);
        self.emit_temporal_plain_time_total_nanoseconds(&field_locals, total_local, function);
        total_local.load(function);
        subsecond_local.load(function);
        function.instruction(&Instruction::I64Add);
        total_local.store(function);
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &field_locals, function);
        self.emit_alloc_temporal_plain_time(
            &field_locals,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;

        self.release_temporal_duration_field_locals(duration_locals, function);
        self.release_temporal_plain_time_field_locals(field_locals, function);
        for local in [total_local, subsecond_local, seconds_local] {
            self.runtime_schema().release_i64_local(local, function);
        }
        argument.clear(function);
        Ok(())
    }

    /// `ValidateTemporalRoundingIncrement` with `inclusive` false: the
    /// increment must divide the unit's maximum and stay strictly below it.
    ///
    /// `unit_local` must name one of the six wall-clock units, and every caller
    /// peels the calendar and day cases off first. The seed is 0 — which is no
    /// unit's maximum — rather than hour's 24, so a unit that escapes that
    /// peeling throws here instead of being silently validated against hour's
    /// bound: `increment >= 0` always holds, so the check below rejects.
    pub(crate) fn emit_temporal_plain_time_validate_increment(
        &mut self,
        unit_local: I64Local,
        increment_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_local = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        maximum_local.store(function);
        for unit in TemporalTimeUnit::ALL {
            unit_local.load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(unit.maximum_rounding_increment()));
            maximum_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        increment_local.load(function);
        maximum_local.load(function);
        function.instruction(&Instruction::I64GeS);
        maximum_local.load(function);
        increment_local.load(function);
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_ROUNDING_INCREMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(maximum_local, function);
        Ok(())
    }

    /// `RoundTime` rounds only the selected field and smaller fields. Keeping
    /// higher fields out of the quotient matters for half-even ties when a
    /// valid increment divides its parent unit an odd number of times.
    pub(crate) fn emit_temporal_round_time_nanoseconds(
        &mut self,
        nanoseconds_local: I64Local,
        unit_local: I64Local,
        quantum_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) {
        let parent_length_local = self.runtime_schema().reserve_i64_local(function);
        let prefix_local = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        parent_length_local.store(function);
        for unit in TemporalTimeUnit::ALL {
            unit_local.load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(
                unit.nanoseconds() * unit.maximum_rounding_increment(),
            ));
            parent_length_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        nanoseconds_local.load(function);
        nanoseconds_local.load(function);
        parent_length_local.load(function);
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Sub);
        prefix_local.store(function);
        nanoseconds_local.load(function);
        prefix_local.load(function);
        function.instruction(&Instruction::I64Sub);
        nanoseconds_local.store(function);
        self.emit_temporal_plain_time_round_nanoseconds(
            nanoseconds_local,
            quantum_local,
            mode_local,
            function,
        );
        nanoseconds_local.load(function);
        prefix_local.load(function);
        function.instruction(&Instruction::I64Add);
        nanoseconds_local.store(function);
        self.runtime_schema()
            .release_i64_local(prefix_local, function);
        self.runtime_schema()
            .release_i64_local(parent_length_local, function);
    }

    /// `increment x nanosecondsPerUnit`, the quantum every rounding step here
    /// works in.
    pub(crate) fn emit_temporal_plain_time_rounding_quantum(
        &mut self,
        unit_local: I64Local,
        increment_local: I64Local,
        quantum_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(1));
        quantum_local.store(function);
        for unit in TemporalTimeUnit::ALL {
            unit_local.load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            quantum_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        quantum_local.load(function);
        increment_local.load(function);
        function.instruction(&Instruction::I64Mul);
        quantum_local.store(function);
    }

    /// Matches a GC string against the unit spellings, leaving
    /// `TemporalUnitSlot::Invalid.code()` when it names none.
    pub(crate) fn emit_temporal_plain_time_unit_from_string(
        &mut self,
        string: &GcLocal<StringValue>,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        output.store(function);
        for unit in TemporalUnit::ALL {
            for spelling in [unit.singular(), unit.plural()] {
                self.emit_temporal_string_matches(string, spelling, function)?;
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unit(unit).code()));
                output.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `round`.
    pub(crate) fn emit_temporal_plain_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let unit = schema.reserve_i64_local(function);
        let increment = schema.reserve_i64_local(function);
        let mode = schema.reserve_i64_local(function);
        let quantum = schema.reserve_i64_local(function);
        let total = schema.reserve_i64_local(function);
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_time_fields_from_receiver(&fields, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_ROUND_REQUIRES_A_ROUNDTO_ARGUMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        increment.store(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        mode.store(function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_plain_time_unit_from_string(&string, unit, function)?;
        string.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&argument, function)?;
        self.emit_temporal_duration_rounding_increment_option(&argument, increment, function)?;
        self.emit_temporal_duration_rounding_mode_option(
            &argument,
            TemporalRoundingMode::HalfExpand,
            mode,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &argument,
            TemporalUnitOptionProperty::SmallestUnit,
            unit,
            function,
        )?;
        unit.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINTIME_PROTOTYPE_ROUND_REQUIRES_SMALLESTUNIT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            unit,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_UNIT_OPTION,
            function,
        )?;
        self.emit_temporal_plain_time_validate_increment(unit, increment, function)?;
        self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, function);
        self.emit_temporal_plain_time_total_nanoseconds(&fields, total, function);
        self.emit_temporal_round_time_nanoseconds(total, unit, quantum, mode, function);
        self.emit_temporal_plain_time_from_nanoseconds(total, &fields, function);
        self.emit_alloc_temporal_plain_time(&fields, TemporalPrototypeSource::Intrinsic, function)?;
        self.release_temporal_plain_time_field_locals(fields, function);
        for local in [total, quantum, mode, increment, unit] {
            schema.release_i64_local(local, function);
        }
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 4.3.x `until` and `since`, both through
    /// `DifferenceTemporalPlainTime`.
    pub(super) fn emit_temporal_plain_time_until_or_since(
        &mut self,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let options = self.runtime_schema().reserve_value_local(function);
        let largest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let smallest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);
        let mode_local = self.runtime_schema().reserve_i64_local(function);
        let original_mode_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let total_local = self.runtime_schema().reserve_i64_local(function);
        let other_total_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let field_locals = self.reserve_temporal_plain_time_field_locals(function);
        let other_locals = self.reserve_temporal_plain_time_field_locals(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other_locals,
            function,
        )?;

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit — the order is observable.
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
                // halfFloor; the sign-symmetric modes are unchanged. The original
                // code is captured first so the four rewrites cannot cascade.
                mode_local.load(function);
                original_mode_local.store(function);
                for mode in TemporalRoundingMode::ALL {
                    if mode.negated() == mode {
                        continue;
                    }
                    original_mode_local.load(function);
                    function.instruction(&Instruction::I64Const(mode.code()));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    function.instruction(&Instruction::I64Const(mode.negated().code()));
                    mode_local.store(function);
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
        smallest_unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        smallest_unit_local.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_UNIT_OPTION,
            function,
        )?;
        // An unset or `"auto"` largestUnit falls back to the larger of hour and
        // the smallest unit.
        largest_unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        largest_unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        largest_unit_local.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_UNIT_OPTION,
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;
        self.emit_temporal_plain_time_validate_increment(
            smallest_unit_local,
            increment_local,
            function,
        )?;

        self.emit_temporal_plain_time_total_nanoseconds(&field_locals, total_local, function);
        self.emit_temporal_plain_time_total_nanoseconds(&other_locals, other_total_local, function);
        other_total_local.load(function);
        total_local.load(function);
        function.instruction(&Instruction::I64Sub);
        total_local.store(function);
        self.emit_temporal_plain_time_rounding_quantum(
            smallest_unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_plain_time_round_nanoseconds(
            total_local,
            quantum_local,
            mode_local,
            function,
        );
        match operation {
            TemporalPlainDifferenceOperation::Until => {}
            TemporalPlainDifferenceOperation::Since => {
                function.instruction(&Instruction::I64Const(0));
                total_local.load(function);
                function.instruction(&Instruction::I64Sub);
                total_local.store(function);
            }
        }
        total_local.load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        seconds_local.store(function);
        total_local.load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        subsecond_local.store(function);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_unit_local,
            &duration_locals,
            function,
        )?;
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals, function);
        self.release_temporal_plain_time_field_locals(other_locals, function);
        self.release_temporal_plain_time_field_locals(field_locals, function);
        for local in [
            subsecond_local,
            seconds_local,
            other_total_local,
            total_local,
            quantum_local,
            original_mode_local,
            mode_local,
            increment_local,
            smallest_unit_local,
            largest_unit_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        argument.clear(function);
        options.clear(function);
        Ok(())
    }

    /// `GetTemporalFractionalSecondDigitsOption`: `"auto"` or an integer in
    /// 0..=9. Anything else is a RangeError; a non-numeric, non-`"auto"` value
    /// is a RangeError too, not a TypeError.
    pub(crate) fn emit_temporal_plain_time_fractional_digits_option(
        &mut self,
        options: &ValueLocals,
        digits: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_temporal_duration_option_get(
            options,
            "fractionalSecondDigits",
            &value,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        digits.store(function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Number.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_string_payload(&value, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        let string = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_string_matches(&string, "auto", function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_FRACTIONALSECONDDIGITS_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        string.clear(function);
        function.instruction(&Instruction::Else);
        value.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        value.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(9.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        value.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        value.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_FRACTIONALSECONDDIGITS_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64TruncSatF64S);
        digits.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        value.clear(function);
        Ok(())
    }

    /// `TimeRecordToString`. `precision_local` is a digit count, or
    /// `TEMPORAL_PRECISION_AUTO` / `TEMPORAL_PRECISION_MINUTE`.
    pub(crate) fn emit_temporal_plain_time_record_to_string(
        &mut self,
        field_locals: &[I64Local; 6],
        precision_local: I64Local,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let number_bits_local = self.runtime_schema().reserve_i64_local(function);
        let fraction_local = self.runtime_schema().reserve_i64_local(function);
        let show_digits_local = self.runtime_schema().reserve_i64_local(function);
        let show_value_local = self.runtime_schema().reserve_i64_local(function);

        let output_string = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        for (index, separator) in [(0_usize, None), (1, Some(":"))] {
            if let Some(separator) = separator {
                let piece = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(separator, function)?,
                    function,
                );
                output_string.replace(
                    self.emit_concat_gc_strings(&output_string, &piece, function),
                    function,
                );
                piece.clear(function);
            }
            field_locals[index].load(function);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            number_bits_local.store(function);
            self.emit_date_append_padded_decimal(&output_string, number_bits_local, 2, function)?;
        }

        precision_local.load(function);
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_MINUTE));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        let piece = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(":", function)?,
            function,
        );
        output_string.replace(
            self.emit_concat_gc_strings(&output_string, &piece, function),
            function,
        );
        piece.clear(function);
        field_locals[2].load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits_local.store(function);
        self.emit_date_append_padded_decimal(&output_string, number_bits_local, 2, function)?;

        field_locals[3].load(function);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        field_locals[4].load(function);
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        field_locals[5].load(function);
        function.instruction(&Instruction::I64Add);
        fraction_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        show_digits_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        show_value_local.store(function);

        // `"auto"` trims trailing zeros and suppresses the fraction entirely
        // when the sub-second part is zero.
        precision_local.load(function);
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        fraction_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        fraction_local.load(function);
        show_value_local.store(function);
        function.instruction(&Instruction::I64Const(9));
        show_digits_local.store(function);
        for _ in 0..8 {
            show_value_local.load(function);
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64RemS);
            function.instruction(&Instruction::I64Eqz);
            self.open_frame(ControlFrameKind::If, function);
            show_value_local.load(function);
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64DivS);
            show_value_local.store(function);
            show_digits_local.load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            show_digits_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        for digits in 1_i64..=9 {
            precision_local.load(function);
            function.instruction(&Instruction::I64Const(digits));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            fraction_local.load(function);
            function.instruction(&Instruction::I64Const(10_i64.pow(9 - digits as u32)));
            function.instruction(&Instruction::I64DivS);
            show_value_local.store(function);
            function.instruction(&Instruction::I64Const(digits));
            show_digits_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        show_digits_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let piece = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(".", function)?,
            function,
        );
        output_string.replace(
            self.emit_concat_gc_strings(&output_string, &piece, function),
            function,
        );
        piece.clear(function);
        show_value_local.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits_local.store(function);
        for digits in 1_u32..=9 {
            show_digits_local.load(function);
            function.instruction(&Instruction::I64Const(digits as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_date_append_padded_decimal(
                &output_string,
                number_bits_local,
                digits,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            show_value_local,
            show_digits_local,
            fraction_local,
            number_bits_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        output.set_reference(&output_string, schema, function);
        output_string.clear(function);
        Ok(())
    }

    /// `Temporal.PlainTime.prototype.toLocaleString`.
    ///
    /// `new Intl.DateTimeFormat(locales, options).format(this)`. The time-only
    /// field set is what makes `{ era: "narrow" }` render a plain time rather
    /// than throwing: `era` never cleared `needDefaults`, so the format falls
    /// back to this type's own hour/minute/second defaults and the `era` is
    /// then masked off.
    pub(crate) fn emit_temporal_plain_time_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_time_fields_from_receiver(&fields, function)?;
        self.release_temporal_plain_time_field_locals(fields, function);
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::PlainTime,
            function,
        )
    }

    /// `ToSecondsStringPrecisionRecord`, shared by time and date-time formatters.
    pub(crate) fn emit_temporal_seconds_string_precision(
        &mut self,
        digits_local: I64Local,
        unit_local: I64Local,
        precision_local: I64Local,
        increment_local: I64Local,
        invalid_unit_message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Minute,
            TemporalUnit::Nanosecond,
            invalid_unit_message,
            function,
        )?;
        // `ToSecondsStringPrecisionRecord`: a named smallestUnit fixes both
        // the precision and the rounding unit, and `fractionalSecondDigits`
        // is then ignored.
        for (unit, precision) in [
            (TemporalUnit::Minute, TEMPORAL_PRECISION_MINUTE),
            (TemporalUnit::Second, 0),
            (TemporalUnit::Millisecond, 3),
            (TemporalUnit::Microsecond, 6),
            (TemporalUnit::Nanosecond, 9),
        ] {
            unit_local.load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(precision));
            precision_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(1));
        increment_local.store(function);
        function.instruction(&Instruction::Else);
        digits_local.load(function);
        precision_local.store(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        unit_local.store(function);
        function.instruction(&Instruction::I64Const(1));
        increment_local.store(function);
        // A digit count picks the coarsest unit that can still show it,
        // with the increment making up the difference: 2 digits is
        // milliseconds rounded to the nearest 10. `scale` is the digit
        // count that unit already provides, so zero digits means whole
        // seconds at increment 1 — not tens of seconds.
        for (low, high, unit, scale) in [
            (0_i64, 0_i64, TemporalUnit::Second, 0_i64),
            (1, 3, TemporalUnit::Millisecond, 3),
            (4, 6, TemporalUnit::Microsecond, 6),
            (7, 9, TemporalUnit::Nanosecond, 9),
        ] {
            digits_local.load(function);
            function.instruction(&Instruction::I64Const(low));
            function.instruction(&Instruction::I64GeS);
            digits_local.load(function);
            function.instruction(&Instruction::I64Const(high));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(unit.code()));
            unit_local.store(function);
            for digits in low..=high {
                digits_local.load(function);
                function.instruction(&Instruction::I64Const(digits));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(10_i64.pow((scale - digits) as u32)));
                increment_local.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `toString` reads options; `toJSON` is fixed at `"auto"` precision.
    pub(crate) fn emit_temporal_plain_time_to_string(
        &mut self,
        mode: TemporalPlainTimeStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields = self.reserve_temporal_plain_time_field_locals(function);
        let precision = schema.reserve_i64_local(function);
        let output = schema.reserve_value_local(function);
        self.emit_temporal_plain_time_fields_from_receiver(&fields, function)?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        precision.store(function);
        match mode {
            TemporalPlainTimeStringMode::ToJson => {}
            TemporalPlainTimeStringMode::ToString => {
                let options = schema.reserve_value_local(function);
                let digits = schema.reserve_i64_local(function);
                let unit = schema.reserve_i64_local(function);
                let rounding_mode = schema.reserve_i64_local(function);
                let increment = schema.reserve_i64_local(function);
                let quantum = schema.reserve_i64_local(function);
                let total = schema.reserve_i64_local(function);
                self.emit_builtin_arg_to_value(0, &options, function);
                self.emit_temporal_duration_options_object(&options, function)?;
                self.emit_temporal_plain_time_fractional_digits_option(&options, digits, function)?;
                self.emit_temporal_duration_rounding_mode_option(
                    &options,
                    TemporalRoundingMode::Trunc,
                    rounding_mode,
                    function,
                )?;
                self.emit_temporal_duration_unit_option(
                    &options,
                    TemporalUnitOptionProperty::SmallestUnit,
                    unit,
                    function,
                )?;
                self.emit_temporal_seconds_string_precision(
                    digits,
                    unit,
                    precision,
                    increment,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINTIME_UNIT_OPTION,
                    function,
                )?;
                self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, function);
                self.emit_temporal_plain_time_total_nanoseconds(&fields, total, function);
                self.emit_temporal_round_time_nanoseconds(
                    total,
                    unit,
                    quantum,
                    rounding_mode,
                    function,
                );
                self.emit_temporal_plain_time_from_nanoseconds(total, &fields, function);
                for local in [total, quantum, increment, rounding_mode, unit, digits] {
                    schema.release_i64_local(local, function);
                }
                options.clear(function);
            }
        }
        self.emit_temporal_plain_time_record_to_string(&fields, precision, &output, function)?;
        self.completion().set_normal(&output, function);
        output.clear(function);
        schema.release_i64_local(precision, function);
        self.release_temporal_plain_time_field_locals(fields, function);
        Ok(())
    }
}
