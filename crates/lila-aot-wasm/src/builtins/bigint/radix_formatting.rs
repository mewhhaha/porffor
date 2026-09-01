use super::*;

#[derive(Debug)]
#[must_use = "a prepared BigInt radix local must be formatted and released"]
struct PreparedBigIntRadixLocal(u32);

impl PreparedBigIntRadixLocal {
    const fn local(&self) -> u32 {
        self.0
    }

    const fn into_local(self) -> u32 {
        self.0
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_bigint_radix_string_result(
        &mut self,
        result: BigIntRadixStringResult,
        bigint_payload_local: u32,
        bigint_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let radix = self.emit_prepare_bigint_radix(result, function)?;

        function.instruction(&Instruction::LocalGet(bigint_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_heap_bigint_to_radix_string_payload(
            bigint_payload_local,
            radix.local(),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_bigint_to_radix_string_payload(bigint_payload_local, radix.local(), function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(radix.into_local());
        Ok(())
    }

    fn emit_prepare_bigint_radix(
        &mut self,
        _result: BigIntRadixStringResult,
        function: &mut Function,
    ) -> Result<PreparedBigIntRadixLocal, EmitError> {
        let radix_local = self.reserve_temp_local();
        let radix_payload_local = self.reserve_temp_local();
        let radix_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, radix_payload_local, radix_tag_local, function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(radix_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(radix_tag_local, radix_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(radix_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "BigInt.prototype.toString radix out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(radix_tag_local);
        self.release_temp_local(radix_payload_local);
        Ok(PreparedBigIntRadixLocal(radix_local))
    }
}
