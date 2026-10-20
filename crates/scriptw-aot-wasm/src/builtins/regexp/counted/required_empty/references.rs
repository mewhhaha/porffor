//! References consume the actual post-body capture fixed point and ordered prefix.
use super::*;

#[derive(Clone, Copy)]
enum CaptureWrite {
    Start,
    End,
    Range,
    None,
}

impl CaptureWrite {
    fn for_opcode(opcode: RegExpOpcode) -> Self {
        match opcode {
            RegExpOpcode::CaptureStart => Self::Start,
            RegExpOpcode::CaptureEnd => Self::End,
            RegExpOpcode::ClearCaptureRange => Self::Range,
            RegExpOpcode::Accept
            | RegExpOpcode::LiteralAscii
            | RegExpOpcode::PositiveAsciiClass
            | RegExpOpcode::Split
            | RegExpOpcode::Jump
            | RegExpOpcode::Whitespace
            | RegExpOpcode::Dot
            | RegExpOpcode::LiteralCodePoint
            | RegExpOpcode::UnicodeProperty
            | RegExpOpcode::NamedBackreference
            | RegExpOpcode::NegativeAsciiClass
            | RegExpOpcode::NumberedBackreference
            | RegExpOpcode::AssertStart
            | RegExpOpcode::AssertEnd
            | RegExpOpcode::NotWhitespace
            | RegExpOpcode::LookaroundStart
            | RegExpOpcode::LookaroundEnd
            | RegExpOpcode::LookaroundFailure
            | RegExpOpcode::ProgressSplit
            | RegExpOpcode::ProgressCheck
            | RegExpOpcode::WordBoundary
            | RegExpOpcode::RepeatBegin
            | RegExpOpcode::RepeatGuard
            | RegExpOpcode::RepeatEnd
            | RegExpOpcode::RepeatExit => Self::None,
        }
    }
}

struct StableReferencesLocals {
    capture: I64Local,
    start: I64Local,
    end: I64Local,
    pc: I64Local,
    address: I64Local,
    opcode: I64Local,
    operand0: I64Local,
    operand1: I64Local,
    candidates: I64Local,
    count: I64Local,
    index: I64Local,
    temporary: I64Local,
    participating: I64Local,
}

impl StableReferencesLocals {
    fn reserve(builder: &FunctionBuilder<'_>, f: &mut Function) -> Self {
        let schema = builder.runtime_schema();
        Self {
            capture: schema.reserve_i64_local(f),
            start: schema.reserve_i64_local(f),
            end: schema.reserve_i64_local(f),
            pc: schema.reserve_i64_local(f),
            address: schema.reserve_i64_local(f),
            opcode: schema.reserve_i64_local(f),
            operand0: schema.reserve_i64_local(f),
            operand1: schema.reserve_i64_local(f),
            candidates: schema.reserve_i64_local(f),
            count: schema.reserve_i64_local(f),
            index: schema.reserve_i64_local(f),
            temporary: schema.reserve_i64_local(f),
            participating: schema.reserve_i64_local(f),
        }
    }

    fn require_replay_empty_capture(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        pair: &CountedLoopLocals,
        current: &CountedMatcherLocals,
        scan: &RequiredEmptyReplayLocals,
        f: &mut Function,
    ) {
        self.capture.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GeU);
        self.capture.load(f);
        workspace.capture_bytes.load(f);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        scan.intersect(f);
        scan.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        workspace.base.load(f);
        self.capture.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.address.store(f);
        builder.emit_regexp_scratch_load_word(self.address, 0, self.start, f);
        builder.emit_regexp_scratch_load_word(self.address, 8, self.end, f);
        self.require_canonical_empty(current, scan, f);
        pair.guard.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        self.pc.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        self.pc.load(f);
        scan.pc.load(f);
        f.instruction(&Instruction::I64GeU);
        scan.eligible.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::BrIf(1));
        current.program.load(f);
        self.pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.address.store(f);
        builder.emit_regexp_instruction_load(self.address, 0, self.opcode, f);
        builder.emit_regexp_instruction_load(self.address, 8, self.operand0, f);
        builder.emit_regexp_instruction_load(self.address, 16, self.operand1, f);
        // Replay this reference's ordered prefix from the actual post-body
        // pair, without changing the workspace. Every allowed write is a
        // constant at this cursor. Positive exact nested bodies are idempotent
        // too, so their physical prefix suffices; their live counters are
        // independently checked by the complete outer instruction scan.
        for opcode in RegExpOpcode::ALL {
            let write = CaptureWrite::for_opcode(opcode);
            if matches!(write, CaptureWrite::None) {
                continue;
            }
            self.opcode.load(f);
            f.instruction(&Instruction::I64Const(opcode as i64));
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::If(BlockType::Empty));
            match write {
                CaptureWrite::Start | CaptureWrite::End => {
                    self.operand0.load(f);
                    self.capture.load(f);
                    f.instruction(&Instruction::I64Eq);
                }
                CaptureWrite::Range => {
                    self.capture.load(f);
                    self.operand0.load(f);
                    f.instruction(&Instruction::I64GeU);
                    self.capture.load(f);
                    self.operand1.load(f);
                    f.instruction(&Instruction::I64LtU);
                    f.instruction(&Instruction::I32And);
                }
                CaptureWrite::None => unreachable!("non-writing opcode has no emitted arm"),
            }
            f.instruction(&Instruction::If(BlockType::Empty));
            match write {
                CaptureWrite::Start => {
                    current.cursor.utf16.load(f);
                    self.start.store(f);
                    current.reverse_mode.load(f);
                    f.instruction(&Instruction::I64Eqz);
                    f.instruction(&Instruction::If(BlockType::Empty));
                    f.instruction(&Instruction::I64Const(-1));
                    self.end.store(f);
                    f.instruction(&Instruction::End);
                }
                CaptureWrite::End => {
                    current.reverse_mode.load(f);
                    f.instruction(&Instruction::I64Eqz);
                    f.instruction(&Instruction::If(BlockType::Empty));
                    self.start.load(f);
                    f.instruction(&Instruction::I64Const(-1));
                    f.instruction(&Instruction::I64Ne);
                    scan.intersect(f);
                    f.instruction(&Instruction::End);
                    current.cursor.utf16.load(f);
                    self.end.store(f);
                }
                CaptureWrite::Range => {
                    f.instruction(&Instruction::I64Const(-1));
                    self.start.store(f);
                    f.instruction(&Instruction::I64Const(-1));
                    self.end.store(f);
                }
                CaptureWrite::None => unreachable!("non-writing opcode has no emitted arm"),
            }
            f.instruction(&Instruction::End);
            f.instruction(&Instruction::End);
        }
        self.pc.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        self.pc.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        // A currently open capture has one unset endpoint and the real
        // reference dispatcher treats it as nonparticipating. This temporary
        // prefix state is allowed; the retained post-body pair above must be
        // canonical so it can seed the next replay.
        self.start.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        self.end.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32Or);
        self.start.load(f);
        self.end.load(f);
        f.instruction(&Instruction::I64Eq);
        self.end.load(f);
        current.input_utf16_len.load(f);
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        scan.intersect(f);
        f.instruction(&Instruction::End);
    }

    fn require_canonical_empty(
        &self,
        current: &CountedMatcherLocals,
        scan: &RequiredEmptyReplayLocals,
        f: &mut Function,
    ) {
        self.start.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        self.end.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.start.load(f);
        self.end.load(f);
        f.instruction(&Instruction::I64Eq);
        self.end.load(f);
        current.input_utf16_len.load(f);
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        scan.intersect(f);
    }

    fn release(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        for local in [
            self.participating,
            self.temporary,
            self.index,
            self.count,
            self.candidates,
            self.operand1,
            self.operand0,
            self.opcode,
            self.address,
            self.pc,
            self.end,
            self.start,
            self.capture,
        ] {
            builder.runtime_schema().release_i64_local(local, f);
        }
    }
}

pub(super) fn emit_numbered(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    scan: &RequiredEmptyReplayLocals,
    f: &mut Function,
) {
    let locals = StableReferencesLocals::reserve(builder, f);
    builder.emit_regexp_instruction_load(scan.address, 8, locals.capture, f);
    locals.require_replay_empty_capture(builder, workspace, pair, current, scan, f);
    locals.release(builder, f);
}

pub(super) fn emit_named(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    scan: &RequiredEmptyReplayLocals,
    f: &mut Function,
) {
    let locals = StableReferencesLocals::reserve(builder, f);
    builder.emit_regexp_instruction_load(scan.address, 8, locals.temporary, f);
    current.named_group_table_ptr.load(f);
    f.instruction(&Instruction::I64Eqz);
    f.instruction(&Instruction::I32Eqz);
    locals.temporary.load(f);
    current.named_group_count.load(f);
    f.instruction(&Instruction::I64LtU);
    f.instruction(&Instruction::I32And);
    scan.intersect(f);
    scan.eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    current.named_records_ptr.load(f);
    locals.temporary.load(f);
    f.instruction(&Instruction::I64Const(24));
    f.instruction(&Instruction::I64Mul);
    f.instruction(&Instruction::I64Add);
    locals.address.store(f);
    builder.emit_regexp_scratch_load_word(locals.address, 8, locals.candidates, f);
    builder.emit_regexp_scratch_load_word(locals.address, 16, locals.count, f);
    locals.candidates.load(f);
    current.named_group_table_ptr.load(f);
    f.instruction(&Instruction::I64Add);
    locals.candidates.store(f);
    f.instruction(&Instruction::I64Const(0));
    locals.index.store(f);
    f.instruction(&Instruction::I64Const(0));
    locals.participating.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    locals.index.load(f);
    locals.count.load(f);
    f.instruction(&Instruction::I64GeU);
    scan.eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::BrIf(1));
    locals.candidates.load(f);
    locals.index.load(f);
    f.instruction(&Instruction::I64Const(8));
    f.instruction(&Instruction::I64Mul);
    f.instruction(&Instruction::I64Add);
    locals.address.store(f);
    builder.emit_regexp_scratch_load_word(locals.address, 0, locals.capture, f);
    locals.require_replay_empty_capture(builder, workspace, pair, current, scan, f);
    scan.eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    locals.participating.load(f);
    locals.start.load(f);
    f.instruction(&Instruction::I64Const(-1));
    f.instruction(&Instruction::I64Ne);
    locals.end.load(f);
    f.instruction(&Instruction::I64Const(-1));
    f.instruction(&Instruction::I64Ne);
    f.instruction(&Instruction::I32And);
    f.instruction(&Instruction::I64ExtendI32U);
    f.instruction(&Instruction::I64Add);
    locals.participating.store(f);
    f.instruction(&Instruction::End);
    locals.index.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    locals.index.store(f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    locals.participating.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64LeU);
    scan.intersect(f);
    f.instruction(&Instruction::End);
    locals.release(builder, f);
}
