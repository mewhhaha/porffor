//! Publish exact source-owned decimal digits after the instruction/range prefix.
use super::*;

impl FunctionBuilder<'_> {
    fn emit_regexp_compile_bound_digits(
        &mut self,
        compiler: &CompilerLocals,
        node: I64Local,
        row: I64Local,
        source_words: (NodeWord, NodeWord, NodeWord),
        output_words: (RegExpRepeatBoundWord, RegExpRepeatBoundWord),
        length: I64Local,
        digit_length: I64Local,
        function: &mut Function,
    ) {
        let start = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        let class = self.runtime_schema().reserve_i64_local(function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let destination = self.runtime_schema().reserve_i64_local(function);
        for (word, output) in [
            (source_words.0, start),
            (source_words.1, end),
            (source_words.2, class),
        ] {
            self.emit_regexp_scratch_load_word(node, word as u64, output, function);
        }
        start.load(function);
        end.load(function);
        function.instruction(&Instruction::I64GtU);
        end.load(function);
        compiler.unit_count.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        end.load(function);
        start.load(function);
        function.instruction(&Instruction::I64Sub);
        digit_length.store(function);
        self.emit_regexp_scratch_store_word(row, output_words.0.offset(), length, function);
        compiler.descriptor.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Add);
        destination.store(function);
        digit_length.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Only the typed default/Annex B zero and one owners have no span.
        class.load(function);
        function.instruction(&Instruction::I64Const(BoundClass::One as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        destination.load(function);
        function.instruction(&Instruction::I32WrapI64);
        class.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::I64Const(1));
        digit_length.store(function);
        function.instruction(&Instruction::Else);
        start.load(function);
        cursor.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(function);
        end.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        compiler.units.load(function);
        cursor.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        digit.store(function);
        destination.load(function);
        function.instruction(&Instruction::I32WrapI64);
        digit.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        self.emit_regexp_scratch_increment(cursor, 1, function);
        self.emit_regexp_scratch_increment(destination, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_store_word(row, output_words.1.offset(), digit_length, function);
        self.emit_regexp_scratch_increment_by_local(length, digit_length, function);
        digit_length.load(function);
        function.instruction(&Instruction::I64Const(
            (REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS - 1) as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64,
        ));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_LIMB_WIDTH as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        compiler.repeat_state_byte_length.load(function);
        function.instruction(&Instruction::I64Add);
        compiler.repeat_state_byte_length.store(function);
        for local in [destination, digit, cursor, class, end, start] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_regexp_compile_repeat_bounds(
        &mut self,
        compiler: &CompilerLocals,
        length: I64Local,
        function: &mut Function,
    ) {
        let rows = self.runtime_schema().reserve_i64_local(function);
        let row = self.runtime_schema().reserve_i64_local(function);
        let slot = self.runtime_schema().reserve_i64_local(function);
        let node = self.runtime_schema().reserve_i64_local(function);
        let node_address = self.runtime_schema().reserve_i64_local(function);
        let kind = self.runtime_schema().reserve_i64_local(function);
        let digit_length = self.runtime_schema().reserve_i64_local(function);
        let padded = self.runtime_schema().reserve_i64_local(function);
        compiler.descriptor.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Add);
        rows.store(function);
        length.load(function);
        compiler.repeat_slot_count.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_BOUND_RECORD_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        length.store(function);
        for local in [slot, compiler.repeat_state_byte_length] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        slot.load(function);
        compiler.repeat_slot_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        rows.load(function);
        slot.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_BOUND_RECORD_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        row.store(function);
        compiler.repeat_rows.load(function);
        slot.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(0)));
        node.store(function);
        self.emit_regexp_node_address(compiler, node, node_address, function);
        self.emit_regexp_scratch_store_word(
            row,
            RegExpRepeatBoundWord::StateOffset.offset(),
            compiler.repeat_state_byte_length,
            function,
        );
        self.emit_regexp_scratch_increment(
            compiler.repeat_state_byte_length,
            REGEXP_REPEAT_STATE_HEADER_SIZE as i64,
            function,
        );
        self.emit_regexp_compile_bound_digits(
            compiler,
            node_address,
            row,
            (
                NodeWord::MinimumDigitsStart,
                NodeWord::MinimumDigitsEnd,
                NodeWord::Minimum,
            ),
            (
                RegExpRepeatBoundWord::MinimumDigitsOffset,
                RegExpRepeatBoundWord::MinimumDigitsLength,
            ),
            length,
            digit_length,
            function,
        );
        self.emit_regexp_scratch_load_word(
            node_address,
            NodeWord::MaximumKind as u64,
            kind,
            function,
        );
        self.emit_regexp_scratch_store_word(
            row,
            RegExpRepeatBoundWord::MaximumKind.offset(),
            kind,
            function,
        );
        kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_bound_digits(
            compiler,
            node_address,
            row,
            (
                NodeWord::MaximumDigitsStart,
                NodeWord::MaximumDigitsEnd,
                NodeWord::Maximum,
            ),
            (
                RegExpRepeatBoundWord::MaximumDigitsOffset,
                RegExpRepeatBoundWord::MaximumDigitsLength,
            ),
            length,
            digit_length,
            function,
        );
        function.instruction(&Instruction::Else);
        for word in [
            RegExpRepeatBoundWord::MaximumDigitsOffset,
            RegExpRepeatBoundWord::MaximumDigitsLength,
        ] {
            self.emit_regexp_scratch_store_word_const(row, word.offset(), 0, function);
        }
        function.instruction(&Instruction::End);
        compiler.repeat_state_byte_length.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        compiler.repeat_state_byte_length.store(function);
        self.emit_regexp_scratch_increment(slot, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        length.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        padded.store(function);
        compiler.descriptor.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        padded.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        padded.load(function);
        length.store(function);
        for local in [
            padded,
            digit_length,
            kind,
            node_address,
            node,
            slot,
            row,
            rows,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
