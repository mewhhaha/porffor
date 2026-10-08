//! Correlated named-zone day boundaries and strict exact transition queries.

use super::super::*;
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::temporal_zone_provider::{
    NormalizedTemporalInstantLocals, RegulatedTemporalIsoRecordLocals, TemporalIsoDateLocals,
    TemporalZoneSnapshotLocals, TemporalZonedAllocationInput,
};
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

const SECONDS_PER_DAY: i64 = NANOSECONDS_PER_TEMPORAL_DAY / 1_000_000_000;
const MAXIMUM_DAY_SPAN_NANOSECONDS: i64 = 3 * NANOSECONDS_PER_TEMPORAL_DAY;

/// Minted only from the actual Instant projection's date. Arbitrary skipped
/// ISO dates cannot produce a positive day span or an unrelated rounding origin.
#[must_use]
pub(super) struct TemporalZoneDayBoundariesLocals<'epoch> {
    origin: &'epoch NormalizedTemporalInstantLocals,
    iso: RegulatedTemporalIsoRecordLocals,
    date: TemporalIsoDateLocals,
    next_date: TemporalIsoDateLocals,
    start: NormalizedTemporalInstantLocals,
    end: NormalizedTemporalInstantLocals,
    span_nanoseconds: I64Local,
}

impl TemporalZoneDayBoundariesLocals<'_> {
    pub(super) fn origin(&self) -> &NormalizedTemporalInstantLocals {
        self.origin
    }
    pub(super) fn start(&self) -> &NormalizedTemporalInstantLocals {
        &self.start
    }
    pub(super) fn end(&self) -> &NormalizedTemporalInstantLocals {
        &self.end
    }
    pub(super) fn span_nanoseconds(&self) -> I64Local {
        self.span_nanoseconds
    }

    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        self.end.release(builder, function);
        self.start.release(builder, function);
        self.next_date.release(builder, function);
        self.date.release(builder, function);
        self.iso.release(builder, function);
        builder
            .runtime_schema()
            .release_i64_local(self.span_nanoseconds, function);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_temporal_zone_day_boundaries<'epoch>(
        &mut self,
        snapshot: &TemporalZoneSnapshotLocals<'epoch>,
        function: &mut Function,
    ) -> Result<TemporalZoneDayBoundariesLocals<'epoch>, EmitError> {
        let span_nanoseconds = self.runtime_schema().reserve_i64_local(function);
        let iso = self.emit_temporal_zone_snapshot_iso_record(snapshot, function)?;
        let date = self.emit_temporal_iso_date_from_record(&iso, function);
        let next_date = self.emit_temporal_add_iso_date_days(&date, 1, function)?;
        let start = self.emit_temporal_get_start_of_day(snapshot.zone(), &date, function)?;
        let end = self.emit_temporal_get_start_of_day(snapshot.zone(), &next_date, function)?;
        let origin = snapshot.epoch();

        // Check seconds before multiplying: even a corrupt pair must not wrap
        // into a plausible positive duration. Each offset is strictly within
        // one day, so a certified calendar day's elapsed span is below 3days.
        (end.floor_seconds()).load(function);
        (start.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Sub);
        (span_nanoseconds).store(function);
        (span_nanoseconds).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (span_nanoseconds).load(function);
        function.instruction(&Instruction::I64Const(3 * SECONDS_PER_DAY));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        (span_nanoseconds).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (end.nanosecond()).load(function);
        (start.nanosecond()).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        (span_nanoseconds).store(function);
        (span_nanoseconds).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LeS);
        (span_nanoseconds).load(function);
        function.instruction(&Instruction::I64Const(MAXIMUM_DAY_SPAN_NANOSECONDS));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);

        // The actual projection proves start <= origin. A backward transition
        // after midnight can revisit this date after the next date's earliest
        // start; day rounding clamps progress to end-1ns before scaling.
        (origin.floor_seconds()).load(function);
        (start.floor_seconds()).load(function);
        function.instruction(&Instruction::I64LtS);
        (origin.floor_seconds()).load(function);
        (start.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Eq);
        (origin.nanosecond()).load(function);
        (start.nanosecond()).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.emit_temporal_provider_corruption_if_i32(function);
        Ok(TemporalZoneDayBoundariesLocals {
            origin,
            iso,
            date,
            next_date,
            start,
            end,
            span_nanoseconds,
        })
    }

    pub(super) fn emit_temporal_zoned_date_time_get_time_zone_transition(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let options = schema.reserve_value_local(function);
        let direction_value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        self.emit_builtin_arg_to_value(0, &options, function);
        options.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        direction_value.copy_from(&options, function);
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(options.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_TRANSITION_DIRECTION_MUST_BE_A_STRING_OR_OBJECT,
            function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_duration_option_get(&options, "direction", &direction_value, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_value_to_string_payload(&direction_value, &pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().copy_from(&pending, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        let direction = self.emit_temporal_transition_direction(&string, function)?;
        string.clear(function);
        pending.clear(function);
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        let transition =
            self.emit_temporal_zone_transition(&zone, &instant, &direction, function)?;
        transition.emit_branches(
            self,
            function,
            |builder, function, result| {
                builder.emit_alloc_temporal_zoned_date_time(
                    TemporalZonedAllocationInput::new(result, &zone, &calendar),
                    TemporalPrototypeSource::Intrinsic,
                    function,
                )
            },
            |builder, function| {
                builder
                    .completion()
                    .value()
                    .set_scalar(ScalarValue::Null, function);
                builder
                    .completion()
                    .set_normal(builder.completion().value(), function);
                Ok(())
            },
        )?;
        transition.release(self, function);
        calendar.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        direction.release(self, function);
        branded.release(function);
        direction_value.clear(function);
        options.clear(function);
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_hours_in_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let number_bits = self.runtime_schema().reserve_i64_local(function);
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &instant, function)?;
        let boundaries = self.emit_temporal_zone_day_boundaries(&snapshot, function)?;
        boundaries.span_nanoseconds().load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(Ieee64::from(3_600_000_000_000.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        number_bits.store(function);
        self.completion().value().set_number(number_bits, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        boundaries.release(self, function);
        snapshot.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        branded.release(function);
        self.runtime_schema()
            .release_i64_local(number_bits, function);
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_start_of_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &instant, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let date = self.emit_temporal_iso_date_from_record(&iso, function);
        let start = self.emit_temporal_get_start_of_day(&zone, &date, function)?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&start, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        start.release(self, function);
        date.release(self, function);
        iso.release(self, function);
        snapshot.release(self, function);
        calendar.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        branded.release(function);
        Ok(())
    }
}
