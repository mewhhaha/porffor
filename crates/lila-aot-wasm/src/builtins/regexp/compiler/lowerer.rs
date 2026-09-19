use super::*;
use lila_ir::{RegExpControlFlow, RegExpInputProgress, RegExpOpcode, RegExpOperandRule};

mod graph;
mod width;

const WIDTH_LIMIT: u64 = REGEXP_MAX_INSTRUCTIONS as u64;
// Width analysis owns this bit after parsing; erased subtrees cannot retain it.
const NODE_WIDTH_REPETITION_LIMIT: u64 = 1 << 63;
const NO_EDGE: u64 = 0xffff;
const PROGRESS_BARRIER: u64 = 1 << 32;
const _: () = assert!(WIDTH_LIMIT < NO_EDGE);

#[derive(Clone, Copy)]
enum LowerWord {
    Constant(u64),
    Local(u32),
}
impl LowerWord {
    fn emit(self, function: &mut Function) {
        match self {
            Self::Constant(word) => {
                function.instruction(&Instruction::I64Const(word as i64));
            }
            Self::Local(local) => {
                function.instruction(&Instruction::LocalGet(local));
            }
        }
    }
}
use LowerWord::{Constant, Local};

#[derive(Clone, Copy)]
#[repr(u64)]
enum LowerTask {
    Quantified = 1,
    Atom,
}

fn set_word(local: u32, word: LowerWord, function: &mut Function) {
    word.emit(function);
    function.instruction(&Instruction::LocalSet(local));
}

fn add_words(left: LowerWord, right: LowerWord, output: u32, function: &mut Function) {
    left.emit(function);
    right.emit(function);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(output));
}

fn indexed_address(base: u32, index: u32, width: u64, function: &mut Function) {
    function.instruction(&Instruction::LocalGet(base));
    function.instruction(&Instruction::LocalGet(index));
    function.instruction(&Instruction::I64Const(width as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
}

impl FunctionBuilder<'_> {
    fn lower_load(
        &self,
        base: u32,
        index: u32,
        width: u64,
        offset: u64,
        output: u32,
        function: &mut Function,
    ) {
        indexed_address(base, index, width, function);
        function.instruction(&Instruction::I64Load(Self::memarg64(offset)));
        function.instruction(&Instruction::LocalSet(output));
    }

    fn lower_store(
        &self,
        base: u32,
        index: u32,
        width: u64,
        offset: u64,
        word: LowerWord,
        function: &mut Function,
    ) {
        indexed_address(base, index, width, function);
        word.emit(function);
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    fn lower_fail_if(
        &self,
        compiler: &CompilerLocals,
        failure: CompileFailure,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, failure, function);
        function.instruction(&Instruction::End);
    }

    fn lower_node_load(
        &self,
        compiler: &CompilerLocals,
        node: u32,
        word: NodeWord,
        output: u32,
        function: &mut Function,
    ) {
        self.lower_load(
            compiler.nodes,
            node,
            NODE_BYTES,
            word as u64,
            output,
            function,
        );
    }

    fn lower_checked_node(&self, compiler: &CompilerLocals, node: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(node));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(node));
        function.instruction(&Instruction::LocalGet(compiler.node_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_instruction(
        &self,
        compiler: &CompilerLocals,
        pc: u32,
        opcode: LowerWord,
        a: LowerWord,
        b: LowerWord,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        indexed_address(compiler.graph_visited, pc, 8, function);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_store(compiler.graph_visited, pc, 8, 0, Constant(1), function);
        for (offset, word) in [(0, opcode), (8, a), (16, b)] {
            self.lower_store(
                compiler.instructions,
                pc,
                REGEXP_INSTRUCTION_WIDTH as u64,
                offset,
                word,
                function,
            );
        }
    }

    fn lower_push(
        &self,
        compiler: &CompilerLocals,
        top: u32,
        task: LowerTask,
        node: u32,
        pc: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(top));
        function.instruction(&Instruction::LocalGet(compiler.task_capacity));
        function.instruction(&Instruction::I64GeU);
        self.lower_fail_if(
            compiler,
            CompileFailure::Resource(CompileResource::Tasks),
            function,
        );
        for (offset, word) in [
            (0, Constant(task as u64)),
            (8, Local(node)),
            (16, Local(pc)),
        ] {
            self.lower_store(compiler.tasks, top, TASK_BYTES, offset, word, function);
        }
        add_words(Local(top), Constant(1), top, function);
    }

    pub(super) fn emit_regexp_runtime_lower(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_regexp_lower_widths(compiler, function);
        let top = self.reserve_temp_local();
        let task = self.reserve_temp_local();
        let node = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let width = self.reserve_temp_local();
        set_word(node, Constant(1), function);
        self.lower_node_load(compiler, node, NodeWord::Width, width, function);
        function.instruction(&Instruction::LocalGet(width));
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_node_load(compiler, node, NodeWord::Flags, task, function);
        function.instruction(&Instruction::LocalGet(task));
        function.instruction(&Instruction::I64Const(NODE_WIDTH_REPETITION_LIMIT as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Instructions),
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Repetition),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        add_words(
            Local(width),
            Constant(1),
            compiler.instruction_count,
            function,
        );
        set_word(pc, Local(width), function);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_ACCEPT),
            Constant(0),
            Constant(0),
            function,
        );
        set_word(top, Constant(0), function);
        set_word(pc, Constant(0), function);
        self.lower_push(compiler, top, LowerTask::Quantified, node, pc, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(top));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(top), Constant(u64::MAX), top, function);
        self.lower_load(compiler.tasks, top, TASK_BYTES, 0, task, function);
        self.lower_load(compiler.tasks, top, TASK_BYTES, 8, node, function);
        self.lower_load(compiler.tasks, top, TASK_BYTES, 16, pc, function);
        self.lower_checked_node(compiler, node, function);
        self.lower_node_load(
            compiler,
            node,
            NodeWord::SourceOffset,
            compiler.cursor,
            function,
        );
        function.instruction(&Instruction::LocalGet(task));
        function.instruction(&Instruction::I64Const(LowerTask::Quantified as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_lower_quantified(compiler, top, node, pc, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(task));
        function.instruction(&Instruction::I64Const(LowerTask::Atom as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.emit_regexp_lower_atom(compiler, top, node, pc, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [width, pc, node, task, top] {
            self.release_temp_local(local);
        }
        self.emit_regexp_lower_graph(compiler, function);
        Ok(())
    }

    fn emit_regexp_lower_quantified(
        &mut self,
        compiler: &CompilerLocals,
        top: u32,
        node: u32,
        start: u32,
        function: &mut Function,
    ) {
        let atom_width = self.reserve_temp_local();
        let minimum = self.reserve_temp_local();
        let maximum = self.reserve_temp_local();
        let flags = self.reserve_temp_local();
        let width = self.reserve_temp_local();
        let after = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let remaining = self.reserve_temp_local();
        let attempt = self.reserve_temp_local();
        let continuation = self.reserve_temp_local();
        let fallback = self.reserve_temp_local();
        let packed = self.reserve_temp_local();
        for (word, local) in [
            (NodeWord::AtomWidth, atom_width),
            (NodeWord::Minimum, minimum),
            (NodeWord::Maximum, maximum),
            (NodeWord::Flags, flags),
            (NodeWord::Width, width),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        function.instruction(&Instruction::LocalGet(atom_width));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(minimum, Constant(0), function);
        set_word(maximum, Constant(0), function);
        function.instruction(&Instruction::End);
        add_words(Local(start), Local(width), after, function);
        set_word(pc, Local(start), function);
        set_word(remaining, Local(minimum), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.lower_push(compiler, top, LowerTask::Atom, node, pc, function);
        add_words(Local(pc), Local(atom_width), pc, function);
        add_words(Local(remaining), Constant(u64::MAX), remaining, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64Const(UNBOUNDED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::LocalGet(minimum));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(remaining));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        add_words(Local(pc), Constant(1), attempt, function);
        self.lower_push(compiler, top, LowerTask::Atom, node, attempt, function);
        add_words(Local(attempt), Local(atom_width), continuation, function);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(fallback, Local(continuation), function);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64Const(UNBOUNDED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            continuation,
            Constant(REGEXP_OPCODE_JUMP),
            Local(pc),
            Constant(0),
            function,
        );
        add_words(Local(continuation), Constant(1), fallback, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(attempt),
            Local(fallback),
            function,
        );
        function.instruction(&Instruction::Else);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(fallback),
            Local(attempt),
            function,
        );
        function.instruction(&Instruction::End);
        set_word(pc, Local(fallback), function);
        function.instruction(&Instruction::Else);
        add_words(Local(continuation), Constant(1), fallback, function);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64Const(UNBOUNDED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(fallback));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(packed));
        self.lower_instruction(
            compiler,
            continuation,
            Constant(REGEXP_OPCODE_PROGRESS_CHECK),
            Local(pc),
            Local(packed),
            function,
        );
        function.instruction(&Instruction::LocalGet(after));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(packed));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_PROGRESS_SPLIT),
            Local(attempt),
            Local(packed),
            function,
        );
        set_word(pc, Local(fallback), function);
        function.instruction(&Instruction::End);
        add_words(Local(remaining), Constant(u64::MAX), remaining, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(after));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for local in [
            packed,
            fallback,
            continuation,
            attempt,
            remaining,
            pc,
            after,
            width,
            flags,
            maximum,
            minimum,
            atom_width,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_lower_atom(
        &mut self,
        compiler: &CompilerLocals,
        top: u32,
        node: u32,
        start: u32,
        function: &mut Function,
    ) {
        let kind = self.reserve_temp_local();
        let width = self.reserve_temp_local();
        let child = self.reserve_temp_local();
        let pc = self.reserve_temp_local();
        let end = self.reserve_temp_local();
        let body_end = self.reserve_temp_local();
        let capture_start = self.reserve_temp_local();
        let capture_end = self.reserve_temp_local();
        let a = self.reserve_temp_local();
        let b = self.reserve_temp_local();
        let opcode = self.reserve_temp_local();
        for (word, local) in [
            (NodeWord::Kind, kind),
            (NodeWord::AtomWidth, width),
            (NodeWord::First, child),
            (NodeWord::CaptureStart, capture_start),
            (NodeWord::CaptureEnd, capture_end),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        add_words(Local(start), Local(width), end, function);
        set_word(pc, Local(start), function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Atom as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (word, local) in [
            (NodeWord::Opcode, opcode),
            (NodeWord::Operand0, a),
            (NodeWord::Operand1, b),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        // An incompletely initialized Atom must not turn the zero opcode into Accept.
        function.instruction(&Instruction::I32Const(0));
        for fact in RegExpOpcode::ALL
            .into_iter()
            .filter(|fact| fact.is_source_atom())
        {
            function.instruction(&Instruction::LocalGet(opcode));
            function.instruction(&Instruction::I64Const(fact as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_instruction(compiler, pc, Local(opcode), Local(a), Local(b), function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_lower_children(compiler, top, child, pc, end, false, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(capture_start));
        function.instruction(&Instruction::LocalGet(capture_end));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_CLEAR_CAPTURE_RANGE),
            Local(capture_start),
            Local(capture_end),
            function,
        );
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::End);
        set_word(body_end, Local(end), function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Capture as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_CAPTURE_START),
            Local(capture_start),
            Constant(0),
            function,
        );
        add_words(Local(pc), Constant(1), pc, function);
        add_words(Local(end), Constant(u64::MAX), body_end, function);
        self.lower_instruction(
            compiler,
            body_end,
            Constant(REGEXP_OPCODE_CAPTURE_END),
            Local(capture_start),
            Constant(0),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::PositiveLookahead as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_LOOKAROUND_START),
            Constant(0),
            Constant(0),
            function,
        );
        add_words(Local(pc), Constant(1), pc, function);
        add_words(Local(pc), Constant(1), a, function);
        add_words(Local(end), Constant(u64::MAX), b, function);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(a),
            Local(b),
            function,
        );
        set_word(pc, Local(a), function);
        add_words(Local(end), Constant(u64::MAX - 1), body_end, function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::NegativeLookahead as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(opcode));
        function.instruction(&Instruction::LocalGet(opcode));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(a));
        self.lower_instruction(
            compiler,
            body_end,
            Constant(REGEXP_OPCODE_LOOKAROUND_END),
            Local(b),
            Local(a),
            function,
        );
        self.lower_instruction(
            compiler,
            b,
            Constant(REGEXP_OPCODE_LOOKAROUND_FAILURE),
            Local(end),
            Local(opcode),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_lower_children(compiler, top, child, pc, body_end, true, function);
        set_word(pc, Local(end), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for local in [
            opcode,
            b,
            a,
            capture_end,
            capture_start,
            body_end,
            end,
            pc,
            child,
            width,
            kind,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_lower_children(
        &mut self,
        compiler: &CompilerLocals,
        top: u32,
        child: u32,
        pc: u32,
        end: u32,
        alternatives: bool,
        function: &mut Function,
    ) {
        let next = self.reserve_temp_local();
        let width = self.reserve_temp_local();
        let attempt = self.reserve_temp_local();
        let exit = self.reserve_temp_local();
        let fallback = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.lower_node_load(compiler, child, NodeWord::Next, next, function);
        self.lower_node_load(compiler, child, NodeWord::Width, width, function);
        if alternatives {
            function.instruction(&Instruction::LocalGet(next));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.lower_push(compiler, top, LowerTask::Quantified, child, pc, function);
            add_words(Local(pc), Local(width), pc, function);
            function.instruction(&Instruction::Else);
            add_words(Local(pc), Constant(1), attempt, function);
            add_words(Local(attempt), Local(width), exit, function);
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
                exit,
                Constant(REGEXP_OPCODE_JUMP),
                Local(end),
                Constant(0),
                function,
            );
            self.lower_push(
                compiler,
                top,
                LowerTask::Quantified,
                child,
                attempt,
                function,
            );
            set_word(pc, Local(fallback), function);
            function.instruction(&Instruction::End);
        } else {
            self.lower_push(compiler, top, LowerTask::Quantified, child, pc, function);
            add_words(Local(pc), Local(width), pc, function);
        }
        set_word(child, Local(next), function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(pc));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for local in [fallback, exit, attempt, width, next] {
            self.release_temp_local(local);
        }
    }
}
