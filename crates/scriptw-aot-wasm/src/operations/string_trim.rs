use super::*;

/// TrimString admits exactly these three boundary policies.
enum EcmaTrimMode {
    Start,
    End,
    Both,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_ecmascript_trim_start_payload_from_locals(
        &mut self,
        string: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<crate::gc_types::GcStackReference<StringValue>, EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(string, EcmaTrimMode::Start, function)
    }

    pub(crate) fn emit_ecmascript_trim_end_payload_from_locals(
        &mut self,
        string: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<crate::gc_types::GcStackReference<StringValue>, EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(string, EcmaTrimMode::End, function)
    }

    pub(crate) fn emit_ecmascript_trim_both_payload_from_locals(
        &mut self,
        string: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<crate::gc_types::GcStackReference<StringValue>, EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(string, EcmaTrimMode::Both, function)
    }

    fn emit_ecmascript_trim_payload_from_locals(
        &mut self,
        string: &GcLocal<StringValue>,
        mode: EcmaTrimMode,
        function: &mut Function,
    ) -> Result<crate::gc_types::GcStackReference<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let start = schema.reserve_i64_local(function);
        let end = schema.reserve_i64_local(function);
        self.emit_gc_string_trim_range(string, start, end, function);
        match mode {
            EcmaTrimMode::Start => {
                let units = schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<StringValue>()
                        .field(StringValueSchema::CODE_UNITS)
                        .read(string, schema, function)
                        .reference(),
                    function,
                );
                schema
                    .array_type::<CodeUnitArray>()
                    .length(&units, schema, function);
                function.instruction(&Instruction::I64ExtendI32U);
                end.store(function);
                units.clear(function);
            }
            EcmaTrimMode::End => {
                function.instruction(&Instruction::I64Const(0));
                start.store(function);
            }
            EcmaTrimMode::Both => {}
        }
        let result = self.emit_gc_string_slice(string, start, end, function);
        schema.release_i64_local(end, function);
        schema.release_i64_local(start, function);
        Ok(result)
    }
}
