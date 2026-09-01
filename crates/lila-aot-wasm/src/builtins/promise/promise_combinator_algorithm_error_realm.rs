use super::*;

#[must_use = "Promise combinator algorithmic error Realm context must be explicitly released"]
pub(super) struct PromiseCombinatorAlgorithmErrorRealmContext {
    type_error_prototype_local: u32,
    range_error_prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_promise_combinator_algorithm_error_realm_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseCombinatorAlgorithmErrorRealmContext {
        let type_error_prototype_local = self.reserve_temp_local();
        let range_error_prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(type_error_prototype_local));
        function.instruction(&Instruction::GlobalGet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(range_error_prototype_local));
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
        for (offset, prototype_local) in [
            (
                HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
            ),
            (
                HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET,
                range_error_prototype_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(intrinsics_local, offset, prototype_local, function);
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
        PromiseCombinatorAlgorithmErrorRealmContext {
            type_error_prototype_local,
            range_error_prototype_local,
        }
    }

    pub(super) fn emit_throw_promise_combinator_type_error(
        &mut self,
        realm: &PromiseCombinatorAlgorithmErrorRealmContext,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            message,
            realm.type_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_throw_promise_combinator_range_error(
        &mut self,
        realm: &PromiseCombinatorAlgorithmErrorRealmContext,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype_local(
            RANGE_ERROR_NAME,
            message,
            realm.range_error_prototype_local,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn release_promise_combinator_algorithm_error_realm_context(
        &mut self,
        realm: PromiseCombinatorAlgorithmErrorRealmContext,
    ) {
        self.release_temp_local(realm.range_error_prototype_local);
        self.release_temp_local(realm.type_error_prototype_local);
    }
}
