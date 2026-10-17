use super::*;

impl DeclaredGcHostImport {
    /// Keep all request owners rooted across the native callback. The syntax
    /// discriminator is separate from both Completion and Script status.
    pub(crate) fn compile_global_script(
        self,
        kind: I32Local,
        source: &GcLocal<StringValue>,
        realm: &GcLocal<RealmRecord>,
        calling_program: &ProgramInstanceLocals,
        completion: &CompletionLocals,
        status: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue, Nullable>, crate::EmitError> {
        if self.import() != GcHostImport::CompileGlobalScript {
            return Err(crate::EmitError::unsupported(
                "global Script compilation requires its distinct declared host import",
            ));
        }
        kind.load(function);
        source.load(schema, function);
        realm.load(schema, function);
        calling_program.reference(schema, function);
        self.emit_call_instruction(function);
        let syntax = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize(GcStackReference::new(), function);
        status.store(function);
        completion.store_call(GcCallResult::emitted(), function);
        Ok(syntax)
    }
}
