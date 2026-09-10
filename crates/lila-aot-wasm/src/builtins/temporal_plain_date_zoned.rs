//! PlainDate conversion to a zoned date-time with an optional wall-clock time.

use super::super::*;
use super::temporal_options::TemporalConversionOverflowOptions;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_temporal_plain_date_to_zoned_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();
        let time_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, field_locals[0]),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, field_locals[1]),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, field_locals[2]),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(time_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(time_zone_tag_local));
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("timeZone")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            key_local,
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(time_zone_tag_local));
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        // Time zone conversion precedes the plainTime getter.
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("plainTime")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            key_local,
            time_payload_local,
            time_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        for local in time_locals {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(time_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_temporal_time(
            time_payload_local,
            time_tag_local,
            TemporalConversionOverflowOptions::Omit,
            &time_locals,
            function,
        )?;
        self.emit_temporal_reject_date_time_lower_bound(&field_locals, function)?;
        function.instruction(&Instruction::End);
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(field_locals[3]));
        function.instruction(&Instruction::I64Const(3600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[4]));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[5]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(time_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_zoned_date_time_day_boundary(
            seconds_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(field_locals[6]));
        function.instruction(&Instruction::I64Const(1000000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[7]));
        function.instruction(&Instruction::I64Const(1000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[8]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.release_temporal_plain_date_time_field_locals(field_locals);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(days_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        self.release_temp_local(offset_seconds_local);
        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(time_tag_local);
        self.release_temp_local(time_payload_local);
        self.release_temp_local(time_zone_tag_local);
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }
}
