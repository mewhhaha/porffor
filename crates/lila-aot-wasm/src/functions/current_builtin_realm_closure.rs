use super::*;
use crate::gc_types::{BuiltinClosureCapture, FunctionObject, GcLocal, GcStackReference, Nullable};

impl FunctionBuilder<'_> {
    /// Capture native algorithm state before publishing a callable in the
    /// executing builtin's defining Realm.
    pub(crate) fn emit_current_builtin_realm_closure_value(
        &mut self,
        meta: &WasmFunctionMeta,
        capture: &GcLocal<BuiltinClosureCapture, Nullable>,
        function: &mut Function,
    ) -> Result<GcStackReference<FunctionObject>, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::FunctionPrototype,
            &prototype,
            function,
        );
        let function_prototype = schema.reserve_gc_local(function).initialize(
            prototype.cast_reference::<FunctionObject>(schema, function),
            function,
        );
        prototype.clear(function);
        let context = RealmFunctionMaterializationContext {
            realm,
            function_prototype,
        };
        let result = self
            .emit_function_value_payload_in_realm_with_capture(meta, &context, capture, function);
        self.release_realm_function_materialization_context(context, function);
        result
    }
}
