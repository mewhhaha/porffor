use super::super::*;
use porffor_ir::{
    REGEXP_INSTRUCTION_WIDTH, REGEXP_OPCODE_ACCEPT, REGEXP_OPCODE_JUMP,
    REGEXP_OPCODE_LITERAL_ASCII, REGEXP_OPCODE_POSITIVE_ASCII_CLASS, REGEXP_OPCODE_SPLIT,
};

impl<'a> FunctionBuilder<'a> {
    /// Compiles the fixed-width ordered-backtracking `RegExpProgram` matcher.
    ///
    /// The flat consuming-atom grammar uses ordered `Split`/`Jump` choices for
    /// repetition. Matching can backtrack, but the helper remains pure: it
    /// reads program/string bytes and writes only caller-provided scratch.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`] (seven i64 params, four i64
    /// results). Params: 0=program pointer, 1=instruction count, 2=input string
    /// payload, 3=start UTF-16 index, 4=sticky, 5=exclusive static-data end,
    /// 6=choice-frame scratch (fallback pc, byte cursor, UTF-16 cursor). Results are found, match start, match end, and status (nonzero
    /// for a corrupt program).
    pub(crate) fn compile_regexp_matcher_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        let input_offset = self.reserve_temp_local();
        let input_len = self.reserve_temp_local();
        let candidate_byte = self.reserve_temp_local();
        let candidate_utf16 = self.reserve_temp_local();
        let match_byte = self.reserve_temp_local();
        let match_utf16 = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let instruction_address = self.reserve_temp_local();
        let opcode = self.reserve_temp_local();
        let operand0 = self.reserve_temp_local();
        let operand1 = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let codepoint = self.reserve_temp_local();
        let byte_advance = self.reserve_temp_local();
        let utf16_advance = self.reserve_temp_local();
        let decode_temp = self.reserve_temp_local();
        let start_on_low_surrogate = self.reserve_temp_local();
        let choice_depth = self.reserve_temp_local();
        let choice_address = self.reserve_temp_local();
        let choice_limit = self.reserve_temp_local();

        // Validate the untrusted program span before deriving an address from
        // its instruction count. The final capacity comparison avoids both
        // multiplication and addition overflow.
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(2, input_offset, input_len, &mut function);
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_limit));
        for local in [candidate_byte, candidate_utf16, start_on_low_surrogate] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        // Seek once from the beginning. If `start` lands on an astral scalar's
        // low surrogate, this advances to the following scalar boundary.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(candidate_utf16));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(input_offset, candidate_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            candidate_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::LocalGet(candidate_utf16));
        function.instruction(&Instruction::LocalGet(utf16_advance));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(start_on_low_surrogate));
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
        self.emit_increment_by_local(candidate_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Preserve a low-surrogate start as an empty-match candidate. Its byte
        // cursor remains at the following scalar boundary, while consuming
        // atoms below are explicitly prevented from using that cursor.
        function.instruction(&Instruction::LocalGet(start_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::LocalSet(candidate_utf16));
        function.instruction(&Instruction::End);

        // A start beyond the string cannot produce a match, including an empty one.
        function.instruction(&Instruction::LocalGet(candidate_utf16));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        // Candidate positions and atom cursors advance incrementally; byte offsets
        // are never exposed as RegExp indices.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_byte));
        function.instruction(&Instruction::LocalSet(match_byte));
        function.instruction(&Instruction::LocalGet(candidate_utf16));
        function.instruction(&Instruction::LocalSet(match_utf16));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(choice_depth));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(instruction_address));
        self.emit_regexp_instruction_load(instruction_address, 0, opcode, &mut function);
        self.emit_regexp_instruction_load(instruction_address, 8, operand0, &mut function);
        self.emit_regexp_instruction_load(instruction_address, 16, operand1, &mut function);

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ACCEPT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(1, candidate_utf16, match_utf16, 0, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        // `Split` records the fallback before taking the primary arm. Both
        // target operands are absolute instruction indices and must be within
        // this untrusted program span.
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_SPLIT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(choice_depth));
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(choice_depth));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_address));
        function.instruction(&Instruction::LocalGet(choice_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Store(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(choice_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Store(Self::memarg8(8)));
        function.instruction(&Instruction::LocalGet(choice_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Store(Self::memarg8(16)));
        function.instruction(&Instruction::LocalGet(choice_depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_depth));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_JUMP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LITERAL_ASCII as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        // A cursor at an astral scalar's low-surrogate half may accept an
        // empty path, but no consuming ASCII atom starts there.
        function.instruction(&Instruction::LocalGet(start_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_regexp_backtrack_or_fail(
            0,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        self.emit_regexp_backtrack_or_fail(
            0,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            &mut function,
        );
        self.emit_load_string_byte(input_offset, match_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            match_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64GeU);
        self.emit_regexp_backtrack_or_fail(
            0,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LITERAL_ASCII as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Ne);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            &mut function,
        );
        function.instruction(&Instruction::Else);
        self.emit_regexp_ascii_class_contains(codepoint, operand0, operand1, &mut function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            &mut function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        self.emit_increment_by_local(pc, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(start_on_low_surrogate));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(candidate_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(candidate_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(input_offset, candidate_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            candidate_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
        self.emit_increment_by_local(candidate_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::End);
        Ok(function)
    }

    /// On an atom failure, restore the latest ordered fallback. If none exists,
    /// branch out of the instruction loop to advance the unanchored candidate.
    fn emit_regexp_backtrack_or_fail(
        &self,
        extra_depth: u32,
        depth: u32,
        address: u32,
        byte: u32,
        utf16: u32,
        pc: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(3 + extra_depth));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(depth));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address));
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(8)));
        function.instruction(&Instruction::LocalSet(byte));
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(16)));
        function.instruction(&Instruction::LocalSet(utf16));
        function.instruction(&Instruction::Br(1 + extra_depth));
        function.instruction(&Instruction::End);
    }

    fn emit_regexp_instruction_load(
        &self,
        address_local: u32,
        delta: u64,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(delta)));
        function.instruction(&Instruction::LocalSet(output_local));
    }

    fn emit_increment_by_local(&self, local: u32, delta_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::LocalGet(delta_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(local));
    }

    fn emit_regexp_ascii_class_contains(
        &self,
        codepoint_local: u32,
        low_bitmap_local: u32,
        high_bitmap_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(low_bitmap_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(high_bitmap_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
    }

    fn emit_regexp_match_result(
        &self,
        found: i64,
        start_local: u32,
        end_local: u32,
        status: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(found));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(status));
    }
}
