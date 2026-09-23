use super::*;
mod decimal;

pub(super) struct ParsedBounds {
    pub(super) minimum: u32,
    pub(super) maximum: u32,
    pub(super) oversized: u32,
    pub(super) matched: u32,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_quantifier(
        &mut self,
        compiler: &CompilerLocals,
        term: u32,
        function: &mut Function,
    ) {
        let bounds = ParsedBounds {
            minimum: self.reserve_temp_local(),
            maximum: self.reserve_temp_local(),
            oversized: self.reserve_temp_local(),
            matched: self.reserve_temp_local(),
        };
        let start = self.reserve_temp_local();
        let unit = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let flags = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let opcode = self.reserve_temp_local();
        copy(function, start, compiler.cursor);
        set(function, bounds.minimum, 1);
        set(function, bounds.maximum, 1);
        set(function, bounds.matched, 0);
        set(function, bounds.oversized, 0);
        peek(compiler, function, compiler.cursor, 0, unit);
        for (token, minimum, maximum) in [(b'?', 0, 1), (b'*', 0, UNBOUNDED), (b'+', 1, UNBOUNDED)]
        {
            eq(function, unit, token as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, bounds.minimum, minimum);
            set(function, bounds.maximum, maximum);
            set(function, bounds.matched, 1);
            self.emit_increment_local(compiler.cursor, 1, function);
            function.instruction(&Instruction::End);
        }
        eq(function, unit, b'{' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_braced_bounds(compiler, &bounds, function);
        function.instruction(&Instruction::End);
        eq(function, bounds.matched, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_node_address(compiler, term, address, function);
        store(function, address, NodeWord::SourceOffset as u64, start);
        load(function, address, NodeWord::Flags as u64, flags);
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b'?' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags));
        self.emit_increment_local(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);
        load(function, address, NodeWord::Kind as u64, kind);
        load(function, address, NodeWord::Opcode as u64, opcode);
        eq(function, kind, NodeKind::Atom as u64);
        eq(function, opcode, REGEXP_OPCODE_ASSERT_START);
        eq(function, opcode, REGEXP_OPCODE_ASSERT_END);
        function.instruction(&Instruction::I32Or);
        eq(function, opcode, REGEXP_OPCODE_WORD_BOUNDARY);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, compiler.cursor, start);
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, kind, NodeKind::PositiveLookahead as u64);
        eq(function, kind, NodeKind::NegativeLookahead as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, bounds.minimum, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(bounds.minimum));
        copy(function, bounds.maximum, bounds.minimum);
        set(function, bounds.oversized, 0);
        function.instruction(&Instruction::End);
        eq(function, bounds.oversized, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_OVERSIZED_BOUNDS as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags));
        function.instruction(&Instruction::End);
        store(function, address, NodeWord::Flags as u64, flags);
        store(function, address, NodeWord::Minimum as u64, bounds.minimum);
        store(function, address, NodeWord::Maximum as u64, bounds.maximum);
        function.instruction(&Instruction::End);
        for local in [
            opcode,
            kind,
            flags,
            address,
            unit,
            start,
            bounds.matched,
            bounds.oversized,
            bounds.maximum,
            bounds.minimum,
        ] {
            self.release_temp_local(local);
        }
    }

    pub(super) fn emit_regexp_parser_braced_bounds(
        &mut self,
        compiler: &CompilerLocals,
        bounds: &ParsedBounds,
        function: &mut Function,
    ) {
        let cursor = self.reserve_temp_local();
        let min_start = self.reserve_temp_local();
        let min_end = self.reserve_temp_local();
        let max_start = self.reserve_temp_local();
        let max_end = self.reserve_temp_local();
        let unit = self.reserve_temp_local();
        let minimum = self.reserve_temp_local();
        let maximum = self.reserve_temp_local();
        let oversized = self.reserve_temp_local();
        set(function, bounds.matched, 0);
        set(function, oversized, 0);
        copy(function, cursor, compiler.cursor);
        self.emit_increment_local(cursor, 1, function);
        copy(function, min_start, cursor);
        self.emit_regexp_parser_decimal(compiler, cursor, minimum, oversized, function);
        copy(function, min_end, cursor);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(min_start));
        function.instruction(&Instruction::LocalGet(min_end));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        copy(function, maximum, minimum);
        copy(function, max_start, min_start);
        copy(function, max_end, min_end);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b',' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(cursor, 1, function);
        copy(function, max_start, cursor);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, maximum, UNBOUNDED);
        function.instruction(&Instruction::Else);
        self.emit_regexp_parser_decimal(compiler, cursor, maximum, oversized, function);
        function.instruction(&Instruction::LocalGet(max_start));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        copy(function, max_end, cursor);
        function.instruction(&Instruction::End);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        eq(function, maximum, UNBOUNDED);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_ordered_decimals(
            compiler, min_start, min_end, max_start, max_end, function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_local(cursor, 1, function);
        copy(function, compiler.cursor, cursor);
        copy(function, bounds.minimum, minimum);
        copy(function, bounds.maximum, maximum);
        copy(function, bounds.oversized, oversized);
        set(function, bounds.matched, 1);
        function.instruction(&Instruction::End);
        for local in [
            oversized, maximum, minimum, unit, max_end, max_start, min_end, min_start, cursor,
        ] {
            self.release_temp_local(local);
        }
    }
}
