use super::super::temporal::TemporalEpochNanosecondsRecord;
use super::super::temporal_options::{
    TemporalRoundingMode, TemporalTimeUnit, TemporalUnit, TemporalUnitOptionProperty,
    TemporalUnitSlot,
};
use super::super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::*;

mod options;

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_temporal_instant_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let quantum = self.reserve_temp_local();
        let mode = self.reserve_temp_local();
        let seconds = self.reserve_temp_local();
        let subseconds = self.reserve_temp_local();
        let day = self.reserve_temp_local();
        let within_day = self.reserve_temp_local();
        let quotient = self.reserve_temp_local();
        let remainder = self.reserve_temp_local();
        let parity = self.reserve_temp_local();
        let positive = self.reserve_temp_local();
        let epoch = UnvalidatedEpochNanoseconds {
            payload_local: self.reserve_temp_local(),
            tag_local: self.reserve_temp_local(),
        };
        self.emit_temporal_instant_record_from_receiver(record, function)?;
        self.emit_temporal_instant_round_settings(quantum, mode, function)?;
        self.emit_temporal_epoch_nanoseconds_pair(
            record,
            TemporalEpochNanosecondsRecord::Instant,
            seconds,
            subseconds,
            function,
        );
        self.emit_temporal_normalize_seconds_and_subseconds(seconds, subseconds, function);
        function.instruction(&Instruction::LocalGet(seconds));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day));
        function.instruction(&Instruction::LocalGet(seconds));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(within_day));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day));
        function.instruction(&Instruction::LocalGet(day));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(subseconds));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day));

        // RoundNumberToIncrementAsIfPositive uses the floor quotient even
        // before the epoch. A day is an exact multiple of every admitted
        // quantum, so its remainder can be computed without a wide product.
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::LocalGet(quantum));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::LocalGet(quantum));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder));
        // Half-even needs the global quotient parity. Keeping only the
        // within-day quotient would misround odd days with a 24-hour quantum.
        function.instruction(&Instruction::LocalGet(day));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::LocalGet(quantum));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(parity));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(positive));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::LocalGet(remainder));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(within_day));
        self.emit_temporal_duration_round_up_i32(
            remainder, quantum, parity, positive, mode, function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::LocalGet(quantum));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day));
        function.instruction(&Instruction::End);
        // within_day may equal one full day after rounding; the exact
        // seconds addition absorbs that carry without another calendar step.
        function.instruction(&Instruction::LocalGet(day));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds));
        function.instruction(&Instruction::LocalGet(within_day));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(subseconds));
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds,
            subseconds,
            epoch.payload_local,
            epoch.tag_local,
            function,
        )?;
        let validated = self.emit_temporal_instant_validated_epoch(epoch, function)?;
        self.emit_alloc_validated_temporal_instant(validated, function)?;
        for local in [
            epoch.tag_local,
            epoch.payload_local,
            positive,
            parity,
            remainder,
            quotient,
            within_day,
            day,
            subseconds,
            seconds,
            mode,
            quantum,
            record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
