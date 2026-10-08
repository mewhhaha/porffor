use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;

impl FunctionBuilder<'_> {
    /// Promise.any creates AggregateError directly in the retained combinator
    /// Realm. It does not call the public constructor or create an own message.
    pub(in crate::builtins) fn emit_promise_any_aggregate_error(
        &mut self,
        errors: &ValueLocals,
        realm: &GcLocal<RealmRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::AggregateErrorPrototype,
            &prototype,
            function,
        );
        let instance = self.emit_prepare_native_error_instance(&prototype, function)?;
        self.emit_append_error_data(instance.header(), "errors", errors, function)?;
        instance.publish(self, result, function);
        prototype.clear(function);
        Ok(())
    }
}
