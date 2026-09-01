use super::*;

#[must_use = "Promise.withResolvers result allocation context must be consumed"]
pub(super) struct PromiseWithResolversResultAllocationContext {
    prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_current_function_promise_with_resolvers_result_allocation_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseWithResolversResultAllocationContext {
        let prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
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
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        PromiseWithResolversResultAllocationContext { prototype_local }
    }

    pub(super) fn emit_install_promise_with_resolvers_result_prototype(
        &mut self,
        result_object_local: u32,
        context: PromiseWithResolversResultAllocationContext,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(
            result_object_local,
            HEAP_PROTOTYPE_OFFSET,
            context.prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            result_object_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.release_temp_local(context.prototype_local);
    }
}
