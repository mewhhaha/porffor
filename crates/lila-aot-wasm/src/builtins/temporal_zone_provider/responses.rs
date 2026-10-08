//! Checked GC provider responses and Temporal instant-range certification.

use super::provider_wire::{NamedTimeZoneDataRequest, TemporalDataResponse};
use super::*;

pub(in crate::builtins) struct CheckedNamedTimeZoneInverseLocals {
    response: TemporalDataResponse,
    kind: I64Local,
    count: I64Local,
    transition: I64Local,
    before: I64Local,
    after: I64Local,
}
impl CheckedNamedTimeZoneInverseLocals {
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        for local in [
            self.after,
            self.before,
            self.transition,
            self.count,
            self.kind,
        ] {
            builder.runtime_schema().release_i64_local(local, function);
        }
        self.response.clear(builder, function);
    }
}

pub(super) struct CheckedTemporalPossibleEpochsLocals {
    data: CheckedNamedTimeZoneInverseLocals,
}
impl CheckedTemporalPossibleEpochsLocals {
    pub(super) fn kind(&self) -> I64Local {
        self.data.kind
    }
    pub(super) fn count(&self) -> I64Local {
        self.data.count
    }
    pub(super) fn transition(&self) -> I64Local {
        self.data.transition
    }
    pub(super) fn before(&self) -> I64Local {
        self.data.before
    }
    pub(super) fn after(&self) -> I64Local {
        self.data.after
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        self.data.release(builder, function);
    }
}

struct NamedTimeZoneInverseInput<'a> {
    zone: &'a ResolvedTemporalZoneLocals,
    local: &'a TemporalLocalCoordinateLocals,
}
impl NamedTimeZoneInverseInput<'_> {
    fn request(&self) -> NamedTimeZoneDataRequest<'_> {
        NamedTimeZoneDataRequest::Inverse {
            zone: self.zone,
            local: self.local,
        }
    }
    fn kind(&self) -> I64Local {
        self.zone.kind_local()
    }
    fn fixed_seconds(&self) -> I64Local {
        self.zone.fixed_offset_seconds()
    }
    fn seconds(&self) -> I64Local {
        self.local.floor_seconds()
    }
    fn nano(&self) -> I64Local {
        self.local.nanosecond()
    }
}

pub(crate) struct OptionalTemporalInstantLocals {
    present: I64Local,
    seconds: I64Local,
    nano: I64Local,
}
impl OptionalTemporalInstantLocals {
    pub(crate) fn emit_branches<P, A>(
        &self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
        present: P,
        absent: A,
    ) -> Result<(), EmitError>
    where
        P: FnOnce(
            &mut FunctionBuilder<'_>,
            &mut Function,
            &NormalizedTemporalInstantLocals,
        ) -> Result<(), EmitError>,
        A: FnOnce(&mut FunctionBuilder<'_>, &mut Function) -> Result<(), EmitError>,
    {
        (self.present).load(function);
        function.instruction(&Instruction::I32WrapI64);
        builder.open_frame(ControlFrameKind::If, function);
        // Only the present guard can expose this proof. The decoder has already
        // established range, strict direction and nanosecond normalization.
        let value =
            builder.emit_temporal_epoch_nanoseconds_bigint(self.seconds, self.nano, function);
        let epoch = builder.emit_temporal_instant_validated_epoch(&value, function)?;
        value.clear(function);
        let instant = NormalizedTemporalInstantLocals::from_epoch(epoch);
        present(builder, function, &instant)?;
        instant.release(builder, function);
        function.instruction(&Instruction::Else);
        absent(builder, function)?;
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(crate) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in [self.nano, self.seconds, self.present] {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_format_rounded_time_zone_offset(
        &mut self,
        snapshot: &TemporalZoneSnapshotLocals<'_>,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let rounded_offset_seconds_local = self.runtime_schema().reserve_i64_local(function);
        // FormatDateTimeUTCOffsetRounded uses nearest-minute halfExpand.
        // Getters continue to expose the exact historical seconds offset.
        (snapshot.offset_seconds()).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        (snapshot.offset_seconds()).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        (snapshot.offset_seconds()).load(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        (rounded_offset_seconds_local).store(function);
        (snapshot.offset_seconds()).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (rounded_offset_seconds_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (rounded_offset_seconds_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_format_fixed_time_zone_offset(
            rounded_offset_seconds_local,
            output,
            function,
        )?;
        self.runtime_schema()
            .release_i64_local(rounded_offset_seconds_local, function);
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_zone_snapshot<'a>(
        &mut self,
        zone: &'a ResolvedTemporalZoneLocals,
        instant: &'a NormalizedTemporalInstantLocals,
        function: &mut Function,
    ) -> Result<TemporalZoneSnapshotLocals<'a>, EmitError> {
        let offset = self.runtime_schema().reserve_i64_local(function);
        zone.kind_local().load(function);
        function.instruction(&Instruction::I64Const(TemporalZoneKind::Named.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let response = self.emit_named_time_zone_data_call(
            NamedTimeZoneDataRequest::Offset { zone, instant },
            function,
        )?;
        response.read_constant_word(0, offset, self, function);
        self.emit_temporal_require_offset_seconds(offset, function);
        response.clear(self, function);
        function.instruction(&Instruction::Else);
        zone.fixed_offset_seconds().load(function);
        offset.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(TemporalZoneSnapshotLocals {
            zone,
            epoch: instant,
            offset,
        })
    }

    pub(super) fn emit_temporal_checked_possible_epochs(
        &mut self,
        zone: &ResolvedTemporalZoneLocals,
        local: &TemporalLocalCoordinateLocals,
        function: &mut Function,
    ) -> Result<CheckedTemporalPossibleEpochsLocals, EmitError> {
        let data = self.emit_checked_temporal_named_inverse(
            NamedTimeZoneInverseInput { zone, local },
            function,
        )?;
        (zone.kind_local()).load(function);
        function.instruction(&Instruction::I64Const(TemporalZoneKind::Named.code()));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        let epoch = self.runtime_schema().reserve_i64_local(function);
        (local.floor_seconds()).load(function);
        (zone.fixed_offset_seconds()).load(function);
        function.instruction(&Instruction::I64Sub);
        (epoch).store(function);
        self.emit_temporal_validate_iso_days_pair(epoch, function)?;
        self.runtime_schema().release_i64_local(epoch, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (data.kind).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_validate_candidate_instants(&data, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(CheckedTemporalPossibleEpochsLocals { data })
    }

    fn emit_checked_temporal_named_inverse(
        &mut self,
        input: NamedTimeZoneInverseInput<'_>,
        function: &mut Function,
    ) -> Result<CheckedNamedTimeZoneInverseLocals, EmitError> {
        let result = CheckedNamedTimeZoneInverseLocals {
            response: self.emit_temporal_fixed_inverse_data(
                input.seconds(),
                input.nano(),
                input.fixed_seconds(),
                function,
            ),
            kind: self.runtime_schema().reserve_i64_local(function),
            count: self.runtime_schema().reserve_i64_local(function),
            transition: self.runtime_schema().reserve_i64_local(function),
            before: self.runtime_schema().reserve_i64_local(function),
            after: self.runtime_schema().reserve_i64_local(function),
        };
        for slot in [result.transition, result.before, result.after] {
            function.instruction(&Instruction::I64Const(0));
            slot.store(function);
        }
        input.kind().load(function);
        function.instruction(&Instruction::I64Const(TemporalZoneKind::Named.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let named = self.emit_named_time_zone_data_call(input.request(), function)?;
        result.response.replace_from(named, self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result
            .response
            .read_constant_word(0, result.kind, self, function);
        result
            .response
            .read_constant_word(8, result.count, self, function);
        (result.kind).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        self.emit_temporal_provider_corruption_if_i32(function);
        (result.kind).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_validate_gap_words(&result, &input, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_validate_candidate_words(&result, &input, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(result)
    }

    fn emit_temporal_validate_gap_words(
        &mut self,
        result: &CheckedNamedTimeZoneInverseLocals,
        input: &NamedTimeZoneInverseInput<'_>,
        function: &mut Function,
    ) {
        (result.response.length()).load(function);
        function.instruction(&Instruction::I64Const(40));
        function.instruction(&Instruction::I64Ne);
        (result.count).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        result
            .response
            .read_constant_word(16, result.transition, self, function);
        result
            .response
            .read_constant_word(24, result.before, self, function);
        result
            .response
            .read_constant_word(32, result.after, self, function);
        self.emit_temporal_require_raw_epoch_context(result.transition, function);
        self.emit_temporal_require_offset_seconds(result.before, function);
        self.emit_temporal_require_offset_seconds(result.after, function);
        (result.after).load(function);
        (result.before).load(function);
        function.instruction(&Instruction::I64LeS);
        self.emit_temporal_provider_corruption_if_i32(function);
        (result.after).load(function);
        (result.before).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64GtS);
        self.emit_temporal_provider_corruption_if_i32(function);
        // Integer boundaries make fractional local containment equivalent to
        // floor-seconds in [T+before,T+after).
        (input.seconds()).load(function);
        (result.transition).load(function);
        (result.before).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64LtS);
        (input.seconds()).load(function);
        (result.transition).load(function);
        (result.after).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        // Global emptiness/sole-nearest-endpoint authority comes from the
        // immutable pinned provider certificate, not this tuple alone.
    }

    fn emit_temporal_validate_candidate_words(
        &mut self,
        result: &CheckedNamedTimeZoneInverseLocals,
        input: &NamedTimeZoneInverseInput<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (result.count).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtU);
        (result.count).load(function);
        function.instruction(&Instruction::I64Const(172_799));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        (result.count).load(function);
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Add);
        (result.response.length()).load(function);
        function.instruction(&Instruction::I64Ne);
        self.emit_temporal_provider_corruption_if_i32(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let pointer = self.runtime_schema().reserve_i64_local(function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let nano = self.runtime_schema().reserve_i64_local(function);
        let offset = self.runtime_schema().reserve_i64_local(function);
        let previous_seconds = self.runtime_schema().reserve_i64_local(function);
        let previous_nano = self.runtime_schema().reserve_i64_local(function);
        for slot in [index, previous_seconds, previous_nano] {
            function.instruction(&Instruction::I64Const(0));
            (slot).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (index).load(function);
        (result.count).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_candidate_coordinate(index, pointer, function);
        result
            .response
            .read_word_at(pointer, 0, seconds, self, function);
        result
            .response
            .read_word_at(pointer, 8, nano, self, function);
        result
            .response
            .read_word_at(pointer, 16, offset, self, function);
        self.emit_temporal_require_raw_epoch_context(seconds, function);
        self.emit_temporal_require_canonical_nano(nano, function);
        self.emit_temporal_require_offset_seconds(offset, function);
        (seconds).load(function);
        (offset).load(function);
        function.instruction(&Instruction::I64Add);
        (input.seconds()).load(function);
        function.instruction(&Instruction::I64Ne);
        (nano).load(function);
        (input.nano()).load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        (index).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (seconds).load(function);
        (previous_seconds).load(function);
        function.instruction(&Instruction::I64LtS);
        (seconds).load(function);
        (previous_seconds).load(function);
        function.instruction(&Instruction::I64Eq);
        (nano).load(function);
        (previous_nano).load(function);
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_copy_pair(seconds, nano, previous_seconds, previous_nano, function);
        self.emit_temporal_advance_cursor(index, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for slot in [
            previous_nano,
            previous_seconds,
            offset,
            nano,
            seconds,
            pointer,
            index,
        ] {
            self.runtime_schema().release_i64_local(slot, function);
        }
        Ok(())
    }
    fn emit_temporal_validate_candidate_instants(
        &mut self,
        result: &CheckedNamedTimeZoneInverseLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index = self.runtime_schema().reserve_i64_local(function);
        let pointer = self.runtime_schema().reserve_i64_local(function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let nano = self.runtime_schema().reserve_i64_local(function);
        // Certify the complete candidate list before selection can expose an Instant.
        function.instruction(&Instruction::I64Const(0));
        (index).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (index).load(function);
        (result.count).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_candidate_coordinate(index, pointer, function);
        result
            .response
            .read_word_at(pointer, 0, seconds, self, function);
        result
            .response
            .read_word_at(pointer, 8, nano, self, function);
        self.emit_temporal_require_instant_pair(seconds, nano, function)?;
        self.emit_temporal_advance_cursor(index, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for slot in [nano, seconds, pointer, index] {
            self.runtime_schema().release_i64_local(slot, function);
        }
        Ok(())
    }
    fn emit_temporal_candidate_coordinate(
        &self,
        index: I64Local,
        pointer: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(16));
        (index).load(function);
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (pointer).store(function);
    }
    pub(super) fn emit_temporal_candidate_word(
        &self,
        result: &CheckedTemporalPossibleEpochsLocals,
        index: I64Local,
        offset: i64,
        out: I64Local,
        function: &mut Function,
    ) {
        let coordinate = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_candidate_coordinate(index, coordinate, function);
        result
            .data
            .response
            .read_word_at(coordinate, offset, out, self, function);
        self.runtime_schema()
            .release_i64_local(coordinate, function);
    }
    pub(in crate::builtins) fn emit_temporal_zone_transition(
        &mut self,
        zone: &ResolvedTemporalZoneLocals,
        instant: &NormalizedTemporalInstantLocals,
        direction: &TemporalTransitionDirectionLocals,
        function: &mut Function,
    ) -> Result<OptionalTemporalInstantLocals, EmitError> {
        let present = self.runtime_schema().reserve_i64_local(function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let nano = self.runtime_schema().reserve_i64_local(function);
        for slot in [present, seconds, nano] {
            function.instruction(&Instruction::I64Const(0));
            (slot).store(function);
        }
        (zone.kind).load(function);
        function.instruction(&Instruction::I64Const(TemporalZoneKind::Named.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let response = self.emit_named_time_zone_data_call(
            NamedTimeZoneDataRequest::Transition {
                zone,
                instant,
                direction,
            },
            function,
        )?;
        response.read_constant_word(0, present, self, function);
        response.read_constant_word(8, seconds, self, function);
        function.instruction(&Instruction::I64Const(0));
        nano.store(function);
        (present).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        self.emit_temporal_provider_corruption_if_i32(function);
        (present).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (seconds).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_temporal_provider_corruption_if_i32(function);
        function.instruction(&Instruction::Else);
        // Invalid host results trap; user input is already range-proved. No
        // extra observable RangeError is introduced by transition decoding.
        self.emit_temporal_instant_pair_invalid_i32(seconds, nano, function);
        self.emit_temporal_provider_corruption_if_i32(function);
        (direction.local).load(function);
        function.instruction(&Instruction::I64Const(
            lila_intl::NamedTimeZoneTransitionDirection::Next as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (seconds).load(function);
        instant.floor_seconds().load(function);
        function.instruction(&Instruction::I64LeS);
        self.emit_temporal_provider_corruption_if_i32(function);
        function.instruction(&Instruction::Else);
        (seconds).load(function);
        instant.floor_seconds().load(function);
        function.instruction(&Instruction::I64GtS);
        (seconds).load(function);
        instant.floor_seconds().load(function);
        function.instruction(&Instruction::I64Eq);
        instant.nanosecond().load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        response.clear(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        Ok(OptionalTemporalInstantLocals {
            present,
            seconds,
            nano,
        })
    }
    pub(in crate::builtins) fn emit_temporal_instant_pair_invalid_i32(
        &self,
        seconds: I64Local,
        nano: I64Local,
        function: &mut Function,
    ) {
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(-8_640_000_000_000));
        function.instruction(&Instruction::I64LtS);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(8_640_000_000_000));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(8_640_000_000_000));
        function.instruction(&Instruction::I64Eq);
        (nano).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }
}
