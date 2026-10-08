//! Rounding produces a new exact epoch before any offset or wall projection.

use super::*;
use crate::builtins::temporal_zone_provider::{
    TemporalInstantRoundingQuantumLocals, TemporalRoundingModeLocals,
};
use crate::builtins::temporal_zoned_date_time_day::TemporalZoneDayBoundariesLocals;

/// Every admitted quantum divides nsPerDay; both Instant endpoints lie on
/// that grid. Monotone rounding preserves the Instant range without an extra
/// spec-visible validation step. Other arithmetic cannot mint this proof.
pub(in crate::builtins) struct CompletedTemporalStringRoundingLocals {
    pair: CompletedTemporalEpochArithmeticLocals,
}

impl CompletedTemporalStringRoundingLocals {
    pub(in crate::builtins) fn floor_seconds(&self) -> I64Local {
        self.pair.floor_seconds()
    }
    pub(in crate::builtins) fn nanosecond(&self) -> I64Local {
        self.pair.nanosecond()
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        self.pair.release(builder, function);
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_temporal_round_instant_for_string(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        quantum: &TemporalInstantRoundingQuantumLocals,
        mode: &TemporalRoundingModeLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let destination = self.reserve_temporal_instant_result(function);
        let floor_seconds = self.runtime_schema().reserve_i64_local(function);
        let nanosecond = self.runtime_schema().reserve_i64_local(function);
        for (source, destination) in [
            (origin.floor_seconds(), floor_seconds),
            (origin.nanosecond(), nanosecond),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_round_seconds_and_subseconds_to_quantum(
            quantum.local(),
            mode.local(),
            floor_seconds,
            nanosecond,
            function,
        );
        let completed = CompletedTemporalStringRoundingLocals {
            pair: CompletedTemporalEpochArithmeticLocals {
                floor_seconds,
                nanosecond,
            },
        };
        // RoundEpochNanoseconds used by string formatting has no interval
        // validation step. The admitted quantum proves the range above.
        let result = self.emit_temporal_normalized_instant_from_string_rounding_into(
            destination,
            &completed,
            function,
        )?;
        completed.release(self, function);
        Ok(result)
    }

    pub(in crate::builtins) fn emit_temporal_round_instant_to_day(
        &mut self,
        boundaries: &TemporalZoneDayBoundariesLocals<'_>,
        mode: &TemporalRoundingModeLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let destination = self.reserve_temporal_instant_result(function);
        let elapsed = self.runtime_schema().reserve_i64_local(function);
        let quotient = self.runtime_schema().reserve_i64_local(function);
        let positive = self.runtime_schema().reserve_i64_local(function);
        let floor_seconds = self.runtime_schema().reserve_i64_local(function);
        let nanosecond = self.runtime_schema().reserve_i64_local(function);
        // Boundaries belong to this origin and prove start <= origin and a
        // positive span. A transition after midnight can put origin beyond
        // the next date's start: ZonedDateTime.round clamps to end - 1 ns.
        (boundaries.origin().floor_seconds()).load(function);
        (boundaries.end().floor_seconds()).load(function);
        function.instruction(&Instruction::I64GtS);
        (boundaries.origin().floor_seconds()).load(function);
        (boundaries.end().floor_seconds()).load(function);
        function.instruction(&Instruction::I64Eq);
        (boundaries.origin().nanosecond()).load(function);
        (boundaries.end().nanosecond()).load(function);
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (boundaries.span_nanoseconds()).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (elapsed).store(function);
        function.instruction(&Instruction::Else);
        // Only the bounded difference is multiplied to nanos, never the full
        // epoch. In this branch origin is strictly below end.
        (boundaries.origin().floor_seconds()).load(function);
        (boundaries.start().floor_seconds()).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (boundaries.origin().nanosecond()).load(function);
        function.instruction(&Instruction::I64Add);
        (boundaries.start().nanosecond()).load(function);
        function.instruction(&Instruction::I64Sub);
        (elapsed).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        (quotient).store(function);
        function.instruction(&Instruction::I64Const(1));
        (positive).store(function);
        self.emit_temporal_duration_round_up_i32(
            elapsed,
            boundaries.span_nanoseconds(),
            quotient,
            positive,
            mode.local(),
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        for (source, destination) in [
            (boundaries.end().floor_seconds(), floor_seconds),
            (boundaries.end().nanosecond(), nanosecond),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        function.instruction(&Instruction::Else);
        for (source, destination) in [
            (boundaries.start().floor_seconds(), floor_seconds),
            (boundaries.start().nanosecond(), nanosecond),
        ] {
            (source).load(function);
            (destination).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let completed = CompletedTemporalEpochArithmeticLocals {
            floor_seconds,
            nanosecond,
        };
        let result = self.emit_temporal_normalized_instant_from_arithmetic_into(
            destination,
            &completed,
            function,
        )?;
        completed.release(self, function);
        for local in [positive, quotient, elapsed] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(result)
    }
}
