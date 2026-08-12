use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BigIntBuiltin {
    Constructor,
    AsIntN,
    AsUintN,
    PrototypeToString,
    PrototypeToLocaleString,
    PrototypeValueOf,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_bigint_builtin(
        &mut self,
        builtin: BigIntBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            BigIntBuiltin::Constructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                if let Some(new_target_tag_local) = self.new_target_tag_local() {
                    function.instruction(&Instruction::LocalGet(new_target_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "BigInt is not a constructor",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                }
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_bigint_locals(
                    arg_tag_local,
                    arg_payload_local,
                    true,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            BigIntBuiltin::AsIntN | BigIntBuiltin::AsUintN => {
                let bits_payload_local = self.reserve_temp_local();
                let bits_tag_local = self.reserve_temp_local();
                let bigint_payload_local = self.reserve_temp_local();
                let bigint_tag_local = self.reserve_temp_local();
                let index_local = self.reserve_temp_local();
                let mask_local = self.reserve_temp_local();
                let sign_local = self.reserve_temp_local();
                let word_payload_local = self.reserve_temp_local();
                let input_sign_local = self.reserve_temp_local();
                let input_limbs_local = self.reserve_temp_local();
                let input_limb_count_local = self.reserve_temp_local();
                let input_magnitude_word_local = self.reserve_temp_local();
                let result_limbs_local = self.reserve_temp_local();
                let result_limb_count_local = self.reserve_temp_local();
                let result_capacity_local = self.reserve_temp_local();
                let limb_index_local = self.reserve_temp_local();
                let limb_local = self.reserve_temp_local();
                let carry_local = self.reserve_temp_local();
                let partial_bits_local = self.reserve_temp_local();
                let result_sign_local = self.reserve_temp_local();
                let record_local = self.reserve_temp_local();
                let fits_immediate_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, bits_payload_local, bits_tag_local, function);
                self.emit_builtin_arg_to_locals(
                    1,
                    bigint_payload_local,
                    bigint_tag_local,
                    function,
                );
                self.emit_value_to_number_payload(bits_tag_local, bits_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(bits_payload_local));
                self.emit_to_index_from_number_payload(
                    bits_payload_local,
                    index_local,
                    "cannot convert value to BigInt",
                    function,
                )?;
                self.emit_to_bigint_value_and_u64_word_from_value_locals(
                    bigint_tag_local,
                    bigint_payload_local,
                    bigint_payload_local,
                    bigint_tag_local,
                    word_payload_local,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(64));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Shl);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(mask_local));
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::LocalGet(mask_local));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::LocalSet(word_payload_local));
                if builtin == BigIntBuiltin::AsIntN {
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Shl);
                    function.instruction(&Instruction::LocalSet(sign_local));
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::LocalGet(mask_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(word_payload_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                if builtin == BigIntBuiltin::AsUintN {
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64LtS);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_alloc_one_limb_bigint(1, word_payload_local, function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    function.instruction(&Instruction::End);
                } else {
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                }
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(bigint_tag_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    bigint_payload_local,
                    HEAP_BIGINT_SIGN_OFFSET,
                    input_sign_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    bigint_payload_local,
                    HEAP_BIGINT_LIMBS_PTR_OFFSET,
                    input_limbs_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    bigint_payload_local,
                    HEAP_BIGINT_LIMBS_LEN_OFFSET,
                    input_limb_count_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(input_magnitude_word_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(input_limbs_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(input_limb_count_local));
                function.instruction(&Instruction::LocalGet(bigint_payload_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(bigint_payload_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(input_sign_local));
                function.instruction(&Instruction::LocalGet(input_sign_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(bigint_payload_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(bigint_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(input_magnitude_word_local));
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(63));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(6));
                function.instruction(&Instruction::I64ShrU);
                function.instruction(&Instruction::LocalSet(result_capacity_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I64Const(63));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::LocalSet(partial_bits_local));
                function.instruction(&Instruction::LocalGet(partial_bits_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalGet(partial_bits_local));
                function.instruction(&Instruction::I64Shl);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(mask_local));

                function.instruction(&Instruction::LocalGet(bigint_tag_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(input_limb_count_local));
                function.instruction(&Instruction::LocalGet(result_capacity_local));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::I32And);
                if builtin == BigIntBuiltin::AsUintN {
                    function.instruction(&Instruction::LocalGet(input_sign_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::I32And);
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(bigint_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(bigint_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);

                function.instruction(&Instruction::LocalGet(result_capacity_local));
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::LocalSet(word_payload_local));
                self.emit_heap_alloc_from_local(word_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(result_limbs_local));
                function.instruction(&Instruction::LocalGet(input_sign_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(carry_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(limb_index_local));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::LocalGet(result_capacity_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::LocalGet(input_limb_count_local));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(input_limbs_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(input_magnitude_word_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(input_limbs_local));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(limb_local));
                function.instruction(&Instruction::LocalGet(input_sign_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(limb_local));
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::I64Xor);
                function.instruction(&Instruction::LocalGet(carry_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(word_payload_local));
                function.instruction(&Instruction::LocalGet(carry_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(carry_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(limb_local));
                function.instruction(&Instruction::LocalSet(word_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(result_limbs_local));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Store(self.buffer_memarg64(0)));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(limb_index_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(result_capacity_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(limb_index_local));
                function.instruction(&Instruction::LocalGet(result_limbs_local));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                function.instruction(&Instruction::LocalGet(mask_local));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::LocalSet(word_payload_local));
                function.instruction(&Instruction::LocalGet(result_limbs_local));
                function.instruction(&Instruction::LocalGet(limb_index_local));
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Store(self.buffer_memarg64(0)));

                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(result_sign_local));
                if builtin == BigIntBuiltin::AsIntN {
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalGet(index_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Const(63));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Shl);
                    function.instruction(&Instruction::LocalSet(sign_local));
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(-1));
                    function.instruction(&Instruction::LocalSet(result_sign_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(carry_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(limb_index_local));
                    function.instruction(&Instruction::Block(BlockType::Empty));
                    function.instruction(&Instruction::Loop(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(limb_index_local));
                    function.instruction(&Instruction::LocalGet(result_capacity_local));
                    function.instruction(&Instruction::I64GeU);
                    function.instruction(&Instruction::BrIf(1));
                    function.instruction(&Instruction::LocalGet(result_limbs_local));
                    function.instruction(&Instruction::LocalGet(limb_index_local));
                    function.instruction(&Instruction::I64Const(8));
                    function.instruction(&Instruction::I64Mul);
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                    function.instruction(&Instruction::I64Const(-1));
                    function.instruction(&Instruction::I64Xor);
                    function.instruction(&Instruction::LocalGet(carry_local));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(word_payload_local));
                    function.instruction(&Instruction::LocalGet(carry_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::LocalSet(carry_local));
                    function.instruction(&Instruction::LocalGet(result_limbs_local));
                    function.instruction(&Instruction::LocalGet(limb_index_local));
                    function.instruction(&Instruction::I64Const(8));
                    function.instruction(&Instruction::I64Mul);
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::I64Store(self.buffer_memarg64(0)));
                    function.instruction(&Instruction::LocalGet(limb_index_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(limb_index_local));
                    function.instruction(&Instruction::Br(0));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(result_limbs_local));
                    function.instruction(&Instruction::LocalGet(result_capacity_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Const(8));
                    function.instruction(&Instruction::I64Mul);
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                    function.instruction(&Instruction::LocalGet(mask_local));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::LocalSet(word_payload_local));
                    function.instruction(&Instruction::LocalGet(result_limbs_local));
                    function.instruction(&Instruction::LocalGet(result_capacity_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Const(8));
                    function.instruction(&Instruction::I64Mul);
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I32WrapI64);
                    function.instruction(&Instruction::LocalGet(word_payload_local));
                    function.instruction(&Instruction::I64Store(self.buffer_memarg64(0)));
                    function.instruction(&Instruction::End);
                }

                function.instruction(&Instruction::LocalGet(result_capacity_local));
                function.instruction(&Instruction::LocalSet(result_limb_count_local));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(result_limb_count_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(result_limbs_local));
                function.instruction(&Instruction::LocalGet(result_limb_count_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(8));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(result_limb_count_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(result_limb_count_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(result_limb_count_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(result_limb_count_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(result_limbs_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));
                function.instruction(&Instruction::LocalSet(word_payload_local));
                function.instruction(&Instruction::LocalGet(result_sign_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Const(i64::MIN));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Const(i64::MAX));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(fits_immediate_local));
                function.instruction(&Instruction::LocalGet(fits_immediate_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(result_sign_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(word_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
                function.instruction(&Instruction::LocalSet(record_local));
                self.store_i64_local_at_offset(
                    record_local,
                    HEAP_BIGINT_SIGN_OFFSET,
                    result_sign_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    record_local,
                    HEAP_BIGINT_LIMBS_PTR_OFFSET,
                    result_limbs_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    record_local,
                    HEAP_BIGINT_LIMBS_LEN_OFFSET,
                    result_limb_count_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    record_local,
                    HEAP_BIGINT_LIMBS_CAP_OFFSET,
                    result_capacity_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(record_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                self.release_temp_local(fits_immediate_local);
                self.release_temp_local(record_local);
                self.release_temp_local(result_sign_local);
                self.release_temp_local(partial_bits_local);
                self.release_temp_local(carry_local);
                self.release_temp_local(limb_local);
                self.release_temp_local(limb_index_local);
                self.release_temp_local(result_capacity_local);
                self.release_temp_local(result_limb_count_local);
                self.release_temp_local(result_limbs_local);
                self.release_temp_local(input_magnitude_word_local);
                self.release_temp_local(input_limb_count_local);
                self.release_temp_local(input_limbs_local);
                self.release_temp_local(input_sign_local);
                self.release_temp_local(word_payload_local);
                self.release_temp_local(sign_local);
                self.release_temp_local(mask_local);
                self.release_temp_local(index_local);
                self.release_temp_local(bigint_tag_local);
                self.release_temp_local(bigint_payload_local);
                self.release_temp_local(bits_tag_local);
                self.release_temp_local(bits_payload_local);
            }
            BigIntBuiltin::PrototypeToString
            | BigIntBuiltin::PrototypeToLocaleString
            | BigIntBuiltin::PrototypeValueOf => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing BigInt prototype receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing BigInt prototype receiver",
                    )
                })?;
                let boxed_kind_local = self.reserve_temp_local();
                let bigint_payload_local = self.reserve_temp_local();
                let bigint_tag_local = self.reserve_temp_local();

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalSet(bigint_payload_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::LocalSet(bigint_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_BOXED_KIND_OFFSET,
                    boxed_kind_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(boxed_kind_local));
                function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BIGINT as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    bigint_payload_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_OBJECT_BOXED_TAG_OFFSET,
                    bigint_tag_local,
                    function,
                );
                function.instruction(&Instruction::Else);
                self.emit_throw_current_function_realm_type_error(
                    "cannot convert value to BigInt",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                self.emit_throw_current_function_realm_type_error(
                    "cannot convert value to BigInt",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                if builtin == BigIntBuiltin::PrototypeValueOf {
                    function.instruction(&Instruction::LocalGet(bigint_payload_local));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::LocalGet(bigint_tag_local));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                } else {
                    function.instruction(&Instruction::LocalGet(bigint_tag_local));
                    function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_heap_bigint_to_string_with_radix_result(
                        bigint_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_bigint_to_string_with_radix_result(bigint_payload_local, function)?;
                    function.instruction(&Instruction::End);
                }

                self.release_temp_local(bigint_tag_local);
                self.release_temp_local(bigint_payload_local);
                self.release_temp_local(boxed_kind_local);
            }
        }
        Ok(())
    }
}
