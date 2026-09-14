use super::*;

#[derive(Clone, Copy)]
pub(super) struct RegExpInputCursor {
    pub(super) byte: u32,
    pub(super) utf16: u32,
    pub(super) on_low_surrogate: u32,
}

#[derive(Clone, Copy)]
pub(super) struct RegExpCharacterLocals {
    pub(super) byte: u32,
    pub(super) codepoint: u32,
    pub(super) byte_advance: u32,
    pub(super) utf16_advance: u32,
    pub(super) decode_temp: u32,
    pub(super) previous_byte: u32,
}

impl<'a> FunctionBuilder<'a> {
    /// Reads one character in the active direction and advances a private
    /// cursor. The caller establishes that input remains and Unicode cursors
    /// are not between paired surrogates. Legacy characters are UTF-16 units.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_regexp_read_character(
        &mut self,
        input_offset: u32,
        input_len: u32,
        cursor: RegExpInputCursor,
        reverse: u32,
        unicode: u32,
        character: u32,
        locals: RegExpCharacterLocals,
        failure_position: u32,
        function: &mut Function,
    ) {
        let RegExpCharacterLocals {
            byte,
            codepoint,
            byte_advance,
            utf16_advance,
            decode_temp,
            previous_byte,
        } = locals;
        function.instruction(&Instruction::LocalGet(reverse));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor.on_low_surrogate));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Moving left from between a pair returns its leading UTF-16 unit.
        self.emit_load_string_byte(input_offset, cursor.byte, byte, function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            cursor.byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(character));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor.on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cursor.byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(previous_byte));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_load_string_byte(input_offset, previous_byte, byte, function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0xc0));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(
            failure_position,
            failure_position,
            RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
            function,
        );
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(previous_byte));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            previous_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::LocalSet(cursor.byte));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(character));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cursor.on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalSet(character));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor.utf16));
        function.instruction(&Instruction::LocalGet(utf16_advance));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(cursor.utf16));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_read_utf16_unit(
            input_offset,
            input_len,
            cursor.byte,
            cursor.utf16,
            cursor.on_low_surrogate,
            character,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(input_offset, cursor.byte, byte, function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            cursor.byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalSet(character));
        self.emit_increment_by_local(cursor.byte, byte_advance, function);
        function.instruction(&Instruction::LocalGet(cursor.utf16));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor.utf16));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    /// Binary searches sorted nonidentity mappings. An absent key is unchanged.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_regexp_canonicalize_character(
        &self,
        table: u32,
        count: u32,
        character: u32,
        low: u32,
        high: u32,
        middle: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalSet(high));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(middle));
        function.instruction(&Instruction::LocalGet(table));
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::LocalSet(high));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(table));
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(middle));
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(middle));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(4)));
        function.instruction(&Instruction::LocalSet(character));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
