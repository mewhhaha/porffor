//! `Temporal.ZonedDateTime.prototype.toLocaleString`: a formatter built with
//! the receiver's zone as `toLocaleStringTimeZone`, applied to its Instant.

use super::*;
use crate::intrinsics::temporal::TemporalIntrinsicFamily;

impl FunctionBuilder<'_> {
    /// Temporal proposal (ECMA-402 amendments)
    /// `Temporal.ZonedDateTime.prototype.toLocaleString(locales, options)`:
    /// `CreateDateTimeFormat(%DateTimeFormat%, locales, options, any, all,
    /// zonedDateTime.[[TimeZone]])`, the calendar check, then
    /// `FormatDateTime(dateTimeFormat, CreateTemporalInstant(epochNs))`.
    pub(crate) fn emit_intl_dtf_zoned_date_time_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_local = self.reserve_temp_local();
        let dtf_payload_local = self.reserve_temp_local();
        let dtf_tag_local = self.reserve_temp_local();
        let calendar_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let compatible_local = self.reserve_temp_local();
        let instant_payload_local = self.reserve_temp_local();
        let instant_tag_local = self.reserve_temp_local();
        let format_payload_local = self.reserve_temp_local();
        let format_tag_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        self.emit_intl_create_date_time_format(
            IntlDateTimeFormatPurpose::ZonedDateTime { time_zone_local },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(dtf_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(dtf_tag_local));

        // A non-ISO calendar must be the formatter's own.
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_local,
            function,
        );
        self.emit_dtf_set_string(expected_local, "iso8601", function);
        self.emit_string_payload_equality_i32(calendar_local, expected_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(compatible_local));
        self.load_i64_to_local_from_offset(
            dtf_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            expected_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            expected_local,
            HEAP_INTL_DTF_CALENDAR_OFFSET,
            expected_local,
            function,
        );
        self.emit_string_payload_equality_i32(calendar_local, expected_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(compatible_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            INTL_DTF_CALENDAR_MISMATCH,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `CreateTemporalInstant(zonedDateTime.[[EpochNanoseconds]])`; the
        // formatter never exposes it.
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            instant_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            instant_tag_local,
            function,
        );
        self.emit_load_current_builtin_temporal_prototype(
            TemporalIntrinsicFamily::Instant,
            calendar_local,
            function,
        );
        self.emit_alloc_temporal_instant(
            instant_payload_local,
            instant_tag_local,
            calendar_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(instant_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(instant_tag_local));

        let format_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get Intl.DateTimeFormat.prototype.format`",
                )
            })?;
        self.emit_direct_js_call(
            &format_getter_meta,
            Some((dtf_payload_local, Some(dtf_tag_local))),
            &[],
            format_payload_local,
            format_tag_local,
            function,
        )?;
        self.emit_function_handle_call(
            format_payload_local,
            format_tag_local,
            None,
            &[(instant_payload_local, instant_tag_local)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        for local in [
            format_tag_local,
            format_payload_local,
            instant_tag_local,
            instant_payload_local,
            compatible_local,
            expected_local,
            calendar_local,
            dtf_tag_local,
            dtf_payload_local,
            time_zone_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
