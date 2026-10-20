use super::*;

#[must_use = "the counted body must consume its paired End/Exit owner"]
pub(super) struct PendingCountedRepeat {
    begin: I64Local,
    guard: I64Local,
    end: I64Local,
    exit: I64Local,
}

impl FunctionBuilder<'_> {
    /// Mirror of static Quantifier::needs_counted_body. The remaining legacy
    /// layouts already retain one body: zero/one optional, star and consuming +.
    pub(super) fn emit_regexp_counted_body_condition(
        &self,
        minimum: I64Local,
        maximum: I64Local,
        maximum_kind: I64Local,
        flags: I64Local,
        function: &mut Function,
    ) {
        minimum.load(function);
        function.instruction(&Instruction::I64Eqz);
        maximum.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LeU);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        minimum.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        maximum.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.emit_regexp_single_body_plus_condition(minimum, maximum_kind, flags, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
    }

    pub(super) fn emit_regexp_begin_counted_repeat(
        &mut self,
        compiler: &CompilerLocals,
        node: I64Local,
        begin: I64Local,
        atom_width: I64Local,
        flags: I64Local,
        function: &mut Function,
    ) -> PendingCountedRepeat {
        let guard = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        let exit = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        add_words(Local(begin), Constant(1), guard, function);
        add_words(Local(begin), Local(atom_width), end, function);
        add_words(Local(end), Constant(2), end, function);
        add_words(Local(end), Constant(1), exit, function);
        compiler.repeat_slot_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        packed.store(function);
        self.lower_instruction(
            compiler,
            begin,
            Constant(REGEXP_OPCODE_REPEAT_BEGIN),
            Local(compiler.repeat_slot_count),
            Constant(0),
            function,
        );
        self.lower_instruction(
            compiler,
            guard,
            Constant(REGEXP_OPCODE_REPEAT_GUARD),
            Local(end),
            Local(packed),
            function,
        );
        compiler.repeat_slot_count.load(function);
        compiler.node_capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_store(
            compiler.repeat_rows,
            compiler.repeat_slot_count,
            8,
            0,
            Local(node),
            function,
        );
        add_words(
            Local(compiler.repeat_slot_count),
            Constant(1),
            compiler.repeat_slot_count,
            function,
        );
        self.runtime_schema().release_i64_local(packed, function);
        PendingCountedRepeat {
            begin,
            guard,
            end,
            exit,
        }
    }

    pub(super) fn emit_regexp_finish_counted_repeat(
        &mut self,
        compiler: &CompilerLocals,
        pending: PendingCountedRepeat,
        function: &mut Function,
    ) {
        self.lower_instruction(
            compiler,
            pending.end,
            Constant(REGEXP_OPCODE_REPEAT_END),
            Local(pending.begin),
            Constant(0),
            function,
        );
        self.lower_instruction(
            compiler,
            pending.exit,
            Constant(REGEXP_OPCODE_REPEAT_EXIT),
            Local(pending.begin),
            Constant(0),
            function,
        );
        for local in [pending.exit, pending.end, pending.guard] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn lower_repeat_instruction(
        &self,
        compiler: &CompilerLocals,
        pc: I64Local,
        opcode: u64,
        begin: I64Local,
        word: I64Local,
        function: &mut Function,
    ) {
        self.lower_require_target(compiler, pc, function);
        for (offset, expected) in [(0, Constant(opcode)), (8, Local(begin)), (16, Constant(0))] {
            self.lower_load(
                compiler.instructions,
                pc,
                REGEXP_INSTRUCTION_WIDTH as u64,
                offset,
                word,
                function,
            );
            word.load(function);
            expected.emit(function);
            function.instruction(&Instruction::I64Ne);
            self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        }
    }

    /// Uses source-sized existing graph work arrays in a separate phase.
    /// graph_visited becomes the per-PC active owner, graph_work the lexical
    /// stack and graph_colors the dense slot admission set. Colors are cleared
    /// before the acyclic proof, and visited is reset by repeatable analysis.
    pub(super) fn emit_regexp_lower_counted_regions(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let pc = self.runtime_schema().reserve_i64_local(function);
        let top = self.runtime_schema().reserve_i64_local(function);
        let owner = self.runtime_schema().reserve_i64_local(function);
        let begin = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let slot = self.runtime_schema().reserve_i64_local(function);
        let opcode = self.runtime_schema().reserve_i64_local(function);
        let a = self.runtime_schema().reserve_i64_local(function);
        let word = self.runtime_schema().reserve_i64_local(function);
        let begins = self.runtime_schema().reserve_i64_local(function);
        let target = self.runtime_schema().reserve_i64_local(function);
        let target_opcode = self.runtime_schema().reserve_i64_local(function);
        let expected = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        for local in [pc, top, begins] {
            set_word(local, Constant(0), function);
        }
        self.lower_clear_graph_colors(compiler, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        pc.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        set_word(owner, Constant(0), function);
        top.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(top), Constant(u64::MAX), slot, function);
        self.lower_load(compiler.graph_work, slot, 8, 0, owner, function);
        function.instruction(&Instruction::End);
        self.lower_store(compiler.graph_visited, pc, 8, 0, Local(owner), function);
        self.lower_load(
            compiler.instructions,
            pc,
            REGEXP_INSTRUCTION_WIDTH as u64,
            0,
            opcode,
            function,
        );
        self.lower_load(
            compiler.instructions,
            pc,
            REGEXP_INSTRUCTION_WIDTH as u64,
            8,
            a,
            function,
        );

        opcode.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_BEGIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(pc), Constant(1), next, function);
        self.lower_require_target(compiler, next, function);
        self.lower_load(
            compiler.instructions,
            next,
            REGEXP_INSTRUCTION_WIDTH as u64,
            0,
            word,
            function,
        );
        word.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_GUARD as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_load(
            compiler.instructions,
            next,
            REGEXP_INSTRUCTION_WIDTH as u64,
            8,
            end,
            function,
        );
        self.lower_load(
            compiler.instructions,
            next,
            REGEXP_INSTRUCTION_WIDTH as u64,
            16,
            slot,
            function,
        );
        slot.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        slot.store(function);
        slot.load(function);
        a.load(function);
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        slot.load(function);
        compiler.repeat_slot_count.load(function);
        function.instruction(&Instruction::I64GeU);
        end.load(function);
        next.load(function);
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_require_target(compiler, end, function);
        self.lower_load(compiler.graph_colors, slot, 8, 0, word, function);
        self.lower_require_zero(compiler, word, function);
        self.lower_store(compiler.graph_colors, slot, 8, 0, Constant(1), function);
        owner.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(
            compiler.instructions,
            owner,
            REGEXP_INSTRUCTION_WIDTH as u64,
            8,
            word,
            function,
        );
        end.load(function);
        word.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.lower_repeat_instruction(compiler, end, REGEXP_OPCODE_REPEAT_END, pc, word, function);
        add_words(Local(end), Constant(1), end, function);
        self.lower_repeat_instruction(compiler, end, REGEXP_OPCODE_REPEAT_EXIT, pc, word, function);
        top.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_store(compiler.graph_work, top, 8, 0, Local(next), function);
        add_words(Local(top), Constant(1), top, function);
        add_words(Local(begins), Constant(1), begins, function);
        function.instruction(&Instruction::End);

        for control in [
            REGEXP_OPCODE_REPEAT_GUARD,
            REGEXP_OPCODE_REPEAT_END,
            REGEXP_OPCODE_REPEAT_EXIT,
        ] {
            opcode.load(function);
            function.instruction(&Instruction::I64Const(control as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            owner.load(function);
            function.instruction(&Instruction::I64Eqz);
            self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            if control == REGEXP_OPCODE_REPEAT_GUARD {
                pc.load(function);
                owner.load(function);
                function.instruction(&Instruction::I64Ne);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            } else {
                add_words(Local(owner), Constant(u64::MAX), begin, function);
                self.lower_load(
                    compiler.instructions,
                    owner,
                    REGEXP_INSTRUCTION_WIDTH as u64,
                    8,
                    end,
                    function,
                );
                if control == REGEXP_OPCODE_REPEAT_EXIT {
                    add_words(Local(end), Constant(1), end, function);
                }
                pc.load(function);
                end.load(function);
                function.instruction(&Instruction::I64Ne);
                a.load(function);
                begin.load(function);
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32Or);
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
                if control == REGEXP_OPCODE_REPEAT_EXIT {
                    add_words(Local(top), Constant(u64::MAX), top, function);
                }
            }
            function.instruction(&Instruction::End);
        }
        opcode.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_ACCEPT as i64));
        function.instruction(&Instruction::I64Eq);
        owner.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_require_zero(compiler, top, function);
        begins.load(function);
        compiler.repeat_slot_count.load(function);
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);

        set_word(pc, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        pc.load(function);
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.lower_load(compiler.graph_visited, pc, 8, 0, owner, function);
        self.lower_load(
            compiler.instructions,
            pc,
            REGEXP_INSTRUCTION_WIDTH as u64,
            0,
            opcode,
            function,
        );
        self.lower_load(
            compiler.instructions,
            pc,
            REGEXP_INSTRUCTION_WIDTH as u64,
            8,
            a,
            function,
        );
        set_word(expected, Local(owner), function);
        opcode.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_BEGIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(pc), Constant(1), expected, function);
        function.instruction(&Instruction::End);
        opcode.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_EXIT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_load(compiler.graph_visited, a, 8, 0, expected, function);
        function.instruction(&Instruction::End);
        self.lower_load(compiler.graph_edges, pc, 8, 0, packed, function);
        for shift in [0, 16] {
            packed.load(function);
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64And);
            target.store(function);
            target.load(function);
            function.instruction(&Instruction::I64Const(NO_EDGE as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.lower_load(compiler.graph_visited, target, 8, 0, word, function);
            word.load(function);
            expected.load(function);
            function.instruction(&Instruction::I64Ne);
            self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
            self.lower_load(
                compiler.instructions,
                target,
                REGEXP_INSTRUCTION_WIDTH as u64,
                0,
                target_opcode,
                function,
            );
            for (destination, permitted) in [
                (
                    REGEXP_OPCODE_REPEAT_GUARD,
                    &[REGEXP_OPCODE_REPEAT_BEGIN, REGEXP_OPCODE_REPEAT_END][..],
                ),
                (REGEXP_OPCODE_REPEAT_EXIT, &[REGEXP_OPCODE_REPEAT_GUARD][..]),
            ] {
                target_opcode.load(function);
                function.instruction(&Instruction::I64Const(destination as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I32Const(1));
                for allowed in permitted {
                    opcode.load(function);
                    function.instruction(&Instruction::I64Const(*allowed as i64));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::I32And);
                }
                self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
        }
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_clear_graph_colors(compiler, function);
        for local in [
            packed,
            expected,
            target_opcode,
            target,
            begins,
            word,
            a,
            opcode,
            slot,
            next,
            end,
            begin,
            owner,
            top,
            pc,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn lower_clear_graph_colors(&self, compiler: &CompilerLocals, function: &mut Function) {
        compiler.graph_colors.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        compiler.instruction_count.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
    }
}
