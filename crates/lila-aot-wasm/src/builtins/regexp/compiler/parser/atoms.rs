use super::escapes::CharacterContext;
use super::quantifiers::ParsedBounds;
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_atom(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let character = self.reserve_temp_local();
        let class = self.reserve_temp_local();
        let matched = self.reserve_temp_local();
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
        let bounds = ParsedBounds {
            minimum: self.reserve_temp_local(),
            maximum: self.reserve_temp_local(),
            oversized: self.reserve_temp_local(),
            matched,
        };
        self.emit_regexp_parser_braced_bounds(compiler, &bounds, function);
        eq(function, matched, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidQuantifier),
            function,
        );
        function.instruction(&Instruction::End);
        for local in [bounds.oversized, bounds.maximum, bounds.minimum] {
            self.release_temp_local(local);
        }
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
            self.emit_increment_local(compiler.cursor, 1, function);
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        eq(function, parser.unit, b'[' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_class(compiler, parser.address, &parser.modifiers, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        eq(function, parser.unit, b'\\' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, character);
        eq(function, character, b'b' as u64);
        eq(function, character, b'B' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_boundary(compiler, parser.address, character, function);
        self.emit_increment_local(compiler.cursor, 2, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        eq(function, character, b'k' as u64);
        eq(function, parser.has_named_capture, 1);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Unsupported(CompileCapability::NamedBackreference),
            function,
        );
        function.instruction(&Instruction::End);
        between(function, character, b'1' as u64, b'9' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_numbered_reference(compiler, parser, matched, function);
        eq(function, matched, 1);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_character(
            compiler,
            CharacterContext::Pattern,
            character,
            class,
            function,
        );
        eq(function, class, 0);
        function.instruction(&Instruction::LocalGet(parser.modifiers.ignore_case));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(character));
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
        bitmap::clear_bitmap(function, compiler.class_bitmap);
        self.emit_regexp_parser_add_character_set(compiler, character, class, function);
        set(function, matched, 0);
        self.emit_regexp_parser_bitmap_ranges(
            compiler,
            parser.address,
            matched,
            &parser.modifiers,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(matched);
        self.release_temp_local(class);
        self.release_temp_local(character);
    }

    fn emit_regexp_parser_numbered_reference(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        matched: u32,
        function: &mut Function,
    ) {
        let cursor = self.reserve_temp_local();
        let capture = self.reserve_temp_local();
        let oversized = self.reserve_temp_local();
        let flags = self.reserve_temp_local();
        set(function, matched, 0);
        set(function, oversized, 0);
        copy(function, cursor, compiler.cursor);
        self.emit_increment_local(cursor, 1, function);
        self.emit_regexp_parser_decimal(compiler, cursor, capture, oversized, function);
        function.instruction(&Instruction::LocalGet(capture));
        function.instruction(&Instruction::LocalGet(parser.total_capture_count));
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
        function.instruction(&Instruction::LocalGet(parser.modifiers.ignore_case));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(
            REGEXP_BACKREFERENCE_IGNORE_CASE as i64,
        ));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags));
        function.instruction(&Instruction::End);
        store(function, parser.address, NodeWord::Operand1 as u64, flags);
        function.instruction(&Instruction::End);
        for local in [flags, oversized, capture, cursor] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_parser_boundary(
        &mut self,
        compiler: &CompilerLocals,
        address: u32,
        marker: u32,
        function: &mut Function,
    ) {
        let first = self.reserve_temp_local();
        let last = self.reserve_temp_local();
        let first_entry = self.reserve_temp_local();
        copy(function, first_entry, compiler.range_count);
        for &(start, end) in REGEXP_WORD_RANGES {
            set(function, first, start as u64);
            set(function, last, end as u64);
            self.emit_regexp_parser_append_range(compiler, first, last, function);
        }
        store_const(
            function,
            address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_WORD_BOUNDARY,
        );
        store(function, address, NodeWord::Operand0 as u64, first_entry);
        eq(function, marker, b'B' as u64);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(
            (REGEXP_WORD_RANGES.len() as i64) << 1,
        ));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(first));
        store(function, address, NodeWord::Operand1 as u64, first);
        store_const(
            function,
            address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        self.release_temp_local(first_entry);
        self.release_temp_local(last);
        self.release_temp_local(first);
    }
}
