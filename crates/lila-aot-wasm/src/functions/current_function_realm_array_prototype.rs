use super::*;

/// A Wasm local proven to contain the active function realm's
/// `%Array.prototype%`, with explicit entry-Realm selection only when the
/// standard builtin has no self-backed function environment.
///
/// The raw local is private and this state is not `Copy`. Array allocation
/// consumes it together with the result payload, so a caller cannot select a
/// current-function realm and then install an unrelated prototype.
#[must_use]
pub(crate) struct CurrentFunctionRealmArrayPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_load_current_function_realm_array_prototype(
        &mut self,
        function: &mut Function,
    ) -> CurrentFunctionRealmArrayPrototypeLocal {
        let prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
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
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
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
        CurrentFunctionRealmArrayPrototypeLocal(prototype_local)
    }

    pub(crate) fn emit_install_current_function_realm_array_prototype(
        &mut self,
        array_payload_local: u32,
        prototype: CurrentFunctionRealmArrayPrototypeLocal,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(
            array_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype.0,
            function,
        );
        self.store_i64_const_at_offset(
            array_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );
        self.release_temp_local(prototype.0);
    }
}
