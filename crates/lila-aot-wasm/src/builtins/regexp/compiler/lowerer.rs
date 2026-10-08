use super::*;
use crate::gc_types::I32Local;
use lila_ir::{RegExpControlFlow, RegExpInputProgress, RegExpOpcode, RegExpOperandRule};

mod counted;
mod finite;
mod graph;
mod width;

const WIDTH_LIMIT: u64 = REGEXP_MAX_INSTRUCTIONS as u64;
const NO_EDGE: u64 = 0xffff;
const PROGRESS_BARRIER: u64 = 1 << 32;
const _: () = assert!(WIDTH_LIMIT < NO_EDGE);

#[derive(Clone, Copy)]
enum LowerWord {
    Constant(u64),
    Local(I64Local),
    Predicate(I32Local),
}
impl LowerWord {
    fn emit(self, function: &mut Function) {
        match self {
            Self::Constant(word) => {
                function.instruction(&Instruction::I64Const(word as i64));
            }
            Self::Local(local) => {
                local.load(function);
            }
            Self::Predicate(local) => {
                local.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
            }
        }
    }
}
use LowerWord::{Constant, Local, Predicate};

#[derive(Clone, Copy)]
#[repr(u64)]
enum LowerTask {
    Quantified = 1,
    Atom,
}

/// Alternatives retain source priority; a sequence owns its validated direction.
/// No caller can accidentally request a sequence without supplying that owner.
enum LowerChildOrder {
    Alternatives,
    Sequence { direction: I64Local },
}

fn set_word(local: I64Local, word: LowerWord, function: &mut Function) {
    word.emit(function);
    local.store(function);
}

fn add_words(left: LowerWord, right: LowerWord, output: I64Local, function: &mut Function) {
    left.emit(function);
    right.emit(function);
    function.instruction(&Instruction::I64Add);
    output.store(function);
}

fn indexed_address(base: I64Local, index: I64Local, width: u64, function: &mut Function) {
    base.load(function);
    index.load(function);
    function.instruction(&Instruction::I64Const(width as i64));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
}

impl FunctionBuilder<'_> {
    fn lower_load(
        &self,
        base: I64Local,
        index: I64Local,
        width: u64,
        offset: u64,
        output: I64Local,
        function: &mut Function,
    ) {
        indexed_address(base, index, width, function);
        function.instruction(&Instruction::I64Load(Self::memarg64(offset)));
        output.store(function);
    }

    fn lower_store(
        &self,
        base: I64Local,
        index: I64Local,
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
        node: I64Local,
        word: NodeWord,
        output: I64Local,
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

    fn lower_checked_node(
        &self,
        compiler: &CompilerLocals,
        node: I64Local,
        function: &mut Function,
    ) {
        node.load(function);
        function.instruction(&Instruction::I64Eqz);
        node.load(function);
        compiler.node_count.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
    }

    fn lower_instruction(
        &self,
        compiler: &CompilerLocals,
        pc: I64Local,
        opcode: LowerWord,
        a: LowerWord,
        b: LowerWord,
        function: &mut Function,
    ) {
        pc.load(function);
        compiler.instruction_count.load(function);
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
        top: I64Local,
        task: LowerTask,
        node: I64Local,
        pc: I64Local,
        function: &mut Function,
    ) {
        top.load(function);
        compiler.task_capacity.load(function);
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
        captures: &CompletedRegExpPattern,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_regexp_lower_widths(compiler, function);
        let top = self.runtime_schema().reserve_i64_local(function);
        let task = self.runtime_schema().reserve_i64_local(function);
        let node = self.runtime_schema().reserve_i64_local(function);
        let pc = self.runtime_schema().reserve_i64_local(function);
        let width = self.runtime_schema().reserve_i64_local(function);
        set_word(node, Constant(1), function);
        self.lower_node_load(compiler, node, NodeWord::Width, width, function);
        width.load(function);
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Instructions),
            function,
        );
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
        set_word(compiler.repeat_slot_count, Constant(0), function);
        set_word(top, Constant(0), function);
        set_word(pc, Constant(0), function);
        self.lower_push(compiler, top, LowerTask::Quantified, node, pc, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        top.load(function);
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
        task.load(function);
        function.instruction(&Instruction::I64Const(LowerTask::Quantified as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_lower_quantified(compiler, top, node, pc, function);
        function.instruction(&Instruction::Else);
        task.load(function);
        function.instruction(&Instruction::I64Const(LowerTask::Atom as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.emit_regexp_lower_atom(compiler, top, node, pc, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [width, pc, node, task, top] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.emit_regexp_lower_graph(compiler, captures, function);
        Ok(())
    }

    /// Both width analysis and emission select the static compiler's single
    /// body plus layout from the same parser-owned nullability authority.
    fn emit_regexp_single_body_plus_condition(
        &self,
        minimum: I64Local,
        maximum_kind: I64Local,
        flags: I64Local,
        function: &mut Function,
    ) {
        minimum.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
    }

    fn emit_regexp_lower_quantified(
        &mut self,
        compiler: &CompilerLocals,
        top: I64Local,
        node: I64Local,
        start: I64Local,
        function: &mut Function,
    ) {
        let atom_width = self.runtime_schema().reserve_i64_local(function);
        let minimum = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let maximum_kind = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        let width = self.runtime_schema().reserve_i64_local(function);
        let after = self.runtime_schema().reserve_i64_local(function);
        let pc = self.runtime_schema().reserve_i64_local(function);
        let remaining = self.runtime_schema().reserve_i64_local(function);
        let attempt = self.runtime_schema().reserve_i64_local(function);
        let continuation = self.runtime_schema().reserve_i64_local(function);
        let fallback = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        for (word, local) in [
            (NodeWord::AtomWidth, atom_width),
            (NodeWord::Minimum, minimum),
            (NodeWord::Maximum, maximum),
            (NodeWord::MaximumKind, maximum_kind),
            (NodeWord::Flags, flags),
            (NodeWord::Width, width),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        atom_width.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(minimum, Constant(0), function);
        set_word(maximum, Constant(0), function);
        set_word(
            maximum_kind,
            Constant(RegExpRepeatMaximumKind::Finite.word()),
            function,
        );
        function.instruction(&Instruction::End);
        add_words(Local(start), Local(width), after, function);
        set_word(pc, Local(start), function);
        self.emit_regexp_counted_body_condition(minimum, maximum, maximum_kind, flags, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        let pending = self
            .emit_regexp_begin_counted_repeat(compiler, node, start, atom_width, flags, function);
        add_words(Local(start), Constant(2), pc, function);
        self.lower_push(compiler, top, LowerTask::Atom, node, pc, function);
        self.emit_regexp_finish_counted_repeat(compiler, pending, function);
        set_word(pc, Local(after), function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_single_body_plus_condition(minimum, maximum_kind, flags, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_push(compiler, top, LowerTask::Atom, node, start, function);
        add_words(Local(start), Local(atom_width), pc, function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(start),
            Local(after),
            function,
        );
        function.instruction(&Instruction::Else);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_SPLIT),
            Local(after),
            Local(start),
            function,
        );
        function.instruction(&Instruction::End);
        set_word(pc, Local(after), function);
        function.instruction(&Instruction::Else);
        minimum.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.lower_push(compiler, top, LowerTask::Atom, node, pc, function);
        add_words(Local(pc), Local(atom_width), pc, function);
        function.instruction(&Instruction::End);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        maximum.load(function);
        minimum.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        remaining.store(function);
        remaining.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(pc), Constant(1), attempt, function);
        self.lower_push(compiler, top, LowerTask::Atom, node, attempt, function);
        add_words(Local(attempt), Local(atom_width), continuation, function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(fallback, Local(continuation), function);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
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
        flags.load(function);
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
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        pc.load(function);
        function.instruction(&Instruction::Else);
        fallback.load(function);
        function.instruction(&Instruction::End);
        packed.store(function);
        self.lower_instruction(
            compiler,
            continuation,
            Constant(REGEXP_OPCODE_PROGRESS_CHECK),
            Local(pc),
            Local(packed),
            function,
        );
        after.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_LAZY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        packed.store(function);
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
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        pc.load(function);
        after.load(function);
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
            maximum_kind,
            maximum,
            minimum,
            atom_width,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_lower_atom(
        &mut self,
        compiler: &CompilerLocals,
        top: I64Local,
        node: I64Local,
        start: I64Local,
        function: &mut Function,
    ) {
        let kind = self.runtime_schema().reserve_i64_local(function);
        let direction = self.runtime_schema().reserve_i64_local(function);
        let width = self.runtime_schema().reserve_i64_local(function);
        let child = self.runtime_schema().reserve_i64_local(function);
        let pc = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        let body_end = self.runtime_schema().reserve_i64_local(function);
        let capture_start = self.runtime_schema().reserve_i64_local(function);
        let capture_end = self.runtime_schema().reserve_i64_local(function);
        let a = self.runtime_schema().reserve_i64_local(function);
        let b = self.runtime_schema().reserve_i64_local(function);
        let opcode = self.runtime_schema().reserve_i64_local(function);
        for (word, local) in [
            (NodeWord::Kind, kind),
            (NodeWord::Direction, direction),
            (NodeWord::AtomWidth, width),
            (NodeWord::First, child),
            (NodeWord::CaptureStart, capture_start),
            (NodeWord::CaptureEnd, capture_end),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        add_words(Local(start), Local(width), end, function);
        set_word(pc, Local(start), function);
        kind.load(function);
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
            opcode.load(function);
            function.instruction(&Instruction::I64Const(fact as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_instruction(compiler, pc, Local(opcode), Local(a), Local(b), function);
        add_words(Local(pc), Constant(1), pc, function);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::FiniteClassSet as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_lower_finite_class(compiler, node, direction, pc, end, function);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_lower_children(
            compiler,
            top,
            child,
            pc,
            end,
            LowerChildOrder::Sequence { direction },
            function,
        );
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Ne);
        capture_start.load(function);
        capture_end.load(function);
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
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Capture as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        direction.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_START as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_END as i64));
        function.instruction(&Instruction::End);
        opcode.store(function);
        self.lower_instruction(
            compiler,
            pc,
            Local(opcode),
            Local(capture_start),
            Constant(0),
            function,
        );
        add_words(Local(pc), Constant(1), pc, function);
        add_words(Local(end), Constant(u64::MAX), body_end, function);
        direction.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_END as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_CAPTURE_START as i64));
        function.instruction(&Instruction::End);
        opcode.store(function);
        self.lower_instruction(
            compiler,
            body_end,
            Local(opcode),
            Local(capture_start),
            Constant(0),
            function,
        );
        function.instruction(&Instruction::End);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::PositiveLookahead as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::PositiveLookbehind as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        a.store(function);
        self.lower_instruction(
            compiler,
            pc,
            Constant(REGEXP_OPCODE_LOOKAROUND_START),
            Local(a),
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
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::NegativeLookahead as i64));
        function.instruction(&Instruction::I64Eq);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::NegativeLookbehind as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        opcode.store(function);
        opcode.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        direction.load(function);
        function.instruction(&Instruction::I64Const(62));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        end.load(function);
        function.instruction(&Instruction::I64Or);
        a.store(function);
        self.lower_instruction(
            compiler,
            body_end,
            Constant(REGEXP_OPCODE_LOOKAROUND_END),
            Local(b),
            Local(a),
            function,
        );
        direction.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        opcode.load(function);
        function.instruction(&Instruction::I64Or);
        opcode.store(function);
        self.lower_instruction(
            compiler,
            b,
            Constant(REGEXP_OPCODE_LOOKAROUND_FAILURE),
            Local(end),
            Local(opcode),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_lower_children(
            compiler,
            top,
            child,
            pc,
            body_end,
            LowerChildOrder::Alternatives,
            function,
        );
        set_word(pc, Local(end), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        pc.load(function);
        end.load(function);
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
            direction,
            kind,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_lower_children(
        &mut self,
        compiler: &CompilerLocals,
        top: I64Local,
        child: I64Local,
        pc: I64Local,
        end: I64Local,
        order: LowerChildOrder,
        function: &mut Function,
    ) {
        let next = self.runtime_schema().reserve_i64_local(function);
        let width = self.runtime_schema().reserve_i64_local(function);
        let attempt = self.runtime_schema().reserve_i64_local(function);
        let exit = self.runtime_schema().reserve_i64_local(function);
        let fallback = self.runtime_schema().reserve_i64_local(function);
        let start = self.runtime_schema().reserve_i64_local(function);
        set_word(start, Local(pc), function);
        if let LowerChildOrder::Sequence { direction } = order {
            direction.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            set_word(pc, Local(end), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        child.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.lower_node_load(compiler, child, NodeWord::Next, next, function);
        self.lower_node_load(compiler, child, NodeWord::Width, width, function);
        match order {
            LowerChildOrder::Alternatives => {
                next.load(function);
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
            }
            LowerChildOrder::Sequence { direction } => {
                direction.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.lower_push(compiler, top, LowerTask::Quantified, child, pc, function);
                add_words(Local(pc), Local(width), pc, function);
                function.instruction(&Instruction::Else);
                // Traverse the source-linked terms once, assigning descending code
                // intervals. Alternatives retain source priority in either direction.
                pc.load(function);
                width.load(function);
                function.instruction(&Instruction::I64Sub);
                pc.store(function);
                self.lower_push(compiler, top, LowerTask::Quantified, child, pc, function);
                function.instruction(&Instruction::End);
            }
        }
        set_word(child, Local(next), function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        pc.load(function);
        if let LowerChildOrder::Sequence { direction } = order {
            direction.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            end.load(function);
            function.instruction(&Instruction::Else);
            start.load(function);
            function.instruction(&Instruction::End);
        } else {
            end.load(function);
        }
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        // The enclosing atom still owns its final forward code interval.
        set_word(pc, Local(end), function);
        for local in [start, fallback, exit, attempt, width, next] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
