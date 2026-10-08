use super::*;

#[must_use = "Promise.try callback TypeError prototype must be consumed"]
pub(super) struct PromiseTryCallbackTypeErrorPrototypeLocal(ValueLocals);
impl FunctionBuilder<'_> {
    pub(super) fn emit_load_promise_try_callback_type_error_prototype(
        &mut self,
        function: &mut Function,
    ) -> PromiseTryCallbackTypeErrorPrototypeLocal {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::TypeErrorPrototype,
            &prototype,
            function,
        );
        realm.clear(function);
        PromiseTryCallbackTypeErrorPrototypeLocal(prototype)
    }
    pub(super) fn emit_throw_promise_try_non_callable_callback(
        &mut self,
        prototype: PromiseTryCallbackTypeErrorPrototypeLocal,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::VALUE_IS_NOT_CALLABLE,
            &prototype.0,
            result,
            function,
        )?;
        prototype.0.clear(function);
        Ok(())
    }
}
