//! `%TypedArray%.prototype.set` offset coercion and early range validation.

use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_typed_array_set_offset(
        &mut self,
        offset_payload_local: u32,
        offset_tag_local: u32,
        offset_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_builtin_arg_to_locals(1, offset_payload_local, offset_tag_local, function);
        self.emit_value_to_number_payload(offset_tag_local, offset_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(offset_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_integer_or_infinity_number_payload_from_number_payload(
            offset_payload_local,
            offset_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(offset_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray.prototype.set offset is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // +Infinity and finite offsets above any realizable TypedArray length
        // must reach the source and target checks before the bounds RangeError.
        function.instruction(&Instruction::LocalGet(offset_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(offset_local));
        Ok(())
    }
}
