use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegExpRangeBound {
    Start,
    End,
}

impl RegExpRangeBound {
    const fn offset(self) -> u64 {
        match self {
            Self::Start => 0,
            Self::End => 4,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// Pushes whether `codepoint_local` fails the canonical range-set matcher
    /// encoded by `first_entry_local` and `packed_count_local`.
    pub(super) fn emit_regexp_unicode_property_mismatch(
        &self,
        range_base_local: I64Local,
        first_entry_local: I64Local,
        packed_count_local: I64Local,
        codepoint_local: I64Local,
        range_count_local: I64Local,
        range_low_local: I64Local,
        range_high_local: I64Local,
        range_middle_local: I64Local,
        function: &mut Function,
    ) {
        // Binary search the sorted, disjoint range slice for the first range
        // whose inclusive end is not below the input code point.
        packed_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        range_count_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        range_low_local.store(function);
        range_count_local.load(function);
        range_high_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        range_low_local.load(function);
        range_high_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        range_low_local.load(function);
        range_high_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        range_middle_local.store(function);
        codepoint_local.load(function);
        self.emit_regexp_range_bound_load(
            range_base_local,
            first_entry_local,
            range_middle_local,
            RegExpRangeBound::End,
            function,
        );
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        range_middle_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        range_low_local.store(function);
        function.instruction(&Instruction::Else);
        range_middle_local.load(function);
        range_high_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        range_low_local.load(function);
        range_count_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        codepoint_local.load(function);
        self.emit_regexp_range_bound_load(
            range_base_local,
            first_entry_local,
            range_low_local,
            RegExpRangeBound::Start,
            function,
        );
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        packed_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::I64Eqz);
    }

    /// Pushes the selected inclusive bound of range-pool entry
    /// `first_entry_local + index_local` as an i64.
    fn emit_regexp_range_bound_load(
        &self,
        range_base_local: I64Local,
        first_entry_local: I64Local,
        index_local: I64Local,
        bound: RegExpRangeBound,
        function: &mut Function,
    ) {
        range_base_local.load(function);
        first_entry_local.load(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(bound.offset())));
    }
}
