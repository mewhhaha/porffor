//! `TemporalInstantToString` for `toString` (8.3.11) and `toJSON` (8.3.13).

use super::super::temporal_options::{TemporalRoundingMode, TemporalUnitOptionProperty};
use super::super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::super::temporal_plain_time_methods::TEMPORAL_PRECISION_AUTO;
use super::*;

/// Which caller of `TemporalInstantToString` is emitting.
///
/// Only `toString` reads an options bag. `toJSON` is
/// `TemporalInstantToString(instant, undefined, auto)` and must read no
/// argument at all: `toJSON/basic.js` passes a Proxy that throws on any
/// property access.
#[derive(Clone, Copy)]
pub(in crate::builtins) enum InstantStringSource {
    ToString,
    ToJson,
}

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_temporal_instant_to_string(
        &mut self,
        source: InstantStringSource,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let precision_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let within_day_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let fields = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::LocalSet(precision_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(quantum_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(time_zone_tag_local));
        match source {
            InstantStringSource::ToJson => {}
            InstantStringSource::ToString => {
                self.emit_builtin_arg_to_locals(
                    0,
                    options_payload_local,
                    options_tag_local,
                    function,
                );
                self.emit_temporal_duration_options_object(
                    options_payload_local,
                    options_tag_local,
                    function,
                )?;
                // Options are read and independently validated in
                // alphabetical order; only then is smallestUnit checked
                // against the `time` category and the time zone resolved.
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
                    TemporalUnitOptionProperty::SmallestUnit,
                    unit_local,
                    function,
                )?;
                self.emit_temporal_duration_option_get(
                    options_payload_local,
                    options_tag_local,
                    "timeZone",
                    time_zone_payload_local,
                    time_zone_tag_local,
                    function,
                )?;
                // `ValidateTemporalUnitValue(smallestUnit, time)` plus the
                // explicit `hour` rejection: the admitted range starts at
                // `minute`.
                self.emit_temporal_seconds_string_precision(
                    digits_local,
                    unit_local,
                    precision_local,
                    increment_local,
                    "Invalid Temporal.Instant unit option",
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(time_zone_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                // `ToTemporalTimeZoneIdentifier`, then the fixed offset the
                // resolved identifier denotes.
                self.emit_temporal_zoned_date_time_time_zone(
                    time_zone_payload_local,
                    time_zone_tag_local,
                    function,
                )?;
                self.emit_temporal_fixed_time_zone_offset_seconds(
                    time_zone_payload_local,
                    offset_seconds_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.emit_temporal_plain_time_rounding_quantum(
                    unit_local,
                    increment_local,
                    quantum_local,
                    function,
                );
            }
        }

        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_floor_milliseconds(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            milliseconds_local,
            remainder_local,
            function,
        );

        // `RoundTemporalInstant` is `RoundNumberToIncrementAsIfPositive` on
        // the whole epoch value. Every admitted quantum divides a day an even
        // number of times, so rounding the non-negative time within the UTC
        // day is exact and keeps `halfEven` ties on the global parity.
        // Milliseconds and the sub-millisecond remainder stay exact at both
        // epoch limits, where the full nanosecond count does not fit an f64.
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

        // `GetISODateTimeFor(outputTimeZone, roundedNs)`: shift the rounded
        // exact time by the zone's offset (zero for the implicit UTC).
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
        for field in &fields[..7] {
            function.instruction(&Instruction::LocalGet(*field));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(*field));
        }
        function.instruction(&Instruction::LocalGet(fields[1]));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fields[1]));
        // Rounding can carry `within_day` to exactly one day; the remainder
        // modulo a day keeps the sub-millisecond digits of the carried value.
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
        // No time zone: the `Z` designator. A time zone:
        // `FormatDateTimeUTCOffsetRounded`, which for the minute-aligned
        // offsets this backend resolves is the offset itself.
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Z")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_format_fixed_time_zone_offset(
            offset_seconds_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_date_time_field_locals(fields);
        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            local_time_payload_local,
            within_day_local,
            day_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            offset_seconds_local,
            quantum_local,
            increment_local,
            precision_local,
            time_zone_tag_local,
            time_zone_payload_local,
            unit_local,
            mode_local,
            digits_local,
            options_tag_local,
            options_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
