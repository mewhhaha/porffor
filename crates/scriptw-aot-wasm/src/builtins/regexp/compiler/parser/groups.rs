use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_capture_census(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let inside_class = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let marker = self.runtime_schema().reserve_i64_local(function);
        for local in [
            inside_class,
            parser.total_capture_count,
            parser.has_named_capture,
            compiler.cursor,
        ] {
            set(function, local, 0);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        compiler.cursor.load(function);
        compiler.unit_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, compiler.cursor, 0, parser.unit);
        eq(function, parser.unit, b'\\' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        for (unit, value) in [(b'[', 1), (b']', 0)] {
            eq(function, parser.unit, unit as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, inside_class, value);
            function.instruction(&Instruction::End);
        }
        eq(function, parser.unit, b'(' as u64);
        eq(function, inside_class, 0);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, next, b'?' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 2, next);
        peek(compiler, function, compiler.cursor, 3, marker);
        eq(function, next, b'<' as u64);
        eq(function, marker, b'=' as u64);
        eq(function, marker, b'!' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(parser.total_capture_count, 1, function);
        set(function, parser.has_named_capture, 1);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_regexp_scratch_increment(parser.total_capture_count, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(marker, function);
        self.runtime_schema().release_i64_local(next, function);
        self.runtime_schema()
            .release_i64_local(inside_class, function);
    }

    pub(super) fn emit_regexp_parser_new_sequence(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        self.emit_regexp_parser_new_node(
            compiler,
            NodeKind::Sequence,
            parser.group,
            parser.sequence,
            parser.address,
            function,
        );
        store_const(
            function,
            parser.address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        self.emit_regexp_parser_append(compiler, parser.group, parser.sequence, function);
    }

    pub(super) fn emit_regexp_parser_open_group(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let kind = self.runtime_schema().reserve_i64_local(function);
        let marker = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_new_node(
            compiler,
            NodeKind::Capture,
            parser.sequence,
            parser.term,
            parser.address,
            function,
        );
        self.emit_regexp_parser_append(compiler, parser.sequence, parser.term, function);
        set(function, kind, NodeKind::Capture as u64);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, marker);
        eq(function, marker, b'?' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, marker);
        set(function, kind, 0);
        for (unit, group) in [
            (b':', NodeKind::NonCapture),
            (b'=', NodeKind::PositiveLookahead),
            (b'!', NodeKind::NegativeLookahead),
        ] {
            eq(function, marker, unit as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, kind, group as u64);
            function.instruction(&Instruction::End);
        }
        eq(function, marker, b'<' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, marker);
        eq(function, marker, b'=' as u64);
        eq(function, marker, b'!' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (unit, group) in [
            (b'=', NodeKind::PositiveLookbehind),
            (b'!', NodeKind::NegativeLookbehind),
        ] {
            eq(function, marker, unit as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, kind, group as u64);
            function.instruction(&Instruction::End);
        }
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        let payload = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        self.emit_regexp_parser_name(compiler, payload, function);
        self.emit_regexp_scratch_increment(compiler.capture_count, 1, function);
        self.emit_regexp_append_capture_name(compiler, parser.term, payload, function);
        set(function, kind, NodeKind::Capture as u64);
        self.runtime_schema().release_i64_local(payload, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, kind, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_modifier_prefix(compiler, &parser.modifiers, function);
        set(function, kind, NodeKind::NonCapture as u64);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_scratch_increment(compiler.capture_count, 1, function);
        function.instruction(&Instruction::End);
        store(function, parser.address, NodeWord::Kind as u64, kind);
        for (word, local) in parser.modifiers.words() {
            store(function, parser.address, word as u64, local);
        }
        copy(function, parser.group, parser.term);
        self.emit_regexp_parser_new_sequence(compiler, parser, function);
        self.runtime_schema().release_i64_local(marker, function);
        self.runtime_schema().release_i64_local(kind, function);
    }

    pub(super) fn emit_regexp_parser_finish_sequence(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let flags = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_node_address(compiler, parser.sequence, address, function);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        flags.store(function);
        store(function, address, NodeWord::CaptureEnd as u64, flags);
        load(function, address, NodeWord::Flags as u64, flags);
        self.emit_regexp_node_address(compiler, parser.group, address, function);
        address.load(function);
        function.instruction(&Instruction::I32WrapI64);
        address.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: NodeWord::Flags as u64,
            align: 3,
            memory_index: 0,
        }));
        flags.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: NodeWord::Flags as u64,
            align: 3,
            memory_index: 0,
        }));
        self.runtime_schema().release_i64_local(address, function);
        self.runtime_schema().release_i64_local(flags, function);
    }

    pub(super) fn emit_regexp_parser_finish_group(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let kind = self.runtime_schema().reserve_i64_local(function);
        let capture = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_finish_sequence(compiler, parser, function);
        self.emit_regexp_node_address(compiler, parser.group, parser.address, function);
        load(function, parser.address, NodeWord::Kind as u64, kind);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        capture.store(function);
        store(
            function,
            parser.address,
            NodeWord::CaptureEnd as u64,
            capture,
        );
        function.instruction(&Instruction::I32Const(0));
        for assertion in [
            NodeKind::PositiveLookahead,
            NodeKind::NegativeLookahead,
            NodeKind::PositiveLookbehind,
            NodeKind::NegativeLookbehind,
        ] {
            eq(function, kind, assertion as u64);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        store_const(
            function,
            parser.address,
            NodeWord::Flags as u64,
            NODE_ATOM_NULLABLE,
        );
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(capture, function);
        self.runtime_schema().release_i64_local(kind, function);
    }

    pub(super) fn emit_regexp_parser_finish_term(
        &mut self,
        compiler: &CompilerLocals,
        parser: &ParserLocals,
        function: &mut Function,
    ) {
        let minimum = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        let capture = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_node_address(compiler, parser.term, parser.address, function);
        load(function, parser.address, NodeWord::Minimum as u64, minimum);
        load(function, parser.address, NodeWord::Flags as u64, flags);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        capture.store(function);
        store(
            function,
            parser.address,
            NodeWord::CaptureEnd as u64,
            capture,
        );
        eq(function, minimum, 0);
        function.instruction(&Instruction::I32Eqz);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_node_address(compiler, parser.sequence, parser.address, function);
        store_const(function, parser.address, NodeWord::Flags as u64, 0);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(capture, function);
        self.runtime_schema().release_i64_local(flags, function);
        self.runtime_schema().release_i64_local(minimum, function);
    }
}
