use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_function_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_array_buffer_prototype_local = self.reserve_temp_local();
        let realm_data_view_prototype_local = self.reserve_temp_local();
        let realm_aggregate_error_prototype_local = self.reserve_temp_local();
        let function_object_local = self.reserve_temp_local();
        let active_constructor_local = self.reserve_temp_local();
        let active_constructor_realm_local = self.reserve_temp_local();
        let meta = self
            .functions
            .get(&StandardBuiltinId::FunctionConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Function`",
                )
            })?;

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(self.new_target_tag_local().unwrap()));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            self.new_target_payload_local().unwrap(),
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            realm_array_buffer_prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            self.new_target_payload_local().unwrap(),
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            realm_data_view_prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            self.new_target_payload_local().unwrap(),
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            realm_aggregate_error_prototype_local,
            function,
        );
        self.emit_function_value_payload(&meta, function)?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::GlobalGet(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(active_constructor_local));
        self.load_i64_to_local_from_offset(
            active_constructor_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            active_constructor_realm_local,
            function,
        );
        self.emit_store_function_defining_realm(
            function_object_local,
            active_constructor_realm_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            realm_array_buffer_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            realm_data_view_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            realm_aggregate_error_prototype_local,
            function,
        );
        self.copy_function_realm_typed_array_prototypes(
            self.new_target_payload_local().unwrap(),
            function_object_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(function_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "dynamic Function constructor unsupported",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(active_constructor_realm_local);
        self.release_temp_local(active_constructor_local);
        self.release_temp_local(function_object_local);
        self.release_temp_local(realm_aggregate_error_prototype_local);
        self.release_temp_local(realm_data_view_prototype_local);
        self.release_temp_local(realm_array_buffer_prototype_local);
        Ok(())
    }
}
