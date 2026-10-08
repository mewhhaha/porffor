//! A closed greedy attempt which completed without progress leaves no choice.
//! The source proof owns the complete attempt and its actual fallback/check.
use super::*;

struct GreedyAttempt {
    pc: I64Local,
    address: I64Local,
    opcode: I64Local,
    a: I64Local,
    b: I64Local,
    begin: I64Local,
    fallback: I64Local,
    end: I64Local,
    progress: I32Local,
    consuming: I32Local,
    admitted: I32Local,
}

impl GreedyAttempt {
    fn read(
        &self,
        builder: &FunctionBuilder<'_>,
        current: &CountedMatcherLocals,
        f: &mut Function,
    ) {
        current.program.load(f);
        self.pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.address.store(f);
        for (offset, output) in [(0, self.opcode), (8, self.a), (16, self.b)] {
            builder.emit_regexp_instruction_load(self.address, offset, output, f);
        }
    }
    fn intersect(&self, scan: &RequiredEmptyReplayLocals, f: &mut Function) {
        scan.eligible.load(f);
        f.instruction(&Instruction::I32And);
        scan.eligible.store(f);
    }
}

pub(super) fn emit_completed_greedy_attempt(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    scan: &RequiredEmptyReplayLocals,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let l = GreedyAttempt {
        pc: schema.reserve_i64_local(f),
        address: schema.reserve_i64_local(f),
        opcode: schema.reserve_i64_local(f),
        a: schema.reserve_i64_local(f),
        b: schema.reserve_i64_local(f),
        begin: schema.reserve_i64_local(f),
        fallback: schema.reserve_i64_local(f),
        end: schema.reserve_i64_local(f),
        progress: schema.reserve_i32_local(f),
        consuming: schema.reserve_i32_local(f),
        admitted: schema.reserve_i32_local(f),
    };
    scan.pc.load(f);
    l.begin.store(f);
    scan.pc.load(f);
    l.pc.store(f);
    l.read(builder, current, f);
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::ProgressSplit as i64));
    f.instruction(&Instruction::I64Eq);
    l.progress.store(f);
    l.a.load(f);
    l.begin.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64Eq);
    l.intersect(scan, f);
    l.progress.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.b.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64And);
    f.instruction(&Instruction::I64Eqz);
    l.intersect(scan, f);
    l.b.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64ShrU);
    l.b.store(f);
    f.instruction(&Instruction::End);
    l.b.load(f);
    l.fallback.store(f);
    l.fallback.load(f);
    l.begin.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64GtU);
    l.fallback.load(f);
    pair.end.load(f);
    f.instruction(&Instruction::I64LeU);
    f.instruction(&Instruction::I32And);
    l.intersect(scan, f);
    scan.eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.fallback.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Sub);
    l.pc.store(f);
    l.read(builder, current, f);
    l.progress.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::ProgressCheck as i64));
    f.instruction(&Instruction::I64Eq);
    l.a.load(f);
    l.begin.load(f);
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::I32And);
    l.b.load(f);
    l.begin.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    l.fallback.load(f);
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::I32And);
    l.intersect(scan, f);
    l.pc.load(f);
    l.end.store(f);
    f.instruction(&Instruction::Else);
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::Jump as i64));
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.a.load(f);
    l.begin.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    f.instruction(&Instruction::I64Eqz);
    f.instruction(&Instruction::I32And);
    l.intersect(scan, f);
    l.pc.load(f);
    l.end.store(f);
    f.instruction(&Instruction::Else);
    l.fallback.load(f);
    l.end.store(f);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    l.begin.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    l.pc.store(f);
    f.instruction(&Instruction::I32Const(0));
    l.consuming.store(f);
    l.progress.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    // The outer greedy no-progress owner discharges every internal choice.
    // Its atom may therefore use closed pure nested alternatives/progress;
    // all exits must still reach this exact terminal Check, and all nested
    // assertion directions and counted row lifetimes remain checked.
    input_only_assertion::emit_input_only_greedy_region(
        builder,
        workspace,
        pair,
        current,
        l.pc,
        l.end,
        scan.eligible,
        f,
    );
    f.instruction(&Instruction::Else);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    l.pc.load(f);
    l.end.load(f);
    f.instruction(&Instruction::I64GeU);
    scan.eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::BrIf(1));
    l.read(builder, current, f);
    f.instruction(&Instruction::I32Const(0));
    l.admitted.store(f);
    for opcode in RegExpOpcode::ALL {
        let consumes = match opcode {
            RegExpOpcode::LiteralAscii
            | RegExpOpcode::PositiveAsciiClass
            | RegExpOpcode::Whitespace
            | RegExpOpcode::Dot
            | RegExpOpcode::LiteralCodePoint
            | RegExpOpcode::UnicodeProperty
            | RegExpOpcode::NegativeAsciiClass
            | RegExpOpcode::NotWhitespace => true,
            RegExpOpcode::AssertStart
            | RegExpOpcode::AssertEnd
            | RegExpOpcode::WordBoundary
            | RegExpOpcode::LookaroundStart => false,
            RegExpOpcode::Accept
            | RegExpOpcode::Split
            | RegExpOpcode::Jump
            | RegExpOpcode::CaptureStart
            | RegExpOpcode::CaptureEnd
            | RegExpOpcode::ClearCaptureRange
            | RegExpOpcode::NumberedBackreference
            | RegExpOpcode::NamedBackreference
            | RegExpOpcode::LookaroundEnd
            | RegExpOpcode::LookaroundFailure
            | RegExpOpcode::ProgressSplit
            | RegExpOpcode::ProgressCheck
            | RegExpOpcode::RepeatBegin
            | RegExpOpcode::RepeatGuard
            | RegExpOpcode::RepeatEnd
            | RegExpOpcode::RepeatExit => continue,
        };
        l.opcode.load(f);
        f.instruction(&Instruction::I64Const(opcode as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I32Const(1));
        l.admitted.store(f);
        if consumes {
            f.instruction(&Instruction::I32Const(1));
            l.consuming.store(f);
        }
        if opcode == RegExpOpcode::LookaroundStart {
            input_only_assertion::emit_input_only_assertion(
                builder,
                workspace,
                pair,
                current,
                l.pc,
                scan.eligible,
                f,
            );
            // Its actual Failure must precede this attempt's check/fallback;
            // a containing assertion cannot disguise a surviving greedy frame.
            l.pc.load(f);
            l.end.load(f);
            f.instruction(&Instruction::I64LtU);
            l.intersect(scan, f);
        }
        f.instruction(&Instruction::End);
    }
    l.admitted.load(f);
    l.intersect(scan, f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    l.pc.store(f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    // Ordinary optional/star attempts must consume on their only success path.
    // Nullable attempts instead have the checked greedy no-progress owner.
    l.consuming.load(f);
    l.progress.load(f);
    f.instruction(&Instruction::I32Or);
    l.intersect(scan, f);
    scan.eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.fallback.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Sub);
    scan.pc.store(f);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    for local in [l.admitted, l.consuming, l.progress] {
        schema.release_i32_local(local, f);
    }
    for local in [
        l.end, l.fallback, l.begin, l.b, l.a, l.opcode, l.address, l.pc,
    ] {
        schema.release_i64_local(local, f);
    }
}
