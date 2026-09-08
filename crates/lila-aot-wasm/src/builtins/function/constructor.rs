use super::*;
use crate::functions::FunctionRealmRevokedRoute;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_function_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // An empty ordinary body and Function.prototype share [[Call]]'s
        // undefined result. Only the emitted body is shared; the new callable
        // has its own constructability, source text, properties and realm.
        let mut empty_body_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototype.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Function.prototype`",
                )
            })?;
        empty_body_meta.name = "anonymous".to_string();
        empty_body_meta.to_string_value = "function anonymous(\n) {\n\n}".to_string();
        empty_body_meta.protocol = FunctionProtocolIr::OrdinaryCallAndConstruct;
        empty_body_meta.strict = false;

        let active_constructor_local = self.reserve_temp_local();
        let active_constructor_realm_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let function_prototype_payload_local = self.reserve_temp_local();
        let function_prototype_tag_local = self.reserve_temp_local();
        let realm_intrinsics_local = self.reserve_temp_local();
        let function_object_local = self.reserve_temp_local();
        let instance_prototype_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let realm_snapshot_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
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

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(active_constructor_local));
        function.instruction(&Instruction::LocalSet(new_target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(new_target_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            prototype_key_local,
            function_prototype_payload_local,
            function_prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            function_prototype_payload_local,
            function_prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(function_prototype_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        let realm_result =
            self.emit_get_function_realm(new_target_payload_local, new_target_tag_local, function);
        let prototype_realm = self.emit_route_function_realm_result(
            realm_result,
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                payload_local: self.result_local,
                tag_local: self.result_tag_local,
            },
            function,
        )?;
        self.load_i64_to_local_from_offset(
            prototype_realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            realm_intrinsics_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            realm_intrinsics_local,
            HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            function_prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(function_prototype_tag_local));
        self.release_resolved_function_realm_local(prototype_realm);
        function.instruction(&Instruction::End);

        self.emit_function_value_payload(&empty_body_meta, function)?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        self.emit_store_function_defining_realm(
            function_object_local,
            active_constructor_realm_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            function_prototype_tag_local,
            function,
        );

        // The function's own prototype object belongs to the active Function
        // constructor's realm, independently of newTarget's prototype realm.
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            instance_prototype_local,
            function,
        );
        self.emit_load_function_defining_realm_object_prototype(
            active_constructor_local,
            object_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            instance_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        for offset in [
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(
                active_constructor_local,
                offset,
                realm_snapshot_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_object_local,
                offset,
                realm_snapshot_local,
                function,
            );
        }
        self.copy_function_realm_typed_array_prototypes(
            active_constructor_local,
            function_object_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(function_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_reject_dynamic_source(
            lila_ir::DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Ordinary),
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(realm_snapshot_local);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(instance_prototype_local);
        self.release_temp_local(function_object_local);
        self.release_temp_local(realm_intrinsics_local);
        self.release_temp_local(function_prototype_tag_local);
        self.release_temp_local(function_prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(active_constructor_realm_local);
        self.release_temp_local(active_constructor_local);
        Ok(())
    }
}
