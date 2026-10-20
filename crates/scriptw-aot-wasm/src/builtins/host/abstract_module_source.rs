use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_host_get_abstract_module_source_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let result = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::AbstractModuleSourceConstructor,
            &result,
            function,
        );
        self.completion().initialize(function);
        self.completion().value().copy_from(&result, function);
        result.clear(function);
        realm.clear(function);
        Ok(())
    }
}
