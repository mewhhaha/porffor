//! Zoned date-time string formatting after exact epoch rounding.

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, StringValuedOption, TemporalRoundingMode, TemporalUnitOptionProperty,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;

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

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_temporal_zoned_date_time_to_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let show_calendar_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let show_offset_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let show_time_zone_local = self.reserve_temp_local();
        let precision_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let within_day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let fields = self.reserve_temporal_plain_date_time_field_locals();
        let components = [
            fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6],
        ];

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
        self.emit_temporal_duration_options_object(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_temporal_string_valued_option::<ShowCalendarName>(
            options_payload_local,
            options_tag_local,
            show_calendar_local,
            "Temporal.ZonedDateTime options must be an object or undefined",
            "Invalid Temporal.ZonedDateTime calendarName option",
            function,
        )?;
        self.emit_temporal_plain_time_fractional_digits_option(
            options_payload_local,
            options_tag_local,
            digits_local,
            function,
        )?;
        self.emit_temporal_string_valued_option::<ShowOffset>(
            options_payload_local,
            options_tag_local,
            show_offset_local,
            "Temporal.ZonedDateTime options must be an object or undefined",
            "Invalid Temporal.ZonedDateTime offset option",
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
            TemporalUnitOptionProperty::SmallestUnit,
            unit_local,
            function,
        )?;
        self.emit_temporal_string_valued_option::<ShowTimeZone>(
            options_payload_local,
            options_tag_local,
            show_time_zone_local,
            "Temporal.ZonedDateTime options must be an object or undefined",
            "Invalid Temporal.ZonedDateTime timeZoneName option",
            function,
        )?;
        self.emit_temporal_seconds_string_precision(
            digits_local,
            unit_local,
            precision_local,
            increment_local,
            "Invalid Temporal.ZonedDateTime unit option",
            function,
        )?;
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_zoned_date_time_local_components(
            record_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            offset_seconds_local,
            time_zone_payload_local,
            local_time_payload_local,
            components,
            function,
        )?;

        // RoundTemporalInstant uses RoundNumberToIncrementAsIfPositive. A UTC
        // day contains an even number of every permitted formatting quantum,
        // so rounding the nonnegative remainder also preserves halfEven ties.
        // Milliseconds and the submillisecond remainder stay exact at both
        // epoch limits; converting the full epoch nanoseconds to f64 would not.
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(within_day_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day_local));
        self.emit_temporal_plain_time_round_nanoseconds(
            within_day_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(local_time_payload_local));
        self.emit_date_components_from_time(
            local_time_payload_local,
            fields[0],
            fields[1],
            fields[2],
            fields[3],
            fields[4],
            fields[5],
            fields[6],
            function,
        );
        for field in components {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(field));
        }
        function.instruction(&Instruction::LocalGet(fields[1]));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fields[1]));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));
        for (destination, divide) in [(fields[7], true), (fields[8], false)] {
            function.instruction(&Instruction::LocalGet(remainder_local));
            function.instruction(&Instruction::I64Const(1_000));
            function.instruction(if divide {
                &Instruction::I64DivU
            } else {
                &Instruction::I64RemU
            });
            function.instruction(&Instruction::LocalSet(destination));
        }
        self.emit_temporal_iso_date_string(
            fields[0],
            fields[1],
            fields[2],
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("T")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_temporal_plain_time_record_to_string(
            &Self::temporal_plain_date_time_time_locals(&fields),
            precision_local,
            piece_payload_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(show_offset_local));
        function.instruction(&Instruction::I64Const(ShowOffset::Never.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_format_fixed_time_zone_offset(
            offset_seconds_local,
            piece_payload_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(show_time_zone_local));
        function.instruction(&Instruction::I64Const(ShowTimeZone::Never.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(show_time_zone_local));
        function.instruction(&Instruction::I64Const(ShowTimeZone::Critical.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("[!")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            time_zone_payload_local,
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
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );
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
        self.release_temporal_plain_date_time_field_locals(fields);
        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            calendar_payload_local,
            within_day_local,
            day_local,
            local_time_payload_local,
            time_zone_payload_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            quantum_local,
            increment_local,
            precision_local,
            show_time_zone_local,
            unit_local,
            mode_local,
            show_offset_local,
            digits_local,
            show_calendar_local,
            options_tag_local,
            options_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
