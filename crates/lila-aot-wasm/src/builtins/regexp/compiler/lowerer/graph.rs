use super::*;

const CHOICE: u64 = 1 << 33;

impl FunctionBuilder<'_> {
    fn lower_require_zero(&self, compiler: &CompilerLocals, value: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_require_target(
        &self,
        compiler: &CompilerLocals,
        target: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(target));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_require_capture(
        &self,
        compiler: &CompilerLocals,
        capture: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(capture));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(capture));
        function.instruction(&Instruction::LocalGet(compiler.capture_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_require_range(
        &mut self,
        compiler: &CompilerLocals,
        first: u32,
        packed_count: u32,
        nonempty: bool,
        function: &mut Function,
    ) {
        let count = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        let packed = self.reserve_temp_local();
        let start = self.reserve_temp_local();
        let previous_end = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(packed_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(count));
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        if nonempty {
            function.instruction(&Instruction::LocalGet(count));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Or);
        }
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(first), Local(count), end, function);
        set_word(cursor, Local(first), function);
        set_word(previous_end, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.ranges, cursor, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(start));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_end));
        function.instruction(&Instruction::LocalGet(start));
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(previous_end));
        add_words(Local(cursor), Constant(1), cursor, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [previous_end, start, packed, cursor, end, count] {
            self.release_temp_local(local);
        }
    }

    fn lower_operand_rule(
        &mut self,
        compiler: &CompilerLocals,
        rule: RegExpOperandRule,
        a: u32,
        b: u32,
        function: &mut Function,
    ) {
        let word = self.reserve_temp_local();
        match rule {
            RegExpOperandRule::Zero => {
                self.lower_require_zero(compiler, a, function);
                self.lower_require_zero(compiler, b, function);
            }
            RegExpOperandRule::Ascii
            | RegExpOperandRule::CodePoint
            | RegExpOperandRule::Modifier
            | RegExpOperandRule::LookaroundStart => {
                let limit = match rule {
                    RegExpOperandRule::Ascii => 0x7f,
                    RegExpOperandRule::CodePoint => 0x10ffff,
                    RegExpOperandRule::Modifier => 2,
                    RegExpOperandRule::LookaroundStart => 1,
                    _ => unreachable!(),
                };
                function.instruction(&Instruction::LocalGet(a));
                function.instruction(&Instruction::I64Const(limit));
                function.instruction(&Instruction::I64GtU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
                self.lower_require_zero(compiler, b, function);
            }
            RegExpOperandRule::Bitmap => {}
            RegExpOperandRule::TargetPair => {
                self.lower_require_target(compiler, a, function);
                self.lower_require_target(compiler, b, function);
            }
            RegExpOperandRule::Target => {
                self.lower_require_target(compiler, a, function);
                self.lower_require_zero(compiler, b, function);
            }
            RegExpOperandRule::Capture => {
                self.lower_require_capture(compiler, a, function);
                self.lower_require_zero(compiler, b, function);
            }
            RegExpOperandRule::CaptureRange => {
                function.instruction(&Instruction::LocalGet(a));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(a));
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::LocalGet(compiler.capture_count));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::I32Or);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::Range | RegExpOperandRule::NonemptyRange => self
                .lower_require_range(
                    compiler,
                    a,
                    b,
                    rule == RegExpOperandRule::NonemptyRange,
                    function,
                ),
            RegExpOperandRule::NamedReference => {
                // This compiler publishes descriptors without a named-group section.
                self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::NumberedReference => {
                self.lower_require_capture(compiler, a, function);
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::I64Const(
                    !(REGEXP_BACKREFERENCE_NONEMPTY | REGEXP_BACKREFERENCE_IGNORE_CASE) as i64,
                ));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::LocalSet(word));
                self.lower_require_zero(compiler, word, function);
            }
            RegExpOperandRule::LookaroundEnd => {
                self.lower_require_target(compiler, a, function);
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::LocalSet(word));
                self.lower_require_target(compiler, word, function);
                self.lower_load(
                    compiler.instructions,
                    a,
                    REGEXP_INSTRUCTION_WIDTH as u64,
                    0,
                    word,
                    function,
                );
                function.instruction(&Instruction::LocalGet(word));
                function.instruction(&Instruction::I64Const(
                    REGEXP_OPCODE_LOOKAROUND_FAILURE as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::LookaroundFailure => {
                self.lower_require_target(compiler, a, function);
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::I64Const(3));
                function.instruction(&Instruction::I64GtU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::ProgressSplit => {
                self.lower_require_target(compiler, a, function);
                function.instruction(&Instruction::LocalGet(b));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64ShrU);
                function.instruction(&Instruction::LocalSet(word));
                self.lower_require_target(compiler, word, function);
            }
            RegExpOperandRule::ProgressCheck => {
                self.lower_require_target(compiler, a, function);
                self.lower_require_target(compiler, b, function);
                self.lower_load(
                    compiler.instructions,
                    a,
                    REGEXP_INSTRUCTION_WIDTH as u64,
                    0,
                    word,
                    function,
                );
                function.instruction(&Instruction::LocalGet(word));
                function.instruction(&Instruction::I64Const(REGEXP_OPCODE_PROGRESS_SPLIT as i64));
                function.instruction(&Instruction::I64Ne);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
        }
        self.release_temp_local(word);
    }

    pub(super) fn emit_regexp_lower_graph(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let pc = self.reserve_temp_local();
        let opcode = self.reserve_temp_local();
        let a = self.reserve_temp_local();
        let b = self.reserve_temp_local();
        let known = self.reserve_temp_local();
        let first = self.reserve_temp_local();
        let second = self.reserve_temp_local();
        let barrier = self.reserve_temp_local();
        let packed = self.reserve_temp_local();
        let word = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_MAX_RANGE_ENTRIES as i64));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(pc, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.ranges, pc, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(a));
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(b));
        function.instruction(&Instruction::LocalGet(a));
        function.instruction(&Instruction::LocalGet(b));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(b));
        function.instruction(&Instruction::I64Const(0x10ffff));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        set_word(compiler.split_count, Constant(0), function);
        set_word(pc, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_visited, pc, 8, 0, word, function);
        function.instruction(&Instruction::LocalGet(word));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for (offset, local) in [(0, opcode), (8, a), (16, b)] {
            self.lower_load(
                compiler.instructions,
                pc,
                REGEXP_INSTRUCTION_WIDTH as u64,
                offset,
                local,
                function,
            );
        }
        set_word(known, Constant(0), function);
        for fact in RegExpOpcode::ALL {
            function.instruction(&Instruction::LocalGet(opcode));
            function.instruction(&Instruction::I64Const(fact as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            set_word(known, Constant(1), function);
            self.lower_operand_rule(compiler, fact.operand_rule(), a, b, function);
            set_word(first, Constant(NO_EDGE), function);
            set_word(second, Constant(NO_EDGE), function);
            match fact.control_flow() {
                RegExpControlFlow::Accept => {}
                RegExpControlFlow::Next => {
                    add_words(Local(pc), Constant(1), first, function);
                    self.lower_require_target(compiler, first, function);
                }
                RegExpControlFlow::Operand0 => set_word(first, Local(a), function),
                RegExpControlFlow::Operand1 => set_word(first, Local(b), function),
                RegExpControlFlow::BothOperands => {
                    set_word(first, Local(a), function);
                    set_word(second, Local(b), function);
                }
                RegExpControlFlow::ProgressSplit => {
                    set_word(first, Local(a), function);
                    function.instruction(&Instruction::LocalGet(b));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64ShrU);
                    function.instruction(&Instruction::LocalSet(second));
                }
                RegExpControlFlow::LookaroundAfter => {
                    function.instruction(&Instruction::LocalGet(b));
                    function.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::LocalSet(first));
                }
            }
            match fact.input_progress() {
                RegExpInputProgress::Consumes | RegExpInputProgress::CheckedOptional => {
                    set_word(barrier, Constant(PROGRESS_BARRIER), function)
                }
                RegExpInputProgress::MayStay => set_word(barrier, Constant(0), function),
                RegExpInputProgress::NumberedReference => {
                    function.instruction(&Instruction::LocalGet(b));
                    function
                        .instruction(&Instruction::I64Const(REGEXP_BACKREFERENCE_NONEMPTY as i64));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::I64Const(32));
                    function.instruction(&Instruction::I64Shl);
                    function.instruction(&Instruction::LocalSet(barrier));
                }
            }
            function.instruction(&Instruction::LocalGet(second));
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalGet(first));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalGet(barrier));
            function.instruction(&Instruction::I64Or);
            if fact.is_choice() {
                function.instruction(&Instruction::I64Const(CHOICE as i64));
                function.instruction(&Instruction::I64Or);
            }
            function.instruction(&Instruction::LocalSet(packed));
            self.lower_store(compiler.graph_edges, pc, 8, 0, Local(packed), function);
            if fact.is_choice() {
                add_words(
                    Local(compiler.split_count),
                    Constant(1),
                    compiler.split_count,
                    function,
                );
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(known));
        function.instruction(&Instruction::I64Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            word, packed, barrier, second, first, known, b, a, opcode, pc,
        ] {
            self.release_temp_local(local);
        }
        self.emit_regexp_lower_acyclic(compiler, function);
        self.emit_regexp_lower_repeatable(compiler, function);
    }

    fn lower_graph_push(
        &self,
        compiler: &CompilerLocals,
        top: u32,
        word: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(top));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_store(compiler.graph_work, top, 8, 0, Local(word), function);
        add_words(Local(top), Constant(1), top, function);
    }

    fn emit_regexp_lower_acyclic(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let root = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let top = self.reserve_temp_local();
        let slot = self.reserve_temp_local();
        let frame = self.reserve_temp_local();
        let edge = self.reserve_temp_local();
        let target = self.reserve_temp_local();
        let color = self.reserve_temp_local();
        let packed = self.reserve_temp_local();
        set_word(root, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(root));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_colors, root, 8, 0, color, function);
        function.instruction(&Instruction::LocalGet(color));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_edges, root, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(PROGRESS_BARRIER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, root, 8, 0, Constant(1), function);
        function.instruction(&Instruction::LocalGet(root));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(frame));
        set_word(top, Constant(0), function);
        self.lower_graph_push(compiler, top, frame, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(top));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(top), Constant(u64::MAX), slot, function);
        self.lower_load(compiler.graph_work, slot, 8, 0, frame, function);
        function.instruction(&Instruction::LocalGet(frame));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(pc));
        function.instruction(&Instruction::LocalGet(frame));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(edge));
        function.instruction(&Instruction::LocalGet(edge));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, pc, 8, 0, Constant(2), function);
        set_word(top, Local(slot), function);
        function.instruction(&Instruction::Else);
        add_words(Local(frame), Constant(1), frame, function);
        self.lower_store(compiler.graph_work, slot, 8, 0, Local(frame), function);
        self.lower_load(compiler.graph_edges, pc, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::LocalGet(edge));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(NO_EDGE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(target));
        function.instruction(&Instruction::LocalGet(target));
        function.instruction(&Instruction::I64Const(NO_EDGE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_colors, target, 8, 0, color, function);
        function.instruction(&Instruction::LocalGet(color));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::LocalGet(color));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_edges, target, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(PROGRESS_BARRIER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, target, 8, 0, Constant(1), function);
        function.instruction(&Instruction::LocalGet(target));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(frame));
        self.lower_graph_push(compiler, top, frame, function);
        function.instruction(&Instruction::Else);
        self.lower_store(compiler.graph_colors, target, 8, 0, Constant(2), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.lower_store(compiler.graph_colors, root, 8, 0, Constant(2), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        add_words(Local(root), Constant(1), root, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [packed, color, target, edge, frame, slot, top, pc, root] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_lower_repeatable(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let root = self.reserve_temp_local();
        let top = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let packed = self.reserve_temp_local();
        let target = self.reserve_temp_local();
        let visited = self.reserve_temp_local();
        let repeatable = self.reserve_temp_local();
        set_word(root, Constant(0), function);
        set_word(compiler.repeatable_split_count, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(root));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_edges, root, 8, 0, packed, function);
        function.instruction(&Instruction::LocalGet(packed));
        function.instruction(&Instruction::I64Const(CHOICE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compiler.graph_visited));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        self.lower_store(compiler.graph_visited, root, 8, 0, Constant(1), function);
        set_word(top, Constant(0), function);
        set_word(repeatable, Constant(0), function);
        self.lower_graph_push(compiler, top, root, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(top));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(repeatable));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(top), Constant(u64::MAX), top, function);
        self.lower_load(compiler.graph_work, top, 8, 0, pc, function);
        self.lower_load(compiler.graph_edges, pc, 8, 0, packed, function);
        for edge in [0, 16] {
            function.instruction(&Instruction::LocalGet(packed));
            function.instruction(&Instruction::I64Const(edge));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(target));
            function.instruction(&Instruction::LocalGet(target));
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(target));
            function.instruction(&Instruction::LocalGet(root));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            set_word(repeatable, Constant(1), function);
            function.instruction(&Instruction::Else);
            self.lower_load(compiler.graph_visited, target, 8, 0, visited, function);
            function.instruction(&Instruction::LocalGet(visited));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.lower_store(compiler.graph_visited, target, 8, 0, Constant(1), function);
            self.lower_graph_push(compiler, top, target, function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        add_words(
            Local(compiler.repeatable_split_count),
            Local(repeatable),
            compiler.repeatable_split_count,
            function,
        );
        function.instruction(&Instruction::End);
        add_words(Local(root), Constant(1), root, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [repeatable, visited, target, packed, pc, top, root] {
            self.release_temp_local(local);
        }
    }
}
