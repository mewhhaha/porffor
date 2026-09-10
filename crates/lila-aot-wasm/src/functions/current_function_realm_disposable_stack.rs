use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Select the method Realm's canonical prototype and allocate together so
    /// the selected prototype cannot be detached from its result object.
    pub(crate) fn emit_alloc_current_function_realm_disposable_stack_object(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            NonArrayRealmIntrinsicSlot::DisposableStackPrototype.offset(),
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        let result =
            self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function);
        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        self.release_temp_local(prototype_local);
        result
    }
}
