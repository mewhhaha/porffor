use super::*;

#[must_use = "created-Realm WeakRef intrinsics must be published"]
pub(super) struct CreatedRealmWeakRefIntrinsics {
    prototype_local: u32,
    constructor_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_weak_ref_intrinsics(
        &mut self,
        realm_record: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        object_prototype_local: u32,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<CreatedRealmWeakRefIntrinsics, EmitError> {
        let constructor_meta = self
            .functions
            .get(&StandardBuiltinId::WeakRefConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `WeakRef`",
                )
            })?;
        let deref_meta = self
            .functions
            .get(&StandardBuiltinId::WeakRefPrototypeDeref.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `WeakRef.prototype.deref`",
                )
            })?;
        let prototype_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let deref_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_store_non_array_realm_intrinsic(
            realm_record.index(),
            NonArrayRealmIntrinsicSlot::WeakRefPrototype,
            prototype_local,
            function,
        );

        self.emit_function_value_payload_in_realm(
            &constructor_meta,
            realm_functions,
            constructor_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            constructor_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data_with_flags(
            constructor_local,
            prototype_local,
            false,
            false,
            false,
            true,
            function,
        )?;

        self.emit_function_value_payload_in_realm(
            &deref_meta,
            realm_functions,
            deref_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            deref_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            deref_local,
            function,
        );
        self.store_i64_local_at_offset(
            deref_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            prototype_local,
            "deref",
            deref_local,
            tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(WEAK_REF_NAME)));
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            value_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(value_local);
        self.release_temp_local(key_local);
        self.release_temp_local(deref_local);
        Ok(CreatedRealmWeakRefIntrinsics {
            prototype_local,
            constructor_local,
        })
    }

    pub(super) fn emit_publish_created_realm_weak_ref_intrinsics(
        &mut self,
        intrinsics: CreatedRealmWeakRefIntrinsics,
        global_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CreatedRealmWeakRefIntrinsics {
            prototype_local,
            constructor_local,
        } = intrinsics;
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            global_local,
            WEAK_REF_NAME,
            constructor_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }
}
