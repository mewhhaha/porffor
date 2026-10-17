use super::*;

impl DeclaredGcHostImport {
    /// Every request edge stays rooted until all returned stack roots are bound.
    pub(crate) fn compile_direct_eval(
        self,
        source: &GcLocal<StringValue>,
        descriptor: &GcLocal<ImmutableByteArray>,
        realm: &GcLocal<RealmRecord>,
        lexical: &GcLocal<Environment, Nullable>,
        variable: &GcLocal<Environment, Nullable>,
        private: &GcLocal<PrivateEnvironment, Nullable>,
        this_value: &ValueLocals,
        new_target: &ValueLocals,
        invocation: &GcLocal<DirectEvalExecutionContext>,
        calling_program: &ProgramInstanceLocals,
        completion: &CompletionLocals,
        status: I32Local,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue, Nullable>, crate::EmitError> {
        if self.import() != GcHostImport::CompileDirectEval {
            return Err(crate::EmitError::unsupported(
                "direct eval compilation requires its distinct typed host import",
            ));
        }
        source.load(schema, function);
        descriptor.load(schema, function);
        realm.load(schema, function);
        lexical.load(schema, function);
        variable.load(schema, function);
        private.load(schema, function);
        this_value.emit(function);
        new_target.emit(function);
        invocation.load(schema, function);
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
