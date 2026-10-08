//! JSON grammar reads UTF-16 units; it never invokes a JavaScript parser.
use super::*;
impl FunctionBuilder<'_> {
    pub(super) fn emit_json_peek(
        &self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        out: I32Local,
        f: &mut Function,
    ) {
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.emit_gc_string_code_unit_i32(text, index, f);
        out.store(f);
        f.instruction(&Instruction::Else);
        json_i32(out, -1, f);
        f.instruction(&Instruction::End);
    }
    pub(super) fn emit_json_bad_text(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_error(realm, NativeErrorKind::SyntaxError, message, f)
    }
    pub(super) fn emit_json_expect_unit(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        expected: i32,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let unit = s.reserve_i32_local(f);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(expected));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_bad_text(realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        json_increment(index, f);
        s.release_i32_local(unit, f);
        Ok(())
    }
    pub(super) fn emit_json_skip_whitespace(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let unit = s.reserve_i32_local(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_json_peek(text, index, length, unit, f);
        f.instruction(&Instruction::I32Const(0));
        for code in [0x20, 0x09, 0x0a, 0x0d] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(code));
            f.instruction(&Instruction::I32Eq);
            f.instruction(&Instruction::I32Or);
        }
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        json_increment(index, f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(unit, f);
    }
    fn emit_json_digit_i32(&self, unit: I32Local, f: &mut Function) {
        unit.load(f);
        f.instruction(&Instruction::I32Const(48));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(9));
        f.instruction(&Instruction::I32LeU);
    }
    fn emit_json_digits(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        require_one: bool,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let unit = s.reserve_i32_local(f);
        self.emit_json_peek(text, index, length, unit, f);
        if require_one {
            self.emit_json_digit_i32(unit, f);
            f.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_json_bad_text(realm, message, f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_json_peek(text, index, length, unit, f);
        self.emit_json_digit_i32(unit, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        json_increment(index, f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(unit, f);
        Ok(())
    }
    fn emit_json_parse_number(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        out: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let start = s.reserve_i64_local(f);
        let unit = s.reserve_i32_local(f);
        let bits = s.reserve_i64_local(f);
        index.load(f);
        start.store(f);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(45));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(48));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        f.instruction(&Instruction::Else);
        self.emit_json_digits(text, index, length, true, realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(46));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        self.emit_json_digits(text, index, length, true, realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(32));
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Const(101));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(43));
        f.instruction(&Instruction::I32Eq);
        unit.load(f);
        f.instruction(&Instruction::I32Const(45));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_json_digits(text, index, length, true, realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let token = s
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(text, start, index, f), f);
        self.emit_string_to_number_payload(&token, bits, f)?;
        out.set_number(bits, f);
        token.clear(f);
        s.release_i64_local(bits, f);
        s.release_i32_local(unit, f);
        s.release_i64_local(start, f);
        Ok(())
    }
    pub(super) fn emit_json_parse_string(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let s = self.runtime_schema();
        self.emit_json_expect_unit(text, index, length, 34, realm, message, f)?;
        let capacity = s.reserve_i32_local(f);
        length.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let construction = StringConstruction::allocate(s, s.reserve_gc_local(f), capacity, f);
        let used = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        let hex = s.reserve_i32_local(f);
        let digit = s.reserve_i32_local(f);
        let valid = s.reserve_i32_local(f);
        json_i32(used, 0, f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(34));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_increment(index, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        unit.load(f);
        f.instruction(&Instruction::I32Const(32));
        f.instruction(&Instruction::I32LtU);
        unit.load(f);
        f.instruction(&Instruction::I32Const(-1));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_bad_text(realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        json_increment(index, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(92));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_peek(text, index, length, unit, f);
        json_increment(index, f);
        json_i32(valid, 0, f);
        for (escape, decoded) in [
            (34, 34),
            (92, 92),
            (47, 47),
            (98, 8),
            (102, 12),
            (110, 10),
            (114, 13),
            (116, 9),
        ] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(escape));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            json_i32(hex, decoded, f);
            json_i32(valid, 1, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        unit.load(f);
        f.instruction(&Instruction::I32Const(117));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        json_i32(hex, 0, f);
        for _ in 0..4 {
            self.emit_json_peek(text, index, length, unit, f);
            json_increment(index, f);
            json_i32(digit, -1, f);
            for (low, high, offset) in [(48, 57, 48), (65, 70, 55), (97, 102, 87)] {
                unit.load(f);
                f.instruction(&Instruction::I32Const(low));
                f.instruction(&Instruction::I32GeU);
                unit.load(f);
                f.instruction(&Instruction::I32Const(high));
                f.instruction(&Instruction::I32LeU);
                f.instruction(&Instruction::I32And);
                self.open_frame(ControlFrameKind::If, f);
                unit.load(f);
                f.instruction(&Instruction::I32Const(offset));
                f.instruction(&Instruction::I32Sub);
                digit.store(f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            digit.load(f);
            f.instruction(&Instruction::I32Const(-1));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_json_bad_text(realm, message, f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            hex.load(f);
            f.instruction(&Instruction::I32Const(4));
            f.instruction(&Instruction::I32Shl);
            digit.load(f);
            f.instruction(&Instruction::I32Or);
            hex.store(f);
        }
        json_i32(valid, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_bad_text(realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        hex.load(f);
        unit.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        construction.write(used, unit, s, f);
        used.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        used.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let full = s
            .reserve_gc_local(f)
            .initialize(construction.publish(s, f), f);
        let zero = s.reserve_i64_local(f);
        let end = s.reserve_i64_local(f);
        json_i64(zero, 0, f);
        used.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        end.store(f);
        let result = s
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(&full, zero, end, f), f);
        full.clear(f);
        s.release_i64_local(end, f);
        s.release_i64_local(zero, f);
        s.release_i32_local(valid, f);
        s.release_i32_local(digit, f);
        s.release_i32_local(hex, f);
        s.release_i32_local(unit, f);
        s.release_i32_local(used, f);
        s.release_i32_local(capacity, f);
        Ok(result)
    }
    pub(super) fn emit_json_parse_primitive(
        &mut self,
        text: &GcLocal<StringValue>,
        index: I64Local,
        length: I64Local,
        realm: &GcLocal<RealmRecord>,
        message: RuntimeErrorMessage,
        out: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let unit = s.reserve_i32_local(f);
        let matched = s.reserve_i32_local(f);
        json_i32(matched, 0, f);
        self.emit_json_peek(text, index, length, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(34));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let string = self.emit_json_parse_string(text, index, length, realm, message, f)?;
        out.set_reference(&string, s, f);
        string.clear(f);
        json_i32(matched, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        for (literal, value) in [
            ("true", ScalarValue::Boolean(true)),
            ("false", ScalarValue::Boolean(false)),
            ("null", ScalarValue::Null),
        ] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(i32::from(literal.as_bytes()[0])));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            for code in literal.bytes() {
                self.emit_json_expect_unit(
                    text,
                    index,
                    length,
                    i32::from(code),
                    realm,
                    message,
                    f,
                )?;
            }
            out.set_scalar(value, f);
            json_i32(matched, 1, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.emit_json_digit_i32(unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(45));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_parse_number(text, index, length, realm, message, out, f)?;
        json_i32(matched, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        matched.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_bad_text(realm, message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i32_local(matched, f);
        s.release_i32_local(unit, f);
        Ok(())
    }
}
