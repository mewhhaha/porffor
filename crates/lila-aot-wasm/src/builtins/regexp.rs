use super::super::*;
use crate::data::REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION;
use lila_ir::{
    REGEXP_INSTRUCTION_WIDTH, REGEXP_OPCODE_ACCEPT, REGEXP_OPCODE_ASSERT_END,
    REGEXP_OPCODE_ASSERT_START, REGEXP_OPCODE_CAPTURE_END, REGEXP_OPCODE_CAPTURE_START,
    REGEXP_OPCODE_CLEAR_CAPTURE_RANGE, REGEXP_OPCODE_DOT, REGEXP_OPCODE_JUMP,
    REGEXP_OPCODE_LITERAL_ASCII, REGEXP_OPCODE_LITERAL_CODE_POINT, REGEXP_OPCODE_LOOKBEHIND_END,
    REGEXP_OPCODE_LOOKBEHIND_FAILURE, REGEXP_OPCODE_LOOKBEHIND_START,
    REGEXP_OPCODE_NAMED_BACKREFERENCE, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
    REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD, REGEXP_OPCODE_NOT_WHITESPACE,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE, REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
    REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD, REGEXP_OPCODE_SPLIT, REGEXP_OPCODE_UNICODE_PROPERTY,
    REGEXP_OPCODE_WHITESPACE, REGEXP_RANGE_ENTRY_WIDTH,
};

impl<'a> FunctionBuilder<'a> {
    /// Compiles the fixed-width ordered-backtracking `RegExpProgram` matcher.
    ///
    /// The flat consuming-atom grammar uses ordered `Split`/`Jump` choices for
    /// repetition. Matching can backtrack, but the helper remains pure: it
    /// reads program/string bytes and writes only caller-provided scratch.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`] (seven i64 params, four i64
    /// results). Param 0 packs the program pointer in its low 32 bits and the
    /// named-group table pointer in its high 32 bits. Param 1=instruction count in the low
    /// 32 bits and capture count in the high 32 bits, 2=input string
    /// payload, 3=start UTF-16 index, 4=sticky in bit 0, Unicode mode in bit 1,
    /// multiline in bit 2, dotAll in bit 3, total split count in bits 4..31,
    /// and repeatable split count in bits
    /// 32..63, 5=exclusive
    /// static-data end,
    /// 6=choice-frame scratch (fallback pc, byte cursor, UTF-16 cursor,
    /// low-surrogate state). Results are found, match start, match end, and
    /// status (1 for a corrupt program, 2 for matcher resource exhaustion).
    pub(crate) fn compile_regexp_matcher_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::RegExpMatcher);
        let input_offset = self.reserve_temp_local();
        let program_ptr = self.reserve_temp_local();
        let named_group_table_ptr = self.reserve_temp_local();
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
        let literal_value = self.reserve_temp_local();
        let literal_advance_byte = self.reserve_temp_local();
        let candidate_on_low_surrogate = self.reserve_temp_local();
        let match_on_low_surrogate = self.reserve_temp_local();
        let choice_depth = self.reserve_temp_local();
        let choice_address = self.reserve_temp_local();
        let choice_limit = self.reserve_temp_local();
        let instruction_count = self.reserve_temp_local();
        let capture_count = self.reserve_temp_local();
        let sticky = self.reserve_temp_local();
        let unicode = self.reserve_temp_local();
        let multiline = self.reserve_temp_local();
        let dot_all = self.reserve_temp_local();
        let split_count = self.reserve_temp_local();
        let repeatable_split_count = self.reserve_temp_local();
        let frame_width = self.reserve_temp_local();
        let capture_index = self.reserve_temp_local();
        let capture_address = self.reserve_temp_local();
        let capture_start = self.reserve_temp_local();
        let named_group_count = self.reserve_temp_local();
        let named_candidate_total = self.reserve_temp_local();
        let named_records_ptr = self.reserve_temp_local();
        let named_record_ptr = self.reserve_temp_local();
        let named_candidate_ptr = self.reserve_temp_local();
        let named_candidate_count = self.reserve_temp_local();
        let named_aggregate_count = self.reserve_temp_local();
        let named_candidate_id = self.reserve_temp_local();
        let named_selected_count = self.reserve_temp_local();
        let named_candidate_start = self.reserve_temp_local();
        let named_capture_end = self.reserve_temp_local();
        let capture_byte = self.reserve_temp_local();
        let capture_utf16 = self.reserve_temp_local();
        let capture_on_low_surrogate = self.reserve_temp_local();
        let compare_byte = self.reserve_temp_local();
        let compare_utf16 = self.reserve_temp_local();
        let compare_on_low_surrogate = self.reserve_temp_local();
        let capture_unit = self.reserve_temp_local();
        let compare_unit = self.reserve_temp_local();
        let reverse_mode = self.reserve_temp_local();
        let lookbehind_frame_depth = self.reserve_temp_local();
        let lookbehind_succeeded = self.reserve_temp_local();
        let previous_byte = self.reserve_temp_local();
        let range_base = self.reserve_temp_local();
        let range_low = self.reserve_temp_local();
        let range_high = self.reserve_temp_local();
        let range_middle = self.reserve_temp_local();
        let range_count = self.reserve_temp_local();
        let effective_multiline = self.reserve_temp_local();
        let effective_dot_all = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(program_ptr));
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(named_group_table_ptr));

        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(instruction_count));
        // The code-point range pool is appended to the instruction stream.
        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(range_base));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(capture_count));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(sticky));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(unicode));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(multiline));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(dot_all));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x0fff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(split_count));
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(repeatable_split_count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(reverse_mode));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lookbehind_succeeded));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_width));

        // Validate the untrusted program span before deriving an address from
        // its instruction count. The final capacity comparison avoids both
        // multiplication and addition overflow.
        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        // Validate named-group metadata eagerly. Result materialization uses
        // this table after a successful match, so it must be safe even when a
        // NamedBackreference instruction is never reached.
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        for (offset, local) in [
            (0, decode_temp),
            (8, named_group_count),
            (16, named_candidate_total),
            (24, named_records_ptr),
        ] {
            function.instruction(&Instruction::LocalGet(named_group_table_ptr));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(offset)));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Const(
            REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_group_count));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(named_aggregate_count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(named_group_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(named_record_ptr));
        // The name payload is also consumed by result materialization. Its
        // low word is the UTF-8 byte length and high word is the absolute
        // static-data offset.
        function.instruction(&Instruction::LocalGet(named_record_ptr));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(byte_advance));
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(literal_value));
        function.instruction(&Instruction::LocalGet(byte_advance));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(literal_value));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(literal_value));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_advance));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(literal_value));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_record_ptr));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(named_candidate_ptr));
        function.instruction(&Instruction::LocalGet(named_record_ptr));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(named_candidate_count));
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::I64Const(STATIC_DATA_OFFSET as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_aggregate_count));
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(named_aggregate_count));
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(named_candidate_id));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(named_candidate_id));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_aggregate_count));
        function.instruction(&Instruction::LocalGet(named_candidate_total));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        // Metadata is carried by the object slot and therefore treated as
        // untrusted at this boundary too.
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::LocalGet(split_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(split_count));
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(2, input_offset, input_len, &mut function);
        // One-shot splits execute once per candidate; only cycle-reentered
        // splits can retain a frame at every consumed-byte position.
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_limit));
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(split_count));
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(choice_limit));
        function.instruction(&Instruction::LocalGet(split_count));
        function.instruction(&Instruction::LocalGet(repeatable_split_count));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_limit));
        function.instruction(&Instruction::LocalGet(choice_limit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, 3, 3, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        for local in [candidate_byte, candidate_utf16, candidate_on_low_surrogate] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        // Seek once from the beginning. A Unicode-mode start inside an astral
        // scalar normalizes to its leading code unit; non-Unicode matching
        // preserves the requested low-surrogate code-unit position.
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
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(candidate_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::LocalSet(candidate_utf16));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::End);
        // The byte cursor remains at the containing scalar in either mode.
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
        self.emit_increment_by_local(candidate_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Preserve a non-Unicode low-surrogate start as an empty-match
        // candidate, retaining its containing scalar's byte cursor.
        function.instruction(&Instruction::LocalGet(candidate_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::LocalSet(candidate_utf16));
        function.instruction(&Instruction::End);

        // A start beyond the string cannot produce a match, including an empty
        // one. A Unicode-normalized low-surrogate start is deliberately below
        // the raw start index while its byte cursor still points into input.
        function.instruction(&Instruction::LocalGet(candidate_utf16));
        function.instruction(&Instruction::LocalGet(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(candidate_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
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
        function.instruction(&Instruction::LocalGet(candidate_on_low_surrogate));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(choice_depth));

        // Each candidate owns a fresh capture vector.  -1/-1 is the unmatched
        // sentinel and is deliberately copied into every saved choice frame.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        for offset in [0u64, 8] {
            function.instruction(&Instruction::LocalGet(capture_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Store(Self::memarg8(offset)));
        }
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        /* Capture instructions are dispatched below, after their three words
         * have been loaded. */
        function.instruction(&Instruction::LocalGet(program_ptr));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(instruction_address));
        self.emit_regexp_instruction_load(instruction_address, 0, opcode, &mut function);
        self.emit_regexp_instruction_load(instruction_address, 8, operand0, &mut function);
        self.emit_regexp_instruction_load(instruction_address, 16, operand1, &mut function);
        // `.`, `^` and `$` carry a RegExp-modifier override in `operand0`: 0
        // defers to the pattern flag, 1 forces the mode on and 2 forces it off.
        for (source, effective) in [
            (multiline, effective_multiline),
            (dot_all, effective_dot_all),
        ] {
            function.instruction(&Instruction::LocalGet(operand0));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(operand0));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(effective));
        }

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LOOKBEHIND_START as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(reverse_mode));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(reverse_mode));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lookbehind_succeeded));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LOOKBEHIND_END as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reverse_mode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(choice_depth));
        function.instruction(&Instruction::LocalSet(lookbehind_frame_depth));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lookbehind_frame_depth));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(lookbehind_frame_depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(lookbehind_frame_depth));
        function.instruction(&Instruction::LocalGet(frame_width));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(choice_address));
        function.instruction(&Instruction::LocalGet(choice_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for (offset, local) in [
            (8, match_byte),
            (16, match_utf16),
            (24, match_on_low_surrogate),
        ] {
            function.instruction(&Instruction::LocalGet(choice_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(offset)));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        for offset in [0u64, 8] {
            function.instruction(&Instruction::LocalGet(capture_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(choice_address));
            function.instruction(&Instruction::LocalGet(capture_index));
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const((32 + offset) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(0)));
            function.instruction(&Instruction::I64Store(Self::memarg8(offset)));
        }
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(lookbehind_succeeded));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(lookbehind_frame_depth));
        function.instruction(&Instruction::LocalSet(choice_depth));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(reverse_mode));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LOOKBEHIND_FAILURE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(reverse_mode));
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(lookbehind_succeeded));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // Capture IDs are one based.  Starts and ends never consume input.
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_START as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reverse_mode));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Store(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Store(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Store(Self::memarg8(8)));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        // ClearCaptureRange is non-consuming and clears the canonical
        // half-open one-based range [operand0, operand1).
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_CLEAR_CAPTURE_RANGE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        for offset in [0u64, 8] {
            function.instruction(&Instruction::LocalGet(capture_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Store(Self::memarg8(offset)));
        }
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        // NamedBackreference selects the single participating capture for a
        // name. Multiple participating candidates are an invalid program
        // state; no participating candidate is the specified empty match.
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NAMED_BACKREFERENCE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NUMBERED_BACKREFERENCE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NUMBERED_BACKREFERENCE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalTee(capture_start));
        function.instruction(&Instruction::LocalSet(named_candidate_start));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(8)));
        function.instruction(&Instruction::LocalSet(named_capture_end));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(named_selected_count));
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(named_capture_end));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(named_selected_count));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(named_group_table_ptr));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(named_group_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_records_ptr));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(named_record_ptr));
        function.instruction(&Instruction::LocalGet(named_record_ptr));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(named_candidate_ptr));
        function.instruction(&Instruction::LocalGet(named_record_ptr));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(named_candidate_count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(named_selected_count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(named_candidate_id));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::LocalGet(named_candidate_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(named_candidate_ptr));
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(named_candidate_start));
        function.instruction(&Instruction::LocalGet(named_candidate_start));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(8)));
        function.instruction(&Instruction::LocalSet(named_capture_end));
        function.instruction(&Instruction::LocalGet(named_capture_end));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(named_candidate_start));
        function.instruction(&Instruction::LocalSet(capture_start));
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::LocalGet(named_capture_end));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(named_capture_end));
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_selected_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(named_selected_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_candidate_id));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(named_candidate_id));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(named_selected_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        // Seek the capture start from the input origin, one UTF-16 code unit
        // at a time. This permits a capture boundary at either astral half.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_byte));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_utf16));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_on_low_surrogate));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_utf16));
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(capture_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(input_offset, capture_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            capture_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(capture_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(capture_on_low_surrogate));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(capture_byte, byte_advance, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(capture_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_utf16));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_utf16));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::LocalSet(compare_byte));
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::LocalSet(compare_utf16));
        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::LocalSet(compare_on_low_surrogate));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_utf16));
        function.instruction(&Instruction::LocalGet(named_capture_end));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        // Decode and advance the capture cursor, retaining its current UTF-16 unit.
        self.emit_load_string_byte(input_offset, capture_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            capture_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(capture_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_unit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(capture_on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalSet(capture_unit));
        self.emit_increment_by_local(capture_byte, byte_advance, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_unit));
        self.emit_increment_by_local(capture_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_utf16));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_utf16));
        function.instruction(&Instruction::LocalGet(compare_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        self.emit_regexp_backtrack_or_fail(
            3,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        self.emit_load_string_byte(input_offset, compare_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            compare_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(compare_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_unit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(compare_on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalSet(compare_unit));
        self.emit_increment_by_local(compare_byte, byte_advance, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_unit));
        self.emit_increment_by_local(compare_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compare_utf16));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_utf16));
        function.instruction(&Instruction::LocalGet(capture_unit));
        function.instruction(&Instruction::LocalGet(compare_unit));
        function.instruction(&Instruction::I64Ne);
        self.emit_regexp_backtrack_or_fail(
            3,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compare_byte));
        function.instruction(&Instruction::LocalSet(match_byte));
        function.instruction(&Instruction::LocalGet(compare_utf16));
        function.instruction(&Instruction::LocalSet(match_utf16));
        function.instruction(&Instruction::LocalGet(compare_on_low_surrogate));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ASSERT_START as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ASSERT_END as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ASSERT_START as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(effective_multiline));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte));
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0x0A));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0x0D));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0xA8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0xA9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0x80));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0xE2));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(effective_multiline));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(input_offset, match_byte, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0x0A));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0x0D));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(0xE2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0x80));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0xA8));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0xA9));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Eqz);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(127));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        self.emit_regexp_backtrack_or_fail(
            2,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        self.emit_load_string_byte(input_offset, match_byte, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Ne);
        self.emit_regexp_backtrack_or_fail(
            2,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(input_offset, match_byte, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Eq);
        self.emit_regexp_backtrack_or_fail(
            3,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_END as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reverse_mode));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Store(Self::memarg8(8)));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(capture_start));
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Store(Self::memarg8(8)));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ACCEPT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A corrupt program cannot manufacture invalid capture boundaries.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        function.instruction(&Instruction::LocalSet(capture_start));
        function.instruction(&Instruction::LocalGet(capture_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(8)));
        function.instruction(&Instruction::LocalSet(operand0));
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
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
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::LocalGet(instruction_count));
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
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(choice_depth));
        function.instruction(&Instruction::LocalGet(frame_width));
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
        function.instruction(&Instruction::LocalGet(choice_address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Store(Self::memarg8(24)));
        // Snapshot the entire current capture vector after the frame header.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        for offset in [0u64, 8] {
            function.instruction(&Instruction::LocalGet(choice_address));
            function.instruction(&Instruction::LocalGet(capture_index));
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const((32 + offset) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(capture_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(offset)));
            function.instruction(&Instruction::I64Store(Self::memarg8(0)));
        }
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
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
        function.instruction(&Instruction::LocalGet(instruction_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reverse_mode));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LITERAL_ASCII as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NEGATIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::I64Eqz);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );

        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(match_byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(previous_byte));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_offset));
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(0xc0));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Const(0x80));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(previous_byte));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(input_offset, previous_byte, byte, &mut function);
        self.emit_decode_utf8_scalar_at_index(
            input_offset,
            previous_byte,
            input_len,
            byte,
            codepoint,
            byte_advance,
            decode_temp,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(previous_byte));
        function.instruction(&Instruction::LocalSet(match_byte));
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
        function.instruction(&Instruction::LocalSet(codepoint));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LITERAL_ASCII as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Ne);
        self.emit_regexp_backtrack_or_fail(
            2,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::LocalGet(utf16_advance));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_utf16));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for line_terminator in [0x000A_i64, 0x000D, 0x2028, 0x2029] {
            function.instruction(&Instruction::LocalGet(codepoint));
            function.instruction(&Instruction::I64Const(line_terminator));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..4 {
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::LocalGet(effective_dot_all));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.emit_regexp_backtrack_or_fail(
            2,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::Else);
        self.emit_regexp_ascii_class_contains(codepoint, operand0, operand1, &mut function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(decode_temp));
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(decode_temp));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::End);
        self.emit_regexp_backtrack_or_fail(
            2,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(match_utf16));
        function.instruction(&Instruction::LocalGet(utf16_advance));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_utf16));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_LITERAL_ASCII as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LITERAL_CODE_POINT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_UNICODE_PROPERTY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_POSITIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NEGATIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_NOT_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        // Canonical operands are part of program validation, even when the
        // current input position cannot satisfy a consuming instruction.
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_NOT_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LITERAL_CODE_POINT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Const(0x10ffff));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_UNICODE_PROPERTY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The referenced slice of the range pool must stay inside static data.
        function.instruction(&Instruction::LocalGet(range_base));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 1, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Dot, an explicit UTF-16/code-point literal, and a negative ASCII
        // class may consume the low half of an astral scalar when matching
        // begins at that code-unit position.
        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LITERAL_CODE_POINT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NEGATIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_NOT_WHITESPACE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.emit_regexp_backtrack_or_fail(
            0,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
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
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
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
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_DOT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for line_terminator in [0x000A_i64, 0x000D, 0x2028, 0x2029] {
            function.instruction(&Instruction::LocalGet(codepoint));
            function.instruction(&Instruction::I64Const(line_terminator));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..4 {
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::LocalGet(effective_dot_all));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_UNICODE_PROPERTY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Binary search the sorted, disjoint range slice
        // `[operand0, operand0 + operand1 >> 1)` for `codepoint`.
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(range_count));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(range_low));
        function.instruction(&Instruction::LocalGet(range_count));
        function.instruction(&Instruction::LocalSet(range_high));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(range_low));
        function.instruction(&Instruction::LocalGet(range_high));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(range_low));
        function.instruction(&Instruction::LocalGet(range_high));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(range_middle));
        function.instruction(&Instruction::LocalGet(codepoint));
        self.emit_regexp_range_bound_load(range_base, operand0, range_middle, 4, &mut function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(range_middle));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(range_low));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(range_middle));
        function.instruction(&Instruction::LocalSet(range_high));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(range_low));
        function.instruction(&Instruction::LocalGet(range_count));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(codepoint));
        self.emit_regexp_range_bound_load(range_base, operand0, range_low, 0, &mut function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(operand1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::I64Eqz);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_NEGATIVE_ASCII_CLASS as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x7f));
        function.instruction(&Instruction::I64LeU);
        self.emit_regexp_ascii_class_contains(codepoint, operand0, operand1, &mut function);
        function.instruction(&Instruction::I32And);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LITERAL_CODE_POINT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::LocalSet(literal_value));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(literal_advance_byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));

        function.instruction(&Instruction::LocalGet(match_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(literal_value));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(literal_advance_byte));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(literal_value));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(literal_value));
        function.instruction(&Instruction::LocalGet(operand0));
        function.instruction(&Instruction::I64Ne);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(literal_advance_byte));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_on_low_surrogate));
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let whitespace_members = [
            0x0009_i64, 0x000A, 0x000B, 0x000C, 0x000D, 0x0020, 0x00A0, 0x1680, 0x2000, 0x2001,
            0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200A, 0x2028, 0x2029,
            0x202F, 0x205F, 0x3000, 0xFEFF,
        ];
        for member in whitespace_members {
            function.instruction(&Instruction::LocalGet(codepoint));
            function.instruction(&Instruction::I64Const(member));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..whitespace_members.len() {
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        self.emit_increment_by_local(pc, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_NOT_WHITESPACE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let whitespace_members = [
            0x0009_i64, 0x000A, 0x000B, 0x000C, 0x000D, 0x0020, 0x00A0, 0x1680, 0x2000, 0x2001,
            0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200A, 0x2028, 0x2029,
            0x202F, 0x205F, 0x3000, 0xFEFF,
        ];
        for member in whitespace_members {
            function.instruction(&Instruction::LocalGet(codepoint));
            function.instruction(&Instruction::I64Const(member));
            function.instruction(&Instruction::I64Eq);
        }
        for _ in 1..whitespace_members.len() {
            function.instruction(&Instruction::I32Or);
        }
        self.emit_regexp_backtrack_or_fail(
            1,
            choice_depth,
            choice_address,
            match_byte,
            match_utf16,
            pc,
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
            &mut function,
        );
        self.emit_increment_by_local(match_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        self.emit_increment_by_local(match_utf16, utf16_advance, &mut function);
        self.emit_increment_by_local(pc, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
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
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
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
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
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
            match_on_low_surrogate,
            capture_count,
            frame_width,
            capture_index,
            capture_address,
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

        function.instruction(&Instruction::LocalGet(sticky));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_on_low_surrogate));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
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
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
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
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::LocalGet(unicode));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Non-Unicode search exposes the low UTF-16 half as the next
        // candidate, retaining the scalar's byte cursor for code-unit atoms.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_increment_by_local(candidate_byte, byte_advance, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(candidate_on_low_surrogate));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(utf16_advance));
        function.instruction(&Instruction::End);
        self.emit_increment_by_local(candidate_utf16, utf16_advance, &mut function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_regexp_match_result(0, candidate_utf16, candidate_utf16, 0, &mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// On an atom failure, restore the latest ordered fallback. If none exists,
    /// branch out of the instruction loop to advance the unanchored candidate.
    ///
    /// `caller_block_depth` is a **raw** branch offset, not a
    /// `ControlTarget`-relative one: this whole matcher is a self-contained
    /// emitted body whose frames it opens and closes itself, so its `Br`
    /// immediates are relative to the position they are written at and the
    /// label-depth work does not move them. See the closing section of
    /// `code_sink.rs` for why this shape is kept rather than converted.
    fn emit_regexp_backtrack_or_fail(
        &self,
        caller_block_depth: u32,
        depth: u32,
        address: u32,
        byte: u32,
        utf16: u32,
        pc: u32,
        on_low_surrogate: u32,
        capture_count: u32,
        frame_width: u32,
        capture_index: u32,
        capture_address: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(3 + caller_block_depth));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(depth));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(depth));
        function.instruction(&Instruction::LocalGet(frame_width));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
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
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(24)));
        function.instruction(&Instruction::LocalSet(on_low_surrogate));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::LocalGet(capture_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_address));
        for offset in [0u64, 8] {
            function.instruction(&Instruction::LocalGet(capture_address));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(address));
            function.instruction(&Instruction::LocalGet(capture_index));
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const((32 + offset) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(0)));
            function.instruction(&Instruction::I64Store(Self::memarg8(offset)));
        }
        function.instruction(&Instruction::LocalGet(capture_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capture_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1 + caller_block_depth));
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

    /// Pushes the inclusive `start` (`field` 0) or `end` (`field` 4) bound of
    /// range-pool entry `first_entry_local + index_local` as an i64.
    fn emit_regexp_range_bound_load(
        &self,
        range_base_local: u32,
        first_entry_local: u32,
        index_local: u32,
        field: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(range_base_local));
        function.instruction(&Instruction::LocalGet(first_entry_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(field)));
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
