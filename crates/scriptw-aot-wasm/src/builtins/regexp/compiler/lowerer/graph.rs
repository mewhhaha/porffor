use super::*;

const CHOICE: u64 = 1 << 33;
const COUNTED_EDGE: u64 = 1 << 34;

impl FunctionBuilder<'_> {
    pub(super) fn lower_require_zero(
        &self,
        compiler: &CompilerLocals,
        value: I64Local,
        function: &mut Function,
    ) {
        value.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    pub(super) fn lower_require_target(
        &self,
        compiler: &CompilerLocals,
        target: I64Local,
        function: &mut Function,
    ) {
        target.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_require_capture(
        &self,
        compiler: &CompilerLocals,
        capture: I64Local,
        function: &mut Function,
    ) {
        capture.load(function);
        function.instruction(&Instruction::I64Eqz);
        capture.load(function);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_require_range(
        &mut self,
        compiler: &CompilerLocals,
        first: I64Local,
        packed_count: I64Local,
        nonempty: bool,
        function: &mut Function,
    ) {
        let count = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        let start = self.runtime_schema().reserve_i64_local(function);
        let previous_end = self.runtime_schema().reserve_i64_local(function);
        packed_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        count.store(function);
        first.load(function);
        compiler.range_count.load(function);
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        count.load(function);
        compiler.range_count.load(function);
        first.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        if nonempty {
            count.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Or);
        }
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(first), Local(count), end, function);
        set_word(cursor, Local(first), function);
        set_word(previous_end, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(function);
        end.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.ranges, cursor, 8, 0, packed, function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        start.store(function);
        cursor.load(function);
        first.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        previous_end.load(function);
        start.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        packed.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        previous_end.store(function);
        add_words(Local(cursor), Constant(1), cursor, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [previous_end, start, packed, cursor, end, count] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn lower_operand_rule(
        &mut self,
        compiler: &CompilerLocals,
        captures: &CompletedRegExpPattern,
        rule: RegExpOperandRule,
        a: I64Local,
        b: I64Local,
        function: &mut Function,
    ) {
        let word = self.runtime_schema().reserve_i64_local(function);
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
                a.load(function);
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
                a.load(function);
                function.instruction(&Instruction::I64Eqz);
                a.load(function);
                b.load(function);
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::I32Or);
                b.load(function);
                compiler.capture_count.load(function);
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
                a.load(function);
                captures.emit_group_count(function);
                function.instruction(&Instruction::I64GeU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(
                    !REGEXP_BACKREFERENCE_IGNORE_CASE as i64,
                ));
                function.instruction(&Instruction::I64And);
                word.store(function);
                self.lower_require_zero(compiler, word, function);
            }
            RegExpOperandRule::NumberedReference => {
                self.lower_require_capture(compiler, a, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(
                    !(REGEXP_BACKREFERENCE_NONEMPTY | REGEXP_BACKREFERENCE_IGNORE_CASE) as i64,
                ));
                function.instruction(&Instruction::I64And);
                word.store(function);
                self.lower_require_zero(compiler, word, function);
            }
            RegExpOperandRule::LookaroundEnd => {
                self.lower_require_target(compiler, a, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
                function.instruction(&Instruction::I64And);
                word.store(function);
                self.lower_require_target(compiler, word, function);
                self.lower_load(
                    compiler.instructions,
                    a,
                    REGEXP_INSTRUCTION_WIDTH as u64,
                    0,
                    word,
                    function,
                );
                word.load(function);
                function.instruction(&Instruction::I64Const(
                    REGEXP_OPCODE_LOOKAROUND_FAILURE as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::LookaroundFailure => {
                self.lower_require_target(compiler, a, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(3));
                function.instruction(&Instruction::I64GtU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::ProgressSplit => {
                self.lower_require_target(compiler, a, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64ShrU);
                word.store(function);
                self.lower_require_target(compiler, word, function);
            }
            RegExpOperandRule::RepeatSlot => {
                a.load(function);
                compiler.repeat_slot_count.load(function);
                function.instruction(&Instruction::I64GeU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
                self.lower_require_zero(compiler, b, function);
            }
            RegExpOperandRule::RepeatGuard => {
                self.lower_require_target(compiler, a, function);
                b.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64ShrU);
                compiler.repeat_slot_count.load(function);
                function.instruction(&Instruction::I64GeU);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
            RegExpOperandRule::RepeatOwner => {
                self.lower_require_target(compiler, a, function);
                self.lower_require_zero(compiler, b, function);
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
                word.load(function);
                function.instruction(&Instruction::I64Const(REGEXP_OPCODE_PROGRESS_SPLIT as i64));
                function.instruction(&Instruction::I64Ne);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            }
        }
        self.runtime_schema().release_i64_local(word, function);
    }

    pub(super) fn emit_regexp_lower_graph(
        &mut self,
        compiler: &CompilerLocals,
        captures: &CompletedRegExpPattern,
        function: &mut Function,
    ) {
        let pc = self.runtime_schema().reserve_i64_local(function);
        let opcode = self.runtime_schema().reserve_i64_local(function);
        let a = self.runtime_schema().reserve_i64_local(function);
        let b = self.runtime_schema().reserve_i64_local(function);
        let known = self.runtime_schema().reserve_i64_local(function);
        let first = self.runtime_schema().reserve_i64_local(function);
        let second = self.runtime_schema().reserve_i64_local(function);
        let barrier = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        let word = self.runtime_schema().reserve_i64_local(function);
        compiler.range_count.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_MAX_RANGE_ENTRIES as i64));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(pc, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        pc.load(function);
        compiler.range_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.ranges, pc, 8, 0, packed, function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        a.store(function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        b.store(function);
        a.load(function);
        b.load(function);
        function.instruction(&Instruction::I64GtU);
        b.load(function);
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
        pc.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_visited, pc, 8, 0, word, function);
        word.load(function);
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
            opcode.load(function);
            function.instruction(&Instruction::I64Const(fact as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            set_word(known, Constant(1), function);
            self.lower_operand_rule(compiler, captures, fact.operand_rule(), a, b, function);
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
                    b.load(function);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64ShrU);
                    second.store(function);
                }
                RegExpControlFlow::RepeatGuard => {
                    add_words(Local(pc), Constant(1), first, function);
                    add_words(Local(a), Constant(1), second, function);
                    self.lower_require_target(compiler, first, function);
                    self.lower_require_target(compiler, second, function);
                }
                RegExpControlFlow::RepeatEnd => {
                    add_words(Local(a), Constant(1), first, function);
                    self.lower_require_target(compiler, first, function);
                }
                RegExpControlFlow::LookaroundAfter => {
                    b.load(function);
                    function.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
                    function.instruction(&Instruction::I64And);
                    first.store(function);
                }
            }
            match fact.input_progress() {
                RegExpInputProgress::Consumes | RegExpInputProgress::CheckedOptional => {
                    set_word(barrier, Constant(PROGRESS_BARRIER), function)
                }
                RegExpInputProgress::MayStay => set_word(barrier, Constant(0), function),
                RegExpInputProgress::CheckedRepeat => {
                    set_word(barrier, Constant(COUNTED_EDGE), function)
                }
                RegExpInputProgress::NumberedReference => {
                    b.load(function);
                    function
                        .instruction(&Instruction::I64Const(REGEXP_BACKREFERENCE_NONEMPTY as i64));
                    function.instruction(&Instruction::I64And);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::I64Const(32));
                    function.instruction(&Instruction::I64Shl);
                    barrier.store(function);
                }
            }
            second.load(function);
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Shl);
            first.load(function);
            function.instruction(&Instruction::I64Or);
            barrier.load(function);
            function.instruction(&Instruction::I64Or);
            if fact.is_choice() {
                function.instruction(&Instruction::I64Const(CHOICE as i64));
                function.instruction(&Instruction::I64Or);
            }
            packed.store(function);
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
        known.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            word, packed, barrier, second, first, known, b, a, opcode, pc,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.emit_regexp_lower_counted_regions(compiler, function);
        self.emit_regexp_lower_acyclic(compiler, function);
        self.emit_regexp_lower_repeatable(compiler, function);
    }

    fn lower_graph_push(
        &self,
        compiler: &CompilerLocals,
        top: I64Local,
        word: I64Local,
        function: &mut Function,
    ) {
        top.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_store(compiler.graph_work, top, 8, 0, Local(word), function);
        add_words(Local(top), Constant(1), top, function);
    }

    fn emit_regexp_lower_acyclic(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let root = self.runtime_schema().reserve_i64_local(function);
        let pc = self.runtime_schema().reserve_i64_local(function);
        let top = self.runtime_schema().reserve_i64_local(function);
        let slot = self.runtime_schema().reserve_i64_local(function);
        let frame = self.runtime_schema().reserve_i64_local(function);
        let edge = self.runtime_schema().reserve_i64_local(function);
        let target = self.runtime_schema().reserve_i64_local(function);
        let color = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        set_word(root, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        root.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_colors, root, 8, 0, color, function);
        color.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_edges, root, 8, 0, packed, function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(PROGRESS_BARRIER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, root, 8, 0, Constant(1), function);
        root.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Shl);
        frame.store(function);
        set_word(top, Constant(0), function);
        self.lower_graph_push(compiler, top, frame, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        top.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(top), Constant(u64::MAX), slot, function);
        self.lower_load(compiler.graph_work, slot, 8, 0, frame, function);
        frame.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64ShrU);
        pc.store(function);
        frame.load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64And);
        edge.store(function);
        edge.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, pc, 8, 0, Constant(2), function);
        set_word(top, Local(slot), function);
        function.instruction(&Instruction::Else);
        add_words(Local(frame), Constant(1), frame, function);
        self.lower_store(compiler.graph_work, slot, 8, 0, Local(frame), function);
        self.lower_load(compiler.graph_edges, pc, 8, 0, packed, function);
        packed.load(function);
        edge.load(function);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(NO_EDGE as i64));
        function.instruction(&Instruction::I64And);
        target.store(function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(COUNTED_EDGE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        edge.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(
            compiler.instructions,
            target,
            REGEXP_INSTRUCTION_WIDTH as u64,
            8,
            target,
            function,
        );
        add_words(Local(target), Constant(1), target, function);
        function.instruction(&Instruction::End);
        target.load(function);
        function.instruction(&Instruction::I64Const(NO_EDGE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_colors, target, 8, 0, color, function);
        color.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        color.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_edges, target, 8, 0, packed, function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(PROGRESS_BARRIER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_store(compiler.graph_colors, target, 8, 0, Constant(1), function);
        target.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Shl);
        frame.store(function);
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
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_lower_repeatable(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let root = self.runtime_schema().reserve_i64_local(function);
        let top = self.runtime_schema().reserve_i64_local(function);
        let pc = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        let target = self.runtime_schema().reserve_i64_local(function);
        let visited = self.runtime_schema().reserve_i64_local(function);
        let repeatable = self.runtime_schema().reserve_i64_local(function);
        set_word(root, Constant(0), function);
        set_word(compiler.repeatable_split_count, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        root.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_edges, root, 8, 0, packed, function);
        packed.load(function);
        function.instruction(&Instruction::I64Const(CHOICE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        compiler.graph_visited.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        compiler.instruction_count.load(function);
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
        top.load(function);
        function.instruction(&Instruction::I64Eqz);
        repeatable.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(top), Constant(u64::MAX), top, function);
        self.lower_load(compiler.graph_work, top, 8, 0, pc, function);
        self.lower_load(compiler.graph_edges, pc, 8, 0, packed, function);
        for edge in [0, 16] {
            packed.load(function);
            function.instruction(&Instruction::I64Const(edge));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64And);
            target.store(function);
            target.load(function);
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            target.load(function);
            root.load(function);
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            set_word(repeatable, Constant(1), function);
            function.instruction(&Instruction::Else);
            self.lower_load(compiler.graph_visited, target, 8, 0, visited, function);
            visited.load(function);
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
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
