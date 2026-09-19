use super::super::temporal::TemporalEpochNanosecondsRecord;
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

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_temporal_instant_add_or_subtract(
        &mut self,
        operation: InstantArithmetic,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let argument_payload = self.reserve_temp_local();
        let argument_tag = self.reserve_temp_local();
        let seconds = self.reserve_temp_local();
        let subseconds = self.reserve_temp_local();
        let duration_seconds = self.reserve_temp_local();
        let duration_subseconds = self.reserve_temp_local();
        let duration = self.reserve_temporal_duration_field_locals();
        let epoch = UnvalidatedEpochNanoseconds {
            payload_local: self.reserve_temp_local(),
            tag_local: self.reserve_temp_local(),
        };

        self.emit_temporal_instant_record_from_receiver(record, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload, argument_tag, function);
        self.emit_to_temporal_duration(argument_payload, argument_tag, &duration, function)?;
        // ToTemporalDuration completes its entire field sweep and validates
        // the duration before the Instant algorithm rejects date units.
        for unit in [
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ] {
            function.instruction(&Instruction::LocalGet(duration.number_bits(unit)));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.Instant arithmetic does not accept date units",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_epoch_nanoseconds_pair(
            record,
            TemporalEpochNanosecondsRecord::Instant,
            seconds,
            subseconds,
            function,
        );
        self.emit_temporal_duration_normalize_seconds(
            &duration,
            TemporalUnit::Hour,
            duration_seconds,
            duration_subseconds,
            function,
        );
        for (epoch_part, duration_part) in [
            (seconds, duration_seconds),
            (subseconds, duration_subseconds),
        ] {
            function.instruction(&Instruction::LocalGet(epoch_part));
            function.instruction(&Instruction::LocalGet(duration_part));
            function.instruction(&match operation {
                InstantArithmetic::Add => Instruction::I64Add,
                InstantArithmetic::Subtract => Instruction::I64Sub,
            });
            function.instruction(&Instruction::LocalSet(epoch_part));
        }
        // Each operand's remainder is signed. Carry before converting the
        // sum to the floor pair required by exact BigInt reconstruction.
        self.emit_temporal_duration_renormalize(seconds, subseconds, function);
        self.emit_temporal_normalize_seconds_and_subseconds(seconds, subseconds, function);
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds,
            subseconds,
            epoch.payload_local,
            epoch.tag_local,
            function,
        )?;
        let validated = self.emit_temporal_instant_validated_epoch(epoch, function)?;
        self.emit_alloc_validated_temporal_instant(validated, function)?;

        self.release_temp_local(epoch.tag_local);
        self.release_temp_local(epoch.payload_local);
        self.release_temporal_duration_field_locals(duration);
        for local in [
            duration_subseconds,
            duration_seconds,
            subseconds,
            seconds,
            argument_tag,
            argument_payload,
            record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_instant_until_or_since(
        &mut self,
        operation: InstantDifference,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let argument_payload = self.reserve_temp_local();
        let argument_tag = self.reserve_temp_local();
        let other_payload = self.reserve_temp_local();
        let other_tag = self.reserve_temp_local();
        let other_record = self.reserve_temp_local();
        let options_payload = self.reserve_temp_local();
        let options_tag = self.reserve_temp_local();
        let seconds = self.reserve_temp_local();
        let subseconds = self.reserve_temp_local();
        let other_seconds = self.reserve_temp_local();
        let other_subseconds = self.reserve_temp_local();
        let duration = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_instant_record_from_receiver(record, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload, argument_tag, function);
        let from = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing builtin meta `Temporal.Instant.from`")
            })?;
        self.emit_direct_js_call(
            &from,
            None,
            &[(argument_payload, argument_tag)],
            other_payload,
            other_tag,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_payload,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record,
            function,
        );
        self.emit_builtin_arg_to_locals(1, options_payload, options_tag, function);
        let plan = match operation {
            InstantDifference::Until => TemporalDateTimeDifferenceSettingsPlan::InstantUntil,
            InstantDifference::Since => TemporalDateTimeDifferenceSettingsPlan::InstantSince,
        };
        let settings = self.emit_temporal_date_time_difference_settings(
            options_payload,
            options_tag,
            plan,
            function,
        )?;
        self.emit_temporal_epoch_nanoseconds_pair(
            record,
            TemporalEpochNanosecondsRecord::Instant,
            seconds,
            subseconds,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_pair(
            other_record,
            TemporalEpochNanosecondsRecord::Instant,
            other_seconds,
            other_subseconds,
            function,
        );
        for (receiver, other) in [(seconds, other_seconds), (subseconds, other_subseconds)] {
            function.instruction(&Instruction::LocalGet(other));
            function.instruction(&Instruction::LocalGet(receiver));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(receiver));
        }
        self.emit_temporal_duration_renormalize(seconds, subseconds, function);
        self.emit_temporal_round_difference_time(
            seconds,
            subseconds,
            settings.smallest_unit_local,
            settings.increment_local,
            settings.mode_local,
            function,
        );
        if matches!(operation, InstantDifference::Since) {
            for local in [seconds, subseconds] {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(local));
            }
        }
        self.emit_temporal_duration_balance(
            seconds,
            subseconds,
            settings.largest_unit_local,
            &duration,
            function,
        )?;
        self.emit_create_temporal_duration(&duration, function)?;

        for local in [
            settings.mode_local,
            settings.increment_local,
            settings.smallest_unit_local,
            settings.largest_unit_local,
        ] {
            self.release_temp_local(local);
        }
        self.release_temporal_duration_field_locals(duration);
        for local in [
            other_subseconds,
            other_seconds,
            subseconds,
            seconds,
            options_tag,
            options_payload,
            other_record,
            other_tag,
            other_payload,
            argument_tag,
            argument_payload,
            record,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
