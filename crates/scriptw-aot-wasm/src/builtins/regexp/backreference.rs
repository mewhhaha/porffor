use super::*;

#[derive(Clone, Copy)]
pub(super) struct RegExpInputCursor {
    pub(super) byte: I64Local,
    pub(super) utf16: I64Local,
    pub(super) on_low_surrogate: I64Local,
}

#[derive(Clone, Copy)]
pub(super) struct RegExpCharacterLocals {
    pub(super) byte: I64Local,
    pub(super) codepoint: I64Local,
    pub(super) byte_advance: I64Local,
    pub(super) utf16_advance: I64Local,
    pub(super) decode_temp: I64Local,
    pub(super) previous_byte: I64Local,
}

impl<'a> FunctionBuilder<'a> {
    /// Reads one character in the active direction and advances a private
    /// cursor. The caller establishes that input remains and Unicode cursors
    /// are not between paired surrogates. Legacy characters are UTF-16 units.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_regexp_read_character(
        &mut self,
        checkpoint: I64Local,
        input_offset: I64Local,
        input_len: I64Local,
        cursor: RegExpInputCursor,
        reverse: I64Local,
        unicode: I64Local,
        character: I64Local,
        locals: RegExpCharacterLocals,
        failure_position: I64Local,
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
        reverse.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.on_low_surrogate.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Moving left from between a pair returns its leading UTF-16 unit.
        self.emit_regexp_scratch_byte(input_offset, cursor.byte, byte, function);
        self.emit_regexp_scratch_decode_scalar(
            input_offset,
            cursor.byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        codepoint.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        character.store(function);
        function.instruction(&Instruction::I64Const(1));
        utf16_advance.store(function);
        function.instruction(&Instruction::I64Const(0));
        cursor.on_low_surrogate.store(function);
        function.instruction(&Instruction::Else);
        cursor.byte.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        previous_byte.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_regexp_scratch_byte(input_offset, previous_byte, byte, function);
        byte.load(function);
        function.instruction(&Instruction::I64Const(0xc0));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        previous_byte.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(
            checkpoint,
            failure_position,
            failure_position,
            RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
            function,
        );
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        previous_byte.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        previous_byte.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_decode_scalar(
            input_offset,
            previous_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        previous_byte.load(function);
        cursor.byte.store(function);
        codepoint.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        unicode.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        codepoint.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        character.store(function);
        function.instruction(&Instruction::I64Const(1));
        utf16_advance.store(function);
        function.instruction(&Instruction::I64Const(1));
        cursor.on_low_surrogate.store(function);
        function.instruction(&Instruction::Else);
        codepoint.load(function);
        character.store(function);
        codepoint.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        utf16_advance.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        cursor.utf16.load(function);
        utf16_advance.load(function);
        function.instruction(&Instruction::I64Sub);
        cursor.utf16.store(function);
        function.instruction(&Instruction::Else);
        unicode.load(function);
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
        self.emit_regexp_scratch_byte(input_offset, cursor.byte, byte, function);
        self.emit_regexp_scratch_decode_scalar(
            input_offset,
            cursor.byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            function,
        );
        codepoint.load(function);
        character.store(function);
        self.emit_regexp_scratch_increment_by_local(cursor.byte, byte_advance, function);
        cursor.utf16.load(function);
        codepoint.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        cursor.utf16.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    /// Binary searches sorted nonidentity mappings. An absent key is unchanged.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_regexp_canonicalize_character(
        &self,
        table: I64Local,
        count: I64Local,
        character: I64Local,
        low: I64Local,
        high: I64Local,
        middle: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        low.store(function);
        count.load(function);
        high.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        low.load(function);
        high.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        low.load(function);
        high.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        middle.store(function);
        table.load(function);
        middle.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        character.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        middle.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        low.store(function);
        function.instruction(&Instruction::Else);
        middle.load(function);
        high.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        low.load(function);
        count.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        table.load(function);
        low.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        middle.store(function);
        middle.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        character.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        middle.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(4)));
        character.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
