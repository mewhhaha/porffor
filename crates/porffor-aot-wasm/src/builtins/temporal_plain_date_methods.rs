//! `Temporal.PlainDate` statics and prototype methods.
//!
//! Split from `temporal_plain_date.rs` (constructor, record, accessors) so the
//! two halves stay readable; both are `impl FunctionBuilder` blocks.

use super::super::*;
use super::temporal_options::{ShowCalendarName, StringValuedOption, TemporalOverflow};

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
            self.emit_temporal_plain_date_calendar(
                calendar_payload_local,
                calendar_tag_local,
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
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
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

    /// `TemporalDateToString`. `builtin` selects whether the `calendarName`
    /// option is read: `toString` reads it, `toJSON` and `toLocaleString` are
    /// fixed at `auto`.
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

        // `FormatCalendarAnnotation`: `auto` suppresses the annotation for the
        // ISO calendar, which is the only calendar this backend has.
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Always.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(show_calendar_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("[!u-ca=")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("[u-ca=")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(calendar_payload_local));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::End);

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
}
