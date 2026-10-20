//! Exact zoned calendar arithmetic. Date fields use the retained calendar and
//! local inverse; time fields are elapsed time on the selected epoch.

use super::super::*;
use super::temporal_zone_provider::{
    BrandedTemporalZonedRecordLocals, NormalizedTemporalInstantLocals,
    RegulatedTemporalIsoRecordLocals, ResolvedTemporalZoneLocals, TemporalCalendarSlotLocals,
    TemporalOverflowLocals, TemporalZonedAllocationInput,
};
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

mod difference;
mod relative;
mod rounding;
pub(super) use relative::{
    TemporalZonedRelativeLargestUnit, TemporalZonedRelativeRoundingSettings,
};
pub(super) use rounding::CompletedTemporalStringRoundingLocals;

#[derive(Clone, Copy)]
pub(super) enum TemporalZonedArithmeticOperation {
    Add,
    Subtract,
}

#[derive(Clone, Copy)]
pub(super) enum TemporalZonedDifferenceOperation {
    Until,
    Since,
}

/// The pair has been produced by an exact epoch operation, rather than by
/// relabelling arbitrary duration or civil-field locals. It has Euclidean
/// nanos, but carries no Instant range proof. Arbitrary duration arithmetic
/// must validate its result; string rounding has a separate grid proof.
pub(super) struct CompletedTemporalEpochArithmeticLocals {
    floor_seconds: I64Local,
    nanosecond: I64Local,
}

impl CompletedTemporalEpochArithmeticLocals {
    pub(super) fn floor_seconds(&self) -> I64Local {
        self.floor_seconds
    }
    pub(super) fn nanosecond(&self) -> I64Local {
        self.nanosecond
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder
            .runtime_schema()
            .release_i64_local(self.nanosecond, function);
        builder
            .runtime_schema()
            .release_i64_local(self.floor_seconds, function);
    }
}

/// Completed balanced ISO arithmetic. Calendar addition checks its prescribed
/// local range before creating this proof; a difference/window probe performs
/// the range checks prescribed by its subsequent inverse operation instead.
/// Other modules cannot manufacture this proof from arbitrary field IDs.
pub(super) struct CompletedTemporalIsoArithmeticLocals {
    fields: [I64Local; 9],
}

impl CompletedTemporalIsoArithmeticLocals {
    pub(super) fn fields(&self) -> &[I64Local; 9] {
        &self.fields
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in self.fields.into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

/// Completed duration conversion and exact zoned difference produce this
/// signed internal duration. Its time pair uses the Duration truncation
/// convention, which is deliberately distinct from a floor-normalized epoch.
pub(super) struct TemporalZonedInternalDurationLocals {
    date: [I64Local; 4],
    seconds: I64Local,
    subsecond: I64Local,
}

impl TemporalZonedInternalDurationLocals {
    pub(super) fn seconds(&self) -> I64Local {
        self.seconds
    }
    pub(super) fn subsecond(&self) -> I64Local {
        self.subsecond
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder
            .runtime_schema()
            .release_i64_local(self.subsecond, function);
        builder
            .runtime_schema()
            .release_i64_local(self.seconds, function);
        for local in self.date.into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_temporal_zoned_internal_duration(
        &mut self,
        operation: TemporalZonedArithmeticOperation,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<TemporalZonedInternalDurationLocals, EmitError> {
        let date = std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_to_temporal_duration(input, &fields, function)?;
        match operation {
            TemporalZonedArithmeticOperation::Add => {}
            TemporalZonedArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&fields, function);
            }
        }
        self.emit_temporal_relative_internal_from_fields(
            &fields, date, seconds, subsecond, function,
        );
        self.release_temporal_duration_field_locals(fields, function);
        Ok(TemporalZonedInternalDurationLocals {
            date,
            seconds,
            subsecond,
        })
    }

    /// AddEpochNanoseconds, keeping whole seconds separate so
    /// the full Temporal range never overflows an i64 nanosecond product.
    fn emit_temporal_zoned_add_elapsed(
        &mut self,
        epoch: &NormalizedTemporalInstantLocals,
        duration: &TemporalZonedInternalDurationLocals,
        function: &mut Function,
    ) -> CompletedTemporalEpochArithmeticLocals {
        let floor_seconds = self.runtime_schema().reserve_i64_local(function);
        let nanosecond = self.runtime_schema().reserve_i64_local(function);
        (epoch.floor_seconds()).load(function);
        (duration.seconds()).load(function);
        function.instruction(&Instruction::I64Add);
        (floor_seconds).store(function);
        (epoch.nanosecond()).load(function);
        (duration.subsecond()).load(function);
        function.instruction(&Instruction::I64Add);
        (nanosecond).store(function);
        // Each remainder has magnitude below 10^9. At most one carry or
        // borrow restores the epoch's [0,10^9) remainder.
        for (compare, adjustment, seconds_adjustment) in [
            (Instruction::I64LtS, 1_000_000_000, -1),
            (Instruction::I64GeS, -1_000_000_000, 1),
        ] {
            (nanosecond).load(function);
            function.instruction(&Instruction::I64Const(if seconds_adjustment < 0 {
                0
            } else {
                1_000_000_000
            }));
            function.instruction(&compare);
            self.open_frame(ControlFrameKind::If, function);
            (nanosecond).load(function);
            function.instruction(&Instruction::I64Const(adjustment));
            function.instruction(&Instruction::I64Add);
            (nanosecond).store(function);
            (floor_seconds).load(function);
            function.instruction(&Instruction::I64Const(seconds_adjustment));
            function.instruction(&Instruction::I64Add);
            (floor_seconds).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        CompletedTemporalEpochArithmeticLocals {
            floor_seconds,
            nanosecond,
        }
    }

    fn emit_temporal_zoned_calendar_add(
        &mut self,
        iso: &RegulatedTemporalIsoRecordLocals,
        calendar: &TemporalCalendarSlotLocals,
        duration: &TemporalZonedInternalDurationLocals,
        overflow: &TemporalOverflowLocals,
        function: &mut Function,
    ) -> Result<RegulatedTemporalIsoRecordLocals, EmitError> {
        let destination = self.reserve_temporal_iso_record_result(function);
        let fields: [I64Local; 9] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        for (source, destination) in iso.fields().iter().zip(fields) {
            (*source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            fields[0],
            fields[1],
            fields[2],
            duration.date[0],
            duration.date[1],
            duration.date[2],
            duration.date[3],
            overflow.local(),
            function,
        )?;
        self.emit_temporal_reject_date_time_lower_bound(&fields, function)?;
        // This private completion is constructed only after CalendarDateAdd
        // and the prescribed intermediate ISODateTimeWithinLimits check.
        let completed = CompletedTemporalIsoArithmeticLocals { fields };
        let result =
            self.emit_temporal_iso_record_from_arithmetic_into(destination, &completed, function)?;
        completed.release(self, function);
        Ok(result)
    }

    pub(super) fn emit_temporal_add_zoned_internal_duration(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        duration: &TemporalZonedInternalDurationLocals,
        overflow: &TemporalOverflowLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let destination = self.reserve_temporal_instant_result(function);
        let selected_seconds = self.runtime_schema().reserve_i64_local(function);
        let selected_nanosecond = self.runtime_schema().reserve_i64_local(function);
        (duration.date[0]).load(function);
        for local in &duration.date[1..] {
            (*local).load(function);
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // A zero date duration preserves the exact occurrence in a fold.
        // Projecting/reinterpreting it would replace a later occurrence with
        // compatible's earlier occurrence even when adding zero.
        let elapsed = self.emit_temporal_zoned_add_elapsed(origin, duration, function);
        for (source, destination) in [
            (elapsed.floor_seconds(), selected_seconds),
            (elapsed.nanosecond(), selected_nanosecond),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        elapsed.release(self, function);
        function.instruction(&Instruction::Else);
        let snapshot = self.emit_temporal_zone_snapshot(zone, origin, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let added =
            self.emit_temporal_zoned_calendar_add(&iso, calendar, duration, overflow, function)?;
        let coordinate = self.emit_temporal_local_coordinate_from_iso_record(&added, function)?;
        let disambiguation = self.emit_temporal_constant_disambiguation(
            super::temporal_options::Disambiguation::Compatible,
            function,
        );
        let intermediate = self.emit_temporal_get_epoch_nanoseconds_for(
            zone,
            &coordinate,
            &disambiguation,
            function,
        )?;
        let elapsed = self.emit_temporal_zoned_add_elapsed(&intermediate, duration, function);
        for (source, destination) in [
            (elapsed.floor_seconds(), selected_seconds),
            (elapsed.nanosecond(), selected_nanosecond),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        elapsed.release(self, function);
        intermediate.release(self, function);
        disambiguation.release(self, function);
        coordinate.release(self, function);
        added.release(self, function);
        iso.release(self, function);
        snapshot.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let completed = CompletedTemporalEpochArithmeticLocals {
            floor_seconds: selected_seconds,
            nanosecond: selected_nanosecond,
        };
        let result = self.emit_temporal_normalized_instant_from_arithmetic_into(
            destination,
            &completed,
            function,
        )?;
        completed.release(self, function);
        Ok(result)
    }

    pub(super) fn emit_temporal_add_duration_to_zoned_date_time(
        &mut self,
        receiver: &BrandedTemporalZonedRecordLocals,
        operation: TemporalZonedArithmeticOperation,
        duration_input: &ValueLocals,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let duration =
            self.emit_temporal_zoned_internal_duration(operation, duration_input, function)?;
        // All observable duration and overflow reads finish before the first
        // projection, calendar operation, inverse query or epoch range check.
        let overflow = self.emit_temporal_zoned_overflow(options, function)?;
        let origin = self.emit_temporal_normalized_instant_from_zoned_record(receiver, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(receiver, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(receiver, function)?;
        let result = self.emit_temporal_add_zoned_internal_duration(
            &origin, &zone, &calendar, &duration, &overflow, function,
        )?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&result, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        result.release(self, function);
        calendar.release(self, function);
        zone.release(self, function);
        origin.release(self, function);
        overflow.release(self, function);
        duration.release(self, function);
        Ok(())
    }
}
