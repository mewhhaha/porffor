use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_host_create_realm_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_alloc_realm_record(0, 1, f)?, f);
        let global = self.emit_created_realm_global_object(&realm, f)?;
        let context = self.emit_realm_function_materialization_context_from_realm(&realm, f);
        let prototype = schema.reserve_value_local(f);
        let value = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            &prototype,
            f,
        );
        let hooks = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        value.set_reference(&global, schema, f);
        self.emit_host_fresh_data(&hooks, "global", &value, f)?;
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::AbstractModuleSourceConstructor,
            &value,
            f,
        );
        self.emit_host_fresh_data(&hooks, "AbstractModuleSource", &value, f)?;
        let detach_meta = self
            .functions
            .get(&HostBuiltinId::DetachArrayBuffer.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("created Realm requires detachArrayBuffer callable")
            })?;
        let detach = schema.reserve_gc_local(f).initialize(
            self.emit_function_value_payload_in_realm(&detach_meta, &context, f)?,
            f,
        );
        value.set_reference(&detach, schema, f);
        self.emit_host_fresh_data(&hooks, "detachArrayBuffer", &value, f)?;
        detach.clear(f);
        value.set_reference(&hooks, schema, f);
        self.emit_host_fresh_data(&global, "$262", &value, f)?;
        let facade = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        value.set_reference(&global, schema, f);
        self.emit_host_fresh_data(&facade, "global", &value, f)?;
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::AbstractModuleSourceConstructor,
            &value,
            f,
        );
        self.emit_host_fresh_data(&facade, "AbstractModuleSource", &value, f)?;
        let eval_meta = self
            .functions
            .get(&HostBuiltinId::RealmEvalScript.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("created Realm requires evalScript callable"))?;
        let eval = schema.reserve_gc_local(f).initialize(
            self.emit_function_value_payload_in_realm(&eval_meta, &context, f)?,
            f,
        );
        value.set_reference(&eval, schema, f);
        self.emit_host_fresh_data(&facade, lila_ir::REALM_EVAL_SCRIPT_METHOD_NAME, &value, f)?;
        eval.clear(f);
        self.completion().initialize(f);
        self.completion().value().set_reference(&facade, schema, f);
        facade.clear(f);
        hooks.clear(f);
        value.clear(f);
        prototype.clear(f);
        self.release_realm_function_materialization_context(context, f);
        global.clear(f);
        realm.clear(f);
        Ok(())
    }

    fn emit_host_fresh_data(
        &mut self,
        object: &GcLocal<OrdinaryObject>,
        name: &str,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let name = schema
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference(name, f)?, f);
        let key = PropertyKeyLocals::from_string(schema, &name, f);
        self.emit_object_append_data_property_with_flags(object, &key, value, true, true, true, f)?;
        key.clear(f);
        name.clear(f);
        Ok(())
    }
}
