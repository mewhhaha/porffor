use super::*;

#[derive(Clone, Copy)]
enum BoundarySide {
    Before,
    After,
}

impl FunctionBuilder<'_> {
    /// Pushes a mismatch without advancing the matcher cursor or changing captures.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_regexp_word_boundary_mismatch(
        &mut self,
        input_offset: u32,
        input_len: u32,
        cursor_byte: u32,
        cursor_on_low_surrogate: u32,
        unicode: u32,
        range_base: u32,
        range_capacity: u32,
        first_entry: u32,
        packed_count_and_polarity: u32,
        candidate_utf16: u32,
        function: &mut Function,
    ) {
        let range_count = self.reserve_temp_local();
        let range_low = self.reserve_temp_local();
        let range_high = self.reserve_temp_local();
        let range_middle = self.reserve_temp_local();
        let packed_count = self.reserve_temp_local();
        let before_word = self.reserve_temp_local();
        let after_word = self.reserve_temp_local();
        let adjacent_byte = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let codepoint = self.reserve_temp_local();
        let byte_advance = self.reserve_temp_local();
        let decode_temp = self.reserve_temp_local();

        // This capacity belongs to the descriptor's range section, excluding
        // all neighboring instruction, name and unrelated allocation bytes.
        function.instruction(&Instruction::LocalGet(packed_count_and_polarity));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(range_count));
        function.instruction(&Instruction::LocalGet(first_entry));
        function.instruction(&Instruction::LocalGet(range_capacity));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(range_count));
        function.instruction(&Instruction::LocalGet(range_capacity));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(first_entry));
        function.instruction(&Instruction::LocalGet(range_count));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(range_capacity));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(range_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(
            candidate_utf16,
            candidate_utf16,
            RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
            function,
        );
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(packed_count_and_polarity));
        function.instruction(&Instruction::I64Const(!1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(packed_count));

        for (side, word) in [
            (BoundarySide::Before, before_word),
            (BoundarySide::After, after_word),
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(word));
            match side {
                BoundarySide::Before => {
                    function.instruction(&Instruction::LocalGet(cursor_byte));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::LocalGet(cursor_on_low_surrogate));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::I32Or);
                }
                BoundarySide::After => {
                    function.instruction(&Instruction::LocalGet(cursor_byte));
                    function.instruction(&Instruction::LocalGet(input_len));
                    function.instruction(&Instruction::I64LtU);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(cursor_byte));
            function.instruction(&Instruction::LocalSet(adjacent_byte));
            if matches!(side, BoundarySide::Before) {
                function.instruction(&Instruction::LocalGet(cursor_on_low_surrogate));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(adjacent_byte));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(adjacent_byte));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                self.emit_load_string_byte(input_offset, adjacent_byte, byte, function);
                function.instruction(&Instruction::LocalGet(byte));
                function.instruction(&Instruction::I64Const(0xc0));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0x80));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(adjacent_byte));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_match_result(
                    candidate_utf16,
                    candidate_utf16,
                    RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
                    function,
                );
                function.instruction(&Instruction::Return);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(adjacent_byte));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(adjacent_byte));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            self.emit_load_string_byte(input_offset, adjacent_byte, byte, function);
            self.emit_decode_utf8_scalar_at_index(
                input_offset,
                adjacent_byte,
                input_len,
                byte,
                codepoint,
                byte_advance,
                decode_temp,
                function,
            );

            // A legacy cursor can sit between the two UTF-16 units represented
            // by one UTF-8 scalar. An adjacent astral scalar is split only in
            // legacy mode; Unicode mode classifies the entire code point.
            function.instruction(&Instruction::LocalGet(cursor_on_low_surrogate));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(codepoint));
            function.instruction(&Instruction::I64Const(0x10000));
            function.instruction(&Instruction::I64Sub);
            match side {
                BoundarySide::Before => {
                    function.instruction(&Instruction::I64Const(10));
                    function.instruction(&Instruction::I64ShrU);
                    function.instruction(&Instruction::I64Const(0xd800));
                }
                BoundarySide::After => {
                    function.instruction(&Instruction::I64Const(0x3ff));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Const(0xdc00));
                }
            }
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(codepoint));
            function.instruction(&Instruction::Else);
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
            match side {
                BoundarySide::Before => {
                    function.instruction(&Instruction::I64Const(0x3ff));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Const(0xdc00));
                }
                BoundarySide::After => {
                    function.instruction(&Instruction::I64Const(10));
                    function.instruction(&Instruction::I64ShrU);
                    function.instruction(&Instruction::I64Const(0xd800));
                }
            }
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(codepoint));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);

            self.emit_regexp_unicode_property_mismatch(
                range_base,
                first_entry,
                packed_count,
                codepoint,
                range_count,
                range_low,
                range_high,
                range_middle,
                function,
            );
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(word));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(before_word));
        function.instruction(&Instruction::LocalGet(after_word));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::LocalGet(packed_count_and_polarity));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eq);

        self.release_temp_local(decode_temp);
        self.release_temp_local(byte_advance);
        self.release_temp_local(codepoint);
        self.release_temp_local(byte);
        self.release_temp_local(adjacent_byte);
        self.release_temp_local(after_word);
        self.release_temp_local(before_word);
        self.release_temp_local(packed_count);
        self.release_temp_local(range_middle);
        self.release_temp_local(range_high);
        self.release_temp_local(range_low);
        self.release_temp_local(range_count);
    }
}
