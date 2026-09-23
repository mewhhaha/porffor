use super::*;

#[must_use = "Promise.try callback TypeError prototype must be consumed"]
pub(super) struct PromiseTryCallbackTypeErrorPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_load_promise_try_callback_type_error_prototype(
        &mut self,
        function: &mut Function,
    ) -> PromiseTryCallbackTypeErrorPrototypeLocal {
        let prototype_local = self.reserve_temp_local();
        // Step 4's Call(callbackfn) throws from Promise.try's own Realm.
        self.emit_load_active_builtin_realm_type_error_prototype(prototype_local, function);
        PromiseTryCallbackTypeErrorPrototypeLocal(prototype_local)
    }

    pub(super) fn emit_throw_promise_try_non_callable_callback(
        &mut self,
        prototype: PromiseTryCallbackTypeErrorPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "value is not callable",
            prototype.0,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(prototype.0);
        result
    }
}
