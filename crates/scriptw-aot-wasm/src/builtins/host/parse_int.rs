use super::*;

/// Exact UTF-16 input after ToString; the cursor trims only ECMAScript whitespace.
pub(super) struct HostNumericText {
    pub(super) string: GcLocal<StringValue>,
    pub(super) length: I64Local,
    pub(super) index: I64Local,
    pub(super) unit: I64Local,
}
impl HostNumericText {
    pub(super) fn read(&self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        builder.emit_gc_string_code_unit_i32(&self.string, self.index, f);
        f.instruction(&Instruction::I64ExtendI32U);
        self.unit.store(f);
    }
    pub(super) fn advance(&self, f: &mut Function) {
        self.index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        self.index.store(f);
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        schema.release_i64_local(self.unit, f);
        schema.release_i64_local(self.index, f);
        schema.release_i64_local(self.length, f);
        self.string.clear(f);
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_host_numeric_text(
        &mut self,
        value: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<HostNumericText, EmitError> {
        let schema = self.runtime_schema();
        self.emit_value_to_string_payload(value, pending, f)?;
        self.emit_host_abrupt_exit(pending, output, exit, f);
        let string = schema
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(schema, f), f);
        let length = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let unit = schema.reserve_i64_local(f);
        let units = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&string, schema, f)
                .reference(),
            f,
        );
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.store(f);
        units.clear(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        let text = HostNumericText {
            string,
            length,
            index,
            unit,
        };
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        text.read(self, f);
        self.emit_ecmascript_whitespace_i32(unit, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        text.advance(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        Ok(text)
    }

    pub(crate) fn compile_host_parse_int_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(f);
        let radix_value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let pending = schema.reserve_completion(f);
        let radix = schema.reserve_i64_local(f);
        let negative = schema.reserve_i32_local(f);
        let strip_prefix = schema.reserve_i32_local(f);
        let saw_digit = schema.reserve_i32_local(f);
        let digit = schema.reserve_i64_local(f);
        let number = schema.reserve_f64_local(f);
        let bits = schema.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_builtin_arg_to_value(1, &radix_value, f);
        output.initialize(f);
        output
            .value()
            .set_scalar(ScalarValue::NumberBits(f64::NAN.to_bits() as i64), f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let text = self.emit_host_numeric_text(&input, &pending, &output, exit, f)?;
        f.instruction(&Instruction::I32Const(0));
        negative.store(f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(45));
        f.instruction(&Instruction::I64Eq);
        negative.store(f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(43));
        f.instruction(&Instruction::I64Eq);
        negative.load(f);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        // ToInt32(radix) follows string conversion and sign processing.
        self.emit_value_to_number_payload(&radix_value, &pending, f)?;
        self.emit_host_abrupt_exit(&pending, &output, exit, f);
        self.emit_to_uint32_i64_from_number_payload(pending.value().scalar(), radix, f);
        radix.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64ExtendI32S);
        radix.store(f);
        f.instruction(&Instruction::I32Const(1));
        strip_prefix.store(f);
        radix.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(10));
        radix.store(f);
        f.instruction(&Instruction::Else);
        radix.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64LtS);
        radix.load(f);
        f.instruction(&Instruction::I64Const(36));
        f.instruction(&Instruction::I64GtS);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        radix.load(f);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Eq);
        strip_prefix.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        strip_prefix.load(f);
        text.index.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Add);
        text.length.load(f);
        f.instruction(&Instruction::I64LeU);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(48));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(88));
        f.instruction(&Instruction::I64Eq);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(120));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        f.instruction(&Instruction::I64Const(16));
        radix.store(f);
        f.instruction(&Instruction::Else);
        text.index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        text.index.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        saw_digit.store(f);
        f.instruction(&Instruction::F64Const(0.0.into()));
        number.store(f);
        let digits_done = self.open_frame(ControlFrameKind::Block, f);
        let next_digit = self.open_frame(ControlFrameKind::Loop, f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(digits_done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        text.read(self, f);
        f.instruction(&Instruction::I64Const(36));
        digit.store(f);
        for (first, last, adjustment) in [(48, 57, 48), (65, 90, 55), (97, 122, 87)] {
            text.unit.load(f);
            f.instruction(&Instruction::I64Const(first));
            f.instruction(&Instruction::I64GeU);
            text.unit.load(f);
            f.instruction(&Instruction::I64Const(last));
            f.instruction(&Instruction::I64LeU);
            f.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, f);
            text.unit.load(f);
            f.instruction(&Instruction::I64Const(adjustment));
            f.instruction(&Instruction::I64Sub);
            digit.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        digit.load(f);
        radix.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(digits_done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(1));
        saw_digit.store(f);
        number.load(f);
        radix.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::F64Mul);
        digit.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::F64Add);
        number.store(f);
        text.advance(f);
        self.emit_branch_to_target(next_digit, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        saw_digit.load(f);
        self.open_frame(ControlFrameKind::If, f);
        negative.load(f);
        self.open_frame(ControlFrameKind::If, f);
        number.load(f);
        f.instruction(&Instruction::F64Neg);
        number.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        number.load(f);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        output.value().set_number(bits, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        text.clear(schema, f);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(bits, f);
        schema.release_f64_local(number, f);
        schema.release_i64_local(digit, f);
        schema.release_i32_local(saw_digit, f);
        schema.release_i32_local(strip_prefix, f);
        schema.release_i32_local(negative, f);
        schema.release_i64_local(radix, f);
        pending.clear(f);
        output.clear(f);
        radix_value.clear(f);
        input.clear(f);
        Ok(())
    }
}
