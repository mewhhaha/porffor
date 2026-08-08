//! `Temporal.PlainDate` statics and prototype methods.
//!
//! Split from `temporal_plain_date.rs` (constructor, record, accessors) so the
//! two halves stay readable; both are `impl FunctionBuilder` blocks.

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, StringValuedOption, TemporalOverflow, TemporalRoundingMode, TemporalUnit,
    TemporalUnitSlot,
};
use super::temporal_plain_year_month::{TemporalPartialDatePrototype, TemporalPartialDateType};

/// `ISO_REFERENCE_YEAR`, the year every `Temporal.PlainMonthDay` stores. 1972
/// is a leap year, so `--02-29` is representable and `toPlainMonthDay` needs no
/// range check of its own. `temporal_plain_month_day.rs` names the same value
/// for its own constructors; it is private there, and this file must not edit
/// a sibling module.
const TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR: i64 = 1972;

impl<'a> FunctionBuilder<'a> {
    /// `GetOptionsObject` followed by a single string-valued option lookup.
    ///
    /// The accepted spellings, the emitted codes and the default when the
    /// property is absent all come from `O`'s [`StringValuedOption`] impl, so
    /// there is no per-call-site slice whose first entry silently decides the
    /// default.
    pub(crate) fn emit_temporal_string_valued_option<O: StringValuedOption>(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        option_local: u32,
        options_type_error: &str,
        option_range_error: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property = O::PROPERTY;
        let property_key_local = self.reserve_temp_local();
        let option_payload_local = self.reserve_temp_local();
        let option_tag_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(O::DEFAULT.code()));
        function.instruction(&Instruction::LocalSet(option_local));
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            options_type_error,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload(property)));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            property_key_local,
            option_payload_local,
            option_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(option_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(option_payload_local, option_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(option_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(recognized_local));
        for accepted in O::ALLOWED {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(accepted.name()),
            ));
            function.instruction(&Instruction::LocalSet(expected_payload_local));
            self.emit_string_payload_equality_i32(
                option_payload_local,
                expected_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(recognized_local));
            function.instruction(&Instruction::I64Const(accepted.code()));
            function.instruction(&Instruction::LocalSet(option_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(recognized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            option_range_error,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            recognized_local,
            expected_payload_local,
            option_tag_local,
            option_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_temporal_plain_date_overflow_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options_payload_local,
            options_tag_local,
            overflow_local,
            "Temporal.PlainDate options must be an object or undefined",
            "Invalid Temporal.PlainDate overflow option",
            function,
        )
    }

    /// `PrepareCalendarFields` for the `« year, month, month-code, day »` key
    /// set, in the alphabetical order Test262's `order-of-operations.js` pins:
    /// `calendar`, then `day`, `month`, `monthCode`, `year`.
    ///
    /// Only reads. Validation is deliberately left to
    /// `emit_temporal_plain_date_resolve_fields`, because
    /// `CalendarResolveFields` runs after `GetTemporalOverflowOption` and the
    /// option read is observable.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_plain_date_read_fields(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        year_local: u32,
        year_present_local: u32,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        day_present_local: u32,
        read_calendar: bool,
        strict_month_code: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();

        if read_calendar {
            function.instruction(&Instruction::I64Const(self.strings.payload("calendar")));
            function.instruction(&Instruction::LocalSet(property_key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                property_key_local,
                calendar_payload_local,
                calendar_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            self.emit_temporal_to_temporal_calendar_identifier(
                calendar_payload_local,
                calendar_tag_local,
                "Temporal.PlainDate calendar must be a string",
                function,
            )?;
        }

        for (property, output_local, output_present_local) in [
            ("day", day_local, day_present_local),
            ("month", month_local, month_present_local),
        ] {
            self.emit_temporal_property_bag_positive_integer(
                argument_payload_local,
                argument_tag_local,
                property,
                property_key_local,
                value_payload_local,
                value_tag_local,
                present_local,
                output_local,
                0,
                "Temporal.PlainDate fields must be finite",
                "Temporal.PlainDate month and day must be positive",
                function,
            )?;
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::LocalSet(output_present_local));
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("monthCode")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        if strict_month_code {
            self.emit_temporal_month_code_string(
                value_payload_local,
                value_tag_local,
                "Temporal.PlainDate monthCode must be a string",
                "Invalid Temporal.PlainDate monthCode",
                function,
            )?;
        } else {
            self.emit_temporal_property_bag_string(
                value_payload_local,
                value_tag_local,
                "Temporal.PlainDate monthCode must be a string",
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            "Temporal.PlainDate fields must be finite",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::LocalSet(year_present_local));

        for local in [
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `CalendarResolveFields` + `RegulateISODate`. Type errors for missing
    /// required keys come first, then the range errors — Test262's
    /// `from/calendarresolvefields-error-ordering.js` asserts exactly that
    /// split.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_plain_date_resolve_fields(
        &mut self,
        year_local: u32,
        year_present_local: u32,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        day_present_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let month_from_code_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();

        for (present_local, message) in [
            (year_present_local, "Temporal.PlainDate fields require year"),
            (day_present_local, "Temporal.PlainDate fields require day"),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                message,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate fields require month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_from_code_local));
        for month in 1_i64..=12 {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(expected_payload_local));
            self.emit_string_payload_equality_i32(
                month_code_payload_local,
                expected_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::LocalSet(month_from_code_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainDate monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDate month and monthCode must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);

        // `RegulateISODate` clamps out-of-range months and days under
        // `constrain`, but a non-positive month or day is never representable
        // and always throws.
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDate month and day must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_plain_date_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;

        self.release_temp_local(expected_payload_local);
        self.release_temp_local(month_from_code_local);
        Ok(())
    }

    /// `RegulateISODate`: clamp under `constrain`, throw under `reject`.
    pub(crate) fn emit_temporal_plain_date_regulate(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(overflow_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        // Clamping cannot rescue a year outside the representable span, so the
        // limit check still runs on the constrained result.
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    /// `ToTemporalDate`. Accepts a branded `Temporal.PlainDate` (cloned), any
    /// other object (read as a property bag), or an ISO string.
    ///
    /// `read_options` is false for `compare` and `equals`, which take no
    /// options argument at all.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_to_temporal_date(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        read_options: bool,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));

        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        if read_options {
            self.emit_temporal_plain_date_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            year_local,
            year_present_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            true,
            false,
            function,
        )?;
        if read_options {
            self.emit_temporal_plain_date_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_plain_date_resolve_fields(
            year_local,
            year_present_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
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
            "Temporal.PlainDate expects a string, a property bag, or a Temporal.PlainDate",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_plain_date_string(
            argument_payload_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        if read_options {
            self.emit_temporal_plain_date_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        function.instruction(&Instruction::End);

        for local in [
            handled_local,
            day_present_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            year_present_local,
            record_local,
            brand_local,
            overflow_local,
            calendar_tag_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.2.2 `Temporal.PlainDate.from`.
    pub(crate) fn emit_temporal_plain_date_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_temporal_to_temporal_date(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            true,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.2.3 `Temporal.PlainDate.compare`.
    pub(crate) fn emit_temporal_plain_date_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let left_year_local = self.reserve_temp_local();
        let left_month_local = self.reserve_temp_local();
        let left_day_local = self.reserve_temp_local();
        let right_year_local = self.reserve_temp_local();
        let right_month_local = self.reserve_temp_local();
        let right_day_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        for (index, year_local, month_local, day_local) in [
            (0, left_year_local, left_month_local, left_day_local),
            (1, right_year_local, right_month_local, right_day_local),
        ] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_temporal_to_temporal_date(
                argument_payload_local,
                argument_tag_local,
                undefined_local,
                undefined_tag_local,
                false,
                year_local,
                month_local,
                day_local,
                calendar_payload_local,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(comparison_local));
        for (left, right) in [
            (left_year_local, right_year_local),
            (left_month_local, right_month_local),
            (left_day_local, right_day_local),
        ] {
            function.instruction(&Instruction::LocalGet(comparison_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(comparison_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(comparison_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            comparison_local,
            right_day_local,
            right_month_local,
            right_year_local,
            left_day_local,
            left_month_local,
            left_year_local,
            calendar_payload_local,
            undefined_tag_local,
            undefined_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `Temporal.PlainDate.prototype.equals`.
    pub(crate) fn emit_temporal_plain_date_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let other_year_local = self.reserve_temp_local();
        let other_month_local = self.reserve_temp_local();
        let other_day_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_temporal_to_temporal_date(
            argument_payload_local,
            argument_tag_local,
            undefined_local,
            undefined_tag_local,
            false,
            other_year_local,
            other_month_local,
            other_day_local,
            other_calendar_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(other_year_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(other_month_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(other_day_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(
            calendar_payload_local,
            other_calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            other_calendar_payload_local,
            other_day_local,
            other_month_local,
            other_year_local,
            undefined_tag_local,
            undefined_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `with`. `CalendarMergeFields` drops the
    /// receiver's `monthCode` as soon as the argument supplies either `month`
    /// or `monthCode`, so supplying `month` alone is never a conflict.
    pub(crate) fn emit_temporal_plain_date_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let new_year_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let new_month_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let new_day_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate.prototype.with requires an object",
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
            "Temporal.PlainDate.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject`: a bag that names a calendar or a time
        // zone is a caller mistake, not a partial date.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_own_property_present(
                argument_payload_local,
                argument_tag_local,
                key_local,
                present_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.PlainDate.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            new_year_local,
            year_present_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate.prototype.with requires at least one date field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_plain_date_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalSet(new_year_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalSet(new_day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalSet(new_month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(month_present_local));
        function.instruction(&Instruction::End);
        for local in [year_present_local, day_present_local] {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(local));
        }

        self.emit_temporal_plain_date_resolve_fields(
            new_year_local,
            year_present_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            new_year_local,
            new_month_local,
            new_day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            day_present_local,
            new_day_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            new_month_local,
            year_present_local,
            new_year_local,
            present_local,
            key_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `withCalendar`. The ISO fields are already
    /// valid, so no range check is needed — only the calendar identifier is
    /// validated.
    pub(crate) fn emit_temporal_plain_date_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let existing_calendar_payload_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                existing_calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_builtin_arg_to_locals(0, calendar_payload_local, calendar_tag_local, function);
        // `withCalendar` requires an explicit identifier, so `undefined` is a
        // TypeError rather than the constructor's `iso8601` default.
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_to_temporal_calendar_identifier(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.PlainDate calendar must be a string",
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            existing_calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Temporal.PlainDate.prototype.toLocaleString`.
    ///
    /// Not a variant of `toString`: the proposal defines it as
    /// `new Intl.DateTimeFormat(locales, options).format(this)`, so it emits
    /// exactly that. Only the brand check belongs here — it is observable
    /// before either argument is read, and only this file knows which internal
    /// slot to name when it fails.
    pub(crate) fn emit_temporal_plain_date_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        self.release_temp_local(record_local);
        self.emit_intl_dtf_temporal_to_locale_string(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
            function,
        )
    }

    /// `TemporalDateToString`. `builtin` selects whether the `calendarName`
    /// option is read: `toString` reads it and `toJSON` is fixed at `auto`.
    pub(crate) fn emit_temporal_plain_date_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let show_calendar_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::LocalSet(show_calendar_local));
        if matches!(
            builtin,
            StandardBuiltinId::TemporalPlainDatePrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_string_valued_option::<ShowCalendarName>(
                options_payload_local,
                options_tag_local,
                show_calendar_local,
                "Temporal.PlainDate options must be an object or undefined",
                "Invalid Temporal.PlainDate calendarName option",
                function,
            )?;
        }

        self.emit_temporal_iso_date_string(
            year_local,
            month_local,
            day_local,
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;

        // `FormatCalendarAnnotation`. Shared with `Temporal.PlainYearMonth`
        // and `Temporal.PlainMonthDay` so the `auto` suppression rule — print
        // the annotation for every calendar except `iso8601` — is decided in
        // one place for all three.
        self.emit_temporal_append_calendar_annotation(
            show_calendar_local,
            calendar_payload_local,
            output_payload_local,
            piece_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            show_calendar_local,
            options_tag_local,
            options_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `PadISOYear` + `-MM-DD`, the ISO date half of both
    /// `TemporalDateToString` and `TemporalDateTimeToString`. Overwrites
    /// `output_payload_local`; `piece_payload_local` and `number_payload_local`
    /// are scratch the caller already owns.
    pub(crate) fn emit_temporal_iso_date_string(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_payload_local: u32,
        piece_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        // `PadISOYear`: four digits inside 0..9999, otherwise a signed six.
        function.instruction(&Instruction::LocalGet(year_local));
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
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(9_999));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(
            output_payload_local,
            number_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for (value_local, separator) in [(month_local, "-"), (day_local, "-")] {
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::LocalGet(value_local));
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
        Ok(())
    }

    /// Temporal deliberately forbids implicit comparison, so `valueOf` always
    /// throws — `a < b` on two dates must be a loud error, not a silent
    /// string comparison.
    pub(crate) fn emit_temporal_plain_date_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate does not support implicit conversion; use compare() or equals()",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    /// Loads the four receiver slots every arithmetic method starts from, after
    /// the `[[InitializedTemporalDate]]` brand check.
    fn emit_temporal_plain_date_receiver_fields(
        &mut self,
        record_local: u32,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `add` and `subtract`, both through
    /// `AddDurationToDate`.
    ///
    /// `ToDateDurationRecordWithoutTime` stops at whole seconds rather than
    /// nanoseconds on purpose. `add/argument-duration-max.js` passes
    /// `{hours: 2400000023}`, which is 8.6e21 nanoseconds and overflows `i64`;
    /// the same span is 8.6e12 seconds, comfortably inside it. The sub-day
    /// remainder is then discarded, which is exactly what "without time" means.
    pub(crate) fn emit_temporal_plain_date_add_or_subtract(
        &mut self,
        subtract: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let day_delta_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_date_receiver_fields(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        // `add/order-of-operations.js`: all ten duration fields are read, and
        // only then `options.overflow`.
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &duration_locals,
            function,
        )?;
        self.emit_temporal_plain_date_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
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

        // `ToDateDurationRecordWithoutTime`. `emit_temporal_duration_normalize_seconds`
        // leaves a whole-second count and a `|subsecond| < 1e9` of the same
        // sign, so `I64DivS` here is the spec's truncation toward zero.
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalGet(
            duration_locals[TemporalUnit::Day.duration_field_index()],
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(day_delta_local));

        self.emit_temporal_add_iso_date(
            year_local,
            month_local,
            day_local,
            duration_locals[TemporalUnit::Year.duration_field_index()],
            duration_locals[TemporalUnit::Month.duration_field_index()],
            duration_locals[TemporalUnit::Week.duration_field_index()],
            day_delta_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temporal_duration_field_locals(duration_locals);
        for local in [
            prototype_payload_local,
            day_delta_local,
            subsecond_local,
            seconds_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `NudgeToCalendarUnit` for a date-only receiver: `smallestUnit` is
    /// `year`, `month` or `week`, none of which has a fixed length, so the
    /// difference cannot be rounded by dividing. The proposal instead dates
    /// both candidates — the already-truncated `r1` in `years/months/weeks`,
    /// and `r2` one `increment` further in the direction of travel — and asks
    /// where `other` falls between them.
    ///
    /// Deliberately *not* `emit_temporal_plain_date_time_nudge_calendar_unit`
    /// with zeroed times: that helper measures the bracket in nanoseconds, and
    /// a `PlainDate` bracket can be 200,000,001 days wide, which is 1.7e22
    /// nanoseconds and overflows `i64`. With no time-of-day to account for, the
    /// same comparison is exact in the day domain.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_plain_date_nudge_calendar_unit(
        &mut self,
        date: [u32; 3],
        other: [u32; 3],
        smallest_unit_local: u32,
        increment_local: u32,
        mode_local: u32,
        years_local: u32,
        months_local: u32,
        weeks_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let overflow_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let step_local = self.reserve_temp_local();
        let receiver_epoch_local = self.reserve_temp_local();
        let other_epoch_local = self.reserve_temp_local();
        let start_epoch_local = self.reserve_temp_local();
        let end_epoch_local = self.reserve_temp_local();
        let numerator_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let start_year_local = self.reserve_temp_local();
        let start_month_local = self.reserve_temp_local();
        let start_day_local = self.reserve_temp_local();
        let end_year_local = self.reserve_temp_local();
        let end_month_local = self.reserve_temp_local();
        let end_day_local = self.reserve_temp_local();
        let nudge_years_local = self.reserve_temp_local();
        let nudge_months_local = self.reserve_temp_local();
        let nudge_weeks_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_temporal_plain_date_epoch_days(
            date[0],
            date[1],
            date[2],
            receiver_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            other[0],
            other[1],
            other[2],
            other_epoch_local,
            function,
        );

        // `DurationSign` of the untruncated difference. With no time-of-day the
        // date comparison is the whole answer.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(receiver_epoch_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(receiver_epoch_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(step_local));
        for (source, destination) in [
            (years_local, nudge_years_local),
            (months_local, nudge_months_local),
            (weeks_local, nudge_weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        for (unit, local) in [
            (TemporalUnit::Year, nudge_years_local),
            (TemporalUnit::Month, nudge_months_local),
            (TemporalUnit::Week, nudge_weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::LocalGet(step_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }

        for (year, month, day) in [
            (start_year_local, start_month_local, start_day_local),
            (end_year_local, end_month_local, end_day_local),
        ] {
            for (source, destination) in [(date[0], year), (date[1], month), (date[2], day)] {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }
        }
        // Both candidates go through `AddISODate`, so a rounding increment that
        // walks either of them off the representable range throws here — which
        // is what `throws-if-rounded-date-outside-valid-iso-range.js` asserts.
        self.emit_temporal_add_iso_date(
            start_year_local,
            start_month_local,
            start_day_local,
            years_local,
            months_local,
            weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_add_iso_date(
            end_year_local,
            end_month_local,
            end_day_local,
            nudge_years_local,
            nudge_months_local,
            nudge_weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            start_year_local,
            start_month_local,
            start_day_local,
            start_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            end_year_local,
            end_month_local,
            end_day_local,
            end_epoch_local,
            function,
        );

        // `numerator` is `other - r1`, signed with the direction of travel;
        // `quantum` is the bracket width, always positive.
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_local));
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(quantum_local));

        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_round_nanoseconds(
            numerator_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::End);
        // `other` lies inside the bracket, so the rounded value is either zero
        // (keep `r1`) or the whole bracket (take `r2`).
        function.instruction(&Instruction::LocalGet(numerator_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (source, destination) in [
            (nudge_years_local, years_local),
            (nudge_months_local, months_local),
            (nudge_weeks_local, weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        for local in [
            nudge_weeks_local,
            nudge_months_local,
            nudge_years_local,
            end_day_local,
            end_month_local,
            end_year_local,
            start_day_local,
            start_month_local,
            start_year_local,
            quantum_local,
            numerator_local,
            end_epoch_local,
            start_epoch_local,
            other_epoch_local,
            receiver_epoch_local,
            step_local,
            sign_local,
            zero_local,
            overflow_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `until` and `since`, both through
    /// `DifferenceTemporalPlainDate`.
    ///
    /// Three things differ from the `PlainDateTime` shape beyond the missing
    /// time half:
    ///
    /// * both unit options are confined to `year..day`, so
    ///   `until/throws-with-time-units.js` gets its RangeError and
    ///   `ValidateTemporalRoundingIncrement` never applies (a date unit has no
    ///   maximum increment);
    /// * a `day` smallestUnit rounds the *day count*, not a nanosecond count.
    ///   `add/argument-duration-max-plus-min-date.js` reaches a 200,000,001-day
    ///   span, and that many nanoseconds does not fit in `i64`;
    /// * `NudgeToCalendarUnit` is fed `r1` already truncated to a multiple of
    ///   `roundingIncrement`, and `BubbleRelativeDuration` folds twelve months
    ///   back into a year afterwards. `until/roundingincrement.js` and
    ///   `until/round-cross-unit-boundary.js` need those two respectively.
    pub(crate) fn emit_temporal_plain_date_until_or_since(
        &mut self,
        since: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let other_year_local = self.reserve_temp_local();
        let other_month_local = self.reserve_temp_local();
        let other_day_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let largest_unit_local = self.reserve_temp_local();
        let smallest_unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let original_mode_local = self.reserve_temp_local();
        let years_local = self.reserve_temp_local();
        let months_local = self.reserve_temp_local();
        let weeks_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let carry_local = self.reserve_temp_local();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_date_receiver_fields(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        // `read_options` is false: `until/order-of-operations.js` records no
        // `overflow` read at all, and reading one here would place it before
        // the four difference settings.
        self.emit_temporal_to_temporal_date(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            other_year_local,
            other_month_local,
            other_day_local,
            other_calendar_payload_local,
            function,
        )?;
        // `DifferenceTemporalPlainDate` step 2: `CalendarEquals` runs
        // immediately after `ToTemporalDate` and before `GetOptionsObject`, so
        // a calendar mismatch is a RangeError even when the options bag would
        // also have thrown.
        self.emit_temporal_require_same_calendar(
            calendar_payload_local,
            other_calendar_payload_local,
            "Temporal.PlainDate until and since require the same calendar",
            function,
        )?;

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit, and every read completes before any
        // validation — `until/options-read-before-algorithmic-validation.js`.
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
            // halfFloor; the sign-symmetric modes are unchanged.
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
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::LocalSet(smallest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Day,
            "Invalid Temporal.PlainDate unit option",
            function,
        )?;
        // An unset or `"auto"` largestUnit falls back to the larger of day and
        // the smallest unit: `day` by default, `year` for
        // `{smallestUnit: "years"}`.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(largest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Day,
            "Invalid Temporal.PlainDate unit option",
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;

        for local in [years_local, months_local, weeks_local, days_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_difference_iso_date(
            [year_local, month_local, day_local],
            [other_year_local, other_month_local, other_day_local],
            largest_unit_local,
            years_local,
            months_local,
            weeks_local,
            days_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `emit_temporal_plain_time_round_nanoseconds` is unit-agnostic: it
        // rounds a signed count to a multiple of the increment under the mode.
        // Applying it to the day count directly avoids the nanosecond scaling
        // that would overflow on a full-range span.
        self.emit_temporal_plain_time_round_nanoseconds(
            days_local,
            increment_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        // `smallestUnit` is year, month or week — the range check above leaves
        // no other option below `day`.
        //
        // Everything under the smallest unit drops, which is both the `trunc`
        // answer and the lower of the two candidates every other mode picks
        // between.
        for (limit, local) in [
            (TemporalUnit::Week.code(), days_local),
            (TemporalUnit::Month.code(), weeks_local),
            (TemporalUnit::Year.code(), months_local),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(limit));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        // `NudgeToCalendarUnit` defines `r1` as
        // `RoundTowardZero(value / increment) * increment`, so the smallest
        // unit is truncated to a multiple of the increment *before* the two
        // candidates are dated. `I64DivS` truncates toward zero, which is the
        // spec's rounding for both signs.
        for (unit, local) in [
            (TemporalUnit::Year, years_local),
            (TemporalUnit::Month, months_local),
            (TemporalUnit::Week, weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_plain_date_nudge_calendar_unit(
            [year_local, month_local, day_local],
            [other_year_local, other_month_local, other_day_local],
            smallest_unit_local,
            increment_local,
            mode_local,
            years_local,
            months_local,
            weeks_local,
            function,
        )?;
        // `BubbleRelativeDuration`, month into year. The nudge can leave twelve
        // months where the caller asked for `largestUnit: "years"`, and only
        // this step turns that into one more year.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::LocalGet(years_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(years_local));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(months_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if since {
            for local in [years_local, months_local, weeks_local, days_local] {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(local));
            }
        }
        // The time tail is identically zero, so there is nothing for
        // `BalanceTimeDuration` to do.
        self.emit_temporal_duration_zero_fields(&duration_locals, function);
        for (source, unit) in [
            (years_local, TemporalUnit::Year),
            (months_local, TemporalUnit::Month),
            (weeks_local, TemporalUnit::Week),
            (days_local, TemporalUnit::Day),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(
                duration_locals[unit.duration_field_index()],
            ));
        }
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        for local in [
            carry_local,
            days_local,
            weeks_local,
            months_local,
            years_local,
            original_mode_local,
            mode_local,
            increment_local,
            smallest_unit_local,
            largest_unit_local,
            other_calendar_payload_local,
            other_day_local,
            other_month_local,
            other_year_local,
            undefined_tag_local,
            undefined_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 3.3.x `toPlainDateTime`. An absent argument means
    /// midnight, not a TypeError.
    ///
    /// `ToTemporalTime` accepts the string, bag, `PlainTime`, `PlainDateTime`
    /// and `ZonedDateTime` forms and pins the alphabetical
    /// hour/microsecond/millisecond/minute/nanosecond/second read order that
    /// `toPlainDateTime/order-of-operations.js` checks.
    pub(crate) fn emit_temporal_plain_date_to_plain_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let time_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_date_receiver_fields(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        for local in time_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &time_locals,
            function,
        )?;
        function.instruction(&Instruction::End);

        for (source, destination) in [
            (year_local, field_locals[0]),
            (month_local, field_locals[1]),
            (day_local, field_locals[2]),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        for index in 0..6 {
            function.instruction(&Instruction::LocalGet(time_locals[index]));
            function.instruction(&Instruction::LocalSet(field_locals[index + 3]));
        }
        // `emit_alloc_temporal_plain_date_time` runs
        // `emit_temporal_reject_date_time_lower_bound` itself, which is what
        // makes `new Temporal.PlainDate(-271821, 4, 19).toPlainDateTime()`
        // throw while the same day at one nanosecond past midnight succeeds
        // (`toPlainDateTime/limits.js`). Repeating the call here would only
        // grow the emitted function.
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_time_field_locals(time_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The shared body of `toPlainYearMonth` and `toPlainMonthDay`: keep the
    /// two ISO fields `kind` stores and replace the third with `kind`'s
    /// reference value.
    fn emit_temporal_plain_date_to_partial_date(
        &mut self,
        kind: TemporalPartialDateType,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_receiver_fields(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        match kind {
            TemporalPartialDateType::PlainYearMonth => {
                // `referenceISODay` is 1 for every calendar-derived year-month.
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(day_local));
                // `ISOYearMonthWithinLimits`, not `ISODateWithinLimits`: the
                // two differ exactly at -271821-04, which is a legal
                // `PlainYearMonth` even though -271821-04-01 is not a legal
                // `PlainDate` (`toPlainYearMonth/limits.js`).
                self.emit_temporal_reject_iso_year_month(
                    year_local,
                    month_local,
                    day_local,
                    function,
                )?;
            }
            TemporalPartialDateType::PlainMonthDay => {
                // 1972 is a leap year, so every month-day the receiver can hold
                // is representable and no range check is needed.
                function.instruction(&Instruction::I64Const(
                    TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
                ));
                function.instruction(&Instruction::LocalSet(year_local));
            }
        }
        self.emit_alloc_temporal_partial_date(
            kind,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
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
