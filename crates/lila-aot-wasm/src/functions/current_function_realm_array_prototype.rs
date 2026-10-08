use super::*;
use crate::gc_types::{ArrayObject, GcLocal, GcStackReference, I64Local};

/// A rooted prototype selected from the executing function's defining Realm.
/// ArrayCreate consumes this owner before publishing its immutable header edge.
#[must_use]
pub(crate) struct CurrentFunctionRealmArrayPrototypeLocal(GcLocal<ArrayObject>);

impl FunctionBuilder<'_> {
    pub(crate) fn emit_load_current_function_realm_array_prototype(
        &mut self,
        function: &mut Function,
    ) -> CurrentFunctionRealmArrayPrototypeLocal {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_gc_local(function).initialize(
            self.emit_load_realm_array_prototype(&realm, function),
            function,
        );
        realm.clear(function);
        CurrentFunctionRealmArrayPrototypeLocal(prototype)
    }

    pub(crate) fn emit_alloc_array_with_current_function_realm_prototype(
        &mut self,
        length: I64Local,
        prototype: CurrentFunctionRealmArrayPrototypeLocal,
        function: &mut Function,
    ) -> Result<GcStackReference<ArrayObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        value.set_reference(&prototype.0, schema, function);
        let result =
            self.emit_alloc_array_payload_with_length_and_prototype(length, &value, function);
        value.clear(function);
        prototype.0.clear(function);
        result
    }
}
