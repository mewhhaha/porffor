use super::*;

impl FunctionBuilder<'_> {
    /// Decimal backreference probing only: an overflow remains outside the
    /// source-sized capture domain. Quantifier counts never use this value.
    pub(in super::super) fn emit_regexp_parser_backreference_decimal(
        &mut self,
        compiler: &CompilerLocals,
        cursor: I64Local,
        value: I64Local,
        oversized: I64Local,
        function: &mut Function,
    ) {
        let digit = self.runtime_schema().reserve_i64_local(function);
        set(function, value, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, cursor, 0, digit);
        between(function, digit, b'0' as u64, b'9' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        digit.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        digit.store(function);
        value.load(function);
        function.instruction(&Instruction::I64Const(((u64::MAX - 1) / 10) as i64));
        function.instruction(&Instruction::I64GtU);
        eq(function, value, (u64::MAX - 1) / 10);
        digit.load(function);
        function.instruction(&Instruction::I64Const(((u64::MAX - 1) % 10) as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, value, u64::MAX - 1);
        set(function, oversized, 1);
        function.instruction(&Instruction::Else);
        value.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        value.store(function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(digit, function);
    }

    /// Retain canonical source spans and classify only zero/one/many. The
    /// classification chooses a physical layout; it never becomes a counter.
    pub(super) fn emit_regexp_parser_bound_digits(
        &mut self,
        compiler: &CompilerLocals,
        cursor: I64Local,
        class: I64Local,
        start: I64Local,
        end: I64Local,
        function: &mut Function,
    ) {
        let digit = self.runtime_schema().reserve_i64_local(function);
        copy(function, start, cursor);
        set(function, class, BoundClass::Zero as u64);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, cursor, 0, digit);
        between(function, digit, b'0' as u64, b'9' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        class.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        class.store(function);
        class.load(function);
        function.instruction(&Instruction::I64Const(BoundClass::Many as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, class, BoundClass::Many as u64);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        copy(function, end, cursor);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        start.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        end.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, start, 0, digit);
        eq(function, digit, b'0' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_increment(start, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(digit, function);
    }

    pub(super) fn emit_regexp_parser_ordered_decimals(
        &mut self,
        compiler: &CompilerLocals,
        min_start: I64Local,
        min_end: I64Local,
        max_start: I64Local,
        max_end: I64Local,
        function: &mut Function,
    ) {
        let left = self.runtime_schema().reserve_i64_local(function);
        let right = self.runtime_schema().reserve_i64_local(function);
        let left_digit = self.runtime_schema().reserve_i64_local(function);
        let right_digit = self.runtime_schema().reserve_i64_local(function);
        let left_len = self.runtime_schema().reserve_i64_local(function);
        let right_len = self.runtime_schema().reserve_i64_local(function);
        for (start, end, cursor, length) in [
            (min_start, min_end, left, left_len),
            (max_start, max_end, right, right_len),
        ] {
            copy(function, cursor, start);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            cursor.load(function);
            end.load(function);
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            peek(compiler, function, cursor, 0, left_digit);
            eq(function, left_digit, b'0' as u64);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(1));
            self.emit_regexp_scratch_increment(cursor, 1, function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            end.load(function);
            cursor.load(function);
            function.instruction(&Instruction::I64Sub);
            length.store(function);
        }
        left_len.load(function);
        right_len.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        left_len.load(function);
        right_len.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        left.load(function);
        min_end.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, left, 0, left_digit);
        peek(compiler, function, right, 0, right_digit);
        left_digit.load(function);
        right_digit.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        left_digit.load(function);
        right_digit.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(left, 1, function);
        self.emit_regexp_scratch_increment(right, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [right_len, left_len, right_digit, left_digit, right, left] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
