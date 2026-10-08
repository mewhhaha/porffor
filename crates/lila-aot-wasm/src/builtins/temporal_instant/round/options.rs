use super::*;
impl FunctionBuilder<'_> {
    /// Shorthand or ordered option reads, then the inclusive full-day divisor.
    pub(super) fn emit_temporal_instant_round_settings(
        &mut self,
        quantum: I64Local,
        mode: I64Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(f);
        let unit = schema.reserve_i64_local(f);
        let increment = schema.reserve_i64_local(f);
        let maximum = schema.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_PROTOTYPE_ROUND_REQUIRES_A_ROUNDTO_ARGUMENT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I64Const(1));
        increment.store(f);
        f.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        mode.store(f);
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let string = schema
            .reserve_gc_local(f)
            .initialize(input.cast_reference::<StringValue>(schema, f), f);
        self.emit_temporal_plain_time_unit_from_string(&string, unit, f)?;
        string.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&input, f)?;
        self.emit_temporal_duration_rounding_increment_option(&input, increment, f)?;
        self.emit_temporal_duration_rounding_mode_option(
            &input,
            TemporalRoundingMode::HalfExpand,
            mode,
            f,
        )?;
        self.emit_temporal_duration_unit_option(
            &input,
            TemporalUnitOptionProperty::SmallestUnit,
            unit,
            f,
        )?;
        unit.load(f);
        f.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_INSTANT_PROTOTYPE_ROUND_REQUIRES_SMALLESTUNIT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            unit,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_INSTANT_UNIT_OPTION,
            f,
        )?;
        f.instruction(&Instruction::I64Const(0));
        maximum.store(f);
        for time_unit in TemporalTimeUnit::ALL {
            unit.load(f);
            f.instruction(&Instruction::I64Const(time_unit.code()));
            f.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::I64Const(
                NANOSECONDS_PER_TEMPORAL_DAY / time_unit.nanoseconds(),
            ));
            maximum.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        increment.load(f);
        maximum.load(f);
        f.instruction(&Instruction::I64GtU);
        maximum.load(f);
        increment.load(f);
        f.instruction(&Instruction::I64RemU);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_INSTANT_ROUNDING_INCREMENT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, f);
        for local in [maximum, increment, unit] {
            schema.release_i64_local(local, f);
        }
        input.clear(f);
        Ok(())
    }
}
