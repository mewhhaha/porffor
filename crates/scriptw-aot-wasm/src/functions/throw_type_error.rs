use super::*;
use crate::gc_types::{
    FunctionContext, FunctionContextSchema, FunctionObject, FunctionObjectSchema, GcLocal,
    ValueLocals,
};
use crate::objects::{AccessorDescriptorLocals, AccessorGetterLocals, AccessorSetterLocals};

impl FunctionBuilder<'_> {
    /// Publish the Realm's one thrower before Function.prototype or an
    /// unmapped Arguments object becomes observable.
    pub(crate) fn emit_initialize_realm_throw_type_error(
        &mut self,
        context: &RealmFunctionMaterializationContext,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant violated: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let schema = self.runtime_schema();
        let thrower = schema.reserve_gc_local(function).initialize(
            self.emit_function_value_payload_in_realm(&meta, context, function)?,
            function,
        );
        let value = schema.reserve_value_local(function);
        value.set_reference(&thrower, schema, function);
        self.emit_store_non_array_realm_intrinsic(
            context.realm(),
            NonArrayRealmIntrinsicSlot::ThrowTypeError,
            &value,
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::OBJECT)
                .read(context.function_prototype(), schema, function)
                .reference(),
            function,
        );
        for name in ["caller", "arguments"] {
            let key = self.emit_function_string_key(name, function)?;
            self.emit_object_append_accessor_property_with_flags(
                &header,
                &key,
                AccessorDescriptorLocals::GetterAndSetter {
                    getter: AccessorGetterLocals::new(&value),
                    setter: AccessorSetterLocals::new(&value),
                },
                false,
                true,
                function,
            )?;
            key.clear(function);
        }
        header.clear(function);
        value.clear(function);
        thrower.clear(function);
        Ok(())
    }

    /// The callable's retained FunctionContext is the only Realm authority.
    pub(crate) fn emit_load_required_function_realm_throw_type_error(
        &mut self,
        function_object: &GcLocal<FunctionObject>,
        result: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CONTEXT)
                .read(function_object, schema, function)
                .reference(),
            function,
        );
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::REALM)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ThrowTypeError,
            result,
            function,
        );
        realm.clear(function);
        context.clear(function);
    }
}
