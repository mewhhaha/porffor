use super::*;

pub(super) struct NfNumericLocals {
    pub(super) kind: u32,
    pub(super) bytes: u32,
}
impl NfNumericLocals {
    pub(super) fn reserve(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            kind: builder.reserve_temp_local(),
            bytes: builder.reserve_temp_local(),
        }
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>) {
        builder.release_temp_local(self.bytes);
        builder.release_temp_local(self.kind);
    }
}

impl FunctionBuilder<'_> {
    fn emit_nf_utf16_input(
        &mut self,
        text: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let byte_length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let first = self.reserve_temp_local();
        let scalar = self.reserve_temp_local();
        let advance = self.reserve_temp_local();
        let temporary = self.reserve_temp_local();
        self.emit_unpack_string_payload(text, offset, length, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(offset, length, byte_length, function);
        function.instruction(&Instruction::LocalGet(byte_length));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalTee(byte_length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(byte_length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_nf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset, index, first, function);
        self.emit_decode_utf8_scalar_at_index(
            offset, index, length, first, scalar, advance, temporary, function,
        );
        function.instruction(&Instruction::LocalGet(scalar));
        function.instruction(&Instruction::I64Const(0xffff));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(scalar));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Store16(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_nf_advance_wire_cursor(cursor, 2, function);
        function.instruction(&Instruction::LocalGet(scalar));
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scalar));
        function.instruction(&Instruction::End);
        // The internal WTF8 decoder also returns lone surrogates verbatim.
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(scalar));
        function.instruction(&Instruction::I64Store16(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_nf_advance_wire_cursor(cursor, 2, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(advance));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(pointer, byte_length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [
            temporary,
            advance,
            scalar,
            first,
            index,
            cursor,
            pointer,
            byte_length,
            length,
            offset,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_nf_observe_numeric(
        &mut self,
        input: TaggedLocals,
        output: &NfNumericLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let observed = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::Number,
            input.payload,
            input.tag,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_tagged_locals(
            observed,
            primitive.payload,
            primitive.tag,
            function,
        );
        self.emit_nf_if_eq(primitive.tag, ValueKind::String.tag() as u64, function);
        self.emit_nf_set_const(
            output.kind,
            NumberNumericKind::String.wire_code() as i64,
            function,
        );
        self.emit_nf_utf16_input(primitive.payload, output.bytes, function)?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(primitive.tag, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_set_const(
            output.kind,
            NumberNumericKind::BigInt.wire_code() as i64,
            function,
        );
        self.emit_bigint_value_to_string_payload(primitive.payload, primitive.tag, function)?;
        function.instruction(&Instruction::LocalSet(output.bytes));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload(primitive.tag, primitive.payload, function)?;
        function.instruction(&Instruction::LocalSet(primitive.payload));
        self.emit_return_current_completion_if_throw(function);
        self.emit_nf_if_eq(primitive.payload, (-0.0f64).to_bits(), function);
        self.emit_nf_set_const(
            output.kind,
            NumberNumericKind::NegativeZero.wire_code() as i64,
            function,
        );
        self.emit_nf_set_string(output.bytes, "", function);
        function.instruction(&Instruction::Else);
        self.emit_nf_set_const(
            output.kind,
            NumberNumericKind::Number.wire_code() as i64,
            function,
        );
        self.emit_number_to_string_payload(primitive.payload, function)?;
        function.instruction(&Instruction::LocalSet(output.bytes));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(primitive.tag);
        self.release_temp_local(primitive.payload);
        Ok(())
    }
}
