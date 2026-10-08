//! `Temporal.PlainDate` statics and prototype methods.
//!
//! Split from `temporal_plain_date.rs` (constructor, record, accessors) so the
//! two halves stay readable; both are `impl FunctionBuilder` blocks.

mod convert;
mod difference;

use crate::gc_types::*;

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, StringValuedOption, TemporalConversionOverflowOptions, TemporalOverflow,
    TemporalRoundingMode, TemporalUnit, TemporalUnitOptionProperty, TemporalUnitSlot,
};
use super::temporal_plain_date::{
    TemporalEraLocals, TemporalMonthFieldContext, TemporalResolvedCalendarYear,
};
use super::temporal_plain_date_time_methods::{
    TemporalPlainArithmeticOperation, TemporalPlainDifferenceOperation,
};
use super::temporal_plain_year_month::TemporalPartialDateType;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use crate::intrinsics::temporal::TemporalPrototypeSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TemporalPlainDateStringMode {
    ToString,
    ToJson,
}

impl<'a> FunctionBuilder<'a> {
    /// `GetOptionsObject` followed by a single string-valued option lookup.
    ///
    /// The accepted spellings, the emitted codes and the default when the
    /// property is absent all come from `O`'s [`StringValuedOption`] impl, so
    /// there is no per-call-site slice whose first entry silently decides the
    /// default.
    pub(crate) fn emit_temporal_string_valued_option<O: StringValuedOption>(
        &mut self,
        options: &ValueLocals,
        output: I64Local,
        object_error: RuntimeErrorMessage,
        range_error: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        function.instruction(&Instruction::I64Const(O::DEFAULT.code()));
        output.store(function);
        options.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_heap_object_like_tag_i32(options.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            object_error,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_temporal_duration_option_get(options, O::PROPERTY, &value, function)?;
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_string_payload(&value, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        let text = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        let matched = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        matched.store(function);
        for accepted in O::ALLOWED {
            self.emit_temporal_string_matches(&text, accepted.name(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(accepted.code()));
            output.store(function);
            function.instruction(&Instruction::I32Const(1));
            matched.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        matched.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            range_error,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(matched, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        value.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_temporal_plain_date_overflow_option(
        &mut self,
        options: &ValueLocals,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options,
            overflow,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_OVERFLOW_OPTION,
            function,
        )
    }

    /// Acquire date fields in alphabetical order after the caller has acquired
    /// the canonical calendar. Month-code syntax precedes the later year Get
    /// and overflow lookup; calendar suitability belongs to field resolution.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_plain_date_read_fields(
        &mut self,
        argument: &ValueLocals,
        calendar: &TemporalCalendarSlotLocals,
        fields: &[I64Local; 3],
        present: &[I64Local; 3],
        acquired_month_code: &ValueLocals,
        code_present: I64Local,
        any_present: I64Local,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let era_slots = self.reserve_temporal_era_slots(function);
        self.emit_temporal_property_bag_positive_integer(
            argument,
            "day",
            present[2],
            fields[2],
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_MUST_BE_FINITE,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_MONTH_AND_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        let era = self.emit_temporal_read_era_fields(
            era_slots,
            argument,
            calendar.calendar_id(),
            function,
        )?;
        self.emit_temporal_property_bag_positive_integer(
            argument,
            "month",
            present[1],
            fields[1],
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_MUST_BE_FINITE,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_MONTH_AND_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        self.emit_temporal_duration_option_get(
            argument,
            "monthCode",
            acquired_month_code,
            function,
        )?;
        acquired_month_code.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        code_present.store(function);
        self.emit_temporal_month_code_string(
            acquired_month_code,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_MONTHCODE_MUST_BE_A_STRING,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
            function,
        )?;
        self.emit_temporal_property_bag_integer(
            argument,
            "year",
            present[0],
            fields[0],
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_MUST_BE_FINITE,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        for local in present
            .iter()
            .copied()
            .chain([code_present])
            .chain(era.present_locals())
        {
            local.load(function);
            function.instruction(&Instruction::I64Or);
        }
        any_present.store(function);
        Ok(era)
    }

    /// `CalendarResolveFields` + `RegulateISODate`. Type errors for missing
    /// required keys come first, then the range errors — Test262's
    /// `from/calendarresolvefields-error-ordering.js` asserts exactly that
    /// split.
    ///
    /// The year arrives as a [`TemporalResolvedCalendarYear`] rather than as a bare
    /// `(year, year-present)` pair, so a bag path that never ran
    /// [`FunctionBuilder::emit_temporal_resolve_era_to_calendar_year`] cannot reach here
    /// — it would answer "fields require year" for a perfectly good
    /// `{ era, eraYear }` bag, which is the exact defect this replaces.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_plain_date_resolve_fields(
        &mut self,
        resolved_year: TemporalResolvedCalendarYear,
        month_local: I64Local,
        month_present_local: I64Local,
        acquired_month_code: &ValueLocals,
        month_code_payload_local: I64Local,
        month_code_present_local: I64Local,
        day_local: I64Local,
        day_present_local: I64Local,
        overflow_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year_local = resolved_year.year_local();
        let year_present_local = resolved_year.year_present_local();

        for (present_local, message) in [
            (
                year_present_local,
                RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_REQUIRE_YEAR,
            ),
            (
                day_present_local,
                RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_REQUIRE_DAY,
            ),
        ] {
            (present_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::TypeError,
                message,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (month_present_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (month_code_present_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_FIELDS_REQUIRE_MONTH_OR_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let resolved_month = self.emit_temporal_resolve_calendar_month(
            resolved_year,
            month_local,
            month_present_local,
            acquired_month_code,
            month_code_payload_local,
            month_code_present_local,
            TemporalMonthFieldContext::PlainDate,
            function,
        )?;

        // Calendar regulation clamps high months/days under `constrain`;
        // a non-positive acquired ordinal or day always throws.
        (month_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        (day_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_MONTH_AND_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_calendar_date_to_iso(
            resolved_month,
            day_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;

        Ok(())
    }

    /// `RegulateISODate`: clamp under `constrain`, throw under `reject`.
    pub(crate) fn emit_temporal_plain_date_regulate(
        &mut self,
        year_local: I64Local,
        month_local: I64Local,
        day_local: I64Local,
        overflow_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.runtime_schema().reserve_i64_local(function);
        (overflow_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        function.instruction(&Instruction::Else);
        (month_local).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(12));
        (month_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        (day_local).load(function);
        (maximum_day_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        (maximum_day_local).load(function);
        (day_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Clamping cannot rescue a year outside the representable span, so the
        // limit check still runs on the constrained result.
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(maximum_day_local, function);
        Ok(())
    }

    pub(crate) fn emit_temporal_plain_date_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_temporal_to_temporal_date(
            &argument,
            TemporalConversionOverflowOptions::Read(&options),
            &fields,
            &calendar_value,
            function,
        )?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        for local in fields {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    fn emit_temporal_plain_date_compare_fields(
        &mut self,
        left: &[I64Local; 3],
        right: &[I64Local; 3],
        comparison: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        comparison.store(function);
        for (left, right) in left.iter().zip(right) {
            comparison.load(function);
            function.instruction(&Instruction::I64Eqz);
            self.open_frame(ControlFrameKind::If, function);
            left.load(function);
            right.load(function);
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(-1));
            comparison.store(function);
            function.instruction(&Instruction::Else);
            left.load(function);
            right.load(function);
            function.instruction(&Instruction::I64GtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            comparison.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_temporal_plain_date_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar = schema.reserve_value_local(function);
        let left: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let right: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let comparison = schema.reserve_i64_local(function);
        for (index, fields) in [(0, &left), (1, &right)] {
            self.emit_builtin_arg_to_value(index, &argument, function);
            self.emit_temporal_to_temporal_date(
                &argument,
                TemporalConversionOverflowOptions::Omit,
                fields,
                &calendar,
                function,
            )?;
        }
        self.emit_temporal_plain_date_compare_fields(&left, &right, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        comparison.store(function);
        self.completion().value().set_number(comparison, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        for local in left.into_iter().chain(right).chain([comparison]) {
            schema.release_i64_local(local, function);
        }
        calendar.clear(function);
        argument.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_plain_date_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let other: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let comparison = schema.reserve_i64_local(function);
        let equal = schema.reserve_i32_local(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &calendar_value, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_temporal_to_temporal_date(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other,
            &other_calendar_value,
            function,
        )?;
        self.emit_temporal_plain_date_compare_fields(&fields, &other, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let calendar = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let other_calendar = schema.reserve_gc_local(function).initialize(
            other_calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_string_payload_equality_i32(&calendar, &other_calendar, function);
        other_calendar.clear(function);
        calendar.clear(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        equal.store(function);
        self.completion().value().set_boolean(equal, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        schema.release_i32_local(equal, function);
        for local in fields.into_iter().chain(other).chain([comparison]) {
            schema.release_i64_local(local, function);
        }
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Brand recovery reads the actual GC record once, then clears its temporary owner.
    fn emit_temporal_plain_date_receiver_fields(
        &mut self,
        fields: &[I64Local; 3],
        calendar: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_temporal_plain_date_record_from_receiver(function)?;
        self.emit_temporal_plain_date_load_record(&record, fields, calendar, function);
        record.clear(function);
        Ok(())
    }

    /// Merge the acquired partial fields with the receiver's calendar fields.
    /// Era resolution excludes the receiver year; supplied month or monthCode
    /// excludes the receiver monthCode. Options follow the complete field sweep.
    pub(crate) fn emit_temporal_plain_date_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let new_fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let present: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let acquired_code = schema.reserve_value_local(function);
        let receiver_code = schema.reserve_value_local(function);
        let forbidden = schema.reserve_value_local(function);
        let encoded_code = schema.reserve_i64_local(function);
        let code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        let projected =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), fields, function);
        for (source, destination) in projected.fields().into_iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        self.emit_temporal_calendar_month_code_payload(&projected, &receiver_code, function)?;
        projected.release(self, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_branded_partial_object(&argument,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_OBJECT, function)?;
        for property in ["calendar", "timeZone"] {
            self.emit_temporal_duration_option_get(&argument, property, &forbidden, function)?;
            forbidden.tag().load(function);
            function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
                RuntimeErrorMessage::TEMPORAL_PLAINDATE_PROTOTYPE_WITH_DOES_NOT_ACCEPT_CALENDAR_OR_TIMEZONE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        acquired_code.set_undefined(function);
        function.instruction(&Instruction::I64Const(0));
        encoded_code.store(function);
        let era = self.emit_temporal_plain_date_read_fields(
            &argument,
            &calendar,
            &new_fields,
            &present,
            &acquired_code,
            code_present,
            any_present,
            function,
        )?;
        any_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_PROTOTYPE_WITH_REQUIRES_AT_LEAST_ONE_DATE_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_plain_date_overflow_option(&options, overflow, function)?;
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            new_fields[0],
            present[0],
            function,
        )?;
        self.emit_temporal_resolved_year_default_to(&resolved_year, fields[0], function);
        present[2].load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        fields[2].load(function);
        new_fields[2].store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        present[1].load(function);
        code_present.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        acquired_code.copy_from(&receiver_code, function);
        function.instruction(&Instruction::I64Const(1));
        code_present.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        present[2].store(function);
        self.emit_temporal_plain_date_resolve_fields(
            resolved_year,
            new_fields[1],
            present[1],
            &acquired_code,
            encoded_code,
            code_present,
            new_fields[2],
            present[2],
            overflow,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            new_fields[0],
            new_fields[1],
            new_fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        for local in fields.into_iter().chain(new_fields).chain(present).chain([
            overflow,
            any_present,
            code_present,
            encoded_code,
        ]) {
            schema.release_i64_local(local, function);
        }
        forbidden.clear(function);
        receiver_code.clear(function);
        acquired_code.clear(function);
        options.clear(function);
        argument.clear(function);
        calendar_value.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_plain_date_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let old_calendar = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &old_calendar, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &argument,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        for local in fields {
            schema.release_i64_local(local, function);
        }
        argument.clear(function);
        old_calendar.clear(function);
        Ok(())
    }

    pub(crate) fn emit_temporal_plain_date_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_temporal_plain_date_record_from_receiver(function)?;
        record.clear(function);
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::PlainDate,
            function,
        )
    }

    /// ToJson selects auto without observing an options argument.
    pub(crate) fn emit_temporal_plain_date_to_string(
        &mut self,
        mode: TemporalPlainDateStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let output = schema.reserve_value_local(function);
        let show_calendar = schema.reserve_i64_local(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        show_calendar.store(function);
        match mode {
            TemporalPlainDateStringMode::ToString => {
                let options = schema.reserve_value_local(function);
                self.emit_builtin_arg_to_value(0, &options, function);
                self.emit_temporal_string_valued_option::<ShowCalendarName>(
                    &options,
                    show_calendar,
                    RuntimeErrorMessage::TEMPORAL_PLAINDATE_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_CALENDARNAME_OPTION,
                    function,
                )?;
                options.clear(function);
            }
            TemporalPlainDateStringMode::ToJson => {}
        }
        self.emit_temporal_iso_date_string(fields[0], fields[1], fields[2], &output, function)?;
        self.emit_temporal_append_calendar_annotation(&calendar, show_calendar, &output, function)?;
        self.completion().set_normal(&output, function);
        calendar.release(self, function);
        for local in fields.into_iter().chain([show_calendar]) {
            schema.release_i64_local(local, function);
        }
        output.clear(function);
        calendar_value.clear(function);
        Ok(())
    }

    /// Format the complete ISO coordinates directly into a GC UTF16 String.
    pub(crate) fn emit_temporal_iso_date_string(
        &mut self,
        year: I64Local,
        month: I64Local,
        day: I64Local,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let text = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let magnitude = schema.reserve_i64_local(function);
        let number = schema.reserve_i64_local(function);
        year.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "-", function)?;
        function.instruction(&Instruction::I64Const(0));
        year.load(function);
        function.instruction(&Instruction::I64Sub);
        magnitude.store(function);
        function.instruction(&Instruction::Else);
        year.load(function);
        magnitude.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        magnitude.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        number.store(function);
        year.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_date_append_padded_decimal(&text, number, 6, function)?;
        function.instruction(&Instruction::Else);
        year.load(function);
        function.instruction(&Instruction::I64Const(9999));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "+", function)?;
        self.emit_date_append_padded_decimal(&text, number, 6, function)?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(&text, number, 4, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for coordinate in [month, day] {
            self.emit_temporal_append_gc_literal(&text, "-", function)?;
            coordinate.load(function);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            number.store(function);
            self.emit_date_append_padded_decimal(&text, number, 2, function)?;
        }
        output.set_reference(&text, schema, function);
        text.clear(function);
        schema.release_i64_local(number, function);
        schema.release_i64_local(magnitude, function);
        Ok(())
    }

    pub(crate) fn emit_temporal_append_gc_literal(
        &mut self,
        output: &GcLocal<StringValue>,
        literal: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let piece = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(literal, function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(output, &piece, function),
            function,
        );
        piece.clear(function);
        Ok(())
    }

    /// Temporal deliberately forbids implicit comparison, so `valueOf` always
    /// throws — `a < b` on two dates must be a loud error, not a silent
    /// string comparison.
    pub(crate) fn emit_temporal_plain_date_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError, RuntimeErrorMessage::TEMPORAL_PLAINDATE_DOES_NOT_SUPPORT_IMPLICIT_CONVERSION_USE_COMPARE_OR_EQUALS, function)
    }

    /// Duration acquisition completes before overflow options. Whole seconds
    /// are used for the time tail so full-range day spans do not overflow i64.
    pub(super) fn emit_temporal_plain_date_add_or_subtract(
        &mut self,
        operation: TemporalPlainArithmeticOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let overflow = schema.reserve_i64_local(function);
        let seconds = schema.reserve_i64_local(function);
        let subsecond = schema.reserve_i64_local(function);
        let day_delta = schema.reserve_i64_local(function);
        let duration = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &calendar_value, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_duration(&argument, &duration, function)?;
        self.emit_temporal_plain_date_overflow_option(&options, overflow, function)?;
        match operation {
            TemporalPlainArithmeticOperation::Add => {}
            TemporalPlainArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&duration, function)
            }
        }
        let date_fields = self.reserve_temporal_duration_date_field_locals(&duration, function);
        self.emit_temporal_duration_normalize_seconds(
            &duration,
            TemporalUnit::Hour,
            seconds,
            subsecond,
            function,
        );
        seconds.load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        date_fields[TemporalUnit::Day.duration_field_index()].load(function);
        function.instruction(&Instruction::I64Add);
        day_delta.store(function);
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_temporal_add_calendar_date(
            &calendar,
            fields[0],
            fields[1],
            fields[2],
            date_fields[TemporalUnit::Year.duration_field_index()],
            date_fields[TemporalUnit::Month.duration_field_index()],
            date_fields[TemporalUnit::Week.duration_field_index()],
            day_delta,
            overflow,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        for local in date_fields {
            schema.release_i64_local(local, function);
        }
        self.release_temporal_duration_field_locals(duration, function);
        for local in fields
            .into_iter()
            .chain([day_delta, subsecond, seconds, overflow])
        {
            schema.release_i64_local(local, function);
        }
        options.clear(function);
        argument.clear(function);
        calendar_value.clear(function);
        Ok(())
    }

    /// An absent time means midnight. The allocation owner checks the complete
    /// date-time lower bound, including the one-nanosecond boundary case.
    pub(crate) fn emit_temporal_plain_date_to_plain_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let time = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_date_receiver_fields(
            &[fields[0], fields[1], fields[2]],
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        for local in time {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_to_temporal_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &time,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (source, destination) in time.iter().zip(fields[3..].iter()) {
            source.load(function);
            destination.store(function);
        }
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_time_field_locals(time, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        argument.clear(function);
        calendar_value.clear(function);
        Ok(())
    }

    /// Publish only the completed calendar partial reference produced by the
    /// projection owner; the caller cannot select a mismatched record kind.
    fn emit_temporal_plain_date_to_partial_date(
        &mut self,
        kind: TemporalPartialDateType,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_plain_date_receiver_fields(&fields, &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            kind,
            fields,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in fields {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        Ok(())
    }

    /// Temporal proposal 3.3.x `toPlainYearMonth`.
    pub(crate) fn emit_temporal_plain_date_to_plain_year_month(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_plain_date_to_partial_date(
            TemporalPartialDateType::PlainYearMonth,
            function,
        )
    }

    /// Temporal proposal 3.3.x `toPlainMonthDay`.
    pub(crate) fn emit_temporal_plain_date_to_plain_month_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_plain_date_to_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            function,
        )
    }
}
