use super::*;

#[must_use = "Promise combinator algorithm error prototypes must be released"]
pub(super) struct PromiseCombinatorAlgorithmErrorRealmContext {
    type_error: ValueLocals,
    range_error: ValueLocals,
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_promise_combinator_algorithm_error_realm_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseCombinatorAlgorithmErrorRealmContext {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let type_error = schema.reserve_value_local(function);
        let range_error = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::TypeErrorPrototype,
            &type_error,
            function,
        );
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::RangeErrorPrototype,
            &range_error,
            function,
        );
        realm.clear(function);
        PromiseCombinatorAlgorithmErrorRealmContext {
            type_error,
            range_error,
        }
    }
    pub(super) fn emit_throw_promise_combinator_type_error(
        &mut self,
        realm: &PromiseCombinatorAlgorithmErrorRealmContext,
        message: RuntimeErrorMessage,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::TypeError,
            message,
            &realm.type_error,
            result,
            function,
        )
    }
    pub(super) fn emit_throw_promise_combinator_range_error(
        &mut self,
        realm: &PromiseCombinatorAlgorithmErrorRealmContext,
        message: RuntimeErrorMessage,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::RangeError,
            message,
            &realm.range_error,
            result,
            function,
        )
    }
    pub(super) fn release_promise_combinator_algorithm_error_realm_context(
        &mut self,
        realm: PromiseCombinatorAlgorithmErrorRealmContext,
        function: &mut Function,
    ) {
        realm.range_error.clear(function);
        realm.type_error.clear(function);
    }
}
