use super::*;

pub(super) enum ArrayCopyWithinDirection {
    Forward,
    Backward,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_array_copy_within_traversal_start(
        &self,
        direction: ArrayCopyWithinDirection,
        from_local: u32,
        to_local: u32,
        count_local: u32,
        direction_local: u32,
        function: &mut Function,
    ) {
        match direction {
            ArrayCopyWithinDirection::Forward => {
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(direction_local));
            }
            ArrayCopyWithinDirection::Backward => {
                function.instruction(&Instruction::LocalGet(from_local));
                function.instruction(&Instruction::LocalGet(count_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(from_local));
                function.instruction(&Instruction::LocalGet(to_local));
                function.instruction(&Instruction::LocalGet(count_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(to_local));
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::LocalSet(direction_local));
            }
        }
    }
}
