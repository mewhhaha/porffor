use super::*;

#[must_use = "PromiseResolve entry root must be released"]
pub(super) struct PromiseResolveOperationRealmContext {
    resolve: ValueLocals,
}
#[must_use = "intrinsic PromiseResolve Realm context must be released"]
pub(super) struct IntrinsicPromiseResolveRealmContext {
    materialization: PromiseInternalFunctionMaterializationContext,
    operation: PromiseResolveOperationRealmContext,
    constructor: ValueLocals,
}

impl FunctionBuilder<'_> {
    fn emit_promise_resolve_internal_function_materialization_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        match authority {
            PromiseResolveRealmAuthority::CurrentFunction => self
                .emit_current_function_promise_internal_function_materialization_context(function),
            PromiseResolveRealmAuthority::ExplicitRealm(realm) => self
                .emit_promise_internal_function_materialization_context_from_realm(realm, function),
            PromiseResolveRealmAuthority::AsyncExecution(context) => self
                .emit_promise_internal_function_materialization_context_from_realm(
                    context.realm(),
                    function,
                ),
        }
    }
    fn emit_promise_resolve_operation_in_context(
        &mut self,
        context: &PromiseInternalFunctionMaterializationContext,
        function: &mut Function,
    ) -> Result<PromiseResolveOperationRealmContext, EmitError> {
        let schema = self.runtime_schema();
        let meta = self
            .functions
            .get(&StandardBuiltinId::PromiseResolve.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Promise.resolve builtin"))?;
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_function_value_payload_in_realm(&meta, context.materialization(), function)?,
            function,
        );
        let resolve = schema.reserve_value_local(function);
        resolve.set_reference(&callable, schema, function);
        callable.clear(function);
        Ok(PromiseResolveOperationRealmContext { resolve })
    }
    pub(super) fn emit_promise_resolve_operation_realm_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> Result<PromiseResolveOperationRealmContext, EmitError> {
        let context = self
            .emit_promise_resolve_internal_function_materialization_context(authority, function);
        let result = self.emit_promise_resolve_operation_in_context(&context, function);
        self.release_promise_internal_function_materialization_context(context, function);
        result
    }
    pub(super) fn emit_intrinsic_promise_resolve_realm_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> Result<IntrinsicPromiseResolveRealmContext, EmitError> {
        let schema = self.runtime_schema();
        let materialization = self
            .emit_promise_resolve_internal_function_materialization_context(authority, function);
        let operation =
            self.emit_promise_resolve_operation_in_context(&materialization, function)?;
        let constructor = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            materialization.realm(),
            NonArrayRealmIntrinsicSlot::PromiseConstructor,
            &constructor,
            function,
        );
        Ok(IntrinsicPromiseResolveRealmContext {
            materialization,
            operation,
            constructor,
        })
    }
    pub(super) fn emit_call_promise_resolve_operation(
        &mut self,
        context: &PromiseResolveOperationRealmContext,
        constructor: &ValueLocals,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arguments = self.emit_pre_evaluated_arg_vector(&[value], function);
        let emitted = self.emit_function_or_proxy_call_with_argv(
            &context.resolve,
            constructor,
            &arguments,
            result,
            function,
        );
        arguments.clear(function);
        emitted
    }
    pub(super) fn release_promise_resolve_operation_realm_context(
        &mut self,
        context: PromiseResolveOperationRealmContext,
        function: &mut Function,
    ) {
        context.resolve.clear(function);
    }
    pub(super) fn release_intrinsic_promise_resolve_realm_context(
        &mut self,
        context: IntrinsicPromiseResolveRealmContext,
        function: &mut Function,
    ) {
        context.constructor.clear(function);
        self.release_promise_resolve_operation_realm_context(context.operation, function);
        self.release_promise_internal_function_materialization_context(
            context.materialization,
            function,
        );
    }
    pub(super) fn emit_intrinsic_promise_resolve_to_locals(
        &mut self,
        context: &IntrinsicPromiseResolveRealmContext,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_call_promise_resolve_operation(
            &context.operation,
            &context.constructor,
            value,
            result,
            function,
        )
    }
}
