use super::*;

/// Canonical constructor and defining Realm are acquired together and consumed
/// by capability publication. User values cannot fabricate this authority.
#[must_use = "intrinsic constructor is consumed by capability allocation"]
pub(crate) struct CurrentFunctionRealmIntrinsicPromiseConstructor {
    constructor: ValueLocals,
    context: PromiseInternalFunctionMaterializationContext,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_current_function_realm_intrinsic_promise_constructor(
        &mut self,
        function: &mut Function,
    ) -> CurrentFunctionRealmIntrinsicPromiseConstructor {
        let schema = self.runtime_schema();
        let context =
            self.emit_current_function_promise_internal_function_materialization_context(function);
        let constructor = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            context.realm(),
            NonArrayRealmIntrinsicSlot::PromiseConstructor,
            &constructor,
            function,
        );
        CurrentFunctionRealmIntrinsicPromiseConstructor {
            constructor,
            context,
        }
    }

    pub(crate) fn emit_new_current_function_realm_intrinsic_promise_capability(
        &mut self,
        constructor: CurrentFunctionRealmIntrinsicPromiseConstructor,
        function: &mut Function,
    ) -> Result<GcLocal<PromiseCapability>, EmitError> {
        let result = self.emit_new_promise_capability(
            &constructor.context,
            &constructor.constructor,
            function,
        );
        constructor.constructor.clear(function);
        self.release_promise_internal_function_materialization_context(
            constructor.context,
            function,
        );
        result
    }
}
