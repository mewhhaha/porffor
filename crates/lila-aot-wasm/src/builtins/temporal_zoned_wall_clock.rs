//! Wall-clock projections of zoned values: `Temporal.ZonedDateTime.prototype.
//! {toPlainTime,withPlainTime}` and `Temporal.Now.{plainDateISO,
//! plainDateTimeISO,plainTimeISO}`, all through `GetISODateTimeFor`,
//! `GetStartOfDay` and `GetEpochNanosecondsFor` on the time-zone boundary.

use super::super::*;
use super::temporal::TemporalEpochNanosecondsRecord;
use super::temporal_options::TemporalConversionOverflowOptions;
use super::temporal_time_zone::TemporalDisambiguationSource;

/// Which `Temporal.Now` wall-clock reader is being emitted.
#[derive(Clone, Copy)]
pub(super) enum TemporalNowWallClock {
    PlainDate,
    PlainDateTime,
    PlainTime,
}

impl<'a> FunctionBuilder<'a> {
    /// `GetISODateTimeFor` of the receiver's record into nine fields.
    fn emit_temporal_zoned_date_time_wall_fields(
        &mut self,
        record_local: u32,
        fields: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_pair(
            record_local,
            TemporalEpochNanosecondsRecord::ZonedDateTime,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_iso_date_time_for(
            time_zone_local,
            seconds_local,
            subsecond_local,
            fields,
            function,
        )?;
        for local in [subsecond_local, seconds_local, time_zone_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Temporal.ZonedDateTime.prototype.toPlainTime`.
    pub(super) fn emit_temporal_zoned_date_time_to_plain_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let fields = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_temporal_zoned_date_time_wall_fields(record_local, &fields, function)?;
        self.emit_alloc_temporal_plain_time(
            &Self::temporal_plain_date_time_time_locals(&fields),
            None,
            function,
        )?;
        self.release_temporal_plain_date_time_field_locals(fields);
        self.release_temp_local(record_local);
        Ok(())
    }

    /// `Temporal.ZonedDateTime.prototype.withPlainTime([plainTimeLike])`: the
    /// start of the local day when the time is absent, otherwise
    /// `GetEpochNanosecondsFor(…, compatible)` of the local date at that time.
    pub(super) fn emit_temporal_zoned_date_time_with_plain_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let fields = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_temporal_zoned_date_time_wall_fields(record_local, &fields, function)?;
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_start_of_day(
            time_zone_payload_local,
            [fields[0], fields[1], fields[2]],
            seconds_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::Else);
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            TemporalConversionOverflowOptions::Omit,
            &Self::temporal_plain_date_time_time_locals(&fields),
            function,
        )?;
        self.emit_temporal_epoch_for_iso_date_time(
            time_zone_payload_local,
            &fields,
            TemporalDisambiguationSource::Compatible,
            seconds_local,
            subsecond_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_local,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(fields);
        for local in [
            prototype_local,
            epoch_tag_local,
            epoch_payload_local,
            subsecond_local,
            seconds_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            argument_tag_local,
            argument_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Temporal.Now.plainDateISO`, `plainDateTimeISO` and `plainTimeISO`:
    /// `SystemDateTime(temporalTimeZoneLike)`, then the ISO record.
    pub(super) fn emit_temporal_now_wall_clock(
        &mut self,
        reader: TemporalNowWallClock,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let fields = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_now_time_zone_argument(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_temporal_now_epoch_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        )?;
        self.emit_temporal_iso_date_time_for(
            time_zone_payload_local,
            seconds_local,
            subsecond_local,
            &fields,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        match reader {
            TemporalNowWallClock::PlainDate => {
                function.instruction(&Instruction::GlobalGet(
                    TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(prototype_local));
                self.emit_alloc_temporal_plain_date(
                    fields[0],
                    fields[1],
                    fields[2],
                    calendar_payload_local,
                    prototype_local,
                    function,
                )?;
            }
            TemporalNowWallClock::PlainDateTime => {
                self.emit_alloc_temporal_plain_date_time(
                    &fields,
                    calendar_payload_local,
                    None,
                    function,
                )?;
            }
            TemporalNowWallClock::PlainTime => {
                self.emit_alloc_temporal_plain_time(
                    &Self::temporal_plain_date_time_time_locals(&fields),
                    None,
                    function,
                )?;
            }
        }

        self.release_temporal_plain_date_time_field_locals(fields);
        for local in [
            prototype_local,
            calendar_payload_local,
            subsecond_local,
            seconds_local,
            time_zone_tag_local,
            time_zone_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
