//! Date owns compatible selection; the native provider owns certified zone data.
use super::*;
use crate::builtins::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::builtins::system_time_zone::ResolvedSystemTimeZoneLocals;
use lila_intl::{IntlHostOp, NamedTimeZoneOffsetSeconds, SortedNamedTimeZoneCandidates};

impl FunctionBuilder<'_> {
    pub(super) fn emit_date_require_offset_seconds(
        &self,
        seconds: I64Local,
        function: &mut Function,
    ) {
        seconds.load(function);
        function.instruction(&Instruction::I64Const(-i64::from(
            NamedTimeZoneOffsetSeconds::MAX_ABSOLUTE_SECONDS,
        )));
        function.instruction(&Instruction::I64LtS);
        seconds.load(function);
        function.instruction(&Instruction::I64Const(i64::from(
            NamedTimeZoneOffsetSeconds::MAX_ABSOLUTE_SECONDS,
        )));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.emit_date_zone_corruption_if(function);
    }

    pub(super) fn emit_date_zone_corruption_if(&self, function: &mut Function) {
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_date_named_zone_reply(
        &mut self,
        operation: IntlHostOp,
        zone: &ResolvedSystemTimeZoneLocals,
        seconds: I64Local,
        nanos: I64Local,
        function: &mut Function,
    ) -> Result<GcLocal<crate::gc_types::ByteArray>, EmitError> {
        let schema = self.runtime_schema();
        let message = IntlByteArrayBuilder::with_operation(operation, schema, function);
        message.append_u64(seconds, schema, function);
        message.append_u64(nanos, schema, function);
        message.append_u64_constant(0, schema, function);
        message.append_utf8(zone.identifier(), schema, function);
        let request = message.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&request, function)?;
        reply.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.emit_date_zone_corruption_if(function);
        let retained = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        reply.clear(function);
        request.clear(function);
        Ok(retained)
    }

    pub(super) fn emit_date_named_offset_into(
        &mut self,
        zone: &ResolvedSystemTimeZoneLocals,
        instant: &DateUtcCoordinateLocals,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reply = self.emit_date_named_zone_reply(
            IntlHostOp::NamedTimeZoneOffset,
            zone,
            instant.floor_seconds(),
            instant.nanosecond(),
            function,
        )?;
        let reader = IntlByteArrayReader::new(&reply, schema, function);
        reader.read_u64(output, schema, function);
        self.emit_date_require_offset_seconds(output, function);
        reader.finish(schema, function);
        reply.clear(function);
        Ok(())
    }

    pub(super) fn emit_date_named_compatible_offset_into(
        &mut self,
        zone: &ResolvedSystemTimeZoneLocals,
        local: &DateLocalCoordinateLocals,
        output: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let reply = self.emit_date_named_zone_reply(
            IntlHostOp::PossibleNamedTimeZoneEpochs,
            zone,
            local.floor_seconds(),
            local.nanosecond(),
            function,
        )?;
        let reader = IntlByteArrayReader::new(&reply, schema, function);
        let kind = schema.reserve_i64_local(function);
        let count = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let seconds = schema.reserve_i64_local(function);
        let nanos = schema.reserve_i64_local(function);
        let offset = schema.reserve_i64_local(function);
        let previous = schema.reserve_i64_local(function);
        reader.read_u64(kind, schema, function);
        reader.read_u64(count, schema, function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        self.emit_date_zone_corruption_if(function);
        kind.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        count.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_date_zone_corruption_if(function);
        reader.read_u64(seconds, schema, function);
        reader.read_u64(offset, schema, function);
        reader.read_u64(nanos, schema, function);
        self.emit_date_require_offset_seconds(offset, function);
        self.emit_date_require_offset_seconds(nanos, function);
        nanos.load(function);
        offset.load(function);
        function.instruction(&Instruction::I64LeS);
        self.emit_date_zone_corruption_if(function);
        // Rewrite containment to subtract bounded offsets from the proved local
        // coordinate. A malformed transition can never overflow the check.
        seconds.load(function);
        local.floor_seconds().load(function);
        offset.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtS);
        self.emit_date_zone_corruption_if(function);
        seconds.load(function);
        local.floor_seconds().load(function);
        nanos.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64LeS);
        self.emit_date_zone_corruption_if(function);
        // The certified catalogue proves global gap emptiness. Date's compatible
        // UTC interpretation uses the before offset, including at range edges.
        offset.load(function);
        output.store(function);
        function.instruction(&Instruction::Else);
        count.load(function);
        function.instruction(&Instruction::I64Eqz);
        count.load(function);
        function.instruction(&Instruction::I64Const(
            SortedNamedTimeZoneCandidates::maximum_count() as i64,
        ));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.emit_date_zone_corruption_if(function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, function);
        reader.read_u64(seconds, schema, function);
        reader.read_u64(nanos, schema, function);
        reader.read_u64(offset, schema, function);
        self.emit_date_require_offset_seconds(offset, function);
        nanos.load(function);
        local.nanosecond().load(function);
        function.instruction(&Instruction::I64Ne);
        self.emit_date_zone_corruption_if(function);
        seconds.load(function);
        local.floor_seconds().load(function);
        offset.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Ne);
        self.emit_date_zone_corruption_if(function);
        index.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        offset.load(function);
        output.store(function);
        function.instruction(&Instruction::Else);
        seconds.load(function);
        previous.load(function);
        function.instruction(&Instruction::I64LeS);
        self.emit_date_zone_corruption_if(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        seconds.load(function);
        previous.store(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        reader.finish(schema, function);
        reply.clear(function);
        for value in [previous, offset, nanos, seconds, index, count, kind] {
            schema.release_i64_local(value, function);
        }
        Ok(())
    }
}
