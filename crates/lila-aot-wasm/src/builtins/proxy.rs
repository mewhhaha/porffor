use super::super::*;
use crate::objects::{ProxyHandlerLocals, ProxySlotLocals, ProxyTargetLocals};

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_proxy_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let execution_realm = self.emit_proxy_creation_execution_realm(function);
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let proxy_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, handler_payload_local, handler_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_proxy_creation_type_error(
            &execution_realm,
            "Proxy target must be object",
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(handler_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_proxy_creation_type_error(
            &execution_realm,
            "Proxy handler must be object",
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_proxy_with_slots(
            ProxySlotLocals::new(
                ProxyTargetLocals::new(target_payload_local, target_tag_local),
                ProxyHandlerLocals::new(handler_payload_local, handler_tag_local),
            ),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(proxy_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("$Proxy.target"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            proxy_payload_local,
            key_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload("$Proxy.handler"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            proxy_payload_local,
            key_local,
            handler_payload_local,
            handler_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(key_local);
        self.release_temp_local(proxy_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_proxy_creation_execution_realm(execution_realm);
        Ok(())
    }

    pub(super) fn compile_proxy_revocable_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let execution_realm = self.emit_proxy_creation_execution_realm(function);
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let proxy_payload_local = self.reserve_temp_local();
        let revoke_target_payload_local = self.reserve_temp_local();
        let revoke_target_tag_local = self.reserve_temp_local();
        let revoke_payload_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let empty_args_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, handler_payload_local, handler_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_proxy_creation_type_error(
            &execution_realm,
            "Proxy target must be object",
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(handler_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_proxy_creation_type_error(
            &execution_realm,
            "Proxy handler must be object",
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_proxy_with_slots(
            ProxySlotLocals::new(
                ProxyTargetLocals::new(target_payload_local, target_tag_local),
                ProxyHandlerLocals::new(handler_payload_local, handler_tag_local),
            ),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(proxy_payload_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("$Proxy.target"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            proxy_payload_local,
            key_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload("$Proxy.handler"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            proxy_payload_local,
            key_local,
            handler_payload_local,
            handler_tag_local,
            function,
        )?;

        self.emit_alloc_proxy_revocable_result_object(
            &execution_realm,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("proxy")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        self.emit_object_define_data(
            result_payload_local,
            key_local,
            proxy_payload_local,
            target_tag_local,
            function,
        )?;

        self.emit_proxy_revoke_target_function(
            &execution_realm,
            revoke_target_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(revoke_target_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, empty_args_payload_local, function)?;
        self.emit_alloc_proxy_revocation_bound_function(
            revoke_target_payload_local,
            revoke_target_tag_local,
            proxy_payload_local,
            &execution_realm,
            empty_args_payload_local,
            revoke_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(revoke_target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(revoke_target_tag_local));
        self.emit_object_define_data_with_configurable(
            revoke_payload_local,
            key_local,
            revoke_target_payload_local,
            revoke_target_tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(revoke_target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(revoke_target_tag_local));
        self.emit_object_define_data_with_configurable(
            revoke_payload_local,
            key_local,
            revoke_target_payload_local,
            revoke_target_tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("revoke")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(revoke_target_tag_local));
        self.emit_object_define_data(
            result_payload_local,
            key_local,
            revoke_payload_local,
            revoke_target_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(zero_local);
        self.release_temp_local(key_local);
        self.release_temp_local(empty_args_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(revoke_payload_local);
        self.release_temp_local(revoke_target_tag_local);
        self.release_temp_local(revoke_target_payload_local);
        self.release_temp_local(proxy_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_proxy_creation_execution_realm(execution_realm);
        Ok(())
    }

    pub(super) fn compile_proxy_revoke_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Proxy revoke receiver",
            )
        })?;
        self.store_i64_const_at_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            PROXY_HANDLER_PAYLOAD_MIN,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        Ok(())
    }
}
