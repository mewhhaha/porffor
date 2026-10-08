use super::super::temporal_options::TemporalUnit;
use super::super::temporal_plain_date_time_methods::TemporalDateTimeDifferenceSettingsPlan;
use super::*;

#[derive(Clone, Copy)]
pub(in crate::builtins) enum InstantArithmetic {
    Add,
    Subtract,
}
#[derive(Clone, Copy)]
pub(in crate::builtins) enum InstantDifference {
    Until,
    Since,
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_instant_add_or_subtract(
        &mut self,
        operation: InstantArithmetic,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_instant_record_from_receiver(f)?;
        let epoch = self.emit_temporal_instant_epoch_from_record(&record, f)?;
        let input = schema.reserve_value_local(f);
        let seconds = schema.reserve_i64_local(f);
        let subseconds = schema.reserve_i64_local(f);
        let duration_seconds = schema.reserve_i64_local(f);
        let duration_subseconds = schema.reserve_i64_local(f);
        let duration = self.reserve_temporal_duration_field_locals(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_to_temporal_duration(&input, &duration, f)?;
        // The complete Duration sweep/validation precedes rejecting date units.
        for unit in [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ] {
            duration.number_bits(unit).load(f);
            f.instruction(&Instruction::I64Eqz);
            f.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::TEMPORAL_INSTANT_ARITHMETIC_DOES_NOT_ACCEPT_DATE_UNITS,
                f,
            )?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        epoch.floor_seconds().load(f);
        seconds.store(f);
        epoch.nanosecond().load(f);
        subseconds.store(f);
        self.emit_temporal_duration_normalize_seconds(
            &duration,
            TemporalUnit::Hour,
            duration_seconds,
            duration_subseconds,
            f,
        );
        for (target, source) in [
            (seconds, duration_seconds),
            (subseconds, duration_subseconds),
        ] {
            target.load(f);
            source.load(f);
            f.instruction(&match operation {
                InstantArithmetic::Add => Instruction::I64Add,
                InstantArithmetic::Subtract => Instruction::I64Sub,
            });
            target.store(f);
        }
        self.emit_temporal_duration_renormalize(seconds, subseconds, f);
        self.emit_temporal_normalize_seconds_and_subseconds(seconds, subseconds, f);
        let value = self.emit_temporal_epoch_nanoseconds_bigint(seconds, subseconds, f);
        let result = self.emit_temporal_instant_validated_epoch(&value, f)?;
        self.emit_alloc_temporal_instant(&result, TemporalPrototypeSource::Intrinsic, f)?;
        result.clear(self, f);
        value.clear(f);
        self.release_temporal_duration_field_locals(duration, f);
        for local in [duration_subseconds, duration_seconds, subseconds, seconds] {
            schema.release_i64_local(local, f);
        }
        input.clear(f);
        epoch.clear(self, f);
        record.clear(f);
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_instant_until_or_since(
        &mut self,
        operation: InstantDifference,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_instant_record_from_receiver(f)?;
        let receiver = self.emit_temporal_instant_epoch_from_record(&record, f)?;
        let input = schema.reserve_value_local(f);
        let options = schema.reserve_value_local(f);
        let seconds = schema.reserve_i64_local(f);
        let subseconds = schema.reserve_i64_local(f);
        let duration = self.reserve_temporal_duration_field_locals(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        let other = self.emit_temporal_to_instant_epoch(&input, f)?;
        self.emit_builtin_arg_to_value(1, &options, f);
        let plan = match operation {
            InstantDifference::Until => TemporalDateTimeDifferenceSettingsPlan::InstantUntil,
            InstantDifference::Since => TemporalDateTimeDifferenceSettingsPlan::InstantSince,
        };
        let settings = self.emit_temporal_date_time_difference_settings(&options, plan, f)?;
        other.floor_seconds().load(f);
        receiver.floor_seconds().load(f);
        f.instruction(&Instruction::I64Sub);
        seconds.store(f);
        other.nanosecond().load(f);
        receiver.nanosecond().load(f);
        f.instruction(&Instruction::I64Sub);
        subseconds.store(f);
        self.emit_temporal_duration_renormalize(seconds, subseconds, f);
        self.emit_temporal_round_difference_time(
            seconds,
            subseconds,
            settings.smallest_unit(),
            settings.rounding_increment(),
            settings.rounding_mode(),
            f,
        );
        if matches!(operation, InstantDifference::Since) {
            for local in [seconds, subseconds] {
                f.instruction(&Instruction::I64Const(0));
                local.load(f);
                f.instruction(&Instruction::I64Sub);
                local.store(f);
            }
        }
        self.emit_temporal_duration_balance(
            seconds,
            subseconds,
            settings.largest_unit(),
            &duration,
            f,
        )?;
        self.emit_create_temporal_duration(&duration, f)?;
        for local in [
            settings.rounding_mode(),
            settings.rounding_increment(),
            settings.smallest_unit(),
            settings.largest_unit(),
        ] {
            schema.release_i64_local(local, f);
        }
        other.clear(self, f);
        self.release_temporal_duration_field_locals(duration, f);
        schema.release_i64_local(subseconds, f);
        schema.release_i64_local(seconds, f);
        options.clear(f);
        input.clear(f);
        receiver.clear(self, f);
        record.clear(f);
        Ok(())
    }
}
