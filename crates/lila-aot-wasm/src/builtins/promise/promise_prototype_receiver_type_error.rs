use super::*;

#[derive(Clone, Copy)]
enum PromisePrototypeReceiverError {
    ThenIncompatible,
    FinallyNonObject,
}
impl PromisePrototypeReceiverError {
    const fn message(self) -> RuntimeErrorMessage {
        match self {
            Self::ThenIncompatible => {
                RuntimeErrorMessage::PROMISE_PROTOTYPE_THEN_CALLED_ON_INCOMPATIBLE_RECEIVER
            }
            Self::FinallyNonObject => {
                RuntimeErrorMessage::PROMISE_PROTOTYPE_FINALLY_CALLED_ON_NON_OBJECT_RECEIVER
            }
        }
    }
}

#[must_use = "selected receiver-error prototype must be consumed"]
pub(super) struct PromisePrototypeReceiverTypeErrorPrototypeLocal(ValueLocals);
impl FunctionBuilder<'_> {
    pub(super) fn emit_load_promise_prototype_receiver_type_error_prototype(
        &mut self,
        function: &mut Function,
    ) -> PromisePrototypeReceiverTypeErrorPrototypeLocal {
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
        PromisePrototypeReceiverTypeErrorPrototypeLocal(prototype)
    }
    fn emit_throw_promise_prototype_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        error: PromisePrototypeReceiverError,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let emitted = self.emit_throw_runtime_error_with_prototype(
            NativeErrorKind::TypeError,
            error.message(),
            &prototype.0,
            result,
            function,
        );
        prototype.0.clear(function);
        emitted
    }
    pub(super) fn emit_throw_promise_then_incompatible_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_promise_prototype_receiver_error(
            prototype,
            PromisePrototypeReceiverError::ThenIncompatible,
            result,
            function,
        )
    }
    pub(super) fn emit_throw_promise_finally_non_object_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_promise_prototype_receiver_error(
            prototype,
            PromisePrototypeReceiverError::FinallyNonObject,
            result,
            function,
        )
    }
}
