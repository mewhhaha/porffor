//! `Temporal.Duration` prototype methods, `from`, `compare`, and the shared
//! option plumbing (`GetOptionsObject`, `GetTemporalUnitValuedOption`,
//! `GetRoundingModeOption`) that the other Temporal types reuse.
//!
//! The unit, rounding-mode and overflow domains live in
//! [`super::temporal_options`]; this module only emits them.

use super::super::*;
use super::temporal_duration::{
    TEMPORAL_DURATION_ALPHABETICAL_FIELDS, TEMPORAL_DURATION_FIELD_NAMES,
};
use super::temporal_options::{
    TemporalRoundingMode, TemporalTimeUnit, TemporalUnit, TemporalUnitSlot, TEMPORAL_UNIT_SECONDS,
};

impl<'a> FunctionBuilder<'a> {
    /// `GetOptionsObject`: `undefined` stays `undefined`, an Object passes
    /// through, everything else is a TypeError.
    pub(crate) fn emit_temporal_duration_options_object(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        let _ = options_payload_local;
        Ok(())
    }

    /// `Get(options, name)`, short-circuited to `undefined` when the options
    /// bag itself is absent.
    pub(crate) fn emit_temporal_duration_option_get(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        name: &str,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_string_matches(
        &mut self,
        value_payload_local: u32,
        literal: &str,
        scratch_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(self.strings.payload(literal)));
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            value_payload_local,
            scratch_local,
            None,
            function,
        );
    }

    /// `GetTemporalUnitValuedOption`. Leaves a unit code in `output_local`:
    /// `TemporalUnitSlot::Unset.code()` when the property is absent, `TemporalUnitSlot::Auto.code()`
    /// for `"auto"` when `allow_auto`, `TemporalUnitSlot::Invalid.code()` for a string
    /// that names no unit.
    pub(crate) fn emit_temporal_duration_unit_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        name: &str,
        allow_auto: bool,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let scratch_local = self.reserve_temp_local();

        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            name,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        if allow_auto {
            self.emit_temporal_string_matches(value_payload_local, "auto", scratch_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
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
        function.instruction(&Instruction::End);

        self.release_temp_local(scratch_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    /// `GetRoundingModeOption`, defaulting to the caller's fallback.
    pub(crate) fn emit_temporal_duration_rounding_mode_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        default_mode: TemporalRoundingMode,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let scratch_local = self.reserve_temp_local();

        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "roundingMode",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(default_mode.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(output_local));
        for mode in TemporalRoundingMode::ALL {
            self.emit_temporal_string_matches(
                value_payload_local,
                mode.name(),
                scratch_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(mode.code()));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.Duration rounding mode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(scratch_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    /// `GetRoundingIncrementOption`: a positive integer below 10^9.
    pub(crate) fn emit_temporal_duration_rounding_increment_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "roundingIncrement",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000_000.0)));
        // `GetRoundingIncrementOption` rejects `integerIncrement > 10**9`, so
        // an increment that truncates to exactly 10**9 is still in range.
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
            "Invalid Temporal.Duration rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    /// A unit code outside `[low, high]` — which includes
    /// [`TemporalUnitSlot::Auto`], [`TemporalUnitSlot::Unset`] and
    /// [`TemporalUnitSlot::Invalid`], none of which is a `TemporalUnit` and so
    /// none of which can be passed as a bound — is a RangeError.
    ///
    /// `low` and `high` are `TemporalUnit`, not `i64`: the old signature shared
    /// its parameter type with the rounding-mode and overflow codes, so
    /// `require_unit_range(unit, HOUR, AUTO)` compiled and silently widened the
    /// accepted range to include `"auto"`.
    pub(crate) fn emit_temporal_require_unit_range(
        &mut self,
        unit_local: u32,
        low: TemporalUnit,
        high: TemporalUnit,
        range_error: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert!(
            low == high || low.is_larger_than(high),
            "unit range bounds are written largest-first",
        );
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(low.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(high.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            range_error,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `largestUnit` must not name a unit smaller than `smallestUnit`. Both
    /// locals hold unit codes, where a *smaller* code is a *larger* unit, so
    /// the test reads backwards; it was hand-emitted at four sites, each with
    /// its own copy of the message.
    pub(crate) fn emit_temporal_require_largest_not_smaller(
        &mut self,
        largest_unit_local: u32,
        smallest_unit_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "smallestUnit must be smaller than largestUnit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `GetTemporalRelativeToOption`, as far as this backend can go: an
    /// absent option is fine, a String or an Object is accepted (and then
    /// ignored, because resolving one needs calendar arithmetic that only the
    /// calendar-unit paths would use), and every other type is a TypeError.
    fn emit_temporal_duration_validate_relative_to(
        &mut self,
        relative_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(relative_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(relative_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(relative_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `ToTemporalPartialDurationRecord`. Reads the ten properties in
    /// alphabetical order — the reads are observable — leaving each present
    /// field in `field_locals` and its presence flag in `present_locals`.
    pub(crate) fn emit_temporal_duration_partial_record(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        field_locals: &[u32; 10],
        present_locals: &[u32; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let bits_local = self.reserve_temp_local();

        for index in 0..10 {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(field_locals[index]));
        }
        for (name, index) in TEMPORAL_DURATION_ALPHABETICAL_FIELDS {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
            self.emit_temporal_duration_field_to_number(
                value_payload_local,
                value_tag_local,
                bits_local,
                function,
            )?;
            // The per-field bound is checked here rather than in a second pass
            // because a partial record has no companion `f64` array to carry.
            function.instruction(&Instruction::LocalGet(bits_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Abs);
            function.instruction(&Instruction::F64Const(Ieee64::from(
                9_223_372_036_854_775_808.0_f64,
            )));
            function.instruction(&Instruction::F64Ge);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.Duration: fields must not exceed the supported range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(bits_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncSatF64S);
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(bits_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    /// `ToTemporalDuration`: an existing Duration is copied, a String is
    /// parsed, anything else must be a property bag.
    pub(crate) fn emit_to_temporal_duration(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        field_locals: &[u32; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let present_locals = self.reserve_temporal_duration_field_locals();
        let any_present_local = self.reserve_temp_local();

        self.emit_temporal_duration_brand_check_i32(
            value_payload_local,
            value_tag_local,
            brand_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_duration_load_record(record_local, field_locals, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_partial_record(
            value_payload_local,
            value_tag_local,
            field_locals,
            &present_locals,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_present_local));
        for local in present_locals.iter() {
            function.instruction(&Instruction::LocalGet(any_present_local));
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(any_present_local));
        }
        function.instruction(&Instruction::LocalGet(any_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration requires at least one duration field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_reject_invalid(field_locals, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration expects a string, a property bag, or a Temporal.Duration",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_parse_string(value_payload_local, field_locals, function)?;
        self.emit_temporal_duration_reject_invalid(field_locals, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(any_present_local);
        self.release_temporal_duration_field_locals(present_locals);
        self.release_temp_local(record_local);
        self.release_temp_local(brand_local);
        Ok(())
    }

    /// Temporal proposal 7.2.2: `Temporal.Duration.from(item)`.
    pub(crate) fn emit_temporal_duration_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &field_locals,
            function,
        )?;
        self.emit_alloc_temporal_duration(&field_locals, None, function)?;

        self.release_temporal_duration_field_locals(field_locals);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    /// Temporal proposal 7.3.15: `Temporal.Duration.prototype.with`.
    pub(crate) fn emit_temporal_duration_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();
        let partial_locals = self.reserve_temporal_duration_field_locals();
        let present_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration.prototype.with requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // A Duration argument is rejected: `with` takes a partial record, and
        // a Duration's own properties all live on the prototype as accessors.
        self.emit_temporal_duration_brand_check_i32(
            argument_payload_local,
            argument_tag_local,
            brand_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration.prototype.with does not accept a Temporal.Duration",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_partial_record(
            argument_payload_local,
            argument_tag_local,
            &partial_locals,
            &present_locals,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_present_local));
        for index in 0..10 {
            function.instruction(&Instruction::LocalGet(any_present_local));
            function.instruction(&Instruction::LocalGet(present_locals[index]));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(any_present_local));
            function.instruction(&Instruction::LocalGet(present_locals[index]));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(partial_locals[index]));
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(any_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration requires at least one duration field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_create_temporal_duration(&field_locals, function)?;

        self.release_temporal_duration_field_locals(present_locals);
        self.release_temporal_duration_field_locals(partial_locals);
        self.release_temporal_duration_field_locals(field_locals);
        self.release_temp_local(any_present_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    /// `DefaultTemporalLargestUnit`: the index of the first non-zero field, or
    /// nanosecond when the duration is blank.
    pub(crate) fn emit_temporal_duration_default_largest_unit(
        &mut self,
        field_locals: &[u32; 10],
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        // Smallest unit first, so the last write wins and leaves the largest
        // non-zero field. The array subscript and the emitted code are the two
        // separate numberings, named as such: this loop used to use one `index`
        // for both.
        for unit in TemporalUnit::ALL.into_iter().rev() {
            function.instruction(&Instruction::LocalGet(
                field_locals[unit.duration_field_index()],
            ));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
    }

    /// Throw a RangeError when any calendar unit is non-zero, which is exactly
    /// when the operation would need a `relativeTo` this backend cannot
    /// resolve.
    pub(crate) fn emit_temporal_duration_reject_calendar_units(
        &mut self,
        field_locals: &[u32; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mut calendar_units = TemporalUnit::ALL
            .into_iter()
            .filter(|unit| unit.is_calendar_unit());
        let first = calendar_units.next().expect("year is a calendar unit");
        function.instruction(&Instruction::LocalGet(
            field_locals[first.duration_field_index()],
        ));
        for unit in calendar_units {
            function.instruction(&Instruction::LocalGet(
                field_locals[unit.duration_field_index()],
            ));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration operation requires relativeTo for calendar units",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `TemporalDurationFromInternal`: spread a signed (seconds, subsecond)
    /// pair back over the unit fields, stopping at `largest_unit_local`.
    pub(crate) fn emit_temporal_duration_balance(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        largest_unit_local: u32,
        field_locals: &[u32; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let sub_local = self.reserve_temp_local();

        self.emit_temporal_duration_zero_fields(field_locals, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, sub_local),
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

        // The seconds-and-above split depends on the largest unit; the
        // sub-second tail is the same for every unit down to millisecond.
        for (index, (unit, _scale)) in TEMPORAL_UNIT_SECONDS.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(largest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            // Year, month and week durations balance the same way a day
            // duration does once the calendar fields are known to be zero.
            if *unit == TemporalUnit::Day {
                function.instruction(&Instruction::LocalGet(largest_unit_local));
                function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I32Or);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(magnitude_local));
            function.instruction(&Instruction::LocalSet(remainder_local));
            for (slot, divisor) in TEMPORAL_UNIT_SECONDS.iter().skip(index) {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(*divisor));
                function.instruction(&Instruction::I64DivU);
                function.instruction(&Instruction::LocalSet(
                    field_locals[slot.duration_field_index()],
                ));
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(*divisor));
                function.instruction(&Instruction::I64RemU);
                function.instruction(&Instruction::LocalSet(remainder_local));
            }
            function.instruction(&Instruction::End);
        }
        // Millisecond, microsecond and nanosecond largest units fold the whole
        // second count down into the sub-second field, which can overflow the
        // `i64` the record holds; that case is reported as a RangeError.
        for (unit, scale) in [
            (TemporalUnit::Millisecond, 1_000_i64),
            (TemporalUnit::Microsecond, 1_000_000),
            (TemporalUnit::Nanosecond, 1_000_000_000),
        ] {
            function.instruction(&Instruction::LocalGet(largest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(magnitude_local));
            function.instruction(&Instruction::I64Const(9_223_372_036_854_775_807 / scale));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.Duration: fields must not exceed the supported range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(magnitude_local));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(
                field_locals[unit.duration_field_index()],
            ));
            function.instruction(&Instruction::End);
        }
        // Sub-second tail: whatever the largest unit, milliseconds and below
        // come straight out of the nanosecond remainder, except that a
        // microsecond or nanosecond largest unit absorbs the coarser slots.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Microsecond.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(
            field_locals[TemporalUnit::Millisecond.duration_field_index()],
        ));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(
            field_locals[TemporalUnit::Millisecond.duration_field_index()],
        ));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(sub_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(
            field_locals[TemporalUnit::Microsecond.duration_field_index()],
        ));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(
            field_locals[TemporalUnit::Microsecond.duration_field_index()],
        ));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(sub_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(
            field_locals[TemporalUnit::Nanosecond.duration_field_index()],
        ));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(
            field_locals[TemporalUnit::Nanosecond.duration_field_index()],
        ));

        for local in field_locals.iter() {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(*local));
        }

        self.release_temp_local(sub_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(magnitude_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    /// Temporal proposal 7.3.18/7.3.19: `add` and `subtract`. Both refuse
    /// calendar units, because balancing years or months needs a reference
    /// point.
    pub(crate) fn emit_temporal_duration_add_or_subtract(
        &mut self,
        subtract: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let largest_unit_local = self.reserve_temp_local();
        let other_largest_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let other_seconds_local = self.reserve_temp_local();
        let other_subsecond_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();
        let other_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &other_locals,
            function,
        )?;
        if subtract {
            for local in other_locals.iter() {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(*local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(*local));
            }
        }
        self.emit_temporal_duration_reject_calendar_units(&field_locals, function)?;
        self.emit_temporal_duration_reject_calendar_units(&other_locals, function)?;
        self.emit_temporal_duration_default_largest_unit(
            &field_locals,
            largest_unit_local,
            function,
        );
        self.emit_temporal_duration_default_largest_unit(
            &other_locals,
            other_largest_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(other_largest_local));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(other_largest_local));
        function.instruction(&Instruction::LocalSet(largest_unit_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_duration_normalize_seconds(
            &other_locals,
            TemporalUnit::Day,
            other_seconds_local,
            other_subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(other_seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalGet(other_subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_unit_local,
            &field_locals,
            function,
        )?;
        self.emit_create_temporal_duration(&field_locals, function)?;

        self.release_temporal_duration_field_locals(other_locals);
        self.release_temporal_duration_field_locals(field_locals);
        for local in [
            other_subsecond_local,
            other_seconds_local,
            subsecond_local,
            seconds_local,
            other_largest_local,
            largest_unit_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Re-establish the invariant that the sub-second remainder is in
    /// (-10^9, 10^9) and shares the sign of the whole-second count.
    pub(crate) fn emit_temporal_duration_renormalize(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        // Opposite signs: borrow a second so both parts agree.
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::End);
    }

    /// Temporal proposal 7.2.3: `Temporal.Duration.compare(one, two, options)`.
    pub(crate) fn emit_temporal_duration_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let relative_payload_local = self.reserve_temp_local();
        let relative_tag_local = self.reserve_temp_local();
        let one_seconds_local = self.reserve_temp_local();
        let one_subsecond_local = self.reserve_temp_local();
        let two_seconds_local = self.reserve_temp_local();
        let two_subsecond_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let one_locals = self.reserve_temporal_duration_field_locals();
        let two_locals = self.reserve_temporal_duration_field_locals();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &one_locals,
            function,
        )?;
        self.emit_builtin_arg_to_locals(1, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &two_locals,
            function,
        )?;
        self.emit_builtin_arg_to_locals(2, options_payload_local, options_tag_local, function);
        self.emit_temporal_duration_options_object(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "relativeTo",
            relative_payload_local,
            relative_tag_local,
            function,
        )?;
        self.emit_temporal_duration_validate_relative_to(relative_tag_local, function)?;

        // Field-for-field equality short-circuits before any range question.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        for index in 0..10 {
            function.instruction(&Instruction::LocalGet(one_locals[index]));
            function.instruction(&Instruction::LocalGet(two_locals[index]));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(result_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_reject_calendar_units(&one_locals, function)?;
        self.emit_temporal_duration_reject_calendar_units(&two_locals, function)?;
        self.emit_temporal_duration_normalize_seconds(
            &one_locals,
            TemporalUnit::Day,
            one_seconds_local,
            one_subsecond_local,
            function,
        );
        self.emit_temporal_duration_normalize_seconds(
            &two_locals,
            TemporalUnit::Day,
            two_seconds_local,
            two_subsecond_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(one_seconds_local));
        function.instruction(&Instruction::LocalGet(two_seconds_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(one_subsecond_local));
        function.instruction(&Instruction::LocalGet(two_subsecond_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(one_subsecond_local));
        function.instruction(&Instruction::LocalGet(two_subsecond_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(one_seconds_local));
        function.instruction(&Instruction::LocalGet(two_seconds_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_duration_field_locals(two_locals);
        self.release_temporal_duration_field_locals(one_locals);
        for local in [
            result_local,
            two_subsecond_local,
            two_seconds_local,
            one_subsecond_local,
            one_seconds_local,
            relative_tag_local,
            relative_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Decide whether a truncated magnitude should be bumped up to the next
    /// increment. Leaves an `i32` on the stack.
    ///
    /// `remainder_local` and `increment_local` are magnitudes; `sign_local`
    /// carries the duration's sign, which is what separates `ceil` from
    /// `floor`.
    pub(crate) fn emit_temporal_duration_round_up_i32(
        &mut self,
        remainder_local: u32,
        increment_local: u32,
        quotient_local: u32,
        sign_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) {
        let decision_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(decision_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // ceil
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Ceil.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        // floor
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Floor.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        // expand
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Expand.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decision_local));
        function.instruction(&Instruction::End);
        // The half-* family: compare 2 x remainder against the increment. The
        // family is contiguous at the top of the code range, which the `const`
        // assertion in `temporal_options` pins, so one `>=` covers all five.
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfCeil.code(),
        ));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decision_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // halfCeil / halfFloor follow their unrounded siblings; halfExpand
        // always expands; halfTrunc never does; halfEven breaks the tie on the
        // parity of the truncated quotient.
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfCeil.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfFloor.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(mode_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfEven.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(decision_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(decision_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.release_temp_local(decision_local);
    }

    /// Round a signed (seconds, subsecond) pair to a multiple of
    /// `quantum_local` nanoseconds. `quantum_local` is at most 10^9 so the
    /// arithmetic never leaves the sub-second slot except through the carry.
    fn emit_temporal_duration_round_subsecond(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        quantum_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let sub_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, sub_local),
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
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient_local));
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            quantum_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(sub_local));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(sub_local));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(subsecond_local));

        self.release_temp_local(quotient_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(sub_local);
        self.release_temp_local(magnitude_local);
        self.release_temp_local(sign_local);
    }

    /// Round a signed (seconds, subsecond) pair to a whole number of
    /// `unit_seconds x increment` seconds.
    fn emit_temporal_duration_round_seconds(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        quantum_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let sub_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let scaled_quantum_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, sub_local),
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
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient_local));
        // Both the remainder and the increment are converted to nanoseconds so
        // the sub-second tail participates in the midpoint comparison exactly.
        // The increment never exceeds a day, so 2 x 86400 x 10^9 stays inside
        // an `i64`.
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(sub_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(scaled_quantum_local));
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            scaled_quantum_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(subsecond_local));

        self.release_temp_local(scaled_quantum_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(sub_local);
        self.release_temp_local(magnitude_local);
        self.release_temp_local(sign_local);
    }

    /// Temporal proposal 7.3.20: `Temporal.Duration.prototype.round`.
    pub(crate) fn emit_temporal_duration_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let relative_payload_local = self.reserve_temp_local();
        let relative_tag_local = self.reserve_temp_local();
        let smallest_local = self.reserve_temp_local();
        let largest_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let default_largest_local = self.reserve_temp_local();
        let maximum_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // A string argument is shorthand for `{ smallestUnit: <string> }`.
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::LocalSet(largest_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(relative_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(relative_tag_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_unit_string_to_code(
            argument_payload_local,
            smallest_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(options_payload_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(options_tag_local));
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "largestUnit",
            true,
            largest_local,
            function,
        )?;
        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "relativeTo",
            relative_payload_local,
            relative_tag_local,
            function,
        )?;
        self.emit_temporal_duration_validate_relative_to(relative_tag_local, function)?;
        self.emit_temporal_duration_rounding_increment_option(
            options_payload_local,
            options_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            options_payload_local,
            options_tag_local,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "smallestUnit",
            false,
            smallest_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration.prototype.round requires largestUnit, smallestUnit, or both",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_require_unit_range(
            smallest_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.Duration unit option",
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::LocalSet(smallest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_require_unit_range(
            largest_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.Duration unit option",
            function,
        )?;
        function.instruction(&Instruction::End);
        // `auto` and an absent largestUnit both mean "the duration's own
        // largest unit, but never smaller than smallestUnit".
        self.emit_temporal_duration_default_largest_unit(
            &field_locals,
            default_largest_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(largest_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_largest_not_smaller(largest_local, smallest_local, function)?;

        // `ValidateTemporalRoundingIncrement`: the increment must be strictly
        // below the unit's maximum and must divide it exactly. Year through
        // day have no maximum at all.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(maximum_local));
        for unit in TemporalTimeUnit::ALL {
            function.instruction(&Instruction::LocalGet(smallest_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(unit.maximum_rounding_increment()));
            function.instruction(&Instruction::LocalSet(maximum_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
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
            "Invalid Temporal.Duration rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_reject_calendar_units(&field_locals, function)?;
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration operation requires relativeTo for calendar units",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_duration_unit_quantum(
            smallest_local,
            increment_local,
            quantum_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_round_seconds(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_round_subsecond(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_local,
            &field_locals,
            function,
        )?;
        self.emit_create_temporal_duration(&field_locals, function)?;

        self.release_temporal_duration_field_locals(field_locals);
        for local in [
            maximum_local,
            default_largest_local,
            subsecond_local,
            seconds_local,
            quantum_local,
            mode_local,
            increment_local,
            largest_local,
            smallest_local,
            relative_tag_local,
            relative_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The rounding quantum for a unit code and increment: seconds for the
    /// day-through-second codes, nanoseconds below that.
    fn emit_temporal_duration_unit_quantum(
        &mut self,
        unit_local: u32,
        increment_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        for (unit, scale) in [
            (3_i64, 86_400_i64),
            (4, 3_600),
            (5, 60),
            (6, 1),
            (7, 1_000_000),
            (8, 1_000),
            (9, 1),
        ] {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(output_local));
    }

    /// Map a unit string to its code, throwing a RangeError when it names no
    /// unit. Shared by the string shorthand of `round` and `total`.
    fn emit_temporal_duration_unit_string_to_code(
        &mut self,
        string_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let scratch_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        function.instruction(&Instruction::LocalSet(output_local));
        for unit in TemporalUnit::ALL {
            for spelling in [unit.singular(), unit.plural()] {
                self.emit_temporal_string_matches(
                    string_payload_local,
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

    /// Temporal proposal 7.3.21: `Temporal.Duration.prototype.total`.
    pub(crate) fn emit_temporal_duration_total(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let relative_payload_local = self.reserve_temp_local();
        let relative_tag_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let scale_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_unit_string_to_code(
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
        self.emit_temporal_duration_option_get(
            argument_payload_local,
            argument_tag_local,
            "relativeTo",
            relative_payload_local,
            relative_tag_local,
            function,
        )?;
        self.emit_temporal_duration_validate_relative_to(relative_tag_local, function)?;
        self.emit_temporal_duration_unit_option(
            argument_payload_local,
            argument_tag_local,
            "unit",
            false,
            unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration.prototype.total requires a unit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.Duration unit option",
            function,
        )?;
        self.emit_temporal_duration_reject_calendar_units(&field_locals, function)?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Duration operation requires relativeTo for calendar units",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        // Split before converting so the quotient keeps full precision even
        // when the second count is close to 2^53.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(scale_local));
        for (unit, scale) in TEMPORAL_UNIT_SECONDS {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::LocalSet(scale_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::Else);
        // Sub-second units multiply instead of divide, so the whole-second
        // part is converted first and the remainder folded in afterwards.
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::LocalSet(scale_local));
        for (unit, scale) in [(7_i64, 1_000_000_i64), (8, 1_000), (9, 1)] {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::LocalSet(scale_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_duration_field_locals(field_locals);
        for local in [
            remainder_local,
            quotient_local,
            subsecond_local,
            seconds_local,
            scale_local,
            unit_local,
            relative_tag_local,
            relative_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 7.3.22/7.3.23: `toString`, `toJSON` and
    /// `toLocaleString`. Only `toString` reads options; the other two always
    /// use `auto` precision.
    pub(crate) fn emit_temporal_duration_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reads_options = matches!(
            builtin,
            StandardBuiltinId::TemporalDurationPrototypeToString
        );
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let scratch_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let smallest_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let hours_local = self.reserve_temp_local();
        let minutes_local = self.reserve_temp_local();
        let default_largest_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::LocalSet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        function.instruction(&Instruction::LocalSet(mode_local));
        if reads_options {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_duration_options_object(
                options_payload_local,
                options_tag_local,
                function,
            )?;
            self.emit_temporal_duration_option_get(
                options_payload_local,
                options_tag_local,
                "fractionalSecondDigits",
                value_payload_local,
                value_tag_local,
                function,
            )?;
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
                "Invalid Temporal.Duration unit option",
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
                "Invalid Temporal.Duration unit option",
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
                smallest_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(smallest_local));
            function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_require_unit_range(
                smallest_local,
                TemporalUnit::Second,
                TemporalUnit::Nanosecond,
                "Invalid Temporal.Duration unit option",
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        // `ToSecondsStringPrecisionRecord`: smallestUnit wins, otherwise the
        // digit count picks both the printed width and the rounding quantum.
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(smallest_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(quantum_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        for digits in 0..=9_i64 {
            function.instruction(&Instruction::LocalGet(digits_local));
            function.instruction(&Instruction::I64Const(digits));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(10_i64.pow((9 - digits) as u32)));
            function.instruction(&Instruction::LocalSet(quantum_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_sign(&field_locals, sign_local, function);
        // Without rounding the components print verbatim; with rounding the
        // whole time part is rebalanced, because a carry out of the seconds
        // has to reach the minutes and hours the way `TemporalDurationFromInternal`
        // would.
        function.instruction(&Instruction::LocalGet(field_locals[4]));
        function.instruction(&Instruction::LocalSet(hours_local));
        function.instruction(&Instruction::LocalGet(field_locals[5]));
        function.instruction(&Instruction::LocalSet(minutes_local));
        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Second,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(hours_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(minutes_local));
        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_duration_round_subsecond(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        // `LargerOfTwoTemporalUnits(defaultLargestUnit, second)`, clamped to
        // hour because the time part never balances up into days.
        self.emit_temporal_duration_default_largest_unit(
            &field_locals,
            default_largest_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::LocalSet(default_largest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(default_largest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(hours_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(default_largest_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(minutes_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Print magnitudes; the sign is a single prefix.
        for local in [seconds_local, subsecond_local, hours_local, minutes_local] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("P")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        for (index, designator) in [(0_usize, "Y"), (1, "M"), (2, "W"), (3, "D")] {
            function.instruction(&Instruction::LocalGet(field_locals[index]));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_duration_append_magnitude(
                output_payload_local,
                field_locals[index],
                number_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(designator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(time_payload_local));
        for (local, designator) in [(hours_local, "H"), (minutes_local, "M")] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_duration_append_magnitude(
                time_payload_local,
                local,
                number_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(designator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                time_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(time_payload_local));
            function.instruction(&Instruction::End);
        }
        // The seconds component is printed when it carries information, when
        // an explicit precision was asked for, or when nothing else would be.
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(field_locals[0]));
        function.instruction(&Instruction::LocalGet(field_locals[1]));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(field_locals[2]));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(field_locals[3]));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(hours_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_duration_append_magnitude(
            time_payload_local,
            seconds_local,
            number_payload_local,
            piece_payload_local,
            function,
        )?;
        self.emit_temporal_duration_append_fraction(
            time_payload_local,
            subsecond_local,
            digits_local,
            number_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("S")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(time_payload_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(time_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(time_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("T")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_concat_string_payloads_local(output_payload_local, time_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_duration_field_locals(field_locals);
        for local in [
            number_payload_local,
            piece_payload_local,
            time_payload_local,
            output_payload_local,
            default_largest_local,
            minutes_local,
            hours_local,
            sign_local,
            subsecond_local,
            seconds_local,
            increment_local,
            quantum_local,
            mode_local,
            smallest_local,
            digits_local,
            scratch_local,
            value_tag_local,
            value_payload_local,
            options_tag_local,
            options_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Append `abs(value)` as a decimal string.
    fn emit_temporal_duration_append_magnitude(
        &mut self,
        output_payload_local: u32,
        value_local: u32,
        number_payload_local: u32,
        piece_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        Ok(())
    }

    /// `FormatFractionalSeconds`. `digits_local` is -1 for `auto`, in which
    /// case the nine-digit padding is trimmed of trailing zeros.
    fn emit_temporal_duration_append_fraction(
        &mut self,
        output_payload_local: u32,
        subsecond_local: u32,
        digits_local: u32,
        number_payload_local: u32,
        piece_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let keep_local = self.reserve_temp_local();
        let scaled_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::LocalSet(keep_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalSet(scaled_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scaled_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(keep_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scaled_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(scaled_local));
        function.instruction(&Instruction::LocalGet(keep_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(keep_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(keep_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(keep_local));

        function.instruction(&Instruction::LocalGet(keep_local));
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
        for position in 0..9_i64 {
            function.instruction(&Instruction::LocalGet(keep_local));
            function.instruction(&Instruction::I64Const(position));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(subsecond_local));
            function.instruction(&Instruction::I64Const(10_i64.pow((8 - position) as u32)));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(number_payload_local));
            self.emit_number_to_string_payload(number_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(scaled_local);
        self.release_temp_local(keep_local);
        Ok(())
    }

    /// `ParseTemporalDurationString`. The grammar is
    /// `Sign? P (nnn[YMWD])* (T (nnn(.fff)?[HMS])*)?`, with a fraction allowed
    /// only on the last time component present, and at least one component
    /// required overall.
    pub(crate) fn emit_temporal_duration_parse_string(
        &mut self,
        string_payload_local: u32,
        field_locals: &[u32; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let digit_count_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let fraction_digits_local = self.reserve_temp_local();
        let has_fraction_local = self.reserve_temp_local();
        let seen_local = self.reserve_temp_local();
        let stage_local = self.reserve_temp_local();
        let nanoseconds_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, length_local, function);
        self.emit_temporal_duration_zero_fields(field_locals, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stage_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_fraction_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(nanoseconds_local));

        self.emit_temporal_duration_peek(
            offset_local,
            cursor_local,
            length_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x2212));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_peek(
            offset_local,
            cursor_local,
            length_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'P' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'p' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));

        // One pass over the components. `stage` is the index of the lowest
        // designator already consumed, so designators must strictly ascend.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_duration_peek(
            offset_local,
            cursor_local,
            length_local,
            byte_local,
            function,
        );
        // The time designator flips the parser into the H/M/S half.
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::LocalSet(stage_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        // A time designator must be followed by at least one component.
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fraction_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fraction_digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        self.emit_temporal_duration_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        // Optional fraction, scaled to nine digits.
        self.emit_temporal_duration_peek(
            offset_local,
            cursor_local,
            length_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_fraction_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::LocalSet(fraction_digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        self.emit_temporal_duration_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(fraction_digits_local));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_peek(
            offset_local,
            cursor_local,
            length_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(seen_local));

        // Designators: Y/M/W/D before the T, H/M/S after it.
        let designators: [(u8, u8, i64, usize); 7] = [
            (b'Y', b'y', 1, 0),
            (b'W', b'w', 3, 2),
            (b'D', b'd', 4, 3),
            (b'H', b'h', 6, 4),
            (b'S', b's', 8, 6),
            (b'M', b'm', 2, 1),
            (b'M', b'm', 7, 5),
        ];
        for (index, (upper, lower, stage, field)) in designators.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(*upper as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(*lower as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            // The two `M` spellings are told apart by whether the parser has
            // already crossed the time designator.
            if index == 5 {
                function.instruction(&Instruction::LocalGet(stage_local));
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I32And);
            } else if index == 6 {
                function.instruction(&Instruction::LocalGet(stage_local));
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(stage_local));
            function.instruction(&Instruction::I64Const(*stage));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::End);
            // Hours, minutes and seconds only exist after the time
            // designator; `P2H` and `P2S` are not durations.
            if *field >= 4 {
                function.instruction(&Instruction::LocalGet(stage_local));
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(valid_local));
                function.instruction(&Instruction::End);
            }
            // Date components and a time component that is not the last one
            // may not carry a fraction.
            if *field < 4 {
                function.instruction(&Instruction::LocalGet(has_fraction_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(valid_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I64Const(*stage));
            function.instruction(&Instruction::LocalSet(stage_local));
            function.instruction(&Instruction::LocalGet(field_locals[*field]));
            function.instruction(&Instruction::LocalGet(value_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(field_locals[*field]));
            if *field >= 4 {
                let scale: i64 = match *field {
                    4 => 3_600,
                    5 => 60,
                    _ => 1,
                };
                function.instruction(&Instruction::LocalGet(fraction_local));
                function.instruction(&Instruction::I64Const(scale));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::LocalSet(nanoseconds_local));
                // A fraction ends the time part: nothing may follow it.
                function.instruction(&Instruction::LocalGet(has_fraction_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(9));
                function.instruction(&Instruction::LocalSet(stage_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(stage_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(seen_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.Duration string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Spread the fractional remainder over the sub-hour fields.
        function.instruction(&Instruction::LocalGet(field_locals[5]));
        function.instruction(&Instruction::LocalGet(nanoseconds_local));
        function.instruction(&Instruction::I64Const(60_000_000_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(field_locals[5]));
        function.instruction(&Instruction::LocalGet(nanoseconds_local));
        function.instruction(&Instruction::I64Const(60_000_000_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(nanoseconds_local));
        for (field, divisor) in [
            (6_usize, 1_000_000_000_i64),
            (7, 1_000_000),
            (8, 1_000),
            (9, 1),
        ] {
            function.instruction(&Instruction::LocalGet(field_locals[field]));
            function.instruction(&Instruction::LocalGet(nanoseconds_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(field_locals[field]));
            function.instruction(&Instruction::LocalGet(nanoseconds_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::LocalSet(nanoseconds_local));
        }
        for local in field_locals.iter() {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(*local));
        }

        for local in [
            nanoseconds_local,
            stage_local,
            seen_local,
            has_fraction_local,
            fraction_digits_local,
            fraction_local,
            digit_count_local,
            value_local,
            sign_local,
            valid_local,
            byte_local,
            cursor_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
        let _ = TEMPORAL_DURATION_FIELD_NAMES;
        Ok(())
    }

    fn emit_temporal_duration_peek(
        &mut self,
        offset_local: u32,
        cursor_local: u32,
        length_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_duration_byte_is_digit(&mut self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }
}
