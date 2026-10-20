use super::*;

#[must_use = "Promise settlement record prototype is consumed by allocation"]
pub(super) struct PromiseSettlementRecordAllocationContext {
    prototype: ValueLocals,
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_self_backed_promise_settlement_record_allocation_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseSettlementRecordAllocationContext {
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
        PromiseSettlementRecordAllocationContext { prototype }
    }
    pub(super) fn emit_alloc_promise_settlement_record(
        &mut self,
        context: PromiseSettlementRecordAllocationContext,
        settlement: PromiseSettlement,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&context.prototype), function)?,
            function,
        );
        let (status, property) = match settlement {
            PromiseSettlement::Fulfill => ("fulfilled", "value"),
            PromiseSettlement::Reject => ("rejected", "reason"),
        };
        let status_string = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(status, function)?,
            function,
        );
        let status_value = schema.reserve_value_local(function);
        status_value.set_reference(&status_string, schema, function);
        for (name, entry) in [("status", &status_value), (property, value)] {
            let string = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(name, function)?,
                function,
            );
            let key = PropertyKeyLocals::from_string(schema, &string, function);
            self.emit_object_append_data_property_with_flags(
                &object, &key, entry, true, true, true, function,
            )?;
            key.clear(function);
            string.clear(function);
        }
        status_value.clear(function);
        status_string.clear(function);
        context.prototype.clear(function);
        Ok(object)
    }
}
