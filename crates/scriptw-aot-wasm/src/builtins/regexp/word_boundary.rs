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
        checkpoint: I64Local,
        input_offset: I64Local,
        input_len: I64Local,
        cursor_byte: I64Local,
        cursor_on_low_surrogate: I64Local,
        unicode: I64Local,
        range_base: I64Local,
        range_capacity: I64Local,
        first_entry: I64Local,
        packed_count_and_polarity: I64Local,
        candidate_utf16: I64Local,
        function: &mut Function,
    ) {
        let range_count = self.runtime_schema().reserve_i64_local(function);
        let range_low = self.runtime_schema().reserve_i64_local(function);
        let range_high = self.runtime_schema().reserve_i64_local(function);
        let range_middle = self.runtime_schema().reserve_i64_local(function);
        let packed_count = self.runtime_schema().reserve_i64_local(function);
        let before_word = self.runtime_schema().reserve_i64_local(function);
        let after_word = self.runtime_schema().reserve_i64_local(function);
        let adjacent_byte = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        let codepoint = self.runtime_schema().reserve_i64_local(function);
        let byte_advance = self.runtime_schema().reserve_i64_local(function);
        let decode_temp = self.runtime_schema().reserve_i64_local(function);

        // This capacity belongs to the descriptor's range section, excluding
        // all neighboring instruction, name and unrelated allocation bytes.
        packed_count_and_polarity.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        range_count.store(function);
        first_entry.load(function);
        range_capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        range_count.load(function);
        range_capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        first_entry.load(function);
        range_count.load(function);
        function.instruction(&Instruction::I64Add);
        range_capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        range_count.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_match_result(
            checkpoint,
            candidate_utf16,
            candidate_utf16,
            RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
            function,
        );
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        packed_count_and_polarity.load(function);
        function.instruction(&Instruction::I64Const(!1));
        function.instruction(&Instruction::I64And);
        packed_count.store(function);

        for (side, word) in [
            (BoundarySide::Before, before_word),
            (BoundarySide::After, after_word),
        ] {
            function.instruction(&Instruction::I64Const(0));
            word.store(function);
            match side {
                BoundarySide::Before => {
                    cursor_byte.load(function);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    cursor_on_low_surrogate.load(function);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::I32Or);
                }
                BoundarySide::After => {
                    cursor_byte.load(function);
                    input_len.load(function);
                    function.instruction(&Instruction::I64LtU);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            cursor_byte.load(function);
            adjacent_byte.store(function);
            if matches!(side, BoundarySide::Before) {
                cursor_on_low_surrogate.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                adjacent_byte.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                adjacent_byte.store(function);
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                self.emit_regexp_scratch_byte(input_offset, adjacent_byte, byte, function);
                byte.load(function);
                function.instruction(&Instruction::I64Const(0xc0));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0x80));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::BrIf(1));
                adjacent_byte.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_match_result(
                    checkpoint,
                    candidate_utf16,
                    candidate_utf16,
                    RegExpMatcherResult::Failed(RegExpMatcherFailure::CorruptProgram),
                    function,
                );
                function.instruction(&Instruction::Return);
                function.instruction(&Instruction::End);
                adjacent_byte.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                adjacent_byte.store(function);
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            self.emit_regexp_scratch_byte(input_offset, adjacent_byte, byte, function);
            self.emit_regexp_scratch_decode_scalar(
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
            cursor_on_low_surrogate.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            codepoint.load(function);
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
            codepoint.store(function);
            function.instruction(&Instruction::Else);
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
            codepoint.store(function);
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
            word.store(function);
            function.instruction(&Instruction::End);
        }
        before_word.load(function);
        after_word.load(function);
        function.instruction(&Instruction::I64Xor);
        packed_count_and_polarity.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eq);

        self.runtime_schema()
            .release_i64_local(decode_temp, function);
        self.runtime_schema()
            .release_i64_local(byte_advance, function);
        self.runtime_schema().release_i64_local(codepoint, function);
        self.runtime_schema().release_i64_local(byte, function);
        self.runtime_schema()
            .release_i64_local(adjacent_byte, function);
        self.runtime_schema()
            .release_i64_local(after_word, function);
        self.runtime_schema()
            .release_i64_local(before_word, function);
        self.runtime_schema()
            .release_i64_local(packed_count, function);
        self.runtime_schema()
            .release_i64_local(range_middle, function);
        self.runtime_schema()
            .release_i64_local(range_high, function);
        self.runtime_schema().release_i64_local(range_low, function);
        self.runtime_schema()
            .release_i64_local(range_count, function);
    }
}
