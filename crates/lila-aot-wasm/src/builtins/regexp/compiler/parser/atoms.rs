use super::escapes::CharacterContext;
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_atom(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let character = self.runtime_schema().reserve_i64_local(function);
        let class = self.runtime_schema().reserve_i64_local(function);
        let matched = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        for unit in b"?*+" {
            eq(function, parser.unit, *unit as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_compile_failure(
                compiler,
                CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
                function,
            );
            function.instruction(&Instruction::End);
        }
        eq(function, parser.unit, b'{' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bounds = self.reserve_regexp_parsed_bounds(function);
        self.emit_regexp_parser_braced_bounds(compiler, &bounds, function);
        copy(function, matched, bounds.matched);
        eq(function, matched, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        self.release_regexp_parsed_bounds(bounds, function);
        function.instruction(&Instruction::End);
        for (unit, opcode, nullable, modifier) in [
            (b'.', REGEXP_OPCODE_DOT, 0, parser.modifiers.dot_all),
            (
                b'^',
                REGEXP_OPCODE_ASSERT_START,
                NODE_ATOM_NULLABLE,
                parser.modifiers.multiline,
            ),
            (
                b'$',
                REGEXP_OPCODE_ASSERT_END,
                NODE_ATOM_NULLABLE,
                parser.modifiers.multiline,
            ),
        ] {
            eq(function, parser.unit, unit as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            store_const(function, parser.address, NodeWord::Opcode as u64, opcode);
            store(
                function,
                parser.address,
                NodeWord::Operand0 as u64,
                modifier,
            );
            store_const(function, parser.address, NodeWord::Flags as u64, nullable);
            self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        eq(function, parser.unit, b'[' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_class(
            compiler,
            character_mode,
            parser.address,
            &parser.modifiers,
            parser.has_named_capture,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        eq(function, parser.unit, b'\\' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, character);
        eq(function, character, b'p' as u64);
        eq(function, character, b'P' as u64);
        function.instruction(&Instruction::I32Or);
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        let property = self.emit_regexp_parser_character_set_operand(
            compiler,
            character_mode,
            CharacterContext::Pattern {
                named_captures: parser.has_named_capture,
            },
            function,
        );
        property.emit_atom(self, compiler, character_mode, parser, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        eq(function, character, b'b' as u64);
        eq(function, character, b'B' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_boundary(
            compiler,
            character_mode,
            parser.address,
            character,
            &parser.modifiers,
            function,
        );
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        eq(function, character, b'k' as u64);
        character_mode.emit_uses_named_capture_grammar(parser.has_named_capture, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        peek(compiler, function, compiler.cursor, 0, character);
        eq(function, character, b'<' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidGroupName),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        self.emit_regexp_parser_name(compiler, character, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        // Operand0 is private canonical workspace data until the completed
        // inventory replaces it with the validated named-group index.
        store_const(
            function,
            parser.address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_NAMED_BACKREFERENCE,
        );
        store(
            function,
            parser.address,
            NodeWord::Operand0 as u64,
            character,
        );
        store_const(
            function,
            parser.address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        set(function, matched, 0);
        parser.modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, matched, REGEXP_BACKREFERENCE_IGNORE_CASE);
        function.instruction(&Instruction::End);
        store(function, parser.address, NodeWord::Operand1 as u64, matched);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        between(function, character, b'1' as u64, b'9' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_numbered_reference(
            compiler,
            character_mode,
            parser,
            matched,
            function,
        );
        eq(function, matched, 1);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_character(
            compiler,
            character_mode,
            CharacterContext::Pattern {
                named_captures: parser.has_named_capture,
            },
            character,
            class,
            function,
        );
        eq(function, class, 0);
        parser.modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        character.load(function);
        function.instruction(&Instruction::I64Const(128));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        store_const(
            function,
            parser.address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_LITERAL_ASCII,
        );
        function.instruction(&Instruction::Else);
        store_const(
            function,
            parser.address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_LITERAL_CODE_POINT,
        );
        function.instruction(&Instruction::End);
        store(
            function,
            parser.address,
            NodeWord::Operand0 as u64,
            character,
        );
        function.instruction(&Instruction::Else);
        bitmap::clear_bitmap(function, compiler.class_bitmap, character_mode);
        self.emit_regexp_parser_add_character_set(
            compiler,
            character_mode,
            &parser.modifiers,
            compiler.class_bitmap,
            character,
            class,
            function,
        );
        set(function, matched, 0);
        self.emit_regexp_parser_bitmap_ranges(
            compiler,
            character_mode,
            parser.address,
            matched,
            &parser.modifiers,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(matched, function);
        self.runtime_schema().release_i64_local(class, function);
        self.runtime_schema().release_i64_local(character, function);
    }

    fn emit_regexp_parser_numbered_reference(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        parser: &ParserLocals,
        matched: I64Local,
        function: &mut Function,
    ) {
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let capture = self.runtime_schema().reserve_i64_local(function);
        let oversized = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        set(function, matched, 0);
        set(function, oversized, 0);
        copy(function, cursor, compiler.cursor);
        self.emit_regexp_scratch_increment(cursor, 1, function);
        self.emit_regexp_parser_backreference_decimal(
            compiler, cursor, capture, oversized, function,
        );
        capture.load(function);
        parser.total_capture_count.load(function);
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, matched, 1);
        copy(function, compiler.cursor, cursor);
        store_const(
            function,
            parser.address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
        );
        store(function, parser.address, NodeWord::Operand0 as u64, capture);
        set(function, flags, 0);
        store_const(
            function,
            parser.address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        parser.modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        flags.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_BACKREFERENCE_IGNORE_CASE as i64,
        ));
        function.instruction(&Instruction::I64Or);
        flags.store(function);
        function.instruction(&Instruction::End);
        store(function, parser.address, NodeWord::Operand1 as u64, flags);
        function.instruction(&Instruction::Else);
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [flags, oversized, capture, cursor] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_parser_boundary(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        address: I64Local,
        marker: I64Local,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let first = self.runtime_schema().reserve_i64_local(function);
        let last = self.runtime_schema().reserve_i64_local(function);
        let first_entry = self.runtime_schema().reserve_i64_local(function);
        copy(function, first_entry, compiler.range_count);
        self.emit_regexp_parser_word_ranges(
            compiler,
            character_mode,
            modifiers,
            first,
            last,
            function,
        );
        store_const(
            function,
            address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_WORD_BOUNDARY,
        );
        store(function, address, NodeWord::Operand0 as u64, first_entry);
        eq(function, marker, b'B' as u64);
        function.instruction(&Instruction::I64ExtendI32U);
        compiler.range_count.load(function);
        first_entry.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        first.store(function);
        store(function, address, NodeWord::Operand1 as u64, first);
        store_const(
            function,
            address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        self.runtime_schema()
            .release_i64_local(first_entry, function);
        self.runtime_schema().release_i64_local(last, function);
        self.runtime_schema().release_i64_local(first, function);
    }
}
