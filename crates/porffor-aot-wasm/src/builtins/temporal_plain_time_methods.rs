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
use super::temporal_options::{
    StringValuedOption, TemporalOverflow, TemporalRoundingMode, TemporalTimeUnit, TemporalUnit,
    TemporalUnitSlot,
};
use super::temporal_plain_time::TEMPORAL_PLAIN_TIME_ALPHABETICAL_FIELDS;

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
        options_payload_local: u32,
        options_tag_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();

        self.emit_temporal_duration_options_object(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "overflow",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(recognized_local));
        for accepted in TemporalOverflow::ALLOWED {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(accepted.name()),
            ));
            function.instruction(&Instruction::LocalSet(expected_payload_local));
            self.emit_string_payload_equality_i32(
                value_payload_local,
                expected_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(recognized_local));
            function.instruction(&Instruction::I64Const(StringValuedOption::code(*accepted)));
            function.instruction(&Instruction::LocalSet(overflow_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(recognized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainTime overflow option",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            recognized_local,
            expected_payload_local,
            value_tag_local,
            value_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ToTemporalTimeRecord`. Reads the six properties in alphabetical order —
    /// the reads are observable — leaving each value in `field_locals` and its
    /// presence flag in `present_locals`. Absent fields are left at
    /// `default_locals` so the same emitter serves both the complete form
    /// (`from`, defaults zero) and the partial form (`with`, defaults the
    /// receiver).
    fn emit_temporal_plain_time_read_fields(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        field_locals: &[u32; 6],
        present_locals: &[u32; 6],
        any_present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let parsed_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_present_local));
        for (property, index) in TEMPORAL_PLAIN_TIME_ALPHABETICAL_FIELDS {
            self.emit_temporal_property_bag_integer(
                argument_payload_local,
                argument_tag_local,
                property,
                property_key_local,
                value_payload_local,
                value_tag_local,
                present_local,
                parsed_local,
                0,
                "Temporal.PlainTime field must be finite",
                function,
            )?;
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(parsed_local));
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(any_present_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(any_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainTime requires at least one time field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for local in [
            parsed_local,
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ToTemporalTime`. Accepts a branded `Temporal.PlainTime` (cloned), any
    /// other object (read as a property bag), or an ISO time string.
    ///
    /// `read_options` is false for `compare`, `equals`, `until` and `since`,
    /// which pass no options through to the conversion.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_to_temporal_time(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        read_options: bool,
        field_locals: &[u32; 6],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let nanoseconds_local = self.reserve_temp_local();
        let present_locals = self.reserve_temporal_plain_time_field_locals();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        for local in field_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }

        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_brand_check_i32(
            argument_payload_local,
            argument_tag_local,
            brand_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_plain_time_load_record(record_local, field_locals, function);
        if read_options {
            self.emit_temporal_plain_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_read_fields(
            argument_payload_local,
            argument_tag_local,
            field_locals,
            &present_locals,
            any_present_local,
            function,
        )?;
        if read_options {
            self.emit_temporal_plain_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_regulate_time(field_locals, overflow_local, function)?;
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
            "Temporal.PlainTime expects a string, a property bag, or a Temporal.PlainTime",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_plain_time_string(
            argument_payload_local,
            field_locals[0],
            field_locals[1],
            field_locals[2],
            nanoseconds_local,
            function,
        )?;
        // The parser hands back one nanosecond count for the whole fraction.
        for (index, divisor) in [(5_usize, 1_000_i64), (4, 1_000), (3, 1_000)] {
            function.instruction(&Instruction::LocalGet(nanoseconds_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemS);
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::LocalGet(nanoseconds_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(nanoseconds_local));
        }
        // The overflow option is read only after the string parses, which is
        // what `observable-get-overflow-argument-string-invalid.js` pins.
        if read_options {
            self.emit_temporal_plain_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);

        self.release_temporal_plain_time_field_locals(present_locals);
        for local in [
            nanoseconds_local,
            any_present_local,
            handled_local,
            overflow_local,
            record_local,
            brand_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.2.2 `Temporal.PlainTime.from`.
    pub(crate) fn emit_temporal_plain_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            true,
            &field_locals,
            function,
        )?;
        self.emit_alloc_temporal_plain_time(&field_locals, None, function)?;

        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Leaves an `i64` in `comparison_local`: -1, 0 or 1 for the two times.
    fn emit_temporal_plain_time_compare_fields(
        &mut self,
        left_locals: &[u32; 6],
        right_locals: &[u32; 6],
        comparison_local: u32,
        function: &mut Function,
    ) {
        let left_total_local = self.reserve_temp_local();
        let right_total_local = self.reserve_temp_local();
        self.emit_temporal_plain_time_total_nanoseconds(left_locals, left_total_local, function);
        self.emit_temporal_plain_time_total_nanoseconds(right_locals, right_total_local, function);
        function.instruction(&Instruction::LocalGet(left_total_local));
        function.instruction(&Instruction::LocalGet(right_total_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(left_total_local));
        function.instruction(&Instruction::LocalGet(right_total_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(comparison_local));
        self.release_temp_local(right_total_local);
        self.release_temp_local(left_total_local);
    }

    /// Temporal proposal 4.2.3 `Temporal.PlainTime.compare`.
    pub(crate) fn emit_temporal_plain_time_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();
        let left_locals = self.reserve_temporal_plain_time_field_locals();
        let right_locals = self.reserve_temporal_plain_time_field_locals();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        for (index, locals) in [(0_usize, &left_locals), (1, &right_locals)] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_to_temporal_time(
                argument_payload_local,
                argument_tag_local,
                undefined_payload_local,
                undefined_tag_local,
                false,
                locals,
                function,
            )?;
        }
        self.emit_temporal_plain_time_compare_fields(
            &left_locals,
            &right_locals,
            comparison_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_time_field_locals(right_locals);
        self.release_temporal_plain_time_field_locals(left_locals);
        for local in [
            comparison_local,
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `Temporal.PlainTime.prototype.equals`.
    pub(crate) fn emit_temporal_plain_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();
        let other_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &other_locals,
            function,
        )?;
        self.emit_temporal_plain_time_compare_fields(
            &field_locals,
            &other_locals,
            comparison_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_time_field_locals(other_locals);
        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            comparison_local,
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `with`. A partial time record replaces only the
    /// fields it names; the rest come from the receiver.
    pub(crate) fn emit_temporal_plain_time_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let probe_payload_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();
        let present_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainTime.prototype.with requires an object",
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
            "Temporal.PlainTime.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject`: a bag that names a calendar or a time
        // zone is a caller mistake, not a partial time. These are ordinary
        // `Get`s, not `HasOwnProperty` checks — the reads are observable, and
        // an explicit `calendar: undefined` is allowed through.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                key_local,
                probe_payload_local,
                present_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.PlainTime.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_plain_time_read_fields(
            argument_payload_local,
            argument_tag_local,
            &field_locals,
            &present_locals,
            any_present_local,
            function,
        )?;
        self.emit_temporal_plain_time_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_regulate_time(&field_locals, overflow_local, function)?;
        self.emit_alloc_temporal_plain_time(&field_locals, None, function)?;

        self.release_temporal_plain_time_field_locals(present_locals);
        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            any_present_local,
            present_local,
            probe_payload_local,
            key_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `add` and `subtract`. The duration's calendar
    /// fields are ignored outright — a `PlainTime` has no date for a year or a
    /// month to land on — and the result wraps around midnight.
    pub(crate) fn emit_temporal_plain_time_add_or_subtract(
        &mut self,
        subtract: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
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
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_plain_time_total_nanoseconds(&field_locals, total_local, function);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &field_locals, function);
        self.emit_alloc_temporal_plain_time(&field_locals, None, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            total_local,
            subsecond_local,
            seconds_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
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
        unit_local: u32,
        increment_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(maximum_local));
        for unit in TemporalTimeUnit::ALL {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.maximum_rounding_increment()));
            function.instruction(&Instruction::LocalSet(maximum_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainTime rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_local);
        Ok(())
    }

    /// `increment x nanosecondsPerUnit`, the quantum every rounding step here
    /// works in.
    pub(crate) fn emit_temporal_plain_time_rounding_quantum(
        &mut self,
        unit_local: u32,
        increment_local: u32,
        quantum_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(quantum_local));
        for unit in TemporalTimeUnit::ALL {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.nanoseconds()));
            function.instruction(&Instruction::LocalSet(quantum_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(quantum_local));
    }

    /// Matches a string payload against the unit spellings, leaving
    /// `TemporalUnitSlot::Invalid.code()` when it names none.
    pub(crate) fn emit_temporal_plain_time_unit_from_payload(
        &mut self,
        value_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let scratch_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        for unit in TemporalUnit::ALL {
            for spelling in [unit.singular(), unit.plural()] {
                self.emit_temporal_string_matches(
                    value_payload_local,
                    spelling,
                    scratch_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unit(unit).code()));
                function.instruction(&Instruction::LocalSet(output_local));
                function.instruction(&Instruction::End);
            }
        }
        self.release_temp_local(scratch_local);
    }

    /// Temporal proposal 4.3.x `round`.
    pub(crate) fn emit_temporal_plain_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainTime.prototype.round requires a roundTo argument",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_unit_from_payload(
            argument_payload_local,
            unit_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            argument_payload_local,
            argument_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            argument_payload_local,
            argument_tag_local,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            argument_payload_local,
            argument_tag_local,
            "smallestUnit",
            false,
            unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainTime.prototype.round requires smallestUnit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainTime unit option",
            function,
        )?;
        self.emit_temporal_plain_time_validate_increment(unit_local, increment_local, function)?;
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_plain_time_total_nanoseconds(&field_locals, total_local, function);
        self.emit_temporal_plain_time_round_nanoseconds(
            total_local,
            quantum_local,
            mode_local,
            function,
        );
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &field_locals, function);
        self.emit_alloc_temporal_plain_time(&field_locals, None, function)?;

        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            total_local,
            quantum_local,
            mode_local,
            increment_local,
            unit_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `until` and `since`, both through
    /// `DifferenceTemporalPlainTime`.
    pub(crate) fn emit_temporal_plain_time_until_or_since(
        &mut self,
        since: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let largest_unit_local = self.reserve_temp_local();
        let smallest_unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let original_mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let other_total_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();
        let other_locals = self.reserve_temporal_plain_time_field_locals();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &other_locals,
            function,
        )?;

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit — the order is observable.
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
            // `NegateRoundingMode`: ceil and floor swap, as do halfCeil and
            // halfFloor; the sign-symmetric modes are unchanged. The original
            // code is captured first so the four rewrites cannot cascade.
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
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::LocalSet(smallest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainTime unit option",
            function,
        )?;
        // An unset or `"auto"` largestUnit falls back to the larger of hour and
        // the smallest unit.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::LocalSet(largest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainTime unit option",
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
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
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
        if since {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(total_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(total_local));
        }
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_unit_local,
            &duration_locals,
            function,
        )?;
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        self.release_temporal_plain_time_field_locals(other_locals);
        self.release_temporal_plain_time_field_locals(field_locals);
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
            undefined_tag_local,
            undefined_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `GetTemporalFractionalSecondDigitsOption`: `"auto"` or an integer in
    /// 0..=9. Anything else is a RangeError; a non-numeric, non-`"auto"` value
    /// is a RangeError too, not a TypeError.
    pub(crate) fn emit_temporal_plain_time_fractional_digits_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let scratch_local = self.reserve_temp_local();

        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "fractionalSecondDigits",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_string_matches(value_payload_local, "auto", scratch_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainTime fractionalSecondDigits option",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(9.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainTime fractionalSecondDigits option",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [scratch_local, value_tag_local, value_payload_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `TimeRecordToString`. `precision_local` is a digit count, or
    /// `TEMPORAL_PRECISION_AUTO` / `TEMPORAL_PRECISION_MINUTE`.
    pub(crate) fn emit_temporal_plain_time_record_to_string(
        &mut self,
        field_locals: &[u32; 6],
        precision_local: u32,
        output_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let show_digits_local = self.reserve_temp_local();
        let show_value_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        for (index, separator) in [(0_usize, None), (1, Some(":"))] {
            if let Some(separator) = separator {
                function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
                function.instruction(&Instruction::LocalSet(piece_payload_local));
                self.emit_concat_string_payloads_local(
                    output_payload_local,
                    piece_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(output_payload_local));
            }
            function.instruction(&Instruction::LocalGet(field_locals[index]));
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(number_payload_local));
            self.emit_date_append_padded_decimal(
                output_payload_local,
                number_payload_local,
                2,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(precision_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_MINUTE));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(":")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(field_locals[2]));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            2,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(field_locals[3]));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(field_locals[4]));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[5]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(show_digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(show_value_local));

        // `"auto"` trims trailing zeros and suppresses the fraction entirely
        // when the sub-second part is zero.
        function.instruction(&Instruction::LocalGet(precision_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::LocalSet(show_value_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::LocalSet(show_digits_local));
        for _ in 0..8 {
            function.instruction(&Instruction::LocalGet(show_value_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64RemS);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(show_value_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(show_value_local));
            function.instruction(&Instruction::LocalGet(show_digits_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(show_digits_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        for digits in 1_i64..=9 {
            function.instruction(&Instruction::LocalGet(precision_local));
            function.instruction(&Instruction::I64Const(digits));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(fraction_local));
            function.instruction(&Instruction::I64Const(10_i64.pow(9 - digits as u32)));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(show_value_local));
            function.instruction(&Instruction::I64Const(digits));
            function.instruction(&Instruction::LocalSet(show_digits_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(show_digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(show_value_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        for digits in 1_u32..=9 {
            function.instruction(&Instruction::LocalGet(show_digits_local));
            function.instruction(&Instruction::I64Const(digits as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_date_append_padded_decimal(
                output_payload_local,
                number_payload_local,
                digits,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            show_value_local,
            show_digits_local,
            fraction_local,
            number_payload_local,
            piece_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 4.3.x `toString`, `toJSON` and `toLocaleString`. Only
    /// `toString` reads options; the other two are fixed at `"auto"` precision.
    pub(crate) fn emit_temporal_plain_time_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let precision_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_time_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::LocalSet(precision_local));
        if matches!(
            builtin,
            StandardBuiltinId::TemporalPlainTimePrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_duration_options_object(
                options_payload_local,
                options_tag_local,
                function,
            )?;
            self.emit_temporal_plain_time_fractional_digits_option(
                options_payload_local,
                options_tag_local,
                digits_local,
                function,
            )?;
            self.emit_temporal_duration_rounding_mode_option(
                options_payload_local,
                options_tag_local,
                TemporalRoundingMode::Trunc,
                mode_local,
                function,
            )?;
            self.emit_temporal_duration_unit_option(
                options_payload_local,
                options_tag_local,
                "smallestUnit",
                false,
                unit_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_require_unit_range(
                unit_local,
                TemporalUnit::Minute,
                TemporalUnit::Nanosecond,
                "Invalid Temporal.PlainTime unit option",
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
                function.instruction(&Instruction::LocalGet(unit_local));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(precision));
                function.instruction(&Instruction::LocalSet(precision_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(increment_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(digits_local));
            function.instruction(&Instruction::LocalSet(precision_local));
            function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
            function.instruction(&Instruction::LocalSet(unit_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(increment_local));
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
                function.instruction(&Instruction::LocalGet(digits_local));
                function.instruction(&Instruction::I64Const(low));
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::LocalGet(digits_local));
                function.instruction(&Instruction::I64Const(high));
                function.instruction(&Instruction::I64LeS);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::LocalSet(unit_local));
                for digits in low..=high {
                    function.instruction(&Instruction::LocalGet(digits_local));
                    function.instruction(&Instruction::I64Const(digits));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function
                        .instruction(&Instruction::I64Const(10_i64.pow((scale - digits) as u32)));
                    function.instruction(&Instruction::LocalSet(increment_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);

            self.emit_temporal_plain_time_rounding_quantum(
                unit_local,
                increment_local,
                quantum_local,
                function,
            );
            self.emit_temporal_plain_time_total_nanoseconds(&field_locals, total_local, function);
            self.emit_temporal_plain_time_round_nanoseconds(
                total_local,
                quantum_local,
                mode_local,
                function,
            );
            self.emit_temporal_plain_time_from_nanoseconds(total_local, &field_locals, function);
        }
        self.emit_temporal_plain_time_record_to_string(
            &field_locals,
            precision_local,
            output_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_time_field_locals(field_locals);
        for local in [
            output_payload_local,
            total_local,
            quantum_local,
            increment_local,
            precision_local,
            mode_local,
            unit_local,
            digits_local,
            options_tag_local,
            options_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
