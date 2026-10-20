use super::*;

/// Only ToIntlMathematicalValue mints this complete native input. Strings keep
/// exact UTF-16; Number and BigInt keep their separate native decimal domains.
pub(in crate::builtins) struct IntlMathematicalValueLocals {
    kind: GcI32DomainLocal<NumberNumericKind>,
    text: GcLocal<StringValue>,
}
impl IntlMathematicalValueLocals {
    pub(in crate::builtins) fn kind(&self) -> &GcI32DomainLocal<NumberNumericKind> {
        &self.kind
    }
    pub(in crate::builtins) fn text(&self) -> &GcLocal<StringValue> {
        &self.text
    }
    pub(in crate::builtins) fn clear(self, schema: &RuntimeSchema, function: &mut Function) {
        self.text.clear(function);
        self.kind.clear(schema, function);
    }
}
impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_mathematical_value(
        &mut self,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<IntlMathematicalValueLocals, EmitError> {
        let schema = self.runtime_schema();
        let primitive = schema.reserve_completion(function);
        primitive.initialize(function);
        self.emit_tagged_to_primitive_locals_pending(
            ToPrimitiveHint::Number,
            input,
            &primitive,
            function,
        )?;
        self.emit_intl_number_adopt_completion(&primitive, function);
        let kind = GcI32DomainLocal::new(schema, NumberNumericKind::NegativeZero, function);
        let text = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        emit_tag_is(primitive.value(), WasmRuntimeValueTag::String, function);
        self.open_frame(ControlFrameKind::If, function);
        kind.set_constant(NumberNumericKind::String, function);
        text.replace(
            primitive
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        function.instruction(&Instruction::Else);
        emit_tag_is(primitive.value(), WasmRuntimeValueTag::BigInt, function);
        self.open_frame(ControlFrameKind::If, function);
        kind.set_constant(NumberNumericKind::BigInt, function);
        let bigint = schema.reserve_gc_local(function).initialize(
            primitive
                .value()
                .cast_reference::<BigIntValue>(schema, function),
            function,
        );
        text.replace(
            self.emit_bigint_value_to_string_payload(&bigint, function)?,
            function,
        );
        bigint.clear(function);
        function.instruction(&Instruction::Else);
        let number = schema.reserve_completion(function);
        number.initialize(function);
        self.emit_value_to_number_payload(primitive.value(), &number, function)?;
        self.emit_intl_number_adopt_completion(&number, function);
        number.value().scalar().load(function);
        function.instruction(&Instruction::I64Const((-0.0f64).to_bits() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        kind.set_constant(NumberNumericKind::Number, function);
        text.replace(
            self.emit_number_to_string_payload(number.value().scalar(), function)?,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        number.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        primitive.clear(function);
        Ok(IntlMathematicalValueLocals { kind, text })
    }
}
