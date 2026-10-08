use super::*;
use crate::functions::RealmFunctionMaterializationContext;

/// Realm and prototype are selected together before any closure escapes.
#[must_use = "the owned Realm context must be released"]
pub(crate) struct PromiseInternalFunctionMaterializationContext(
    RealmFunctionMaterializationContext,
);
impl PromiseInternalFunctionMaterializationContext {
    pub(super) fn realm(&self) -> &GcLocal<RealmRecord> {
        self.0.realm()
    }
    pub(super) fn materialization(&self) -> &RealmFunctionMaterializationContext {
        &self.0
    }
}

/// Each algorithm entry admits only its actual immutable capture family.
/// An element callback cannot accidentally publish a resolving context.
pub(super) enum PromiseInternalFunction<'a> {
    Resolve(&'a GcLocal<PromiseResolvingContext>),
    Reject(&'a GcLocal<PromiseResolvingContext>),
    CapabilityExecutor(&'a GcLocal<PromiseCapabilityExecutorContext>),
    AllElement(&'a GcLocal<PromiseElementContext>),
    SettledFulfillElement(&'a GcLocal<PromiseElementContext>),
    SettledRejectElement(&'a GcLocal<PromiseElementContext>),
    AnyRejectElement(&'a GcLocal<PromiseElementContext>),
    AllKeyedElement(&'a GcLocal<PromiseKeyedElementContext>),
    SettledFulfillKeyedElement(&'a GcLocal<PromiseKeyedElementContext>),
    SettledRejectKeyedElement(&'a GcLocal<PromiseKeyedElementContext>),
    ThenFinally(&'a GcLocal<PromiseFinallyContext>),
    CatchFinally(&'a GcLocal<PromiseFinallyContext>),
    ValueThunk(&'a GcLocal<PromiseFinallyValueContext>),
    Thrower(&'a GcLocal<PromiseFinallyValueContext>),
}
impl<'a> PromiseInternalFunction<'a> {
    fn publication(self) -> (StandardBuiltinId, BuiltinClosurePayload<'a>) {
        match self {
            Self::Resolve(value) => (
                StandardBuiltinId::PromiseResolveFunction,
                BuiltinClosurePayload::PromiseResolving(value),
            ),
            Self::Reject(value) => (
                StandardBuiltinId::PromiseRejectFunction,
                BuiltinClosurePayload::PromiseResolving(value),
            ),
            Self::CapabilityExecutor(value) => (
                StandardBuiltinId::PromiseCapabilityExecutor,
                BuiltinClosurePayload::PromiseCapabilityExecutor(value),
            ),
            Self::AllElement(value) => (
                StandardBuiltinId::PromiseAllResolveElement,
                BuiltinClosurePayload::PromiseElement(value),
            ),
            Self::SettledFulfillElement(value) => (
                StandardBuiltinId::PromiseAllSettledResolveElement,
                BuiltinClosurePayload::PromiseElement(value),
            ),
            Self::SettledRejectElement(value) => (
                StandardBuiltinId::PromiseAllSettledRejectElement,
                BuiltinClosurePayload::PromiseElement(value),
            ),
            Self::AnyRejectElement(value) => (
                StandardBuiltinId::PromiseAnyRejectElement,
                BuiltinClosurePayload::PromiseElement(value),
            ),
            Self::AllKeyedElement(value) => (
                StandardBuiltinId::PromiseAllKeyedResolveElement,
                BuiltinClosurePayload::PromiseKeyedElement(value),
            ),
            Self::SettledFulfillKeyedElement(value) => (
                StandardBuiltinId::PromiseAllSettledKeyedResolveElement,
                BuiltinClosurePayload::PromiseKeyedElement(value),
            ),
            Self::SettledRejectKeyedElement(value) => (
                StandardBuiltinId::PromiseAllSettledKeyedRejectElement,
                BuiltinClosurePayload::PromiseKeyedElement(value),
            ),
            Self::ThenFinally(value) => (
                StandardBuiltinId::PromiseThenFinally,
                BuiltinClosurePayload::PromiseFinally(value),
            ),
            Self::CatchFinally(value) => (
                StandardBuiltinId::PromiseCatchFinally,
                BuiltinClosurePayload::PromiseFinally(value),
            ),
            Self::ValueThunk(value) => (
                StandardBuiltinId::PromiseValueThunk,
                BuiltinClosurePayload::PromiseFinallyValue(value),
            ),
            Self::Thrower(value) => (
                StandardBuiltinId::PromiseThrower,
                BuiltinClosurePayload::PromiseFinallyValue(value),
            ),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_promise_internal_function_materialization_context_from_realm(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        PromiseInternalFunctionMaterializationContext(
            self.emit_realm_function_materialization_context_from_realm(realm, function),
        )
    }
    pub(crate) fn emit_current_function_promise_internal_function_materialization_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let context = self
            .emit_promise_internal_function_materialization_context_from_realm(&realm, function);
        realm.clear(function);
        context
    }
    pub(super) fn emit_promise_record_internal_function_materialization_context(
        &mut self,
        promise: &GcLocal<PromiseObject>,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        let schema = self.runtime_schema();
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseObject>()
                .field(PromiseObjectSchema::REALM)
                .read(promise, schema, function)
                .reference(),
            function,
        );
        let context = self
            .emit_promise_internal_function_materialization_context_from_realm(&realm, function);
        realm.clear(function);
        context
    }
    pub(super) fn emit_promise_internal_function_value(
        &mut self,
        entry: PromiseInternalFunction<'_>,
        context: &PromiseInternalFunctionMaterializationContext,
        function: &mut Function,
    ) -> Result<GcLocal<FunctionObject>, EmitError> {
        let schema = self.runtime_schema();
        let (builtin, payload) = entry.publication();
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "missing Promise algorithm entry {}",
                    builtin.debug_name()
                ))
            })?;
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<BuiltinClosureCapture>()
                    .publish(payload, schema, function)
                    .nullable(),
                function,
            );
        let result = schema.reserve_gc_local(function).initialize(
            self.emit_function_value_payload_in_realm_with_capture(
                &meta, &context.0, &capture, function,
            )?,
            function,
        );
        capture.clear(function);
        Ok(result)
    }
    fn emit_load_promise_internal_function_capture(
        &self,
        kind: BuiltinClosureCaptureKind,
        function: &mut Function,
    ) -> GcLocal<BuiltinClosureCapture> {
        let schema = self.runtime_schema();
        let context = self
            .body_entry_locals()
            .expect("Promise algorithm has an ordinary entry")
            .function_context()
            .expect("Promise algorithm context");
        let capture = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(context, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let actual = schema.reserve_i32_local(function);
        schema
            .struct_type::<BuiltinClosureCapture>()
            .field(BuiltinClosureCaptureSchema::KIND)
            .read(&capture, schema, function)
            .store(actual, function);
        actual.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(kind)));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        schema.release_i32_local(actual, function);
        capture
    }
    pub(crate) fn release_promise_internal_function_materialization_context(
        &mut self,
        context: PromiseInternalFunctionMaterializationContext,
        function: &mut Function,
    ) {
        self.release_realm_function_materialization_context(context.0, function);
    }
}

macro_rules! captured_promise_context {
    ($method:ident, $kind:ident, $field:ident, $ty:ident) => {
        impl FunctionBuilder<'_> {
            pub(super) fn $method(&self, function: &mut Function) -> GcLocal<$ty> {
                let schema = self.runtime_schema();
                let capture = self.emit_load_promise_internal_function_capture(
                    BuiltinClosureCaptureKind::$kind,
                    function,
                );
                let context = schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<BuiltinClosureCapture>()
                        .field(BuiltinClosureCaptureSchema::$field)
                        .read(&capture, schema, function)
                        .reference()
                        .require_non_null(function),
                    function,
                );
                capture.clear(function);
                context
            }
        }
    };
}
captured_promise_context!(
    emit_promise_resolving_context,
    PromiseResolving,
    RESOLVING,
    PromiseResolvingContext
);
captured_promise_context!(
    emit_promise_capability_executor_context,
    PromiseCapabilityExecutor,
    CAPABILITY_EXECUTOR,
    PromiseCapabilityExecutorContext
);
captured_promise_context!(
    emit_promise_element_context,
    PromiseElement,
    ELEMENT,
    PromiseElementContext
);
captured_promise_context!(
    emit_promise_keyed_element_context,
    PromiseKeyedElement,
    KEYED_ELEMENT,
    PromiseKeyedElementContext
);
captured_promise_context!(
    emit_promise_finally_context,
    PromiseFinally,
    FINALLY,
    PromiseFinallyContext
);
captured_promise_context!(
    emit_promise_finally_value_context,
    PromiseFinallyValue,
    FINALLY_VALUE,
    PromiseFinallyValueContext
);
