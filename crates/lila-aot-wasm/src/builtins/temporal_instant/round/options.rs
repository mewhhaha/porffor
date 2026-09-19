use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Resolve the shorthand or ordered options, then validate an inclusive
    /// full-day increment. Duration differences use a different bound.
    pub(super) fn emit_temporal_instant_round_settings(
        &mut self,
        quantum_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let maximum_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant.prototype.round requires a roundTo argument",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_unit_from_payload(
            argument_payload_local,
            unit_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            argument_payload_local,
            argument_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            argument_payload_local,
            argument_tag_local,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            argument_payload_local,
            argument_tag_local,
            TemporalUnitOptionProperty::SmallestUnit,
            unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Instant.prototype.round requires smallestUnit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.Instant unit option",
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(maximum_local));
        for unit in TemporalTimeUnit::ALL {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(
                NANOSECONDS_PER_TEMPORAL_DAY / unit.nanoseconds(),
            ));
            function.instruction(&Instruction::LocalSet(maximum_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.Instant rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        for local in [
            maximum_local,
            increment_local,
            unit_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
