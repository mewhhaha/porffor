use super::*;
use crate::intrinsics::intl::intl_constructor_properties;

#[must_use = "created-Realm Intl intrinsics must be published"]
pub(super) struct CreatedRealmIntlIntrinsics(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_intl_intrinsics(
        &mut self,
        members: IntlNamespaceMembers,
        realm: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        object_prototype: u32,
        function: &mut Function,
    ) -> Result<CreatedRealmIntlIntrinsics, EmitError> {
        let namespace = self.reserve_temp_local();
        let constructor = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let callable = self.reserve_temp_local();
        let tag = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype), None, function)?;
        function.instruction(&Instruction::LocalSet(namespace));
        self.emit_created_realm_intl_callable(
            StandardBuiltinId::IntlGetCanonicalLocales,
            realm_functions,
            callable,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        self.emit_object_define_local_data(
            namespace,
            "getCanonicalLocales",
            callable,
            tag,
            function,
        )?;

        // The same witness installs the whole represented namespace in both
        // entry and created Realms. No per-member gate can silently omit one.
        for (name, builtin) in members.in_installation_order() {
            let properties = intl_constructor_properties(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "missing created-Realm intrinsic properties for {}",
                    builtin.debug_name(),
                ))
            })?;
            self.emit_alloc_plain_object_with_prototype(Some(object_prototype), None, function)?;
            function.instruction(&Instruction::LocalSet(prototype));
            self.emit_store_non_array_realm_intrinsic(
                realm.index(),
                properties.prototype_slot,
                prototype,
                function,
            );
            self.emit_created_realm_intl_callable(builtin, realm_functions, constructor, function)?;
            self.emit_set_function_prototype_data_with_flags(
                constructor,
                prototype,
                false,
                false,
                false,
                true,
                function,
            )?;
            for (receiver, entries) in [
                (constructor, properties.constructor),
                (prototype, properties.prototype),
            ] {
                for property in entries {
                    self.emit_created_realm_intl_callable(
                        property.builtin,
                        realm_functions,
                        callable,
                        function,
                    )?;
                    self.emit_define_intl_intrinsic_function_property(
                        receiver, property, callable, function,
                    )?;
                }
            }
            self.emit_define_intl_intrinsic_to_string_tag(
                prototype,
                properties.prototype_name,
                function,
            )?;
            self.emit_object_append_local_data_property_with_flags(
                namespace,
                name,
                constructor,
                tag,
                true,
                false,
                true,
                function,
            )?;
        }
        self.emit_define_intl_intrinsic_to_string_tag(namespace, "Intl", function)?;

        self.release_temp_local(tag);
        self.release_temp_local(callable);
        self.release_temp_local(prototype);
        self.release_temp_local(constructor);
        Ok(CreatedRealmIntlIntrinsics(namespace))
    }

    fn emit_created_realm_intl_callable(
        &mut self,
        builtin: StandardBuiltinId,
        realm_functions: &RealmFunctionMaterializationContext,
        callable: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!("missing {} metadata", builtin.debug_name()))
            })?;
        self.emit_function_value_payload_in_realm(&meta, realm_functions, callable, function)?;
        // Intl's ordinary coercion and error emitters read the active builtin
        // environment as a function object and then resolve its defining Realm.
        self.store_i64_local_at_offset(
            callable,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            callable,
            function,
        );
        Ok(())
    }

    pub(super) fn emit_publish_created_realm_intl_intrinsics(
        &mut self,
        intrinsics: CreatedRealmIntlIntrinsics,
        global: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        let result =
            self.emit_object_define_local_data(global, "Intl", intrinsics.0, tag, function);
        self.release_temp_local(tag);
        self.release_temp_local(intrinsics.0);
        result
    }
}
