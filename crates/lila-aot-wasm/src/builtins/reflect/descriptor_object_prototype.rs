use super::*;

#[must_use = "the Reflect descriptor Object prototype must be consumed"]
pub(super) struct ReflectDescriptorObjectPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_reflect_descriptor_object_prototype(
        &mut self,
        function: &mut Function,
    ) -> ReflectDescriptorObjectPrototypeLocal {
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
        ReflectDescriptorObjectPrototypeLocal(prototype_local)
    }

    pub(super) fn emit_alloc_reflect_descriptor_object(
        &mut self,
        prototype: ReflectDescriptorObjectPrototypeLocal,
        descriptor_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ReflectDescriptorObjectPrototypeLocal(prototype_local) = prototype;
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        self.release_temp_local(prototype_local);
        Ok(())
    }
}
