use super::*;

pub(super) enum ArrayCopyWithinDirection {
    Forward,
    Backward,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_array_copy_within_traversal_start(
        &self,
        direction: ArrayCopyWithinDirection,
        from_local: I64Local,
        to_local: I64Local,
        count_local: I64Local,
        direction_local: I64Local,
        function: &mut Function,
    ) {
        match direction {
            ArrayCopyWithinDirection::Forward => {
                function.instruction(&Instruction::I64Const(1));
                direction_local.store(function);
            }
            ArrayCopyWithinDirection::Backward => {
                from_local.load(function);
                count_local.load(function);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                from_local.store(function);
                to_local.load(function);
                count_local.load(function);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                to_local.store(function);
                function.instruction(&Instruction::I64Const(-1));
                direction_local.store(function);
            }
        }
    }
}
