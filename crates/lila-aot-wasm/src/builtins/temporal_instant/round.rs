use super::super::temporal_options::{
    TemporalRoundingMode, TemporalTimeUnit, TemporalUnit, TemporalUnitOptionProperty,
    TemporalUnitSlot,
};
use super::super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::*;
mod options;

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_instant_round(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_instant_record_from_receiver(f)?;
        let epoch = self.emit_temporal_instant_epoch_from_record(&record, f)?;
        let quantum = schema.reserve_i64_local(f);
        let mode = schema.reserve_i64_local(f);
        let seconds = schema.reserve_i64_local(f);
        let subseconds = schema.reserve_i64_local(f);
        self.emit_temporal_instant_round_settings(quantum, mode, f)?;
        epoch.floor_seconds().load(f);
        seconds.store(f);
        epoch.nanosecond().load(f);
        subseconds.store(f);
        self.emit_temporal_round_seconds_and_subseconds_to_quantum(
            quantum, mode, seconds, subseconds, f,
        );
        let value = self.emit_temporal_epoch_nanoseconds_bigint(seconds, subseconds, f);
        let result = self.emit_temporal_instant_validated_epoch(&value, f)?;
        self.emit_alloc_temporal_instant(&result, TemporalPrototypeSource::Intrinsic, f)?;
        result.clear(self, f);
        value.clear(f);
        for local in [subseconds, seconds, mode, quantum] {
            schema.release_i64_local(local, f);
        }
        epoch.clear(self, f);
        record.clear(f);
        Ok(())
    }
}
