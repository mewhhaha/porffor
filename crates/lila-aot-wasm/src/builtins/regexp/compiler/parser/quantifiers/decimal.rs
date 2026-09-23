use super::*;

impl FunctionBuilder<'_> {
    /// Keeps a finite saturated value distinct from the unbounded sentinel.
    /// The caller compares the original decimal spans before using this value.
    pub(in super::super) fn emit_regexp_parser_decimal(
        &mut self,
        compiler: &CompilerLocals,
        cursor: u32,
        value: u32,
        oversized: u32,
        function: &mut Function,
    ) {
        let digit = self.reserve_temp_local();
        set(function, value, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, cursor, 0, digit);
        between(function, digit, b'0' as u64, b'9' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(((u64::MAX - 1) / 10) as i64));
        function.instruction(&Instruction::I64GtU);
        eq(function, value, (u64::MAX - 1) / 10);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(((u64::MAX - 1) % 10) as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, value, u64::MAX - 1);
        set(function, oversized, 1);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(value));
        function.instruction(&Instruction::End);
        self.emit_increment_local(cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(digit);
    }

    pub(super) fn emit_regexp_parser_ordered_decimals(
        &mut self,
        compiler: &CompilerLocals,
        min_start: u32,
        min_end: u32,
        max_start: u32,
        max_end: u32,
        function: &mut Function,
    ) {
        let left = self.reserve_temp_local();
        let right = self.reserve_temp_local();
        let left_digit = self.reserve_temp_local();
        let right_digit = self.reserve_temp_local();
        let left_len = self.reserve_temp_local();
        let right_len = self.reserve_temp_local();
        for (start, end, cursor, length) in [
            (min_start, min_end, left, left_len),
            (max_start, max_end, right, right_len),
        ] {
            copy(function, cursor, start);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(cursor));
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            peek(compiler, function, cursor, 0, left_digit);
            eq(function, left_digit, b'0' as u64);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(1));
            self.emit_increment_local(cursor, 1, function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::LocalGet(cursor));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(length));
        }
        function.instruction(&Instruction::LocalGet(left_len));
        function.instruction(&Instruction::LocalGet(right_len));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(left_len));
        function.instruction(&Instruction::LocalGet(right_len));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(left));
        function.instruction(&Instruction::LocalGet(min_end));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, left, 0, left_digit);
        peek(compiler, function, right, 0, right_digit);
        function.instruction(&Instruction::LocalGet(left_digit));
        function.instruction(&Instruction::LocalGet(right_digit));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(left_digit));
        function.instruction(&Instruction::LocalGet(right_digit));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_local(left, 1, function);
        self.emit_increment_local(right, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [right_len, left_len, right_digit, left_digit, right, left] {
            self.release_temp_local(local);
        }
    }
}
