use super::*;

impl DeclaredGcHostImport {
    /// Store the nullable syntax discriminator before consuming the remaining
    /// exact Completion stack. Host capability/internal errors trap instead.
    pub(crate) fn compile_dynamic_function(
        self,
        kind: I32Local,
        new_target: &ValueLocals,
        sources: &GcLocal<ValueArray>,
        active_constructor: &GcLocal<FunctionContext>,
        caller_environment: &GcLocal<Environment, Nullable>,
        completion: &CompletionLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue, Nullable>, crate::EmitError> {
        if self.import() != GcHostImport::CompileDynamicFunction {
            return Err(crate::EmitError::unsupported(
                "native Function compilation requires its declared host import",
            ));
        }
        kind.load(function);
        new_target.emit(function);
        sources.load(schema, function);
        active_constructor.load(schema, function);
        caller_environment.load(schema, function);
        self.emit_call_instruction(function);
        let syntax = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize(GcStackReference::new(), function);
        completion.store_call(GcCallResult::emitted(), function);
        Ok(syntax)
    }
}
