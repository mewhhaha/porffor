//! Each Check owns a newly entered canonical progress attempt. An older Run
//! cannot supply a missing frame when source control skips the current Split.
use super::*;

fn mark(
    l: &AssertionRegions,
    scratch: &RegionScratch,
    builder: &FunctionBuilder<'_>,
    f: &mut Function,
) {
    l.depth.load(f);
    f.instruction(&Instruction::I64Eqz);
    f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    f.instruction(&Instruction::I64Const(0));
    f.instruction(&Instruction::Else);
    l.first.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::End);
    l.owner.store(f);
    scratch.write(
        Table::ProgressOwners,
        l.pc,
        l.owner,
        builder.runtime_schema(),
        f,
    );
}

pub(super) fn emit_scopes(
    l: &AssertionRegions,
    scratch: &RegionScratch,
    builder: &FunctionBuilder<'_>,
    current: &CountedMatcherLocals,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    l.root_first.load(f);
    l.pc.store(f);
    l.root_end.load(f);
    l.end.store(f);
    f.instruction(&Instruction::I64Const(0));
    l.depth.store(f);
    f.instruction(&Instruction::I64Const(0));
    l.first.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    l.pc.load(f);
    l.root_end.load(f);
    f.instruction(&Instruction::I64GeU);
    l.eligible.load(f);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::BrIf(1));
    mark(l, scratch, builder, f);
    l.read(builder, current, l.pc, f);
    l.pc.load(f);
    l.end.load(f);
    f.instruction(&Instruction::I64Eq);
    l.depth.load(f);
    f.instruction(&Instruction::I64Eqz);
    f.instruction(&Instruction::I32Eqz);
    f.instruction(&Instruction::I32And);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.require_opcode(RegExpOpcode::ProgressCheck, f);
    l.a.load(f);
    l.first.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    l.first.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::I32And);
    l.intersect(f);
    l.add(l.depth, l.depth, -1, f);
    scratch.read(Table::ParentFirst, l.depth, l.first, schema, f);
    scratch.read(Table::ParentEnd, l.depth, l.end, schema, f);
    f.instruction(&Instruction::Else);
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::ProgressSplit as i64));
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.a.load(f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64Eq);
    l.intersect(f);
    l.b.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64ShrU);
    l.child_failure.store(f);
    l.child_failure.load(f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    f.instruction(&Instruction::I64GtU);
    l.child_failure.load(f);
    l.end.load(f);
    f.instruction(&Instruction::I64LeU);
    f.instruction(&Instruction::I32And);
    l.intersect(f);
    l.add(l.child_end, l.child_failure, -1, f);
    l.require_target_owner(scratch, l.child_end, builder, f);
    l.eligible.load(f);
    f.instruction(&Instruction::If(BlockType::Empty));
    l.read(builder, current, l.child_end, f);
    l.require_opcode(RegExpOpcode::ProgressCheck, f);
    l.a.load(f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    l.pc.load(f);
    f.instruction(&Instruction::I64Eq);
    l.b.load(f);
    l.child_failure.load(f);
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::I32And);
    l.intersect(f);
    scratch.write(Table::ParentFirst, l.depth, l.first, schema, f);
    scratch.write(Table::ParentEnd, l.depth, l.end, schema, f);
    l.pc.load(f);
    l.first.store(f);
    l.child_end.load(f);
    l.end.store(f);
    l.add(l.depth, l.depth, 1, f);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::Else);
    l.opcode.load(f);
    f.instruction(&Instruction::I64Const(RegExpOpcode::ProgressCheck as i64));
    f.instruction(&Instruction::I64Ne);
    l.intersect(f);
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    l.add(l.pc, l.pc, 1, f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    l.depth.load(f);
    f.instruction(&Instruction::I64Eqz);
    l.intersect(f);
    // The root's ordinary continuation is outside every nested progress scope.
    f.instruction(&Instruction::I64Const(0));
    l.owner.store(f);
    scratch.write(Table::ProgressOwners, l.root_end, l.owner, schema, f);
}
