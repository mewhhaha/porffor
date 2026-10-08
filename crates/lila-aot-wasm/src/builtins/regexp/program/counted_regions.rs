//! Linear structured-region proof before a GC byte descriptor becomes usable.
use super::*;
use crate::gc_types::{ByteArray, GcOperand, RuntimeSchema};
use lila_ir::{RegExpControlFlow, RegExpOpcode};

#[derive(Clone, Copy)]
enum RegionTable {
    Seen,
    Stack,
    Owners,
}

/// Three bounded native u32 tables in an existing GC byte array. Every exit
/// clears the owner; neither numeric bounds nor the private byte arena size it.
struct RegionScratch {
    bytes: GcLocal<ByteArray>,
    slots: I64Local,
    instructions: I64Local,
}
impl RegionScratch {
    fn new(
        view: &ValidatedRegExpProgramLayoutLocals,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) -> Self {
        let count = schema.reserve_i32_local(f);
        view.repeat_slot_count().load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let bytes = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<ByteArray>()
                .filled(GcOperand::i32(0), count, f),
            f,
        );
        schema.release_i32_local(count, f);
        Self {
            bytes,
            slots: view.repeat_slot_count(),
            instructions: view.instruction_count(),
        }
    }
    fn address(
        &self,
        table: RegionTable,
        index: I64Local,
        byte: u32,
        out: crate::gc_types::I32Local,
        f: &mut Function,
    ) {
        match table {
            RegionTable::Seen => {
                f.instruction(&Instruction::I64Const(0));
            }
            RegionTable::Stack => self.slots.load(f),
            RegionTable::Owners => {
                self.slots.load(f);
                self.instructions.load(f);
                f.instruction(&Instruction::I64Add);
            }
        }
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
        table: RegionTable,
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
        table: RegionTable,
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

fn set(out: I64Local, value: u64, f: &mut Function) {
    f.instruction(&Instruction::I64Const(value as i64));
    out.store(f);
}
fn add(out: I64Local, base: I64Local, delta: i64, f: &mut Function) {
    base.load(f);
    f.instruction(&Instruction::I64Const(delta));
    f.instruction(&Instruction::I64Add);
    out.store(f);
}

impl FunctionBuilder<'_> {
    fn read_regexp_region_instruction(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        pc: I64Local,
        offset: u64,
        out: I64Local,
        rejected: ControlTarget,
        f: &mut Function,
    ) {
        pc.load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(rejected, f);
        let address = self.runtime_schema().reserve_i64_local(f);
        view.instructions().load(f);
        pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(offset as i64));
        f.instruction(&Instruction::I64Add);
        address.store(f);
        view.read_word_in_range(address, out, self, f);
        self.runtime_schema().release_i64_local(address, f);
    }
    fn require_regexp_region_pair(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        pc: I64Local,
        opcode: RegExpOpcode,
        begin: I64Local,
        word: I64Local,
        rejected: ControlTarget,
        f: &mut Function,
    ) {
        self.read_regexp_region_instruction(view, pc, 0, word, rejected, f);
        word.load(f);
        f.instruction(&Instruction::I64Const(opcode as i64));
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, f);
        self.read_regexp_region_instruction(view, pc, 8, word, rejected, f);
        word.load(f);
        begin.load(f);
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, f);
        self.read_regexp_region_instruction(view, pc, 16, word, rejected, f);
        word.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(rejected, f);
    }
    fn require_regexp_region_successor(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        scratch: &RegionScratch,
        target: I64Local,
        expected: I64Local,
        source: RegExpOpcode,
        word: I64Local,
        rejected: ControlTarget,
        f: &mut Function,
    ) {
        target.load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(rejected, f);
        scratch.read(RegionTable::Owners, target, word, self.runtime_schema(), f);
        word.load(f);
        expected.load(f);
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, f);
        self.read_regexp_region_instruction(view, target, 0, word, rejected, f);
        if !matches!(source, RegExpOpcode::RepeatBegin | RegExpOpcode::RepeatEnd) {
            word.load(f);
            f.instruction(&Instruction::I64Const(RegExpOpcode::RepeatGuard as i64));
            f.instruction(&Instruction::I64Eq);
            self.emit_branch_if_to_target(rejected, f);
        }
        if source != RegExpOpcode::RepeatGuard {
            word.load(f);
            f.instruction(&Instruction::I64Const(RegExpOpcode::RepeatExit as i64));
            f.instruction(&Instruction::I64Eq);
            self.emit_branch_if_to_target(rejected, f);
        }
    }

    pub(super) fn emit_validate_regexp_counted_regions(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        rejected: ControlTarget,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let scratch = RegionScratch::new(view, schema, f);
        let valid = schema.reserve_i32_local(f);
        let known = schema.reserve_i32_local(f);
        let pc = schema.reserve_i64_local(f);
        let top = schema.reserve_i64_local(f);
        let owner = schema.reserve_i64_local(f);
        let begin = schema.reserve_i64_local(f);
        let end = schema.reserve_i64_local(f);
        let next = schema.reserve_i64_local(f);
        let slot = schema.reserve_i64_local(f);
        let opcode = schema.reserve_i64_local(f);
        let a = schema.reserve_i64_local(f);
        let b = schema.reserve_i64_local(f);
        let word = schema.reserve_i64_local(f);
        let begins = schema.reserve_i64_local(f);
        let expected = schema.reserve_i64_local(f);
        let target = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I32Const(0));
        valid.store(f);
        let failed = self.open_frame(ControlFrameKind::Block, f);
        for local in [pc, top, begins] {
            set(local, 0, f);
        }
        let scanned = self.open_frame(ControlFrameKind::Block, f);
        let scanning = self.open_frame(ControlFrameKind::Loop, f);
        pc.load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(scanned, f);
        set(owner, 0, f);
        top.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        add(slot, top, -1, f);
        scratch.read(RegionTable::Stack, slot, owner, schema, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        scratch.write(RegionTable::Owners, pc, owner, schema, f);
        self.read_regexp_region_instruction(view, pc, 0, opcode, failed, f);
        self.read_regexp_region_instruction(view, pc, 8, a, failed, f);
        self.read_regexp_region_instruction(view, pc, 16, b, failed, f);
        opcode.load(f);
        f.instruction(&Instruction::I64Const(RegExpOpcode::RepeatBegin as i64));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        b.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(failed, f);
        add(next, pc, 1, f);
        self.read_regexp_region_instruction(view, next, 0, word, failed, f);
        word.load(f);
        f.instruction(&Instruction::I64Const(RegExpOpcode::RepeatGuard as i64));
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(failed, f);
        self.read_regexp_region_instruction(view, next, 8, end, failed, f);
        self.read_regexp_region_instruction(view, next, 16, slot, failed, f);
        slot.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64ShrU);
        slot.store(f);
        slot.load(f);
        view.repeat_slot_count().load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(failed, f);
        slot.load(f);
        a.load(f);
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(failed, f);
        end.load(f);
        next.load(f);
        f.instruction(&Instruction::I64LeU);
        self.emit_branch_if_to_target(failed, f);
        scratch.read(RegionTable::Seen, slot, word, schema, f);
        word.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(failed, f);
        set(word, 1, f);
        scratch.write(RegionTable::Seen, slot, word, schema, f);
        owner.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.read_regexp_region_instruction(view, owner, 8, word, failed, f);
        end.load(f);
        word.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(failed, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.require_regexp_region_pair(view, end, RegExpOpcode::RepeatEnd, pc, word, failed, f);
        add(end, end, 1, f);
        self.require_regexp_region_pair(view, end, RegExpOpcode::RepeatExit, pc, word, failed, f);
        top.load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(failed, f);
        scratch.write(RegionTable::Stack, top, next, schema, f);
        add(top, top, 1, f);
        add(begins, begins, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        for control in [
            RegExpOpcode::RepeatGuard,
            RegExpOpcode::RepeatEnd,
            RegExpOpcode::RepeatExit,
        ] {
            opcode.load(f);
            f.instruction(&Instruction::I64Const(control as i64));
            f.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, f);
            owner.load(f);
            f.instruction(&Instruction::I64Eqz);
            self.emit_branch_if_to_target(failed, f);
            if control == RegExpOpcode::RepeatGuard {
                pc.load(f);
                owner.load(f);
                f.instruction(&Instruction::I64Ne);
                self.emit_branch_if_to_target(failed, f);
            } else {
                add(begin, owner, -1, f);
                self.read_regexp_region_instruction(view, owner, 8, end, failed, f);
                if control == RegExpOpcode::RepeatExit {
                    add(end, end, 1, f);
                }
                pc.load(f);
                end.load(f);
                f.instruction(&Instruction::I64Ne);
                a.load(f);
                begin.load(f);
                f.instruction(&Instruction::I64Ne);
                f.instruction(&Instruction::I32Or);
                self.emit_branch_if_to_target(failed, f);
                if control == RegExpOpcode::RepeatExit {
                    add(top, top, -1, f);
                }
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        opcode.load(f);
        f.instruction(&Instruction::I64Const(RegExpOpcode::Accept as i64));
        f.instruction(&Instruction::I64Eq);
        owner.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        self.emit_branch_if_to_target(failed, f);
        add(pc, pc, 1, f);
        self.emit_branch_to_target(scanning, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        top.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(failed, f);
        begins.load(f);
        view.repeat_slot_count().load(f);
        f.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(failed, f);

        set(pc, 0, f);
        let joined = self.open_frame(ControlFrameKind::Block, f);
        let joining = self.open_frame(ControlFrameKind::Loop, f);
        pc.load(f);
        view.instruction_count().load(f);
        f.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(joined, f);
        scratch.read(RegionTable::Owners, pc, expected, schema, f);
        self.read_regexp_region_instruction(view, pc, 0, opcode, failed, f);
        self.read_regexp_region_instruction(view, pc, 8, a, failed, f);
        self.read_regexp_region_instruction(view, pc, 16, b, failed, f);
        f.instruction(&Instruction::I32Const(0));
        known.store(f);
        for source in RegExpOpcode::ALL {
            opcode.load(f);
            f.instruction(&Instruction::I64Const(source as i64));
            f.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::I32Const(1));
            known.store(f);
            match source {
                RegExpOpcode::RepeatBegin => add(expected, pc, 1, f),
                RegExpOpcode::RepeatExit => {
                    scratch.read(RegionTable::Owners, a, expected, schema, f)
                }
                RegExpOpcode::Accept
                | RegExpOpcode::LiteralAscii
                | RegExpOpcode::PositiveAsciiClass
                | RegExpOpcode::Split
                | RegExpOpcode::Jump
                | RegExpOpcode::CaptureStart
                | RegExpOpcode::CaptureEnd
                | RegExpOpcode::ClearCaptureRange
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
                | RegExpOpcode::RepeatGuard
                | RegExpOpcode::RepeatEnd => {}
            }
            // These are the shared opcode's actual runtime successors, including
            // assertion continuation and End -> Guard, not proof summaries.
            match source.control_flow() {
                RegExpControlFlow::Accept => {}
                RegExpControlFlow::Next => {
                    add(target, pc, 1, f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
                RegExpControlFlow::Operand0 | RegExpControlFlow::Operand1 => {
                    if source.control_flow() == RegExpControlFlow::Operand0 {
                        a.load(f);
                    } else {
                        b.load(f);
                    }
                    target.store(f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
                RegExpControlFlow::BothOperands | RegExpControlFlow::ProgressSplit => {
                    a.load(f);
                    target.store(f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                    b.load(f);
                    if source.control_flow() == RegExpControlFlow::ProgressSplit {
                        f.instruction(&Instruction::I64Const(1));
                        f.instruction(&Instruction::I64ShrU);
                    }
                    target.store(f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
                RegExpControlFlow::LookaroundAfter => {
                    b.load(f);
                    f.instruction(&Instruction::I64Const(0x3fff_ffff_ffff_ffff));
                    f.instruction(&Instruction::I64And);
                    target.store(f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
                RegExpControlFlow::RepeatGuard => {
                    add(target, pc, 1, f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                    add(target, a, 1, f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
                RegExpControlFlow::RepeatEnd => {
                    add(target, a, 1, f);
                    self.require_regexp_region_successor(
                        view, &scratch, target, expected, source, word, failed, f,
                    );
                }
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        known.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(failed, f);
        add(pc, pc, 1, f);
        self.emit_branch_to_target(joining, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(1));
        valid.store(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        scratch.bytes.clear(f);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(rejected, f);
        for local in [
            target, expected, begins, word, b, a, opcode, slot, next, end, begin, owner, top, pc,
        ] {
            schema.release_i64_local(local, f);
        }
        schema.release_i32_local(known, f);
        schema.release_i32_local(valid, f);
    }
}
