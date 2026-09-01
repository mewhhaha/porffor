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
        range_base_local: u32,
        first_entry_local: u32,
        packed_count_local: u32,
        codepoint_local: u32,
        range_count_local: u32,
        range_low_local: u32,
        range_high_local: u32,
        range_middle_local: u32,
        function: &mut Function,
    ) {
        // Binary search the sorted, disjoint range slice for the first range
        // whose inclusive end is not below the input code point.
        function.instruction(&Instruction::LocalGet(packed_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(range_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(range_low_local));
        function.instruction(&Instruction::LocalGet(range_count_local));
        function.instruction(&Instruction::LocalSet(range_high_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(range_low_local));
        function.instruction(&Instruction::LocalGet(range_high_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(range_low_local));
        function.instruction(&Instruction::LocalGet(range_high_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(range_middle_local));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        self.emit_regexp_range_bound_load(
            range_base_local,
            first_entry_local,
            range_middle_local,
            RegExpRangeBound::End,
            function,
        );
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(range_middle_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(range_low_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(range_middle_local));
        function.instruction(&Instruction::LocalSet(range_high_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(range_low_local));
        function.instruction(&Instruction::LocalGet(range_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(codepoint_local));
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
        function.instruction(&Instruction::LocalGet(packed_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::I64Eqz);
    }

    /// Pushes the selected inclusive bound of range-pool entry
    /// `first_entry_local + index_local` as an i64.
    fn emit_regexp_range_bound_load(
        &self,
        range_base_local: u32,
        first_entry_local: u32,
        index_local: u32,
        bound: RegExpRangeBound,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(range_base_local));
        function.instruction(&Instruction::LocalGet(first_entry_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(bound.offset())));
    }
}
