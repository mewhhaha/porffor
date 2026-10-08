use super::*;

mod decimal;

pub(super) struct ParsedBounds {
    pub(super) minimum: I64Local,
    pub(super) maximum: I64Local,
    pub(super) maximum_kind: I64Local,
    pub(super) minimum_start: I64Local,
    pub(super) minimum_end: I64Local,
    pub(super) maximum_start: I64Local,
    pub(super) maximum_end: I64Local,
    pub(super) matched: I64Local,
}

impl FunctionBuilder<'_> {
    pub(super) fn reserve_regexp_parsed_bounds(&mut self, function: &mut Function) -> ParsedBounds {
        ParsedBounds {
            minimum: self.runtime_schema().reserve_i64_local(function),
            maximum: self.runtime_schema().reserve_i64_local(function),
            maximum_kind: self.runtime_schema().reserve_i64_local(function),
            minimum_start: self.runtime_schema().reserve_i64_local(function),
            minimum_end: self.runtime_schema().reserve_i64_local(function),
            maximum_start: self.runtime_schema().reserve_i64_local(function),
            maximum_end: self.runtime_schema().reserve_i64_local(function),
            matched: self.runtime_schema().reserve_i64_local(function),
        }
    }
    pub(super) fn release_regexp_parsed_bounds(
        &mut self,
        bounds: ParsedBounds,
        function: &mut Function,
    ) {
        for local in [
            bounds.matched,
            bounds.maximum_end,
            bounds.maximum_start,
            bounds.minimum_end,
            bounds.minimum_start,
            bounds.maximum_kind,
            bounds.maximum,
            bounds.minimum,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
    pub(super) fn emit_regexp_parser_quantifier(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        term: I64Local,
        function: &mut Function,
    ) {
        let bounds = self.reserve_regexp_parsed_bounds(function);
        let start = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        let kind = self.runtime_schema().reserve_i64_local(function);
        let opcode = self.runtime_schema().reserve_i64_local(function);
        copy(function, start, compiler.cursor);
        for local in [bounds.minimum, bounds.maximum] {
            set(function, local, BoundClass::One as u64);
        }
        for local in [
            bounds.maximum_kind,
            bounds.minimum_start,
            bounds.minimum_end,
            bounds.maximum_start,
            bounds.maximum_end,
            bounds.matched,
        ] {
            set(function, local, 0);
        }
        peek(compiler, function, compiler.cursor, 0, unit);
        for (token, minimum, maximum, maximum_kind) in [
            (
                b'?',
                BoundClass::Zero,
                BoundClass::One,
                RegExpRepeatMaximumKind::Finite,
            ),
            (
                b'*',
                BoundClass::Zero,
                BoundClass::Many,
                RegExpRepeatMaximumKind::Unbounded,
            ),
            (
                b'+',
                BoundClass::One,
                BoundClass::Many,
                RegExpRepeatMaximumKind::Unbounded,
            ),
        ] {
            eq(function, unit, token as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, bounds.minimum, minimum as u64);
            set(function, bounds.maximum, maximum as u64);
            set(function, bounds.maximum_kind, maximum_kind.word());
            set(function, bounds.matched, 1);
            self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
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
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64Or);
        flags.store(function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
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
        // Annex B permits quantified lookahead only. Lookbehind remains an
        // Assertion and cannot acquire a quantifier in the non-Unicode grammar.
        eq(function, kind, NodeKind::PositiveLookbehind as u64);
        eq(function, kind, NodeKind::NegativeLookbehind as u64);
        function.instruction(&Instruction::I32Or);
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
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, compiler.cursor, start);
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, bounds.minimum, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        bounds.minimum.store(function);
        copy(function, bounds.maximum, bounds.minimum);
        set(
            function,
            bounds.maximum_kind,
            RegExpRepeatMaximumKind::Finite.word(),
        );
        for local in [
            bounds.minimum_start,
            bounds.minimum_end,
            bounds.maximum_start,
            bounds.maximum_end,
        ] {
            set(function, local, 0);
        }
        function.instruction(&Instruction::End);
        for (word, local) in [
            (NodeWord::Flags, flags),
            (NodeWord::Minimum, bounds.minimum),
            (NodeWord::Maximum, bounds.maximum),
            (NodeWord::MaximumKind, bounds.maximum_kind),
            (NodeWord::MinimumDigitsStart, bounds.minimum_start),
            (NodeWord::MinimumDigitsEnd, bounds.minimum_end),
            (NodeWord::MaximumDigitsStart, bounds.maximum_start),
            (NodeWord::MaximumDigitsEnd, bounds.maximum_end),
        ] {
            store(function, address, word as u64, local);
        }
        function.instruction(&Instruction::End);
        for local in [opcode, kind, flags, address, unit, start] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.release_regexp_parsed_bounds(bounds, function);
    }

    /// A failed Annex B brace probe publishes neither a cursor nor digit spans.
    /// Canonical digits stay in the source workspace until final publication.
    pub(super) fn emit_regexp_parser_braced_bounds(
        &mut self,
        compiler: &CompilerLocals,
        bounds: &ParsedBounds,
        function: &mut Function,
    ) {
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let min_start = self.runtime_schema().reserve_i64_local(function);
        let min_end = self.runtime_schema().reserve_i64_local(function);
        let max_start = self.runtime_schema().reserve_i64_local(function);
        let max_end = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let minimum = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let maximum_kind = self.runtime_schema().reserve_i64_local(function);
        set(function, bounds.matched, 0);
        set(
            function,
            maximum_kind,
            RegExpRepeatMaximumKind::Finite.word(),
        );
        copy(function, cursor, compiler.cursor);
        self.emit_regexp_scratch_increment(cursor, 1, function);
        self.emit_regexp_parser_bound_digits(
            compiler, cursor, minimum, min_start, min_end, function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        min_start.load(function);
        min_end.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        copy(function, maximum, minimum);
        copy(function, max_start, min_start);
        copy(function, max_end, min_end);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b',' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(cursor, 1, function);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(
            function,
            maximum_kind,
            RegExpRepeatMaximumKind::Unbounded.word(),
        );
        set(function, maximum, BoundClass::Many as u64);
        set(function, max_start, 0);
        set(function, max_end, 0);
        function.instruction(&Instruction::Else);
        self.emit_regexp_parser_bound_digits(
            compiler, cursor, maximum, max_start, max_end, function,
        );
        max_start.load(function);
        max_end.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        peek(compiler, function, cursor, 0, unit);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        eq(
            function,
            maximum_kind,
            RegExpRepeatMaximumKind::Finite.word(),
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_ordered_decimals(
            compiler, min_start, min_end, max_start, max_end, function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(cursor, 1, function);
        copy(function, compiler.cursor, cursor);
        for (output, value) in [
            (bounds.minimum, minimum),
            (bounds.maximum, maximum),
            (bounds.maximum_kind, maximum_kind),
            (bounds.minimum_start, min_start),
            (bounds.minimum_end, min_end),
            (bounds.maximum_start, max_start),
            (bounds.maximum_end, max_end),
        ] {
            copy(function, output, value);
        }
        set(function, bounds.matched, 1);
        function.instruction(&Instruction::End);
        for local in [
            maximum_kind,
            maximum,
            minimum,
            unit,
            max_end,
            max_start,
            min_end,
            min_start,
            cursor,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
