use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_initialize_realm_eval_intrinsic(
        &mut self,
        realm_local: u32,
        eval_local: u32,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        let previous_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_EVAL_FUNCTION_OFFSET,
            previous_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(previous_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            intrinsics_local,
            NonArrayRealmIntrinsicSlot::EvalFunction.offset(),
            eval_local,
            function,
        );
        self.release_temp_local(previous_local);
        self.release_temp_local(intrinsics_local);
    }

    pub(crate) fn emit_load_realm_eval_intrinsic_to_local(
        &mut self,
        realm_local: u32,
        eval_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            eval_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            eval_local,
            HEAP_REALM_INTRINSICS_EVAL_FUNCTION_OFFSET,
            eval_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(eval_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }
}
