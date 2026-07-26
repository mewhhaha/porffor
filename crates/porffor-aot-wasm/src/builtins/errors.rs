use super::super::*;

#[derive(Clone, Copy)]
pub(crate) enum NewTargetPrototypeFallback {
    CurrentGlobal,
    FunctionSnapshot(u64),
    RealmIntrinsic(u64),
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_alloc_error_instance_from_locals(
        &mut self,
        prototype_payload_local: u32,
        message_payload_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_install_error_cause_from_arg(
        &mut self,
        error_object_local: u32,
        options_arg_index: usize,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let cause_key_local = self.reserve_temp_local();
        let has_cause_local = self.reserve_temp_local();
        let cause_payload_local = self.reserve_temp_local();
        let cause_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            options_arg_index,
            options_payload_local,
            options_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("cause")));
        function.instruction(&Instruction::LocalSet(cause_key_local));
        self.emit_object_has_property_i32(
            options_payload_local,
            options_tag_local,
            cause_key_local,
            has_cause_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_cause_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            cause_key_local,
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        self.emit_object_define_data(
            error_object_local,
            cause_key_local,
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cause_tag_local);
        self.release_temp_local(cause_payload_local);
        self.release_temp_local(has_cause_local);
        self.release_temp_local(cause_key_local);
        self.release_temp_local(options_tag_local);
        self.release_temp_local(options_payload_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_aggregate_error_instance_from_locals(
        &mut self,
        message_payload_local: Option<u32>,
        errors_payload_local: u32,
        prototype_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("errors")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_suppressed_error_instance_from_locals(
        &mut self,
        message_payload_local: Option<u32>,
        error_payload_local: u32,
        error_tag_local: u32,
        suppressed_payload_local: u32,
        suppressed_tag_local: u32,
        prototype_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("error")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("suppressed")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            suppressed_payload_local,
            suppressed_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_error_new_target_prototype_to_local(
        &mut self,
        default_prototype_global_index: u32,
        fallback_realm_prototype_offset: Option<u64>,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fallback = fallback_realm_prototype_offset
            .map(NewTargetPrototypeFallback::FunctionSnapshot)
            .unwrap_or(NewTargetPrototypeFallback::CurrentGlobal);
        self.emit_new_target_prototype_to_local(
            default_prototype_global_index,
            fallback,
            prototype_payload_local,
            function,
        )
    }

    pub(crate) fn emit_new_target_prototype_to_local(
        &mut self,
        default_prototype_global_index: u32,
        fallback: NewTargetPrototypeFallback,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let realm_source_payload_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let prototype_realm_local = self.reserve_temp_local();
        let prototype_realm_revoked_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalSet(realm_source_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            realm_source_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(default_prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(prototype_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        match fallback {
            NewTargetPrototypeFallback::CurrentGlobal => {
                function.instruction(&Instruction::GlobalGet(default_prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_payload_local));
            }
            NewTargetPrototypeFallback::FunctionSnapshot(offset) => {
                self.load_i64_to_local_from_offset(
                    realm_source_payload_local,
                    offset,
                    prototype_payload_local,
                    function,
                );
            }
            NewTargetPrototypeFallback::RealmIntrinsic(offset) => {
                self.emit_get_function_realm_to_locals(
                    new_target_payload_local,
                    new_target_tag_local,
                    prototype_realm_local,
                    prototype_realm_revoked_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(prototype_realm_revoked_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "cannot get function realm from a revoked Proxy",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_load_realm_intrinsic_prototype_or_global(
                    prototype_realm_local,
                    offset,
                    default_prototype_global_index,
                    prototype_payload_local,
                    function,
                );
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_realm_revoked_local);
        self.release_temp_local(prototype_realm_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(realm_source_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_aggregate_error_new_target_prototype_to_local(
        &mut self,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_error_new_target_prototype_to_local(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            Some(HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET),
            prototype_payload_local,
            function,
        )
    }

    pub(crate) fn emit_runtime_error_object(
        &mut self,
        name: &str,
        _message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(error_prototype_global_index(name)),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_throw_runtime_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_runtime_error_object(name, message, payload_local, tag_local, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let prototype_offset = error_realm_prototype_offset(name);

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(name, message, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        if let Some(offset) = prototype_offset {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                offset,
                prototype_local,
                function,
            );
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(prototype_local));
        }
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(error_prototype_global_index(name)));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local(
            name,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_type_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            TYPE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_current_function_realm_range_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            RANGE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_runtime_error_with_prototype_local(
        &mut self,
        name: &str,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(message)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_throw_runtime_error_to_active_handler(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(name, message, payload_local, tag_local, function)?;
        if !self.is_main() {
            self.emit_return_current_completion(function);
        } else if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, extra_depth, function);
        } else {
            self.emit_return_current_completion(function);
        }
        Ok(())
    }

    pub(crate) fn emit_capture_throw_error_name(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        Ok(())
    }

    pub(crate) fn emit_aggregate_error_iterable_to_list_payload(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            input_payload_local,
            input_tag_local,
            payload_local,
            "AggregateError errors input must be iterable",
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            input_payload_local,
            input_tag_local,
            payload_local,
            "AggregateError errors input must be iterable",
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            input_payload_local,
            input_tag_local,
            input_payload_local,
            input_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError errors input must be iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((input_payload_local, Some(input_tag_local))),
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            next_payload_local,
            next_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_alloc_array_payload_with_length(index_local, payload_local, function)?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_handle_call(
            next_payload_local,
            next_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_write(
            payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError errors input must be iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
