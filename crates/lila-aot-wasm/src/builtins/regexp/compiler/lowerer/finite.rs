use super::*;

/// Only this loader can construct a reference to a completed finite atom.
/// Its node, header and row spans belong to the private compiler workspace.
#[must_use]
struct CheckedFiniteClassAtom {
    header: I64Local,
    rows: I64Local,
    count: I64Local,
    singleton_opcode: I64Local,
    singleton_a: I64Local,
    singleton_b: I64Local,
    empty: I32Local,
}

#[must_use]
struct CheckedFiniteClassString {
    instructions: I64Local,
    length: I64Local,
}

#[derive(Clone, Copy)]
enum FiniteSpan {
    Atom,
    Strings,
    Instructions,
}

impl FiniteSpan {
    const fn bytes(self) -> u64 {
        match self {
            Self::Atom => FiniteClassSetAtomWord::BYTES,
            Self::Strings => FiniteClassSetStringWord::BYTES,
            Self::Instructions => REGEXP_INSTRUCTION_WIDTH as u64,
        }
    }
}

impl CheckedFiniteClassAtom {
    fn load(
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        node: I64Local,
        function: &mut Function,
    ) -> Self {
        let atom = Self {
            header: emitter.runtime_schema().reserve_i64_local(function),
            rows: emitter.runtime_schema().reserve_i64_local(function),
            count: emitter.runtime_schema().reserve_i64_local(function),
            singleton_opcode: emitter.runtime_schema().reserve_i64_local(function),
            singleton_a: emitter.runtime_schema().reserve_i64_local(function),
            singleton_b: emitter.runtime_schema().reserve_i64_local(function),
            empty: emitter.runtime_schema().reserve_i32_local(function),
        };
        let empty_word = emitter.runtime_schema().reserve_i64_local(function);
        emitter.lower_node_load(compiler, node, NodeWord::Kind, atom.header, function);
        atom.header.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::FiniteClassSet as i64));
        function.instruction(&Instruction::I64Ne);
        emitter.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        emitter.lower_node_load(compiler, node, NodeWord::Operand0, atom.header, function);
        emitter.lower_finite_span(
            compiler,
            atom.header,
            Constant(1),
            FiniteSpan::Atom,
            function,
        );
        for (word, local) in [
            (FiniteClassSetAtomWord::RowsPointer, atom.rows),
            (FiniteClassSetAtomWord::RowCount, atom.count),
            (
                FiniteClassSetAtomWord::SingletonOpcode,
                atom.singleton_opcode,
            ),
            (FiniteClassSetAtomWord::SingletonOperand0, atom.singleton_a),
            (FiniteClassSetAtomWord::SingletonOperand1, atom.singleton_b),
            (FiniteClassSetAtomWord::ContainsEmpty, empty_word),
        ] {
            atom.header.load(function);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(
                word as u64,
            )));
            local.store(function);
        }
        empty_word.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        emitter.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        empty_word.load(function);
        function.instruction(&Instruction::I32WrapI64);
        atom.empty.store(function);
        emitter
            .runtime_schema()
            .release_i64_local(empty_word, function);
        emitter.lower_finite_span(
            compiler,
            atom.rows,
            Local(atom.count),
            FiniteSpan::Strings,
            function,
        );
        emitter.lower_finite_character_opcode(compiler, atom.singleton_opcode, function);
        atom
    }

    fn string(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        index: I64Local,
        function: &mut Function,
    ) -> CheckedFiniteClassString {
        index.load(function);
        self.count.load(function);
        function.instruction(&Instruction::I64GeU);
        emitter.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        let string = CheckedFiniteClassString {
            instructions: emitter.runtime_schema().reserve_i64_local(function),
            length: emitter.runtime_schema().reserve_i64_local(function),
        };
        for (word, local) in [
            (
                FiniteClassSetStringWord::InstructionsPointer,
                string.instructions,
            ),
            (FiniteClassSetStringWord::CodePointLength, string.length),
        ] {
            emitter.lower_load(
                self.rows,
                index,
                FiniteClassSetStringWord::BYTES,
                word as u64,
                local,
                function,
            );
        }
        string.length.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtU);
        emitter.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        emitter.lower_finite_span(
            compiler,
            string.instructions,
            Local(string.length),
            FiniteSpan::Instructions,
            function,
        );
        string
    }

    fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        emitter
            .runtime_schema()
            .release_i32_local(self.empty, function);
        for local in [
            self.singleton_b,
            self.singleton_a,
            self.singleton_opcode,
            self.count,
            self.rows,
            self.header,
        ] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl CheckedFiniteClassString {
    fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        emitter
            .runtime_schema()
            .release_i64_local(self.length, function);
        emitter
            .runtime_schema()
            .release_i64_local(self.instructions, function);
    }
}

impl FunctionBuilder<'_> {
    /// Check division before multiplication/address addition. Empty row tables
    /// may use the absent pointer; every loaded header or instruction is nonempty.
    fn lower_finite_span(
        &mut self,
        compiler: &CompilerLocals,
        pointer: I64Local,
        count: LowerWord,
        span: FiniteSpan,
        function: &mut Function,
    ) {
        let end = self.runtime_schema().reserve_i64_local(function);
        count.emit(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        end.store(function);
        end.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 32));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        pointer.load(function);
        compiler.heap_checkpoint.load(function);
        function.instruction(&Instruction::I64LtU);
        pointer.load(function);
        end.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        pointer.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        count.emit(function);
        end.load(function);
        pointer.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(span.bytes() as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(end, function);
    }

    fn lower_finite_character_opcode(
        &self,
        compiler: &CompilerLocals,
        opcode: I64Local,
        function: &mut Function,
    ) {
        opcode.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_LITERAL_CODE_POINT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        opcode.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_UNICODE_PROPERTY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    pub(super) fn emit_regexp_finite_class_width(
        &mut self,
        compiler: &CompilerLocals,
        node: I64Local,
        flags: I64Local,
        width: I64Local,
        function: &mut Function,
    ) {
        let atom = CheckedFiniteClassAtom::load(self, compiler, node, function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let previous_length = self.runtime_schema().reserve_i64_local(function);
        let choices = self.runtime_schema().reserve_i64_local(function);
        // Nullable publication belongs to the same completed empty-member fact
        // used by enclosing groups and the existing quantified progress owner.
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        atom.empty.load(function);
        function.instruction(&Instruction::I32Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(index, Constant(0), function);
        set_word(previous_length, Constant(u64::MAX), function);
        set_word(width, Constant(1), function); // The singleton matcher is always present.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        atom.count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        let string = atom.string(self, compiler, index, function);
        string.length.load(function);
        previous_length.load(function);
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(previous_length, Local(string.length), function);
        self.lower_width_add(width, string.length, width, function);
        string.release(self, function);
        add_words(Local(index), Constant(1), index, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // One Split/Jump pair for every alternative except the final one.
        add_words(Local(atom.count), Predicate(atom.empty), choices, function);
        choices.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        choices.store(function);
        self.lower_width_add(width, choices, width, function);
        for local in [choices, previous_length, index] {
            self.runtime_schema().release_i64_local(local, function);
        }
        atom.release(self, function);
    }

    pub(super) fn emit_regexp_lower_finite_class(
        &mut self,
        compiler: &CompilerLocals,
        node: I64Local,
        direction: I64Local,
        pc: I64Local,
        end: I64Local,
        function: &mut Function,
    ) {
        let atom = CheckedFiniteClassAtom::load(self, compiler, node, function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let position = self.runtime_schema().reserve_i64_local(function);
        let instruction_index = self.runtime_schema().reserve_i64_local(function);
        let attempt = self.runtime_schema().reserve_i64_local(function);
        let exit = self.runtime_schema().reserve_i64_local(function);
        let fallback = self.runtime_schema().reserve_i64_local(function);
        let opcode = self.runtime_schema().reserve_i64_local(function);
        let a = self.runtime_schema().reserve_i64_local(function);
        let b = self.runtime_schema().reserve_i64_local(function);
        set_word(index, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        atom.count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        let string = atom.string(self, compiler, index, function);
        add_words(Local(pc), Constant(1), attempt, function);
        add_words(Local(attempt), Local(string.length), exit, function);
        add_words(Local(exit), Constant(1), fallback, function);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(attempt),
            Local(fallback),
            function,
        );
        set_word(pc, Local(attempt), function);
        set_word(position, Constant(0), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        position.load(function);
        string.length.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        direction.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        position.load(function);
        function.instruction(&Instruction::Else);
        string.length.load(function);
        position.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        instruction_index.store(function);
        for (offset, local) in [(0, opcode), (8, a), (16, b)] {
            self.lower_load(
                string.instructions,
                instruction_index,
                REGEXP_INSTRUCTION_WIDTH as u64,
                offset,
                local,
                function,
            );
        }
        self.lower_finite_character_opcode(compiler, opcode, function);
        self.lower_instruction(compiler, pc, Local(opcode), Local(a), Local(b), function);
        add_words(Local(pc), Constant(1), pc, function);
        add_words(Local(position), Constant(1), position, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_instruction(
            compiler,
            exit,
            Constant(REGEXP_OPCODE_JUMP),
            Local(end),
            Constant(0),
            function,
        );
        set_word(pc, Local(fallback), function);
        string.release(self, function);
        add_words(Local(index), Constant(1), index, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        atom.empty.load(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(pc), Constant(1), attempt, function);
        add_words(Local(attempt), Constant(1), exit, function);
        add_words(Local(exit), Constant(1), fallback, function);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(attempt),
            Local(fallback),
            function,
        );
        self.lower_instruction(
            compiler,
            attempt,
            Local(atom.singleton_opcode),
            Local(atom.singleton_a),
            Local(atom.singleton_b),
            function,
        );
        self.lower_instruction(
            compiler,
            exit,
            Constant(REGEXP_OPCODE_JUMP),
            Local(end),
            Constant(0),
            function,
        );
        set_word(pc, Local(fallback), function);
        function.instruction(&Instruction::Else);
        self.lower_instruction(
            compiler,
            pc,
            Local(atom.singleton_opcode),
            Local(atom.singleton_a),
            Local(atom.singleton_b),
            function,
        );
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::End);
        pc.load(function);
        end.load(function);
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for local in [
            b,
            a,
            opcode,
            fallback,
            exit,
            attempt,
            instruction_index,
            position,
            index,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        atom.release(self, function);
    }
}
