use super::*;

impl FunctionBuilder<'_> {
    /// Field initializers read the original normalized key from their class
    /// context. The cached String/Symbol never re-runs user key conversion.
    pub(crate) fn emit_class_field_function_name(
        &mut self,
        callable: &crate::gc_types::GcLocal<crate::gc_types::FunctionObject>,
        name: &ClassFieldNameIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::gc_types::{FunctionContext, FunctionContextSchema, StoredValue, ValueArray};
        let meta = self.current_function_meta().ok_or_else(|| {
            EmitError::unsupported("compiler invariant: field named evaluation has no source owner")
        })?;
        if !matches!(
            meta.class_element_execution_kind,
            ClassElementExecutionKind::InstanceFieldInitializer
                | ClassElementExecutionKind::StaticFieldInitializer
        ) {
            return Err(EmitError::unsupported(
                "compiler invariant: named function is outside its field initializer",
            ));
        }
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        match name {
            ClassFieldNameIr::Static(name) => {
                let string = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(name, function)?,
                    function,
                );
                value.set_reference(&string, schema, function);
                string.clear(function);
            }
            ClassFieldNameIr::Computed(slot) => {
                let context = self
                    .body_entry_locals()
                    .and_then(|entry| entry.function_context())
                    .expect("source initializer owns complete class captures");
                let keys = schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<FunctionContext>()
                        .field(FunctionContextSchema::FIELD_KEYS)
                        .read(context, schema, function)
                        .reference()
                        .require_non_null(function),
                    function,
                );
                let index = schema.reserve_i32_local(function);
                function.instruction(&Instruction::I32Const(*slot as i32));
                index.store(function);
                let stored = schema.reserve_gc_local(function).initialize(
                    schema
                        .array_type::<ValueArray>()
                        .read(&keys, index, schema, function)
                        .reference(),
                    function,
                );
                schema
                    .struct_type::<StoredValue>()
                    .read_into(&stored, &value, schema, function);
                stored.clear(function);
                schema.release_i32_local(index, function);
                keys.clear(function);
            }
        }
        let key = self.emit_value_to_property_key_locals(&value, function)?;
        self.emit_set_function_name(callable, &key, FunctionNamePrefix::None, function)?;
        key.clear(function);
        value.clear(function);
        Ok(())
    }
}
