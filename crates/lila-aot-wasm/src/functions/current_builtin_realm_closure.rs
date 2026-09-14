use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Create an escaping builtin closure in the active builtin's Realm.
    /// The shared capture slot owns algorithm state while ENV keeps the
    /// function identity used by coercion and generated-error Realm lookup.
    pub(crate) fn emit_current_builtin_realm_closure_value(
        &mut self,
        meta: &WasmFunctionMeta,
        capture_local: u32,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        // Zero ENV denotes an entry builtin. Its canonical callable prototype
        // retains that Realm even when public constructor bindings change.
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        self.load_i64_to_local_from_offset(
            function_prototype_local,
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
            HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(function_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(intrinsics_local);

        let context = RealmFunctionMaterializationContext {
            realm: RealmRecordLocal(realm_local),
            function_prototype_local,
        };
        let result = (|| {
            self.emit_function_value_payload_in_realm(
                meta,
                &context,
                function_object_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                function_object_local,
                HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
                capture_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_object_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                function_object_local,
                function,
            );
            Ok(())
        })();
        self.release_realm_function_materialization_context(context);
        self.release_temp_local(realm_local);
        result
    }
}
