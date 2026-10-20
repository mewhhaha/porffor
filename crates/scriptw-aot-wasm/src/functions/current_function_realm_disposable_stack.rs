use super::*;
use crate::gc_types::{GcStackReference, OrdinaryObject};

impl FunctionBuilder<'_> {
    /// Allocate the ordinary header with the executing builtin's defining
    /// Realm prototype before its resource-stack record is published.
    pub(crate) fn emit_alloc_current_function_realm_disposable_stack_object(
        &mut self,
        function: &mut Function,
    ) -> Result<GcStackReference<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::DisposableStackPrototype,
            &prototype,
            function,
        );
        let result = self.emit_alloc_plain_object_with_prototype(Some(&prototype), function);
        prototype.clear(function);
        realm.clear(function);
        result
    }
}
