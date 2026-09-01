use super::*;

/// The inseparable Realm-owned fields installed on an escaping Promise
/// algorithm closure before that function can be exposed to user code.
///
/// The context is deliberately non-`Copy` and private to this module. Its
/// factories prove either the active Promise builtin's defining Realm or the
/// Promise record's stored Realm, then derive every header field from the same
/// intrinsic table.
#[must_use = "Promise internal function Realm context must be explicitly released"]
pub(super) struct PromiseInternalFunctionMaterializationContext {
    realm_local: u32,
    function_prototype_local: u32,
    type_error_prototype_local: u32,
    range_error_prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_promise_internal_function_materialization_context_from_realm(
        &mut self,
        realm_local: u32,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        let function_prototype_local = self.reserve_temp_local();
        let type_error_prototype_local = self.reserve_temp_local();
        let range_error_prototype_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

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
                HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
                function_prototype_local,
            ),
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

        self.release_temp_local(intrinsics_local);
        PromiseInternalFunctionMaterializationContext {
            realm_local,
            function_prototype_local,
            type_error_prototype_local,
            range_error_prototype_local,
        }
    }

    pub(super) fn emit_current_function_promise_internal_function_materialization_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        let realm_local = self.reserve_temp_local();
        let active_function_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(active_function_local));
        self.load_i64_to_local_from_offset(
            active_function_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        self.release_temp_local(active_function_local);
        self.emit_promise_internal_function_materialization_context_from_realm(
            realm_local,
            function,
        )
    }

    pub(super) fn emit_promise_record_internal_function_materialization_context(
        &mut self,
        promise_record_local: u32,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        let realm_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            promise_record_local,
            HEAP_PROMISE_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_promise_internal_function_materialization_context_from_realm(
            realm_local,
            function,
        )
    }

    pub(super) fn emit_promise_internal_function_value(
        &mut self,
        meta: &WasmFunctionMeta,
        context: &PromiseInternalFunctionMaterializationContext,
        closure_context_local: u32,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        self.emit_store_function_defining_realm(
            function_object_local,
            context.realm_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_PROTOTYPE_OFFSET,
            context.function_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            function_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            context.type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
            context.range_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
            closure_context_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            function_object_local,
            function,
        );
        Ok(())
    }

    pub(super) fn emit_load_promise_internal_function_context(
        &mut self,
        context_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
            context_local,
            function,
        );
    }

    pub(super) fn emit_load_promise_internal_function_realm_intrinsics(
        &self,
        context: &PromiseInternalFunctionMaterializationContext,
        intrinsics_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            context.realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
    }

    pub(super) fn release_promise_internal_function_materialization_context(
        &mut self,
        context: PromiseInternalFunctionMaterializationContext,
    ) {
        self.release_temp_local(context.range_error_prototype_local);
        self.release_temp_local(context.type_error_prototype_local);
        self.release_temp_local(context.function_prototype_local);
        self.release_temp_local(context.realm_local);
    }
}
