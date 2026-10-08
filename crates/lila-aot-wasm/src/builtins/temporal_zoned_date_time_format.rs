//! Zoned date-time serialization projects the newly rounded exact epoch.

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, StringValuedOption, TemporalRoundingMode, TemporalUnitOptionProperty,
};
use crate::gc_types::*;

/// The `valueOf` TypeError text, interned unconditionally in the `data.rs`
/// string pool next to the sibling Instant message.
const TEMPORAL_ZONED_DATE_TIME_VALUE_OF_MESSAGE: RuntimeErrorMessage =
    RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_DOES_NOT_SUPPORT_IMPLICIT_CONVERSION_USE_COMPARE_OR_EQUALS;

#[derive(Clone, Copy)]
enum ShowOffset {
    Auto,
    Never,
}

impl StringValuedOption for ShowOffset {
    const PROPERTY: &'static str = "offset";
    const DEFAULT: Self = Self::Auto;
    const ALLOWED: &'static [Self] = &[Self::Auto, Self::Never];

    fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Never => "never",
        }
    }

    fn code(self) -> i64 {
        match self {
            Self::Auto => 0,
            Self::Never => 1,
        }
    }
}

#[derive(Clone, Copy)]
enum ShowTimeZone {
    Auto,
    Never,
    Critical,
}

impl StringValuedOption for ShowTimeZone {
    const PROPERTY: &'static str = "timeZoneName";
    const DEFAULT: Self = Self::Auto;
    const ALLOWED: &'static [Self] = &[Self::Auto, Self::Never, Self::Critical];

    fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Never => "never",
            Self::Critical => "critical",
        }
    }

    fn code(self) -> i64 {
        match self {
            Self::Auto => 0,
            Self::Never => 1,
            Self::Critical => 2,
        }
    }
}

/// Where the string core reads its options bag from. A `bool` here would put
/// `(true, function)` and `(false, function)` at adjacent call sites, where a
/// transposition compiles and makes `toJSON` observe its argument — the same
/// silent wrong answer the direction domains in
/// `temporal_zoned_date_time_methods.rs` exist to prevent.
#[derive(Clone, Copy)]
enum ZonedDateTimeStringOptions {
    FromArgument,
    Undefined,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_temporal_zoned_date_time_to_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_to_string_core(
            ZonedDateTimeStringOptions::FromArgument,
            function,
        )
    }

    pub(super) fn emit_temporal_zoned_date_time_to_json(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_to_string_core(
            ZonedDateTimeStringOptions::Undefined,
            function,
        )
    }

    fn emit_temporal_zoned_date_time_to_string_core(
        &mut self,
        options: ZonedDateTimeStringOptions,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let options_value = schema.reserve_value_local(function);
        let output = schema.reserve_value_local(function);
        let piece = schema.reserve_value_local(function);
        let show_calendar_local = schema.reserve_i64_local(function);
        let digits_local = schema.reserve_i64_local(function);
        let show_offset_local = schema.reserve_i64_local(function);
        let mode_local = schema.reserve_i64_local(function);
        let unit_local = schema.reserve_i64_local(function);
        let show_time_zone_local = schema.reserve_i64_local(function);
        let precision_local = schema.reserve_i64_local(function);
        let increment_local = schema.reserve_i64_local(function);
        let quantum_local = schema.reserve_i64_local(function);
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        match options {
            ZonedDateTimeStringOptions::FromArgument => {
                self.emit_builtin_arg_to_value(0, &options_value, function)
            }
            ZonedDateTimeStringOptions::Undefined => options_value.set_undefined(function),
        }
        self.emit_temporal_duration_options_object(&options_value, function)?;
        self.emit_temporal_string_valued_option::<ShowCalendarName>(
            &options_value,
            show_calendar_local,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_CALENDARNAME_OPTION,
            function,
        )?;
        self.emit_temporal_plain_time_fractional_digits_option(
            &options_value,
            digits_local,
            function,
        )?;
        self.emit_temporal_string_valued_option::<ShowOffset>(
            &options_value,
            show_offset_local,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_OFFSET_OPTION,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            &options_value,
            TemporalRoundingMode::Trunc,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &options_value,
            TemporalUnitOptionProperty::SmallestUnit,
            unit_local,
            function,
        )?;
        self.emit_temporal_string_valued_option::<ShowTimeZone>(
            &options_value,
            show_time_zone_local,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_TIMEZONENAME_OPTION,
            function,
        )?;
        self.emit_temporal_seconds_string_precision(
            digits_local,
            unit_local,
            precision_local,
            increment_local,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_UNIT_OPTION,
            function,
        )?;
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        let mode = self.emit_temporal_validated_rounding_mode(mode_local, function)?;
        let quantum =
            self.emit_temporal_validated_instant_rounding_quantum(quantum_local, function)?;
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        // Internal BigInt rounding has no Instant range check. Both offset and
        // ISO projection consume this newly rounded exact epoch.
        let rounded_epoch =
            self.emit_temporal_round_instant_for_string(&instant, &quantum, &mode, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &rounded_epoch, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let fields = iso.fields();
        self.emit_temporal_iso_date_string(fields[0], fields[1], fields[2], &output, function)?;
        let text = schema.reserve_gc_local(function).initialize(
            output.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_append_gc_literal(&text, "T", function)?;
        self.emit_temporal_plain_time_record_to_string(
            &Self::temporal_plain_date_time_time_locals(fields),
            precision_local,
            &piece,
            function,
        )?;
        let time_text = schema.reserve_gc_local(function).initialize(
            piece.cast_reference::<StringValue>(schema, function),
            function,
        );
        text.replace(
            self.emit_concat_gc_strings(&text, &time_text, function),
            function,
        );
        time_text.clear(function);
        show_offset_local.load(function);
        function.instruction(&Instruction::I64Const(ShowOffset::Never.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_format_rounded_time_zone_offset(&snapshot, &piece, function)?;
        let offset_text = schema.reserve_gc_local(function).initialize(
            piece.cast_reference::<StringValue>(schema, function),
            function,
        );
        text.replace(
            self.emit_concat_gc_strings(&text, &offset_text, function),
            function,
        );
        offset_text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        show_time_zone_local.load(function);
        function.instruction(&Instruction::I64Const(ShowTimeZone::Never.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        show_time_zone_local.load(function);
        function.instruction(&Instruction::I64Const(ShowTimeZone::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_append_gc_literal(&text, "[!", function)?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_append_gc_literal(&text, "[", function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        text.replace(
            self.emit_concat_gc_strings(&text, zone.identifier(), function),
            function,
        );
        self.emit_temporal_append_gc_literal(&text, "]", function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output.set_reference(&text, schema, function);
        text.clear(function);
        self.emit_temporal_append_calendar_annotation(
            &calendar,
            show_calendar_local,
            &output,
            function,
        )?;
        self.completion().set_normal(&output, function);
        iso.release(self, function);
        snapshot.release(self, function);
        rounded_epoch.release(self, function);
        calendar.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        quantum.release(self, function);
        mode.release(self, function);
        branded.release(function);
        for local in [
            quantum_local,
            increment_local,
            precision_local,
            show_time_zone_local,
            unit_local,
            mode_local,
            show_offset_local,
            digits_local,
            show_calendar_local,
        ] {
            schema.release_i64_local(local, function);
        }
        piece.clear(function);
        output.clear(function);
        options_value.clear(function);
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_value_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            TEMPORAL_ZONED_DATE_TIME_VALUE_OF_MESSAGE,
            function,
        )
    }

    pub(super) fn emit_temporal_zoned_date_time_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let locales = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        self.emit_builtin_arg_to_value(0, &locales, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_intl_dtf_format_temporal_zoned(branded.record(), &locales, &options, function)?;
        branded.release(function);
        options.clear(function);
        locales.clear(function);
        Ok(())
    }
}
