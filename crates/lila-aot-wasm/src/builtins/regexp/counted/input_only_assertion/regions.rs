//! Source-sized assertion ancestry proves discharged nested bodies without
//! recursion or a repetition-sized scratch allocation.
use super::*;
use crate::gc_types::{ByteArray, GcLocal, GcOperand, I32Local, RuntimeSchema};
use lila_ir::RegExpOpcode;
mod progress;

#[derive(Clone, Copy)]
enum Table {
    Owners,
    ParentFirst,
    ParentEnd,
    ParentDirection,
    ProgressOwners,
}

#[derive(Clone, Copy)]
enum RegionRoot {
    Assertion(I64Local),
    RequiredBody,
    GreedyAttempt { first: I64Local, end: I64Local },
}

struct RegionScratch {
    bytes: GcLocal<ByteArray>,
    count: I64Local,
}
impl RegionScratch {
    fn new(count: I64Local, schema: &RuntimeSchema, f: &mut Function) -> Self {
        let length = schema.reserve_i32_local(f);
        count.load(f);
        f.instruction(&Instruction::I64Const(20));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I32WrapI64);
        length.store(f);
        let bytes = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<ByteArray>()
                .filled(GcOperand::i32(0), length, f),
            f,
        );
        schema.release_i32_local(length, f);
        Self { bytes, count }
    }
    fn address(&self, table: Table, index: I64Local, byte: u32, out: I32Local, f: &mut Function) {
        let ordinal = match table {
            Table::Owners => 0,
            Table::ParentFirst => 1,
            Table::ParentEnd => 2,
            Table::ParentDirection => 3,
            Table::ProgressOwners => 4,
        };
        self.count.load(f);
        f.instruction(&Instruction::I64Const(ordinal));
        f.instruction(&Instruction::I64Mul);
        index.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Const(byte as i64));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        out.store(f);
    }
    fn read(
        &self,
        table: Table,
        index: I64Local,
        out: I64Local,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        let at = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        out.store(f);
        for n in 0..4 {
            self.address(table, index, n, at, f);
            schema
                .array_type::<ByteArray>()
                .read(&self.bytes, at, schema, f)
                .store(byte, f);
            out.load(f);
            byte.load(f);
            f.instruction(&Instruction::I64ExtendI32U);
            f.instruction(&Instruction::I64Const(n as i64 * 8));
            f.instruction(&Instruction::I64Shl);
            f.instruction(&Instruction::I64Or);
            out.store(f);
        }
        schema.release_i32_local(byte, f);
        schema.release_i32_local(at, f);
    }
    fn write(
        &self,
        table: Table,
        index: I64Local,
        value: I64Local,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        let at = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        for n in 0..4 {
            self.address(table, index, n, at, f);
            value.load(f);
            f.instruction(&Instruction::I64Const(n as i64 * 8));
            f.instruction(&Instruction::I64ShrU);
            f.instruction(&Instruction::I32WrapI64);
            byte.store(f);
            schema.array_type::<ByteArray>().write(
                &self.bytes,
                at,
                GcOperand::i32_local(byte),
                schema,
                f,
            );
        }
        schema.release_i32_local(byte, f);
        schema.release_i32_local(at, f);
    }
}

struct AssertionRegions {
    eligible: I32Local,
    admitted: I32Local,
    pc: I64Local,
    address: I64Local,
    opcode: I64Local,
    a: I64Local,
    b: I64Local,
    first: I64Local,
    end: I64Local,
    direction: I64Local,
    depth: I64Local,
    root_first: I64Local,
    root_end: I64Local,
    root_failure: I64Local,
    child_first: I64Local,
    child_end: I64Local,
    child_failure: I64Local,
    child_direction: I64Local,
    flags: I64Local,
    target: I64Local,
    owner: I64Local,
    other_owner: I64Local,
}
impl AssertionRegions {
    fn intersect(&self, f: &mut Function) {
        self.eligible.load(f);
        f.instruction(&Instruction::I32And);
        self.eligible.store(f);
    }
    fn read(
        &self,
        builder: &FunctionBuilder<'_>,
        current: &CountedMatcherLocals,
        pc: I64Local,
        f: &mut Function,
    ) {
        current.program.load(f);
        pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.address.store(f);
        for (offset, output) in [(0, self.opcode), (8, self.a), (16, self.b)] {
            builder.emit_regexp_instruction_load(self.address, offset, output, f);
        }
    }
    fn require_opcode(&self, opcode: RegExpOpcode, f: &mut Function) {
        self.opcode.load(f);
        f.instruction(&Instruction::I64Const(opcode as i64));
        f.instruction(&Instruction::I64Eq);
        self.intersect(f);
    }
    fn add(&self, output: I64Local, input: I64Local, delta: i64, f: &mut Function) {
        input.load(f);
        f.instruction(&Instruction::I64Const(delta));
        f.instruction(&Instruction::I64Add);
        output.store(f);
    }
    /// Every read follows the same checked canonical source extent. Children
    /// restore their actual parent's direction on both polarity exits.
    fn header(
        &self,
        builder: &FunctionBuilder<'_>,
        current: &CountedMatcherLocals,
        source: I64Local,
        limit: I64Local,
        parent_direction: I64Local,
        f: &mut Function,
    ) {
        source.load(f);
        f.instruction(&Instruction::I64Const(3));
        f.instruction(&Instruction::I64Add);
        limit.load(f);
        f.instruction(&Instruction::I64LtU);
        self.intersect(f);
        self.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.read(builder, current, source, f);
        self.require_opcode(RegExpOpcode::LookaroundStart, f);
        self.a.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64LeU);
        self.b.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32And);
        self.intersect(f);
        self.a.load(f);
        self.child_direction.store(f);
        self.add(self.target, source, 1, f);
        self.read(builder, current, self.target, f);
        self.require_opcode(RegExpOpcode::Split, f);
        self.add(self.child_first, source, 2, f);
        self.a.load(f);
        self.child_first.load(f);
        f.instruction(&Instruction::I64Eq);
        self.b.load(f);
        self.child_first.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32And);
        self.b.load(f);
        limit.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::I32And);
        self.intersect(f);
        self.b.load(f);
        self.child_failure.store(f);
        self.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.read(builder, current, self.child_failure, f);
        self.require_opcode(RegExpOpcode::LookaroundFailure, f);
        self.a.load(f);
        self.child_failure.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Eq);
        self.b.load(f);
        f.instruction(&Instruction::I64Const(3));
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        self.b.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64ShrU);
        parent_direction.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.intersect(f);
        self.b.load(f);
        self.flags.store(f);
        self.add(self.child_end, self.child_failure, -1, f);
        self.read(builder, current, self.child_end, f);
        self.require_opcode(RegExpOpcode::LookaroundEnd, f);
        self.a.load(f);
        self.child_failure.load(f);
        f.instruction(&Instruction::I64Eq);
        self.b.load(f);
        f.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
        f.instruction(&Instruction::I64And);
        self.child_failure.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.b.load(f);
        f.instruction(&Instruction::I64Const(63));
        f.instruction(&Instruction::I64ShrU);
        self.flags.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.b.load(f);
        f.instruction(&Instruction::I64Const(62));
        f.instruction(&Instruction::I64ShrU);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64And);
        parent_direction.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        self.intersect(f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
    }
    fn mark(
        &self,
        scratch: &RegionScratch,
        pc: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.add(self.owner, self.first, -1, f);
        scratch.write(Table::Owners, pc, self.owner, builder.runtime_schema(), f);
    }
    fn require_target_owner(
        &self,
        scratch: &RegionScratch,
        target: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        target.load(f);
        self.root_first.load(f);
        f.instruction(&Instruction::I64GeU);
        target.load(f);
        self.root_end.load(f);
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        self.intersect(f);
        self.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        scratch.read(
            Table::Owners,
            self.pc,
            self.owner,
            builder.runtime_schema(),
            f,
        );
        scratch.read(
            Table::Owners,
            target,
            self.other_owner,
            builder.runtime_schema(),
            f,
        );
        self.owner.load(f);
        self.other_owner.load(f);
        f.instruction(&Instruction::I64Eq);
        self.intersect(f);
        f.instruction(&Instruction::End);
    }
    fn require_progress_owner(
        &self,
        scratch: &RegionScratch,
        target: I64Local,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        self.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        scratch.read(
            Table::ProgressOwners,
            self.pc,
            self.owner,
            builder.runtime_schema(),
            f,
        );
        scratch.read(
            Table::ProgressOwners,
            target,
            self.other_owner,
            builder.runtime_schema(),
            f,
        );
        self.owner.load(f);
        self.other_owner.load(f);
        f.instruction(&Instruction::I64Eq);
        self.intersect(f);
        f.instruction(&Instruction::End);
    }
}

pub(super) fn emit_closed_assertion(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    source: I64Local,
    eligible: I32Local,
    evidence: AssertionStateEvidence,
    f: &mut Function,
) {
    emit_regions(
        builder,
        workspace,
        pair,
        current,
        RegionRoot::Assertion(source),
        eligible,
        evidence,
        f,
    );
}

pub(super) fn emit_required_body(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    eligible: I32Local,
    f: &mut Function,
) {
    emit_regions(
        builder,
        workspace,
        pair,
        current,
        RegionRoot::RequiredBody,
        eligible,
        AssertionStateEvidence::CompleteChoiceSnapshots,
        f,
    );
}

pub(super) fn emit_independent_body(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    eligible: I32Local,
    evidence: AssertionStateEvidence,
    f: &mut Function,
) {
    emit_regions(
        builder,
        workspace,
        pair,
        current,
        RegionRoot::RequiredBody,
        eligible,
        evidence,
        f,
    );
}

pub(super) fn emit_greedy_region(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    first: I64Local,
    end: I64Local,
    eligible: I32Local,
    f: &mut Function,
) {
    emit_regions(
        builder,
        workspace,
        pair,
        current,
        RegionRoot::GreedyAttempt { first, end },
        eligible,
        AssertionStateEvidence::InputOnly,
        f,
    );
}

fn emit_regions(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    root: RegionRoot,
    eligible: I32Local,
    evidence: AssertionStateEvidence,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let l = AssertionRegions {
        eligible,
        admitted: schema.reserve_i32_local(f),
        pc: schema.reserve_i64_local(f),
        address: schema.reserve_i64_local(f),
        opcode: schema.reserve_i64_local(f),
        a: schema.reserve_i64_local(f),
        b: schema.reserve_i64_local(f),
        first: schema.reserve_i64_local(f),
        end: schema.reserve_i64_local(f),
        direction: schema.reserve_i64_local(f),
        depth: schema.reserve_i64_local(f),
        root_first: schema.reserve_i64_local(f),
        root_end: schema.reserve_i64_local(f),
        root_failure: schema.reserve_i64_local(f),
        child_first: schema.reserve_i64_local(f),
        child_end: schema.reserve_i64_local(f),
        child_failure: schema.reserve_i64_local(f),
        child_direction: schema.reserve_i64_local(f),
        flags: schema.reserve_i64_local(f),
        target: schema.reserve_i64_local(f),
        owner: schema.reserve_i64_local(f),
        other_owner: schema.reserve_i64_local(f),
    };
    match root {
        RegionRoot::Assertion(source) => {
            l.header(builder, current, source, pair.end, current.reverse_mode, f)
        }
        RegionRoot::RequiredBody => {
            l.add(l.child_first, pair.guard, 1, f);
            pair.end.load(f);
            l.child_end.store(f);
            current.reverse_mode.load(f);
            l.child_direction.store(f);
            pair.end.load(f);
            l.child_failure.store(f);
        }
        RegionRoot::GreedyAttempt { first, end } => {
            first.load(f);
            l.child_first.store(f);
            end.load(f);
            l.child_end.store(f);
            current.reverse_mode.load(f);
            l.child_direction.store(f);
            end.load(f);
            l.child_failure.store(f);
        }
    }
    eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    let scratch = RegionScratch::new(current.instructions, schema, f);
    for (input, output) in [
        (l.child_first, l.first),
        (l.child_first, l.root_first),
        (l.child_end, l.end),
        (l.child_end, l.root_end),
        (l.child_failure, l.root_failure),
        (l.child_direction, l.direction),
        (l.child_first, l.pc),
    ] {
        input.load(f);
        output.store(f);
    }
    f.instruction(&Instruction::I64Const(0));
    l.depth.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::BrIf(1));
    l.pc.load(f);
    l.end.load(f);
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.mark(&scratch, l.end, builder, f);
    l.depth.load(f);
    f.instruction(&Instruction::I64Eqz);
    f.instruction(&Instruction::BrIf(2));
    l.add(l.target, l.end, 1, f);
    l.mark(&scratch, l.target, builder, f);
    l.add(l.pc, l.end, 2, f);
    l.add(l.depth, l.depth, -1, f);
    scratch.read(Table::ParentFirst, l.depth, l.first, schema, f);
    scratch.read(Table::ParentEnd, l.depth, l.end, schema, f);
    scratch.read(Table::ParentDirection, l.depth, l.direction, schema, f);
    f.instruction(&Instruction::Br(1));
    f.instruction(&Instruction::End);
    l.mark(&scratch, l.pc, builder, f);
    l.read(builder, current, l.pc, f);
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::LookaroundStart as i64));
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.header(builder, current, l.pc, l.end, l.direction, f);
    eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    scratch.write(Table::ParentFirst, l.depth, l.first, schema, f);
    scratch.write(Table::ParentEnd, l.depth, l.end, schema, f);
    scratch.write(Table::ParentDirection, l.depth, l.direction, schema, f);
    l.add(l.depth, l.depth, 1, f);
    for (input, output) in [
        (l.child_first, l.first),
        (l.child_end, l.end),
        (l.child_direction, l.direction),
    ] {
        input.load(f);
        output.store(f);
    }
    l.add(l.target, l.pc, 1, f);
    l.mark(&scratch, l.target, builder, f);
    l.first.load(f);
    l.pc.store(f);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::Br(1));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::I32Const(0));
    l.admitted.store(f);
    for opcode in RegExpOpcode::ALL {
        match opcode {
            RegExpOpcode::LiteralAscii
            | RegExpOpcode::PositiveAsciiClass
            | RegExpOpcode::Whitespace
            | RegExpOpcode::Dot
            | RegExpOpcode::LiteralCodePoint
            | RegExpOpcode::UnicodeProperty
            | RegExpOpcode::NegativeAsciiClass
            | RegExpOpcode::NotWhitespace
            | RegExpOpcode::AssertStart
            | RegExpOpcode::AssertEnd
            | RegExpOpcode::WordBoundary
            | RegExpOpcode::Split
            | RegExpOpcode::Jump
            | RegExpOpcode::ProgressSplit
            | RegExpOpcode::ProgressCheck
            | RegExpOpcode::RepeatBegin
            | RegExpOpcode::RepeatGuard
            | RegExpOpcode::RepeatEnd
            | RegExpOpcode::RepeatExit => {}
            RegExpOpcode::CaptureStart
            | RegExpOpcode::CaptureEnd
            | RegExpOpcode::ClearCaptureRange
            | RegExpOpcode::NumberedBackreference
            | RegExpOpcode::NamedBackreference => match evidence {
                AssertionStateEvidence::CompleteChoiceSnapshots
                | AssertionStateEvidence::IndependentIterations { .. } => {}
                AssertionStateEvidence::InputOnly => continue,
            },
            RegExpOpcode::Accept
            | RegExpOpcode::LookaroundStart
            | RegExpOpcode::LookaroundEnd
            | RegExpOpcode::LookaroundFailure => continue,
        }
        l.opcode.load(f);
        f.instruction(&Instruction::I64Const(opcode as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I32Const(1));
        l.admitted.store(f);
        if let AssertionStateEvidence::IndependentIterations {
            clear_first,
            clear_end,
            zero_width,
        } = evidence
        {
            match opcode {
                // Assertion execution restores all three cursor words before
                // its continuation. At body depth zero, successful consumption
                // instead advances in the unchanged parent direction.
                RegExpOpcode::LiteralAscii
                | RegExpOpcode::PositiveAsciiClass
                | RegExpOpcode::Whitespace
                | RegExpOpcode::Dot
                | RegExpOpcode::LiteralCodePoint
                | RegExpOpcode::UnicodeProperty
                | RegExpOpcode::NegativeAsciiClass
                | RegExpOpcode::NotWhitespace
                | RegExpOpcode::NumberedBackreference
                | RegExpOpcode::NamedBackreference => {
                    l.depth.load(f);
                    f.instruction(&Instruction::I64Eqz);
                    f.instruction(&Instruction::If(BlockType::Empty));
                    f.instruction(&Instruction::I32Const(0));
                    zero_width.store(f);
                    f.instruction(&Instruction::End);
                }
                RegExpOpcode::CaptureStart | RegExpOpcode::CaptureEnd => {
                    l.a.load(f);
                    clear_first.load(f);
                    f.instruction(&Instruction::I64GeU);
                    l.a.load(f);
                    clear_end.load(f);
                    f.instruction(&Instruction::I64LtU);
                    f.instruction(&Instruction::I32And);
                    l.intersect(f);
                }
                RegExpOpcode::ClearCaptureRange => {
                    l.a.load(f);
                    clear_first.load(f);
                    f.instruction(&Instruction::I64GeU);
                    l.b.load(f);
                    clear_end.load(f);
                    f.instruction(&Instruction::I64LeU);
                    f.instruction(&Instruction::I32And);
                    l.intersect(f);
                }
                RegExpOpcode::AssertStart
                | RegExpOpcode::AssertEnd
                | RegExpOpcode::WordBoundary
                | RegExpOpcode::Split
                | RegExpOpcode::Jump
                | RegExpOpcode::ProgressSplit
                | RegExpOpcode::ProgressCheck
                | RegExpOpcode::RepeatBegin
                | RegExpOpcode::RepeatGuard
                | RegExpOpcode::RepeatEnd
                | RegExpOpcode::RepeatExit => {}
                RegExpOpcode::Accept
                | RegExpOpcode::LookaroundStart
                | RegExpOpcode::LookaroundEnd
                | RegExpOpcode::LookaroundFailure => {
                    unreachable!("closed assertion structure has its own admission")
                }
            }
        }
        if opcode == RegExpOpcode::RepeatBegin {
            let child = CountedLoopLocals::admit_at_begin(builder, workspace, current, l.pc, f);
            child.end.load(f);
            l.end.load(f);
            f.instruction(&Instruction::I64LtU);
            child.slot.load(f);
            pair.slot.load(f);
            f.instruction(&Instruction::I64Ne);
            f.instruction(&Instruction::I32And);
            builder.emit_regexp_scratch_load_word(
                child.state,
                RegExpRepeatStateWord::Active.offset(),
                child.temporary,
                f,
            );
            child.temporary.load(f);
            f.instruction(&Instruction::I64Eqz);
            f.instruction(&Instruction::I32And);
            l.intersect(f);
            child
                .minimum
                .validate_canonical(builder, workspace, child.state, f);
            child
                .maximum
                .validate_canonical(builder, workspace, child.state, f);
            child.release(builder, f);
        }
        f.instruction(&Instruction::End);
    }
    l.admitted.load(f);
    l.intersect(f);
    l.add(l.pc, l.pc, 1, f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    progress::emit_scopes(&l, &scratch, builder, current, f);
    // Every ordinary/progress edge stays in its exact assertion owner. A
    // target inside an enclosing extent alone is insufficient for nesting.
    l.root_first.load(f);
    l.pc.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    l.pc.load(f);
    l.root_end.load(f);
    f.instruction(&Instruction::I64GeU);
    eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::BrIf(1));
    l.read(builder, current, l.pc, f);
    for opcode in [
        RegExpOpcode::Split,
        RegExpOpcode::Jump,
        RegExpOpcode::ProgressSplit,
        RegExpOpcode::ProgressCheck,
    ] {
        l.opcode.load(f);
        f.instruction(&Instruction::I64Const(opcode as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        l.require_target_owner(&scratch, l.a, builder, f);
        if opcode == RegExpOpcode::ProgressSplit {
            l.b.load(f);
            f.instruction(&Instruction::I64Const(1));
            f.instruction(&Instruction::I64ShrU);
            l.b.store(f);
        }
        if opcode != RegExpOpcode::Jump {
            l.require_target_owner(&scratch, l.b, builder, f);
        }
        match opcode {
            RegExpOpcode::Split | RegExpOpcode::Jump => {
                l.require_progress_owner(&scratch, l.a, builder, f);
                if opcode == RegExpOpcode::Split {
                    l.require_progress_owner(&scratch, l.b, builder, f);
                }
            }
            RegExpOpcode::ProgressSplit => {
                l.require_progress_owner(&scratch, l.b, builder, f);
            }
            RegExpOpcode::ProgressCheck => {}
            _ => {
                unreachable!("the closed source-edge list contains only choice/jump/progress words")
            }
        }
        f.instruction(&Instruction::End);
    }
    l.add(l.pc, l.pc, 1, f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    if let RegionRoot::Assertion(source) = root {
        l.root_failure.load(f);
        source.store(f);
    }
    f.instruction(&Instruction::End);
    scratch.bytes.clear(f);
    f.instruction(&Instruction::End);
    schema.release_i32_local(l.admitted, f);
    for local in [
        l.other_owner,
        l.owner,
        l.target,
        l.flags,
        l.child_direction,
        l.child_failure,
        l.child_end,
        l.child_first,
        l.root_failure,
        l.root_end,
        l.root_first,
        l.depth,
        l.direction,
        l.end,
        l.first,
        l.b,
        l.a,
        l.opcode,
        l.address,
        l.pc,
    ] {
        schema.release_i64_local(local, f);
    }
}
