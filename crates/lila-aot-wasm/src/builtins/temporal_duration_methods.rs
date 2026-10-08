//! `Temporal.Duration` prototype methods, `from`, `compare`, and the shared
//! option plumbing (`GetOptionsObject`, `GetTemporalUnitValuedOption`,
//! `GetRoundingModeOption`) that the other Temporal types reuse.
//!
//! The unit, rounding-mode and overflow domains live in
//! [`super::temporal_options`]; this module only emits them.

use super::super::*;
use super::temporal_duration::{
    TemporalDurationFields, TemporalDurationNumberProjection, TemporalDurationSubsecondUnit,
    TEMPORAL_DURATION_ALPHABETICAL_FIELDS, TEMPORAL_DURATION_FIELD_NAMES,
};
use super::temporal_options::{
    TemporalRoundingMode, TemporalTimeUnit, TemporalUnit, TemporalUnitOptionProperty,
    TemporalUnitSlot, TEMPORAL_UNIT_SECONDS,
};
use super::temporal_zone_provider::TemporalRelativeToKind;
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

pub(super) enum TemporalDurationStringMode {
    ToString,
    ToJson,
}

/// Completed input Duration fields. Only actual ToTemporalDuration or
/// receiver brand recovery below produces this owned proof.
#[must_use]
pub(super) struct CompletedTemporalDurationRecordLocals {
    fields: TemporalDurationFields,
}

impl CompletedTemporalDurationRecordLocals {
    pub(super) fn fields(&self) -> &TemporalDurationFields {
        &self.fields
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder.release_temporal_duration_field_locals(self.fields, function);
    }
}

/// Borrowed view of the existing round option locals after their complete
/// validation and default-largest selection. No other owner can construct it.
pub(super) struct CompletedTemporalDurationRoundOptionsLocals {
    largest_unit: I64Local,
    smallest_unit: I64Local,
    rounding_increment: I64Local,
    rounding_mode: I64Local,
}

impl CompletedTemporalDurationRoundOptionsLocals {
    pub(super) fn largest_unit(&self) -> I64Local {
        self.largest_unit
    }
    pub(super) fn smallest_unit(&self) -> I64Local {
        self.smallest_unit
    }
    pub(super) fn rounding_increment(&self) -> I64Local {
        self.rounding_increment
    }
    pub(super) fn rounding_mode(&self) -> I64Local {
        self.rounding_mode
    }
}

/// The required total unit has passed the complete Year..Nanosecond check.
pub(super) struct TemporalDurationTotalUnitLocals {
    unit: I64Local,
}
impl TemporalDurationTotalUnitLocals {
    pub(super) fn local(&self) -> I64Local {
        self.unit
    }
}

impl FunctionBuilder<'_> {
    fn emit_temporal_completed_duration_record_from_input(
        &mut self,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<CompletedTemporalDurationRecordLocals, EmitError> {
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_to_temporal_duration(input, &fields, function)?;
        Ok(CompletedTemporalDurationRecordLocals { fields })
    }

    fn emit_temporal_completed_duration_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<CompletedTemporalDurationRecordLocals, EmitError> {
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_duration_fields_from_receiver(&fields, function)?;
        Ok(CompletedTemporalDurationRecordLocals { fields })
    }
}

enum TemporalDurationArithmeticOperation {
    Add,
    Subtract,
}

impl<'a> FunctionBuilder<'a> {
    /// GetOptionsObject preserves absent options and observes no properties.
    pub(crate) fn emit_temporal_duration_options_object(
        &mut self,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        options.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.emit_is_heap_object_like_tag_i32(options.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Get is skipped only when the options bag itself is absent.
    pub(crate) fn emit_temporal_duration_option_get(
        &mut self,
        options: &ValueLocals,
        name: &str,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        output.set_undefined(function);
        options.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(name, function)?,
            function,
        );
        let key = crate::operations::PropertyKeyLocals::from_string(schema, &name, function);
        let pending = schema.reserve_completion(function);
        self.emit_object_read(options, options, &key, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        output.copy_from(pending.value(), function);
        pending.clear(function);
        key.clear(function);
        name.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_temporal_string_matches(
        &mut self,
        value: &GcLocal<StringValue>,
        literal: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let other = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(literal, function)?,
            function,
        );
        self.emit_string_payload_equality_i32(value, &other, function);
        other.clear(function);
        Ok(())
    }

    /// Unknown spellings reject at read time; recognized units/auto/absence
    /// remain available for the caller's later allowed-range validation.
    pub(crate) fn emit_temporal_duration_unit_option(
        &mut self,
        options: &ValueLocals,
        property: TemporalUnitOptionProperty,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_temporal_duration_option_get(options, property.name(), &value, function)?;
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        output.store(function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
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
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        output.store(function);
        self.emit_temporal_string_matches(&string, "auto", function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        output.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for unit in TemporalUnit::ALL {
            for spelling in [unit.singular(), unit.plural()] {
                self.emit_temporal_string_matches(&string, spelling, function)?;
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unit(unit).code()));
                output.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        output.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Invalid.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        value.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_rounding_mode_option(
        &mut self,
        options: &ValueLocals,
        default_mode: TemporalRoundingMode,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_temporal_duration_option_get(options, "roundingMode", &value, function)?;
        function.instruction(&Instruction::I64Const(default_mode.code()));
        output.store(function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
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
        function.instruction(&Instruction::I64Const(-1));
        output.store(function);
        for mode in TemporalRoundingMode::ALL {
            self.emit_temporal_string_matches(&string, mode.name(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(mode.code()));
            output.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        output.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_ROUNDING_MODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        value.clear(function);
        Ok(())
    }

    /// GetRoundingIncrementOption accepts truncations through exactly 10^9.
    pub(crate) fn emit_temporal_duration_rounding_increment_option(
        &mut self,
        options: &ValueLocals,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_temporal_duration_option_get(options, "roundingIncrement", &value, function)?;
        function.instruction(&Instruction::I64Const(1));
        output.store(function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_number_payload(&value, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        let number = pending.value().scalar();
        number.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Lt);
        number.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000_000.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        number.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        number.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_ROUNDING_INCREMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        number.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        output.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        value.clear(function);
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
        unit_local: I64Local,
        low: TemporalUnit,
        high: TemporalUnit,
        range_error: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert!(
            low == high || low.is_larger_than(high),
            "unit range bounds are written largest-first",
        );
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(low.code()));
        function.instruction(&Instruction::I64LtS);
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(high.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            range_error,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `largestUnit` must not name a unit smaller than `smallestUnit`. Both
    /// locals hold unit codes, where a *smaller* code is a *larger* unit, so
    /// the test reads backwards; it was hand-emitted at four sites, each with
    /// its own copy of the message.
    pub(crate) fn emit_temporal_require_largest_not_smaller(
        &mut self,
        largest_unit_local: I64Local,
        smallest_unit_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (largest_unit_local).load(function);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::SMALLESTUNIT_MUST_BE_SMALLER_THAN_LARGESTUNIT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Observable alphabetical property sweep, retaining exact Number bits.
    pub(crate) fn emit_temporal_duration_partial_record(
        &mut self,
        argument: &ValueLocals,
        fields: &TemporalDurationFields,
        present: &[I64Local; 10],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        for index in 0..10 {
            function.instruction(&Instruction::I64Const(0));
            present[index].store(function);
            function.instruction(&Instruction::I64Const(0));
            fields.number_bits_locals()[index].store(function);
        }
        for (name, index) in TEMPORAL_DURATION_ALPHABETICAL_FIELDS {
            self.emit_temporal_duration_option_get(argument, name, &value, function)?;
            value.tag().load(function);
            function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            present[index].store(function);
            self.emit_temporal_duration_field_to_number(
                &value,
                fields.number_bits_locals()[index],
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        value.clear(function);
        Ok(())
    }

    /// Abstract conversion observes private slots, String grammar, or a full
    /// property bag. It never calls the mutable public Duration.from property.
    pub(crate) fn emit_to_temporal_duration(
        &mut self,
        input: &ValueLocals,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.emit_temporal_duration_brand_check_i32(input, function);
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            input.cast_reference::<TemporalDurationObject>(schema, function),
            function,
        );
        self.emit_temporal_duration_load_record(&record, fields, function);
        record.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        let present: [I64Local; 10] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let any = schema.reserve_i64_local(function);
        self.emit_temporal_duration_partial_record(input, fields, &present, function)?;
        function.instruction(&Instruction::I64Const(0));
        any.store(function);
        for local in present {
            any.load(function);
            local.load(function);
            function.instruction(&Instruction::I64Or);
            any.store(function);
        }
        any.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_REQUIRES_AT_LEAST_ONE_DURATION_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_reject_invalid(fields, function)?;
        schema.release_i64_local(any, function);
        for local in present.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        function.instruction(&Instruction::Else);
        input.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_DURATION, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            input.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_duration_parse_string(&string, fields, function)?;
        self.emit_temporal_duration_reject_invalid(fields, function)?;
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_temporal_duration(&argument, &fields, function)?;
        self.emit_alloc_temporal_duration(&fields, TemporalPrototypeSource::Intrinsic, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        argument.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_duration_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_duration_field_locals(function);
        let partial = self.reserve_temporal_duration_field_locals(function);
        let present: [I64Local; 10] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let any = schema.reserve_i64_local(function);
        self.emit_temporal_duration_fields_from_receiver(&fields, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_brand_check_i32(&argument, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_DURATION, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_partial_record(&argument, &partial, &present, function)?;
        function.instruction(&Instruction::I64Const(0));
        any.store(function);
        for index in 0..10 {
            any.load(function);
            present[index].load(function);
            function.instruction(&Instruction::I64Or);
            any.store(function);
            present[index].load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            partial.number_bits_locals()[index].load(function);
            fields.number_bits_locals()[index].store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        any.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_REQUIRES_AT_LEAST_ONE_DURATION_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_create_temporal_duration(&fields, function)?;
        schema.release_i64_local(any, function);
        for local in present.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        self.release_temporal_duration_field_locals(partial, function);
        self.release_temporal_duration_field_locals(fields, function);
        argument.clear(function);
        Ok(())
    }

    /// `DefaultTemporalLargestUnit`: the index of the first non-zero field, or
    /// nanosecond when the duration is blank.
    pub(crate) fn emit_temporal_duration_default_largest_unit(
        &mut self,
        field_locals: &TemporalDurationFields,
        output_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        (output_local).store(function);
        // Smallest unit first, so the last write wins and leaves the largest
        // non-zero field. The array subscript and the emitted code are the two
        // separate numberings, named as such: this loop used to use one `index`
        // for both.
        for unit in TemporalUnit::ALL.into_iter().rev() {
            (field_locals.number_bits(unit)).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(unit.code()));
            (output_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
    }

    /// Throw a RangeError when any calendar unit is non-zero, which is exactly
    /// when the operation would need a `relativeTo` this backend cannot
    /// resolve.
    pub(crate) fn emit_temporal_duration_reject_calendar_units(
        &mut self,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mut calendar_units = TemporalUnit::ALL
            .into_iter()
            .filter(|unit| unit.is_calendar_unit());
        let first = calendar_units.next().expect("year is a calendar unit");
        (field_locals.number_bits(first)).load(function);
        for unit in calendar_units {
            (field_locals.number_bits(unit)).load(function);
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPERATION_REQUIRES_RELATIVETO_FOR_CALENDAR_UNITS,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `TemporalDurationFromInternal`: spread a signed (seconds, subsecond)
    /// pair back over the unit fields, stopping at `largest_unit_local`.
    pub(crate) fn emit_temporal_duration_balance(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        largest_unit_local: I64Local,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let negative = self.runtime_schema().reserve_i64_local(function);
        let magnitude = self.runtime_schema().reserve_i64_local(function);
        let remaining_seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        let component = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_duration_zero_fields(fields, function);
        // `DifferenceISODateTime` can hand over a mixed-sign pair (whole
        // seconds one way, a subsecond rest the other). Fold the subsecond
        // carry first, then borrow one second across the sign boundary, so
        // the magnitude split below sees a consistent pair. The single
        // borrow suffices because the carry fold leaves `|subsecond| < 1e9`.
        (seconds_local).load(function);
        (magnitude).store(function);
        (subsecond_local).load(function);
        (subsecond).store(function);
        (subsecond).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        (component).store(function);
        (magnitude).load(function);
        (component).load(function);
        function.instruction(&Instruction::I64Add);
        (magnitude).store(function);
        (subsecond).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond).store(function);
        for (seconds_sign, subsecond_sign, adjust) in [(true, true, 1), (false, false, -1)] {
            (magnitude).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            if !seconds_sign {
                function.instruction(&Instruction::I32Eqz);
                (magnitude).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
            }
            (subsecond).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            if !subsecond_sign {
                function.instruction(&Instruction::I32Eqz);
                (subsecond).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            (magnitude).load(function);
            function.instruction(&Instruction::I64Const(adjust));
            function.instruction(&Instruction::I64Add);
            (magnitude).store(function);
            (subsecond).load(function);
            function.instruction(&Instruction::I64Const(1_000_000_000 * adjust));
            function.instruction(&Instruction::I64Sub);
            (subsecond).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (magnitude).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (negative).store(function);
        for local in [magnitude, subsecond] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            (local).load(function);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            (local).load(function);
            function.instruction(&Instruction::End);
            (local).store(function);
        }
        for (index, (unit, _)) in TEMPORAL_UNIT_SECONDS.iter().enumerate() {
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            if *unit == TemporalUnit::Day {
                (largest_unit_local).load(function);
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I32Or);
            }
            self.open_frame(ControlFrameKind::If, function);
            (magnitude).load(function);
            (remaining_seconds).store(function);
            for (slot, divisor) in TEMPORAL_UNIT_SECONDS.iter().skip(index) {
                (remaining_seconds).load(function);
                function.instruction(&Instruction::I64Const(*divisor));
                function.instruction(&Instruction::I64DivU);
                (component).store(function);
                self.emit_temporal_duration_set_integer_field(fields, *slot, component, function);
                (remaining_seconds).load(function);
                function.instruction(&Instruction::I64Const(*divisor));
                function.instruction(&Instruction::I64RemU);
                (remaining_seconds).store(function);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for unit in TemporalDurationSubsecondUnit::ALL {
            let divisor = unit.nanoseconds();
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.temporal_unit().code()));
            function.instruction(&Instruction::I64LeS);
            self.open_frame(ControlFrameKind::If, function);
            (subsecond).load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            (component).store(function);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.temporal_unit().code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_duration_scaled_time_number(
                magnitude,
                component,
                TemporalDurationNumberProjection::BalancedField(unit),
                fields.number_bits(unit.temporal_unit()),
                function,
            );
            function.instruction(&Instruction::Else);
            self.emit_temporal_duration_set_integer_field(
                fields,
                unit.temporal_unit(),
                component,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            (subsecond).load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemU);
            (subsecond).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (negative).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_negate_fields(fields, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [component, subsecond, remaining_seconds, magnitude, negative] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// Temporal proposal 7.3.18/7.3.19: `add` and `subtract`. Both refuse
    /// calendar units, because balancing years or months needs a reference
    /// point.
    pub(crate) fn emit_temporal_duration_add(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_arithmetic(TemporalDurationArithmeticOperation::Add, function)
    }

    pub(crate) fn emit_temporal_duration_subtract(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_duration_arithmetic(
            TemporalDurationArithmeticOperation::Subtract,
            function,
        )
    }

    fn emit_temporal_duration_arithmetic(
        &mut self,
        operation: TemporalDurationArithmeticOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let largest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let other_largest_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let other_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let other_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let field_locals = self.reserve_temporal_duration_field_locals(function);
        let other_locals = self.reserve_temporal_duration_field_locals(function);

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_temporal_duration(&argument, &other_locals, function)?;
        match operation {
            TemporalDurationArithmeticOperation::Add => {}
            TemporalDurationArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&other_locals, function);
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
        (other_largest_local).load(function);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (other_largest_local).load(function);
        (largest_unit_local).store(function);
        self.pop_control(ControlFrameKind::If);
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
        (seconds_local).load(function);
        (other_seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        (other_subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (subsecond_local).store(function);
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_unit_local,
            &field_locals,
            function,
        )?;
        self.emit_create_temporal_duration(&field_locals, function)?;

        self.release_temporal_duration_field_locals(other_locals, function);
        self.release_temporal_duration_field_locals(field_locals, function);
        for local in [
            other_subsecond_local,
            other_seconds_local,
            subsecond_local,
            seconds_local,
            other_largest_local,
            largest_unit_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        argument.clear(function);
        Ok(())
    }

    /// Re-establish the invariant that the sub-second remainder is in
    /// (-10^9, 10^9) and shares the sign of the whole-second count.
    pub(crate) fn emit_temporal_duration_renormalize(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) {
        (seconds_local).load(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond_local).store(function);
        // Opposite signs: borrow a second so both parts agree.
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Add);
        (subsecond_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        (subsecond_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Temporal proposal 7.2.3: `Temporal.Duration.compare(one, two, options)`.
    pub(crate) fn emit_temporal_duration_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let options = self.runtime_schema().reserve_value_local(function);
        let relative_value = self.runtime_schema().reserve_value_local(function);
        let one_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let one_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let two_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let two_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let result_local = self.runtime_schema().reserve_i64_local(function);
        let one_largest_local = self.runtime_schema().reserve_i64_local(function);
        let two_largest_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_builtin_arg_to_value(0, &argument, function);
        let one_locals =
            self.emit_temporal_completed_duration_record_from_input(&argument, function)?;
        self.emit_builtin_arg_to_value(1, &argument, function);
        let two_locals =
            self.emit_temporal_completed_duration_record_from_input(&argument, function)?;
        let mut relative = self.emit_temporal_absent_relative_to(function);
        self.emit_builtin_arg_to_value(2, &options, function);
        self.emit_temporal_duration_options_object(&options, function)?;
        self.emit_temporal_duration_option_get(&options, "relativeTo", &relative_value, function)?;
        self.emit_temporal_duration_relative_to_option_into(
            &mut relative,
            &relative_value,
            function,
        )?;

        // Field-for-field equality short-circuits before any range question.
        function.instruction(&Instruction::I64Const(1));
        (result_local).store(function);
        for index in 0..10 {
            (one_locals.fields().number_bits_locals()[index]).load(function);
            (two_locals.fields().number_bits_locals()[index]).load(function);
            function.instruction(&Instruction::I64Ne);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(0));
            (result_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (result_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Calendar units compare through the `relativeTo` date; day and time
        // units compare as 24-hour days without one. A zoned `relativeTo`
        // with a date-category operand additionally validates both
        // `AddZonedDateTime` targets, so those route relatively too.
        self.emit_temporal_duration_default_largest_unit(
            one_locals.fields(),
            one_largest_local,
            function,
        );
        self.emit_temporal_duration_default_largest_unit(
            two_locals.fields(),
            two_largest_local,
            function,
        );
        (one_largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        (two_largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        (one_largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::I64LtS);
        (two_largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPERATION_REQUIRES_RELATIVETO_FOR_CALENDAR_UNITS,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        relative.emit_zoned_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_compare_zoned_relative(
                &one_locals,
                &two_locals,
                context,
                result_local,
                function,
            )
        })?;
        relative.emit_plain_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_compare_plain_relative(
                one_locals.fields(),
                two_locals.fields(),
                context,
                result_local,
                function,
            )
        })?;
        (result_local).load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_normalize_seconds(
            one_locals.fields(),
            TemporalUnit::Day,
            one_seconds_local,
            one_subsecond_local,
            function,
        );
        self.emit_temporal_duration_normalize_seconds(
            two_locals.fields(),
            TemporalUnit::Day,
            two_seconds_local,
            two_subsecond_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        (result_local).store(function);
        (one_seconds_local).load(function);
        (two_seconds_local).load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (one_subsecond_local).load(function);
        (two_subsecond_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (result_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (one_subsecond_local).load(function);
        (two_subsecond_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (result_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (one_seconds_local).load(function);
        (two_seconds_local).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (result_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (result_local).load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);

        relative.release(self, function);
        two_locals.release(self, function);
        one_locals.release(self, function);
        for local in [
            two_largest_local,
            one_largest_local,
            result_local,
            two_subsecond_local,
            two_seconds_local,
            one_subsecond_local,
            one_seconds_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        relative_value.clear(function);
        options.clear(function);
        argument.clear(function);
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
        remainder_local: I64Local,
        increment_local: I64Local,
        quotient_local: I64Local,
        sign_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) {
        let decision_local = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (decision_local).store(function);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // ceil
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Ceil.code()));
        function.instruction(&Instruction::I64Eq);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        // floor
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Floor.code()));
        function.instruction(&Instruction::I64Eq);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        // expand
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Expand.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (decision_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The half-* family: compare 2 x remainder against the increment. The
        // family is contiguous at the top of the code range, which the `const`
        // assertion in `temporal_options` pins, so one `>=` covers all five.
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfCeil.code(),
        ));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (increment_local).load(function);
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (decision_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        // halfCeil / halfFloor follow their unrounded siblings; halfExpand
        // always expands; halfTrunc never does; halfEven breaks the tie on the
        // parity of the truncated quotient.
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfCeil.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfFloor.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        (mode_local).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfEven.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        (quotient_local).load(function);
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
        (decision_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (decision_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.runtime_schema()
            .release_i64_local(decision_local, function);
    }

    /// Reconcile a signed (seconds, subsecond) pair in place: fold the
    /// subsecond carry, then borrow one second across a sign boundary, so
    /// the magnitude split sees the true total. `DifferenceISODateTime`
    /// hands mixed-sign pairs to the relative nudge (whole seconds one
    /// way, a subsecond rest the other); rounding the legs independently
    /// would shift the result by a whole unit. Consistent pairs pass
    /// through untouched. (`emit_temporal_duration_balance` inlines the
    /// same sequence.)
    pub(super) fn emit_temporal_duration_reconcile_pair(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) {
        let carry_local = self.runtime_schema().reserve_i64_local(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        (carry_local).store(function);
        (seconds_local).load(function);
        (carry_local).load(function);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond_local).store(function);
        for (seconds_negative, subsecond_positive, adjust) in [(true, true, 1), (false, false, -1)]
        {
            (seconds_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            if !seconds_negative {
                function.instruction(&Instruction::I32Eqz);
                (seconds_local).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
            }
            (subsecond_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            if !subsecond_positive {
                function.instruction(&Instruction::I32Eqz);
                (subsecond_local).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            (seconds_local).load(function);
            function.instruction(&Instruction::I64Const(adjust));
            function.instruction(&Instruction::I64Add);
            (seconds_local).store(function);
            (subsecond_local).load(function);
            function.instruction(&Instruction::I64Const(1_000_000_000 * adjust));
            function.instruction(&Instruction::I64Sub);
            (subsecond_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.runtime_schema()
            .release_i64_local(carry_local, function);
    }

    /// Round a signed (seconds, subsecond) pair to a multiple of
    /// `quantum_local` nanoseconds. `quantum_local` is at most 10^9 so the
    /// arithmetic never leaves the sub-second slot except through the carry.
    pub(super) fn emit_temporal_duration_round_subsecond(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        quantum_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) {
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let magnitude_local = self.runtime_schema().reserve_i64_local(function);
        let sub_local = self.runtime_schema().reserve_i64_local(function);
        let remainder_local = self.runtime_schema().reserve_i64_local(function);
        let quotient_local = self.runtime_schema().reserve_i64_local(function);
        let parity_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_temporal_duration_reconcile_pair(seconds_local, subsecond_local, function);
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (sign_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, sub_local),
        ] {
            (source).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            (source).load(function);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            (source).load(function);
            function.instruction(&Instruction::End);
            (destination).store(function);
        }
        (sub_local).load(function);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64RemU);
        (remainder_local).store(function);
        (sub_local).load(function);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64DivU);
        (quotient_local).store(function);
        // Half-even uses the quotient of the whole duration, including seconds.
        // Only its low bit is needed, so multiplication cannot lose parity.
        (magnitude_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        (quantum_local).load(function);
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Mul);
        (quotient_local).load(function);
        function.instruction(&Instruction::I64Add);
        (parity_local).store(function);
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            quantum_local,
            parity_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (quotient_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        (quotient_local).load(function);
        function.instruction(&Instruction::End);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (sub_local).store(function);
        (sub_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        (sub_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        (sub_local).store(function);
        (magnitude_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (magnitude_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (magnitude_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (seconds_local).store(function);
        (sub_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (subsecond_local).store(function);

        self.runtime_schema()
            .release_i64_local(parity_local, function);
        self.runtime_schema()
            .release_i64_local(quotient_local, function);
        self.runtime_schema()
            .release_i64_local(remainder_local, function);
        self.runtime_schema().release_i64_local(sub_local, function);
        self.runtime_schema()
            .release_i64_local(magnitude_local, function);
        self.runtime_schema()
            .release_i64_local(sign_local, function);
    }

    /// Round a signed (seconds, subsecond) pair to a whole number of
    /// `unit_seconds x increment` seconds.
    pub(super) fn emit_temporal_duration_round_seconds(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        quantum_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) {
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let magnitude_local = self.runtime_schema().reserve_i64_local(function);
        let sub_local = self.runtime_schema().reserve_i64_local(function);
        let remainder_local = self.runtime_schema().reserve_i64_local(function);
        let quotient_local = self.runtime_schema().reserve_i64_local(function);
        let scaled_quantum_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_temporal_duration_reconcile_pair(seconds_local, subsecond_local, function);
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (sign_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (source, destination) in [
            (seconds_local, magnitude_local),
            (subsecond_local, sub_local),
        ] {
            (source).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            (source).load(function);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            (source).load(function);
            function.instruction(&Instruction::End);
            (destination).store(function);
        }
        (magnitude_local).load(function);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64DivU);
        (quotient_local).store(function);
        // Encode the exact midpoint comparison in quarters. Whole seconds
        // may span the full date range; only the subsecond remainder is scaled.
        (magnitude_local).load(function);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (scaled_quantum_local).store(function);
        (sub_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        (remainder_local).store(function);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        (remainder_local).store(function);
        (scaled_quantum_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (scaled_quantum_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (scaled_quantum_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        (scaled_quantum_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (remainder_local).store(function);
        (magnitude_local).load(function);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        (sub_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (remainder_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(4));
        (scaled_quantum_local).store(function);
        self.emit_temporal_duration_round_up_i32(
            remainder_local,
            scaled_quantum_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (quotient_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        (quotient_local).load(function);
        function.instruction(&Instruction::End);
        (quantum_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (seconds_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (subsecond_local).store(function);

        self.runtime_schema()
            .release_i64_local(scaled_quantum_local, function);
        self.runtime_schema()
            .release_i64_local(quotient_local, function);
        self.runtime_schema()
            .release_i64_local(remainder_local, function);
        self.runtime_schema().release_i64_local(sub_local, function);
        self.runtime_schema()
            .release_i64_local(magnitude_local, function);
        self.runtime_schema()
            .release_i64_local(sign_local, function);
    }

    /// Temporal proposal 7.3.20: `Temporal.Duration.prototype.round`.
    pub(crate) fn emit_temporal_duration_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let options = self.runtime_schema().reserve_value_local(function);
        let relative_value = self.runtime_schema().reserve_value_local(function);
        let smallest_local = self.runtime_schema().reserve_i64_local(function);
        let largest_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);
        let mode_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let default_largest_local = self.runtime_schema().reserve_i64_local(function);
        let maximum_local = self.runtime_schema().reserve_i64_local(function);

        let field_locals = self.emit_temporal_completed_duration_record_from_receiver(function)?;
        let mut relative = self.emit_temporal_absent_relative_to(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A string argument is shorthand for `{ smallestUnit: <string> }`.
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        (largest_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (increment_local).store(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        (mode_local).store(function);
        relative_value.set_undefined(function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_duration_unit_string_to_code(&string, smallest_local, function)?;
        string.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&argument, function)?;
        options.copy_from(&argument, function);
        self.emit_temporal_duration_unit_option(
            &options,
            TemporalUnitOptionProperty::LargestUnit,
            largest_local,
            function,
        )?;
        self.emit_temporal_duration_option_get(&options, "relativeTo", &relative_value, function)?;
        self.emit_temporal_duration_relative_to_option_into(
            &mut relative,
            &relative_value,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(&options, increment_local, function)?;
        self.emit_temporal_duration_rounding_mode_option(
            &options,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &options,
            TemporalUnitOptionProperty::SmallestUnit,
            smallest_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::TEMPORAL_DURATION_PROTOTYPE_ROUND_REQUIRES_LARGESTUNIT_SMALLESTUNIT_OR_BOTH, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_require_unit_range(
            smallest_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        (smallest_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_require_unit_range(
            largest_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // `auto` and an absent largestUnit both mean "the duration's own
        // largest unit, but never smaller than smallestUnit".
        self.emit_temporal_duration_default_largest_unit(
            field_locals.fields(),
            default_largest_local,
            function,
        );
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (default_largest_local).load(function);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (default_largest_local).load(function);
        function.instruction(&Instruction::Else);
        (smallest_local).load(function);
        function.instruction(&Instruction::End);
        (largest_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_largest_not_smaller(largest_local, smallest_local, function)?;

        // `ValidateTemporalRoundingIncrement`: the increment must be strictly
        // below the unit's maximum and must divide it exactly. Year through
        // day have no maximum at all.
        function.instruction(&Instruction::I64Const(0));
        (maximum_local).store(function);
        for unit in TemporalTimeUnit::ALL {
            (smallest_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(unit.maximum_rounding_increment()));
            (maximum_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (maximum_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (increment_local).load(function);
        (maximum_local).load(function);
        function.instruction(&Instruction::I64GeS);
        (maximum_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_ROUNDING_INCREMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // An increment above 1 cannot round a date unit while balancing to
        // a larger one.
        (increment_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtS);
        (largest_local).load(function);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::TEMPORAL_DURATION_PROTOTYPE_ROUND_CANNOT_BALANCE_DATE_UNITS_WITH_A_ROUNDING_INCREMENT_ABOVE_1, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let round_options = CompletedTemporalDurationRoundOptionsLocals {
            largest_unit: largest_local,
            smallest_unit: smallest_local,
            rounding_increment: increment_local,
            rounding_mode: mode_local,
        };

        // Without `relativeTo`, calendar units in the duration or the options
        // cannot resolve. A present `relativeTo` takes the calendar-aware
        // branch below instead.
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_reject_calendar_units(field_locals.fields(), function)?;
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        (largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPERATION_REQUIRES_RELATIVETO_FOR_CALENDAR_UNITS,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A present `relativeTo` runs the calendar-aware path:
        // `DifferencePlainDateTimeWithRounding` between the relativeTo
        // instant and the duration's target, then `TemporalDurationFromInternal`.
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        relative.emit_zoned_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_round_zoned_relative(
                &field_locals,
                context,
                &round_options,
                function,
            )
        })?;
        relative.emit_plain_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_round_plain_relative(
                field_locals.fields(),
                context,
                &round_options,
                function,
            )
        })?;
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_normalize_seconds(
            field_locals.fields(),
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
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
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
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_local,
            field_locals.fields(),
            function,
        )?;
        self.emit_create_temporal_duration(field_locals.fields(), function)?;

        relative.release(self, function);
        field_locals.release(self, function);
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
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        relative_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// The rounding quantum for a unit code and increment: seconds for the
    /// day-through-second codes, nanoseconds below that.
    pub(super) fn emit_temporal_duration_unit_quantum(
        &mut self,
        unit_local: I64Local,
        increment_local: I64Local,
        output_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(1));
        (output_local).store(function);
        for (unit, scale) in [
            (3_i64, 86_400_i64),
            (4, 3_600),
            (5, 60),
            (6, 1),
            (7, 1_000_000),
            (8, 1_000),
            (9, 1),
        ] {
            (unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(scale));
            (output_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (output_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (output_local).store(function);
    }

    /// Map a unit string to its code, throwing a RangeError when it names no
    /// unit. Shared by the string shorthand of `round` and `total`.
    fn emit_temporal_duration_unit_string_to_code(
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

    /// Temporal proposal 7.3.21: `Temporal.Duration.prototype.total`.
    pub(crate) fn emit_temporal_duration_total(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument = self.runtime_schema().reserve_value_local(function);
        let relative_value = self.runtime_schema().reserve_value_local(function);
        let unit_local = self.runtime_schema().reserve_i64_local(function);
        let scale_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let quotient_local = self.runtime_schema().reserve_i64_local(function);
        let remainder_local = self.runtime_schema().reserve_i64_local(function);

        let field_locals = self.emit_temporal_completed_duration_record_from_receiver(function)?;
        let mut relative = self.emit_temporal_absent_relative_to(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_duration_unit_string_to_code(&string, unit_local, function)?;
        string.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&argument, function)?;
        self.emit_temporal_duration_option_get(&argument, "relativeTo", &relative_value, function)?;
        self.emit_temporal_duration_relative_to_option_into(
            &mut relative,
            &relative_value,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &argument,
            TemporalUnitOptionProperty::Unit,
            unit_local,
            function,
        )?;
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_PROTOTYPE_TOTAL_REQUIRES_A_UNIT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
            function,
        )?;
        let total_unit = TemporalDurationTotalUnitLocals { unit: unit_local };

        // Without `relativeTo`, calendar units in the duration or the unit
        // option cannot resolve.
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_reject_calendar_units(field_locals.fields(), function)?;
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_DURATION_OPERATION_REQUIRES_RELATIVETO_FOR_CALENDAR_UNITS,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A present `relativeTo` totals through
        // `DifferencePlainDateTimeWithTotal`.
        (relative.kind_local()).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        relative.emit_zoned_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_total_zoned_relative(
                &field_locals,
                context,
                &total_unit,
                function,
            )
        })?;
        relative.emit_plain_branch(self, function, |builder, function, context| {
            builder.emit_temporal_duration_total_plain_relative(
                field_locals.fields(),
                context,
                &total_unit,
                function,
            )
        })?;
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_normalize_seconds(
            field_locals.fields(),
            TemporalUnit::Day,
            seconds_local,
            subsecond_local,
            function,
        );
        // Split before converting so the quotient keeps full precision even
        // when the second count is close to 2^53.
        function.instruction(&Instruction::I64Const(1));
        (scale_local).store(function);
        for (unit, scale) in TEMPORAL_UNIT_SECONDS {
            (unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(scale));
            (scale_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        // `DivideNormalizedTimeDuration` divides the EXACT total by the
        // unit with a single rounding: a quotient-plus-fraction sum
        // double-rounds (`precision-exact-mathematical-values-6`). The
        // total needs 128 bits (|seconds| < 2^53 times 10^9 overflows
        // u64), so the multiply below splits into 32-bit limbs; 10^9
        // fits 30 bits, which keeps every partial product in range.
        let total_negative_local = self.runtime_schema().reserve_i64_local(function);
        let total_limb0_local = self.runtime_schema().reserve_i64_local(function);
        let total_limb1_local = self.runtime_schema().reserve_i64_local(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (total_negative_local).store(function);
        for local in [seconds_local, subsecond_local] {
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
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (total_limb0_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (total_limb1_local).store(function);
        (total_limb0_local).load(function);
        (total_limb1_local).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        (remainder_local).store(function);
        (total_limb1_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        (remainder_local).load(function);
        (total_limb0_local).load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        (quotient_local).store(function);
        (remainder_local).load(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (remainder_local).load(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (remainder_local).load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        (quotient_local).load(function);
        function.instruction(&Instruction::I64Add);
        (quotient_local).store(function);
        (remainder_local).store(function);
        (scale_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (scale_local).store(function);
        self.emit_u128_div_to_f64(
            quotient_local,
            remainder_local,
            scale_local,
            seconds_local,
            function,
        );
        (total_negative_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(-0x7fff_ffff_ffff_ffff - 1));
        function.instruction(&Instruction::I64Xor);
        (seconds_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(total_limb1_local, function);
        self.runtime_schema()
            .release_i64_local(total_limb0_local, function);
        self.runtime_schema()
            .release_i64_local(total_negative_local, function);
        (seconds_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::Else);
        // Subsecond totals need a single rounding of the exact rational.
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (quotient_local).store(function);
        for local in [seconds_local, subsecond_local] {
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
        function.instruction(&Instruction::I64Const(0));
        (remainder_local).store(function);
        for unit in TemporalDurationSubsecondUnit::ALL {
            (unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.temporal_unit().code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_duration_scaled_time_number(
                seconds_local,
                subsecond_local,
                TemporalDurationNumberProjection::Total(unit),
                remainder_local,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (quotient_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        (remainder_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::Else);
        (remainder_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);

        relative.release(self, function);
        field_locals.release(self, function);
        for local in [
            remainder_local,
            quotient_local,
            subsecond_local,
            seconds_local,
            scale_local,
            unit_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        relative_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 7.3.22/7.3.23: ISO `toString` and `toJSON`.
    /// Only `toString` reads ISO precision options; `toJSON` uses `auto`.
    /// Localized formatting has its own compiled DurationFormat initializer.
    pub(crate) fn emit_temporal_duration_to_string(
        &mut self,
        mode: TemporalDurationStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reads_options = match mode {
            TemporalDurationStringMode::ToString => true,
            TemporalDurationStringMode::ToJson => false,
        };
        let schema = self.runtime_schema();
        let options = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let digits_local = self.runtime_schema().reserve_i64_local(function);
        let smallest_local = self.runtime_schema().reserve_i64_local(function);
        let mode_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);
        let seconds_local = self.runtime_schema().reserve_i64_local(function);
        let subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let hours_local = self.runtime_schema().reserve_i64_local(function);
        let minutes_local = self.runtime_schema().reserve_i64_local(function);
        let default_largest_local = self.runtime_schema().reserve_i64_local(function);
        let rounded_largest_local = self.runtime_schema().reserve_i64_local(function);
        let internal_date_locals = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let output = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let time = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let piece = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let time_length = schema.reserve_i64_local(function);
        let number_payload_local = self.runtime_schema().reserve_i64_local(function);
        let field_locals = self.reserve_temporal_duration_field_locals(function);

        self.emit_temporal_duration_fields_from_receiver(&field_locals, function)?;
        function.instruction(&Instruction::I64Const(-1));
        (digits_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        (smallest_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        (mode_local).store(function);
        if reads_options {
            self.emit_builtin_arg_to_value(0, &options, function);
            self.emit_temporal_duration_options_object(&options, function)?;
            self.emit_temporal_duration_option_get(
                &options,
                "fractionalSecondDigits",
                &value,
                function,
            )?;
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
            string.clear(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
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
                RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            value.scalar().load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Floor);
            function.instruction(&Instruction::I64TruncSatF64S);
            digits_local.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.emit_temporal_duration_rounding_mode_option(
                &options,
                TemporalRoundingMode::Trunc,
                mode_local,
                function,
            )?;
            self.emit_temporal_duration_unit_option(
                &options,
                TemporalUnitOptionProperty::SmallestUnit,
                smallest_local,
                function,
            )?;
            (smallest_local).load(function);
            function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
            function.instruction(&Instruction::I64Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_require_unit_range(
                smallest_local,
                TemporalUnit::Second,
                TemporalUnit::Nanosecond,
                RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_UNIT_OPTION,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // `ToSecondsStringPrecisionRecord`: smallestUnit wins, otherwise the
        // digit count picks both the printed width and the rounding quantum.
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        (digits_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        (increment_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (quantum_local).store(function);
        (digits_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        for digits in 0..=9_i64 {
            (digits_local).load(function);
            function.instruction(&Instruction::I64Const(digits));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(10_i64.pow((9 - digits) as u32)));
            (quantum_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_sign(&field_locals, sign_local, function);
        // Without rounding the components print verbatim; with rounding the
        // whole time part is rebalanced, because a carry out of the seconds
        // has to reach the minutes and hours the way `TemporalDurationFromInternal`
        // would.
        (field_locals.number_bits_locals()[4]).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        (hours_local).store(function);
        (field_locals.number_bits_locals()[5]).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        (minutes_local).store(function);
        self.emit_temporal_duration_normalize_seconds(
            &field_locals,
            TemporalUnit::Second,
            seconds_local,
            subsecond_local,
            function,
        );
        (quantum_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (hours_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (minutes_local).store(function);
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
        // `TemporalDurationFromInternal` with
        // `LargerOfTwoTemporalUnits(defaultLargestUnit, second)`: the
        // rounded time balances up through days (never further) and the
        // result validates, so an out-of-range rounding throws.
        self.emit_temporal_duration_default_largest_unit(
            &field_locals,
            default_largest_local,
            function,
        );
        (default_largest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (default_largest_local).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::End);
        (rounded_largest_local).store(function);
        for (unit, destination) in [
            (TemporalUnit::Year, internal_date_locals[0]),
            (TemporalUnit::Month, internal_date_locals[1]),
            (TemporalUnit::Week, internal_date_locals[2]),
            (TemporalUnit::Day, internal_date_locals[3]),
        ] {
            (field_locals.number_bits(unit)).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            (destination).store(function);
        }
        self.emit_temporal_relative_balance_internal(
            internal_date_locals,
            seconds_local,
            subsecond_local,
            rounded_largest_local,
            &field_locals,
            function,
        )?;
        self.emit_temporal_duration_reject_invalid(&field_locals, function)?;
        // The print below reads the balanced time back out of the fields.
        for (unit, destination) in [
            (TemporalUnit::Hour, hours_local),
            (TemporalUnit::Minute, minutes_local),
            (TemporalUnit::Second, seconds_local),
        ] {
            (field_locals.number_bits(unit)).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            (destination).store(function);
        }
        (field_locals.number_bits(TemporalUnit::Millisecond)).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        (field_locals.number_bits(TemporalUnit::Microsecond)).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (field_locals.number_bits(TemporalUnit::Nanosecond)).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Add);
        (subsecond_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Print magnitudes; the sign is a single prefix.
        for local in [seconds_local, subsecond_local, hours_local, minutes_local] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            (local).load(function);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            (local).load(function);
            function.instruction(&Instruction::End);
            (local).store(function);
        }

        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        piece.replace(
            self.emit_interned_string_reference("-", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &piece, function),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        piece.replace(
            self.emit_interned_string_reference("P", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &piece, function),
            function,
        );
        for (index, designator) in [(0_usize, "Y"), (1, "M"), (2, "W"), (3, "D")] {
            (field_locals.number_bits_locals()[index]).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            (field_locals.number_bits_locals()[index]).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            (number_payload_local).store(function);
            self.emit_temporal_duration_append_magnitude(
                &output,
                number_payload_local,
                number_payload_local,
                &piece,
                function,
            )?;
            piece.replace(
                self.emit_interned_string_reference(designator, function)?,
                function,
            );
            output.replace(
                self.emit_concat_gc_strings(&output, &piece, function),
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        for (local, designator) in [(hours_local, "H"), (minutes_local, "M")] {
            (local).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_duration_append_magnitude(
                &time,
                local,
                number_payload_local,
                &piece,
                function,
            )?;
            piece.replace(
                self.emit_interned_string_reference(designator, function)?,
                function,
            );
            time.replace(
                self.emit_concat_gc_strings(&time, &piece, function),
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // The seconds component is printed when it carries information, when
        // an explicit precision was asked for, or when nothing else would be.
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        (digits_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        (field_locals.number_bits_locals()[0]).load(function);
        (field_locals.number_bits_locals()[1]).load(function);
        function.instruction(&Instruction::I64Or);
        (field_locals.number_bits_locals()[2]).load(function);
        function.instruction(&Instruction::I64Or);
        (field_locals.number_bits_locals()[3]).load(function);
        function.instruction(&Instruction::I64Or);
        (hours_local).load(function);
        function.instruction(&Instruction::I64Or);
        (minutes_local).load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_append_magnitude(
            &time,
            seconds_local,
            number_payload_local,
            &piece,
            function,
        )?;
        self.emit_temporal_duration_append_fraction(
            &time,
            subsecond_local,
            digits_local,
            number_payload_local,
            &piece,
            function,
        )?;
        piece.replace(
            self.emit_interned_string_reference("S", function)?,
            function,
        );
        time.replace(
            self.emit_concat_gc_strings(&time, &piece, function),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_native_gc_string_length(&time, time_length, function);
        time_length.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        piece.replace(
            self.emit_interned_string_reference("T", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &piece, function),
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &time, function),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion()
            .value()
            .set_reference(&output, schema, function);
        self.completion()
            .set_normal(self.completion().value(), function);

        self.release_temporal_duration_field_locals(field_locals, function);
        for local in [
            number_payload_local,
            time_length,
            internal_date_locals[3],
            internal_date_locals[2],
            internal_date_locals[1],
            internal_date_locals[0],
            rounded_largest_local,
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
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        piece.clear(function);
        time.clear(function);
        output.clear(function);
        pending.clear(function);
        value.clear(function);
        options.clear(function);
        Ok(())
    }

    /// Append `abs(value)` as a decimal string.
    fn emit_temporal_duration_append_magnitude(
        &mut self,
        output: &GcLocal<StringValue>,
        value_local: I64Local,
        number_payload_local: I64Local,
        piece: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (value_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        (value_local).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        (value_local).load(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        (number_payload_local).store(function);
        piece.replace(
            self.emit_number_to_string_payload(number_payload_local, function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(output, piece, function),
            function,
        );
        Ok(())
    }

    /// `FormatFractionalSeconds`. `digits_local` is -1 for `auto`, in which
    /// case the nine-digit padding is trimmed of trailing zeros.
    fn emit_temporal_duration_append_fraction(
        &mut self,
        output: &GcLocal<StringValue>,
        subsecond_local: I64Local,
        digits_local: I64Local,
        number_payload_local: I64Local,
        piece: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let keep_local = self.runtime_schema().reserve_i64_local(function);
        let scaled_local = self.runtime_schema().reserve_i64_local(function);

        (digits_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (digits_local).load(function);
        function.instruction(&Instruction::Else);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(9));
        (keep_local).store(function);
        (subsecond_local).load(function);
        (scaled_local).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (scaled_local).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        (keep_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(1));
        (scaled_local).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        (scaled_local).store(function);
        (keep_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (keep_local).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (keep_local).load(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (keep_local).store(function);

        (keep_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        piece.replace(
            self.emit_interned_string_reference(".", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(output, piece, function),
            function,
        );
        for position in 0..9_i64 {
            (keep_local).load(function);
            function.instruction(&Instruction::I64Const(position));
            function.instruction(&Instruction::I64GtS);
            self.open_frame(ControlFrameKind::If, function);
            (subsecond_local).load(function);
            function.instruction(&Instruction::I64Const(10_i64.pow((8 - position) as u32)));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            (number_payload_local).store(function);
            piece.replace(
                self.emit_number_to_string_payload(number_payload_local, function)?,
                function,
            );
            output.replace(
                self.emit_concat_gc_strings(output, piece, function),
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(scaled_local, function);
        self.runtime_schema()
            .release_i64_local(keep_local, function);
        Ok(())
    }

    /// `ParseTemporalDurationString`. The grammar is
    /// `Sign? P (nnn[YMWD])* (T (nnn(.fff)?[HMS])*)?`, with a fraction allowed
    /// only on the last time component present, and at least one component
    /// required overall.
    pub(crate) fn emit_temporal_duration_parse_string(
        &mut self,
        string: &GcLocal<StringValue>,
        field_locals: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length_local = self.runtime_schema().reserve_i64_local(function);
        let cursor_local = self.runtime_schema().reserve_i64_local(function);
        let code_unit_local = self.runtime_schema().reserve_i64_local(function);
        let valid_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let value_local = self.runtime_schema().reserve_i64_local(function);
        let digit_count_local = self.runtime_schema().reserve_i64_local(function);
        let fraction_local = self.runtime_schema().reserve_i64_local(function);
        let fraction_digits_local = self.runtime_schema().reserve_i64_local(function);
        let has_fraction_local = self.runtime_schema().reserve_i64_local(function);
        let seen_local = self.runtime_schema().reserve_i64_local(function);
        let stage_local = self.runtime_schema().reserve_i64_local(function);
        let nanoseconds_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_native_gc_string_length(string, length_local, function);
        let integer_fields: [I64Local; 10] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        for local in integer_fields {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        function.instruction(&Instruction::I64Const(1));
        (valid_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (cursor_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (sign_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (seen_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (stage_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (has_fraction_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (nanoseconds_local).store(function);

        self.emit_temporal_duration_peek(
            string,
            cursor_local,
            length_local,
            code_unit_local,
            function,
        );
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(0x2212));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (sign_local).store(function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_peek(
            string,
            cursor_local,
            length_local,
            code_unit_local,
            function,
        );
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'P' as i64));
        function.instruction(&Instruction::I64Eq);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'p' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);

        // One pass over the components. `stage` is the index of the lowest
        // designator already consumed, so designators must strictly ascend.
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (valid_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_duration_peek(
            string,
            cursor_local,
            length_local,
            code_unit_local,
            function,
        );
        // The time designator flips the parser into the H/M/S half.
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (stage_local).load(function);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(5));
        (stage_local).store(function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        // A time designator must be followed by at least one component.
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        (value_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (digit_count_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (fraction_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (fraction_digits_local).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_gc_string_code_unit_i32(string, cursor_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        code_unit_local.store(function);
        self.emit_temporal_duration_code_unit_is_digit(code_unit_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        (value_local).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        (value_local).store(function);
        // Every valid whole date/time component is below 2^53. Check the
        // magnitude after each digit, allowing arbitrarily many leading zeros.
        (value_local).load(function);
        function.instruction(&Instruction::I64Const(9_007_199_254_740_992));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (digit_count_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (digit_count_local).store(function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (digit_count_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Optional fraction, scaled to nine digits.
        self.emit_temporal_duration_peek(
            string,
            cursor_local,
            length_local,
            code_unit_local,
            function,
        );
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (has_fraction_local).store(function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        (fraction_digits_local).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_gc_string_code_unit_i32(string, cursor_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        code_unit_local.store(function);
        self.emit_temporal_duration_code_unit_is_digit(code_unit_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        (fraction_digits_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        function.instruction(&Instruction::Br(3));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (fraction_digits_local).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        (fraction_digits_local).store(function);
        (fraction_local).load(function);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        (fraction_digits_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (fraction_local).store(function);
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (fraction_digits_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_duration_peek(
            string,
            cursor_local,
            length_local,
            code_unit_local,
            function,
        );
        (cursor_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (cursor_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (seen_local).store(function);

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
            (code_unit_local).load(function);
            function.instruction(&Instruction::I64Const(*upper as i64));
            function.instruction(&Instruction::I64Eq);
            (code_unit_local).load(function);
            function.instruction(&Instruction::I64Const(*lower as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            // The two `M` spellings are told apart by whether the parser has
            // already crossed the time designator.
            if index == 5 {
                (stage_local).load(function);
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I32And);
            } else if index == 6 {
                (stage_local).load(function);
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::I32And);
            }
            self.open_frame(ControlFrameKind::If, function);
            (stage_local).load(function);
            function.instruction(&Instruction::I64Const(*stage));
            function.instruction(&Instruction::I64GeS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(0));
            (valid_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            // Hours, minutes and seconds only exist after the time
            // designator; `P2H` and `P2S` are not durations.
            if *field >= 4 {
                (stage_local).load(function);
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::I64LtS);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(0));
                (valid_local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            // Date components and a time component that is not the last one
            // may not carry a fraction.
            if *field < 4 {
                (has_fraction_local).load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(0));
                (valid_local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I64Const(*stage));
            (stage_local).store(function);
            (integer_fields[*field]).load(function);
            (value_local).load(function);
            function.instruction(&Instruction::I64Add);
            (integer_fields[*field]).store(function);
            if *field >= 4 {
                let scale: i64 = match *field {
                    4 => 3_600,
                    5 => 60,
                    _ => 1,
                };
                (fraction_local).load(function);
                function.instruction(&Instruction::I64Const(scale));
                function.instruction(&Instruction::I64Mul);
                (nanoseconds_local).store(function);
                // A fraction ends the time part: nothing may follow it.
                (has_fraction_local).load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I64Const(9));
                (stage_local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (stage_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        (seen_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (valid_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (valid_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_STRING,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Spread the fractional remainder over the sub-hour fields.
        (integer_fields[5]).load(function);
        (nanoseconds_local).load(function);
        function.instruction(&Instruction::I64Const(60_000_000_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        (integer_fields[5]).store(function);
        (nanoseconds_local).load(function);
        function.instruction(&Instruction::I64Const(60_000_000_000));
        function.instruction(&Instruction::I64RemU);
        (nanoseconds_local).store(function);
        for (field, divisor) in [
            (6_usize, 1_000_000_000_i64),
            (7, 1_000_000),
            (8, 1_000),
            (9, 1),
        ] {
            (integer_fields[field]).load(function);
            (nanoseconds_local).load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Add);
            (integer_fields[field]).store(function);
            (nanoseconds_local).load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemU);
            (nanoseconds_local).store(function);
        }
        for (unit, local) in TemporalUnit::ALL.into_iter().zip(integer_fields) {
            (local).load(function);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            (field_locals.number_bits(unit)).store(function);
        }
        for local in integer_fields.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
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
            code_unit_local,
            cursor_local,
            length_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        let _ = TEMPORAL_DURATION_FIELD_NAMES;
        Ok(())
    }

    fn emit_temporal_duration_peek(
        &mut self,
        string: &GcLocal<StringValue>,
        cursor_local: I64Local,
        length_local: I64Local,
        code_unit_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (code_unit_local).store(function);
        (cursor_local).load(function);
        (length_local).load(function);
        function.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_gc_string_code_unit_i32(string, cursor_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        code_unit_local.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_duration_code_unit_is_digit(
        &mut self,
        code_unit_local: I64Local,
        function: &mut Function,
    ) {
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        (code_unit_local).load(function);
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }
}
