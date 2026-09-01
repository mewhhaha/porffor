use super::*;
use crate::intrinsics::collections::CollectionPrototypeIntrinsic;

#[must_use = "created-Realm weak collection intrinsics must be published"]
pub(super) struct CreatedRealmWeakCollectionIntrinsics {
    weak_map_prototype_local: u32,
    weak_map_constructor_local: u32,
    weak_set_prototype_local: u32,
    weak_set_constructor_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_weak_collection_intrinsics(
        &mut self,
        realm_record: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        object_prototype_local: u32,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<CreatedRealmWeakCollectionIntrinsics, EmitError> {
        let weak_set_constructor_meta = self
            .functions
            .get(&StandardBuiltinId::WeakSetConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `WeakSet`",
                )
            })?;
        let weak_map_constructor_meta = self
            .functions
            .get(&StandardBuiltinId::WeakMapConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `WeakMap`",
                )
            })?;

        let weak_set_intrinsic = CollectionPrototypeIntrinsic::WeakSet;
        let weak_set_prototype_local = self.reserve_temp_local();
        let weak_set_constructor_local = self.reserve_temp_local();
        let weak_set_method_local = self.reserve_temp_local();
        let weak_set_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(weak_set_prototype_local));
        self.emit_store_non_array_realm_intrinsic(
            realm_record.index(),
            weak_set_intrinsic.realm_slot(),
            weak_set_prototype_local,
            function,
        );
        self.emit_function_value_payload_in_realm(
            &weak_set_constructor_meta,
            realm_functions,
            weak_set_constructor_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            weak_set_constructor_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            weak_set_constructor_local,
            function,
        );
        self.store_i64_local_at_offset(
            weak_set_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data_with_flags(
            weak_set_constructor_local,
            weak_set_prototype_local,
            false,
            false,
            false,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("add", StandardBuiltinId::WeakSetPrototypeAdd),
            ("delete", StandardBuiltinId::WeakSetPrototypeDelete),
            ("has", StandardBuiltinId::WeakSetPrototypeHas),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload_in_realm(
                &meta,
                realm_functions,
                weak_set_method_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                weak_set_method_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                weak_set_method_local,
                function,
            );
            self.store_i64_local_at_offset(
                weak_set_method_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(weak_set_tag_local));
            self.emit_object_define_local_data(
                weak_set_prototype_local,
                name,
                weak_set_method_local,
                weak_set_tag_local,
                function,
            )?;
        }
        self.emit_collection_prototype_to_string_tag(
            weak_set_intrinsic,
            weak_set_prototype_local,
            function,
        )?;
        self.release_temp_local(weak_set_tag_local);
        self.release_temp_local(weak_set_method_local);

        let weak_map_intrinsic = CollectionPrototypeIntrinsic::WeakMap;
        let weak_map_prototype_local = self.reserve_temp_local();
        let weak_map_constructor_local = self.reserve_temp_local();
        let weak_map_method_local = self.reserve_temp_local();
        let weak_map_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(weak_map_prototype_local));
        self.emit_store_non_array_realm_intrinsic(
            realm_record.index(),
            weak_map_intrinsic.realm_slot(),
            weak_map_prototype_local,
            function,
        );
        self.emit_function_value_payload_in_realm(
            &weak_map_constructor_meta,
            realm_functions,
            weak_map_constructor_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            weak_map_constructor_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            weak_map_constructor_local,
            function,
        );
        self.store_i64_local_at_offset(
            weak_map_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data_with_flags(
            weak_map_constructor_local,
            weak_map_prototype_local,
            false,
            false,
            false,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("delete", StandardBuiltinId::WeakMapPrototypeDelete),
            ("get", StandardBuiltinId::WeakMapPrototypeGet),
            (
                "getOrInsert",
                StandardBuiltinId::WeakMapPrototypeGetOrInsert,
            ),
            (
                "getOrInsertComputed",
                StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed,
            ),
            ("has", StandardBuiltinId::WeakMapPrototypeHas),
            ("set", StandardBuiltinId::WeakMapPrototypeSet),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload_in_realm(
                &meta,
                realm_functions,
                weak_map_method_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                weak_map_method_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                weak_map_method_local,
                function,
            );
            self.store_i64_local_at_offset(
                weak_map_method_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(weak_map_tag_local));
            self.emit_object_define_local_data(
                weak_map_prototype_local,
                name,
                weak_map_method_local,
                weak_map_tag_local,
                function,
            )?;
        }
        self.emit_collection_prototype_to_string_tag(
            weak_map_intrinsic,
            weak_map_prototype_local,
            function,
        )?;
        self.release_temp_local(weak_map_tag_local);
        self.release_temp_local(weak_map_method_local);
        Ok(CreatedRealmWeakCollectionIntrinsics {
            weak_map_prototype_local,
            weak_map_constructor_local,
            weak_set_prototype_local,
            weak_set_constructor_local,
        })
    }

    pub(super) fn emit_publish_created_realm_weak_collection_intrinsics(
        &mut self,
        intrinsics: CreatedRealmWeakCollectionIntrinsics,
        global_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CreatedRealmWeakCollectionIntrinsics {
            weak_map_prototype_local,
            weak_map_constructor_local,
            weak_set_prototype_local,
            weak_set_constructor_local,
        } = intrinsics;

        let weak_map_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(weak_map_tag_local));
        self.emit_object_define_local_data(
            global_local,
            WEAK_MAP_NAME,
            weak_map_constructor_local,
            weak_map_tag_local,
            function,
        )?;
        self.release_temp_local(weak_map_tag_local);
        self.release_temp_local(weak_map_constructor_local);
        self.release_temp_local(weak_map_prototype_local);

        let weak_set_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(weak_set_tag_local));
        self.emit_object_define_local_data(
            global_local,
            WEAK_SET_NAME,
            weak_set_constructor_local,
            weak_set_tag_local,
            function,
        )?;
        self.release_temp_local(weak_set_tag_local);
        self.release_temp_local(weak_set_constructor_local);
        self.release_temp_local(weak_set_prototype_local);
        Ok(())
    }
}
