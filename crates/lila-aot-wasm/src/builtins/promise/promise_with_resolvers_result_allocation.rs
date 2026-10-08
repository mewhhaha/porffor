use super::*;

#[must_use = "Promise.withResolvers prototype is consumed by allocation"]
pub(super) struct PromiseWithResolversResultAllocationContext {
    prototype: ValueLocals,
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_current_function_promise_with_resolvers_result_allocation_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseWithResolversResultAllocationContext {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            &prototype,
            function,
        );
        realm.clear(function);
        PromiseWithResolversResultAllocationContext { prototype }
    }
    pub(super) fn emit_alloc_promise_with_resolvers_result(
        &mut self,
        context: PromiseWithResolversResultAllocationContext,
        capability: &GcLocal<PromiseCapability>,
        function: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&context.prototype), function)?,
            function,
        );
        let value = schema.reserve_value_local(function);
        for (name, field) in [
            ("promise", PromiseCapabilitySchema::PROMISE),
            ("resolve", PromiseCapabilitySchema::RESOLVE),
            ("reject", PromiseCapabilitySchema::REJECT),
        ] {
            let stored = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<PromiseCapability>()
                    .field(field)
                    .read(capability, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&stored, &value, schema, function);
            let string = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(name, function)?,
                function,
            );
            let key = PropertyKeyLocals::from_string(schema, &string, function);
            self.emit_object_append_data_property_with_flags(
                &object, &key, &value, true, true, true, function,
            )?;
            key.clear(function);
            string.clear(function);
            stored.clear(function);
        }
        value.clear(function);
        context.prototype.clear(function);
        Ok(object)
    }
}
