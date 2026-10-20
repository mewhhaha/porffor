//! A cursor bound is a machine word; its comparison and translation still read
//! every source-sized counter limb. No huge decimal is narrowed to that word.
use super::*;

impl NaturalCounterLocals<MinimumCounter> {
    pub(in crate::builtins::regexp::counted) fn greater_than_bounded(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        bound: I64Local,
        f: &mut Function,
    ) {
        self.validate_canonical(builder, workspace, state, f);
        let schema = builder.runtime_schema();
        let remaining = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let digit = schema.reserve_i64_local(f);
        let comparison = schema.reserve_i64_local(f);
        bound.load(f);
        remaining.store(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I64Const(0));
        comparison.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.load_limb(index, address, value, f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64RemU);
        digit.store(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64DivU);
        remaining.store(f);
        value.load(f);
        digit.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::If(BlockType::Empty));
        value.load(f);
        digit.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::End);
        comparison.store(f);
        f.instruction(&Instruction::End);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        remaining.load(f);
        f.instruction(&Instruction::I64Eqz);
        comparison.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        for local in [comparison, digit, value, address, index, remaining] {
            schema.release_i64_local(local, f);
        }
    }

    pub(in crate::builtins::regexp::counted) fn subtract_bounded(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        bound: I64Local,
        f: &mut Function,
    ) {
        self.greater_than_bounded(builder, workspace, state, bound, f);
        f.instruction(&Instruction::I32Eqz);
        CountedLoopLocals::reject(workspace, builder, f);
        let schema = builder.runtime_schema();
        let remaining = schema.reserve_i64_local(f);
        let used = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let probe = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let digit = schema.reserve_i64_local(f);
        let borrow = schema.reserve_i64_local(f);
        self.load_used(builder, state, used, f);
        bound.load(f);
        remaining.store(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I64Const(0));
        borrow.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        used.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.load_limb(index, address, value, f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64RemU);
        borrow.load(f);
        f.instruction(&Instruction::I64Add);
        digit.store(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64DivU);
        remaining.store(f);
        value.load(f);
        digit.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        value.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64Add);
        digit.load(f);
        f.instruction(&Instruction::I64Sub);
        value.store(f);
        f.instruction(&Instruction::I64Const(1));
        borrow.store(f);
        f.instruction(&Instruction::Else);
        value.load(f);
        digit.load(f);
        f.instruction(&Instruction::I64Sub);
        value.store(f);
        f.instruction(&Instruction::I64Const(0));
        borrow.store(f);
        f.instruction(&Instruction::End);
        Self::store_limb(address, value, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        remaining.load(f);
        borrow.load(f);
        f.instruction(&Instruction::I64Or);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        CountedLoopLocals::reject(workspace, builder, f);
        used.load(f);
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        probe.store(f);
        self.load_limb(probe, address, value, f);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(1));
        probe.load(f);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.store_used(builder, state, index, f);
        for local in [borrow, digit, value, address, probe, index, used, remaining] {
            schema.release_i64_local(local, f);
        }
    }

    pub(in crate::builtins::regexp::counted) fn retain_bounded(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        bound: I64Local,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let remaining = schema.reserve_i64_local(f);
        let used = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        bound.load(f);
        f.instruction(&Instruction::I64Eqz);
        CountedLoopLocals::reject(workspace, builder, f);
        bound.load(f);
        remaining.store(f);
        f.instruction(&Instruction::I64Const(0));
        used.store(f);
        // Prove capacity before clearing the original source-sized row.
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        remaining.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        used.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        used.store(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64DivU);
        remaining.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        used.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GtU);
        CountedLoopLocals::reject(workspace, builder, f);
        self.clear(builder, state, f);
        bound.load(f);
        remaining.store(f);
        f.instruction(&Instruction::I64Const(0));
        used.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        remaining.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64RemU);
        value.store(f);
        self.limb_address(used, address, f);
        Self::store_limb(address, value, f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64DivU);
        remaining.store(f);
        used.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        used.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.store_used(builder, state, used, f);
        for local in [value, address, used, remaining] {
            schema.release_i64_local(local, f);
        }
    }
}
