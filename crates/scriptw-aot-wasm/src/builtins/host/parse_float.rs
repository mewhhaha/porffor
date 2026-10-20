use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_host_parse_float_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let pending = schema.reserve_completion(f);
        let start = schema.reserve_i64_local(f);
        let exponent = schema.reserve_i64_local(f);
        let bits = schema.reserve_i64_local(f);
        let negative = schema.reserve_i32_local(f);
        let saw_digit = schema.reserve_i32_local(f);
        let infinity = schema.reserve_i32_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        output.initialize(f);
        output
            .value()
            .set_scalar(ScalarValue::NumberBits(f64::NAN.to_bits() as i64), f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let text = self.emit_host_numeric_text(&input, &pending, &output, exit, f)?;
        text.index.load(f);
        start.store(f);
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
        text.index.load(f);
        exponent.store(f);
        text.index.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Add);
        text.length.load(f);
        f.instruction(&Instruction::I64LeU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I32Const(1));
        infinity.store(f);
        for unit in b"Infinity" {
            text.read(self, f);
            infinity.load(f);
            text.unit.load(f);
            f.instruction(&Instruction::I64Const(i64::from(*unit)));
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::I32And);
            infinity.store(f);
            text.advance(f);
        }
        infinity.load(f);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        bits.store(f);
        negative.load(f);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(f64::NEG_INFINITY.to_bits() as i64));
        bits.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output.value().set_number(bits, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        exponent.load(f);
        text.index.store(f);
        f.instruction(&Instruction::I32Const(0));
        saw_digit.store(f);
        self.emit_host_decimal_digits(&text, saw_digit, f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(46));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        self.emit_host_decimal_digits(&text, saw_digit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        saw_digit.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        text.index.load(f);
        exponent.store(f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(69));
        f.instruction(&Instruction::I64Eq);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(101));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(43));
        f.instruction(&Instruction::I64Eq);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(45));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        text.advance(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        saw_digit.store(f);
        self.emit_host_decimal_digits(&text, saw_digit, f);
        saw_digit.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        exponent.load(f);
        text.index.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let token = schema.reserve_gc_local(f).initialize(
            self.emit_gc_string_slice(&text.string, start, text.index, f),
            f,
        );
        self.emit_string_to_number_payload(&token, bits, f)?;
        output.value().set_number(bits, f);
        token.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        text.clear(schema, f);
        self.completion().copy_from(&output, f);
        schema.release_i32_local(infinity, f);
        schema.release_i32_local(saw_digit, f);
        schema.release_i32_local(negative, f);
        schema.release_i64_local(bits, f);
        schema.release_i64_local(exponent, f);
        schema.release_i64_local(start, f);
        pending.clear(f);
        output.clear(f);
        input.clear(f);
        Ok(())
    }

    fn emit_host_decimal_digits(
        &mut self,
        text: &parse_int::HostNumericText,
        saw_digit: I32Local,
        f: &mut Function,
    ) {
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        text.index.load(f);
        text.length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        text.read(self, f);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(48));
        f.instruction(&Instruction::I64LtU);
        text.unit.load(f);
        f.instruction(&Instruction::I64Const(57));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(1));
        saw_digit.store(f);
        text.advance(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
    }
}
