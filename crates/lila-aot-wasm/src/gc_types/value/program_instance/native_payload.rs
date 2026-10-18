use super::*;
impl ProgramInstanceLocals {
    pub(super) fn fresh_native_payload_slot(
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcLocal<NativeSourcePayloadArray, NonNullable> {
        function.native_program_payload_slot();
        function.instruction(&Instruction::ArrayNewFixed {
            array_type_index: schema
                .array_type::<NativeSourcePayloadArray>()
                .type_index()
                .raw(),
            array_size: 0,
        });
        schema
            .reserve_gc_local::<NativeSourcePayloadArray, NonNullable>(function)
            .initialize(GcStackReference::new(), function)
    }

    pub(crate) const fn native_payload_field() -> u32 {
        ProgramInstanceSchema::NATIVE_SOURCE_PAYLOAD.ordinal().raw()
    }
    fn native_context_schema(schema: &RuntimeSchema) -> crate::native_program_contexts::Schema {
        crate::native_program_contexts::Schema {
            program_type: schema.struct_type::<ProgramInstance>().type_index().raw(),
            payload_type: schema
                .array_type::<NativeSourcePayloadArray>()
                .type_index()
                .raw(),
            payload_field: ProgramInstanceSchema::NATIVE_SOURCE_PAYLOAD.ordinal().raw(),
        }
    }

    pub(crate) fn mark_native_context(&self, schema: &RuntimeSchema, function: &mut Function) {
        function.bind_native_program_context(Self::native_context_schema(schema), self.owner.index);
    }

    pub(crate) fn mark_native_bootstrap(schema: &RuntimeSchema, function: &mut Function) {
        function.bind_native_bootstrap_context(Self::native_context_schema(schema));
    }
}
