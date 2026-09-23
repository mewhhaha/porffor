use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Temporal method results belong to the active builtin's Realm, even
    /// when the receiver, arguments and mutable constructor properties do not.
    pub(crate) fn emit_load_current_builtin_temporal_prototype(
        &mut self,
        family: TemporalIntrinsicFamily,
        prototype: u32,
        function: &mut Function,
    ) {
        let realm = self.reserve_temp_local();
        let intrinsics = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(family.prototype_global()));
        function.instruction(&Instruction::LocalSet(prototype));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics,
            family.prototype_slot().offset(),
            prototype,
            function,
        );
        // Both bootstrap paths publish these slots before any user code.
        function.instruction(&Instruction::LocalGet(prototype));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(intrinsics);
        self.release_temp_local(realm);
    }
}
