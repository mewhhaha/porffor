//! A copied Run must retain an actual nested source pair. Its private arena
//! reader checks ancestor identity; this join checks the original source row.
use super::*;
use crate::builtins::regexp::matcher_workspace::CheckedTemplateRunNode;

pub(super) fn emit_source_node(
    node: &CheckedTemplateRunNode<'_>,
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    eligible: I32Local,
    f: &mut Function,
) {
    let schema = builder.runtime_schema();
    let pc = schema.reserve_i64_local(f);
    let address = schema.reserve_i64_local(f);
    let opcode = schema.reserve_i64_local(f);
    let row = schema.reserve_i64_local(f);
    let minimum_capacity = schema.reserve_i64_local(f);
    let maximum_capacity = schema.reserve_i64_local(f);
    let maximum_kind = schema.reserve_i64_local(f);
    let matched = schema.reserve_i32_local(f);
    node.row_offset(row, f);
    node.minimum_capacity(minimum_capacity, f);
    node.maximum_capacity(maximum_capacity, f);
    node.maximum_kind(maximum_kind, f);
    f.instruction(&Instruction::I32Const(0));
    matched.store(f);
    pair.guard.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    pc.store(f);
    f.instruction(&Instruction::Block(BlockType::Empty));
    f.instruction(&Instruction::Loop(BlockType::Empty));
    pc.load(f);
    pair.end.load(f);
    f.instruction(&Instruction::I64GeU);
    matched.load(f);
    f.instruction(&Instruction::I32Or);
    f.instruction(&Instruction::BrIf(1));
    current.program.load(f);
    pc.load(f);
    f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
    f.instruction(&Instruction::I64Mul);
    f.instruction(&Instruction::I64Add);
    address.store(f);
    builder.emit_regexp_instruction_load(address, 0, opcode, f);
    opcode.load(f);
    f.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_BEGIN as i64));
    f.instruction(&Instruction::I64Eq);
    f.instruction(&Instruction::If(BlockType::Empty));
    let child = CountedLoopLocals::admit_at_begin(builder, workspace, current, pc, f);
    child.state.load(f);
    workspace.base.load(f);
    f.instruction(&Instruction::I64Sub);
    row.load(f);
    f.instruction(&Instruction::I64Eq);
    child.slot.load(f);
    pair.slot.load(f);
    f.instruction(&Instruction::I64Ne);
    f.instruction(&Instruction::I32And);
    child.end.load(f);
    pair.end.load(f);
    f.instruction(&Instruction::I64LtU);
    f.instruction(&Instruction::I32And);
    for (actual, selected) in [
        (child.minimum.capacity(), minimum_capacity),
        (child.maximum.capacity(), maximum_capacity),
        (child.maximum_kind, maximum_kind),
    ] {
        actual.load(f);
        selected.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
    }
    matched.store(f);
    child.release(builder, f);
    f.instruction(&Instruction::End);
    pc.load(f);
    f.instruction(&Instruction::I64Const(1));
    f.instruction(&Instruction::I64Add);
    pc.store(f);
    f.instruction(&Instruction::Br(0));
    f.instruction(&Instruction::End);
    f.instruction(&Instruction::End);
    matched.load(f);
    eligible.load(f);
    f.instruction(&Instruction::I32And);
    eligible.store(f);
    schema.release_i32_local(matched, f);
    for local in [
        maximum_kind,
        maximum_capacity,
        minimum_capacity,
        row,
        opcode,
        address,
        pc,
    ] {
        schema.release_i64_local(local, f);
    }
}
