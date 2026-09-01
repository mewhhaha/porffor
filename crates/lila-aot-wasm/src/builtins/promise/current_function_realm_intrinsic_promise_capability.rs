use super::*;

/// The canonical `%Promise%` constructor loaded from the executing function's
/// defining Realm.
///
/// The private, non-copyable local can only be consumed by intrinsic Promise
/// capability allocation, so a request method cannot pair the constructor with
/// another Realm or an arbitrary representation tag.
#[must_use = "intrinsic Promise constructor must be consumed by capability allocation"]
pub(crate) struct CurrentFunctionRealmIntrinsicPromiseConstructor {
    constructor_payload_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_current_function_realm_intrinsic_promise_constructor(
        &mut self,
        function: &mut Function,
    ) -> CurrentFunctionRealmIntrinsicPromiseConstructor {
        let constructor_payload_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
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
            HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        CurrentFunctionRealmIntrinsicPromiseConstructor {
            constructor_payload_local,
        }
    }

    pub(crate) fn emit_new_current_function_realm_intrinsic_promise_capability(
        &mut self,
        constructor: CurrentFunctionRealmIntrinsicPromiseConstructor,
        capability_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        let result = self.emit_new_promise_capability(
            constructor.constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        );
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor.constructor_payload_local);
        result
    }
}
