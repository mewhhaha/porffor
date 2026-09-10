use super::*;
use lila_ir::DISPOSABLE_STACK_NAME;

#[must_use = "created-Realm DisposableStack intrinsics must be published"]
pub(super) struct CreatedRealmDisposableStackIntrinsics {
    prototype_local: u32,
    constructor_local: u32,
}

enum DisposableStackProperty {
    Method(&'static [&'static str]),
    Getter(&'static str),
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_disposable_stack_intrinsics(
        &mut self,
        realm_record: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        object_prototype_local: u32,
        type_error_prototype_local: u32,
        reference_error_prototype_local: u32,
        suppressed_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<CreatedRealmDisposableStackIntrinsics, EmitError> {
        let constructor_meta = self
            .functions
            .get(&StandardBuiltinId::DisposableStackConstructor.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing DisposableStack metadata"))?;
        let prototype_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let method_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_store_non_array_realm_intrinsic(
            realm_record.index(),
            NonArrayRealmIntrinsicSlot::DisposableStackPrototype,
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
        for (offset, prototype) in [
            (
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
            ),
            (
                HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
                reference_error_prototype_local,
            ),
            (
                HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
                suppressed_error_prototype_local,
            ),
        ] {
            self.store_i64_local_at_offset(constructor_local, offset, prototype, function);
        }
        self.emit_set_function_prototype_data_with_flags(
            constructor_local,
            prototype_local,
            false,
            false,
            false,
            true,
            function,
        )?;

        for (builtin, property) in [
            (
                StandardBuiltinId::DisposableStackPrototypeAdopt,
                DisposableStackProperty::Method(&["adopt"]),
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeDefer,
                DisposableStackProperty::Method(&["defer"]),
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeMove,
                DisposableStackProperty::Method(&["move"]),
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeUse,
                DisposableStackProperty::Method(&["use"]),
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeDispose,
                DisposableStackProperty::Method(&["dispose", "Symbol.dispose"]),
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeDisposedGetter,
                DisposableStackProperty::Getter("disposed"),
            ),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!("missing {} metadata", builtin.debug_name()))
                })?;
            self.emit_function_value_payload_in_realm(
                &meta,
                realm_functions,
                method_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                method_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                method_local,
                function,
            );
            for (offset, prototype) in [
                (
                    HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                    type_error_prototype_local,
                ),
                (
                    HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
                    reference_error_prototype_local,
                ),
                (
                    HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
                    suppressed_error_prototype_local,
                ),
            ] {
                self.store_i64_local_at_offset(method_local, offset, prototype, function);
            }
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            match property {
                DisposableStackProperty::Method(names) => {
                    for name in names {
                        self.emit_object_define_local_data(
                            prototype_local,
                            name,
                            method_local,
                            tag_local,
                            function,
                        )?;
                    }
                }
                DisposableStackProperty::Getter(name) => {
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_append_accessor_property_with_flags(
                        prototype_local,
                        key_local,
                        Some((method_local, tag_local)),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }
            }
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(DISPOSABLE_STACK_NAME),
        ));
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
        self.release_temp_local(method_local);
        Ok(CreatedRealmDisposableStackIntrinsics {
            prototype_local,
            constructor_local,
        })
    }

    pub(super) fn emit_publish_created_realm_disposable_stack_intrinsics(
        &mut self,
        intrinsics: CreatedRealmDisposableStackIntrinsics,
        global_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CreatedRealmDisposableStackIntrinsics {
            prototype_local,
            constructor_local,
        } = intrinsics;
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            global_local,
            DISPOSABLE_STACK_NAME,
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
