//! QuoteJSONString keeps surrogate pairs and escapes lone UTF-16 surrogates.
use super::*;
impl FunctionBuilder<'_> {
    pub(super) fn emit_json_quote(
        &mut self,
        string: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let s = self.runtime_schema();
        let length = s.reserve_i64_local(f);
        self.emit_native_gc_string_length(string, length, f);
        let capacity = s.reserve_i32_local(f);
        length.load(f);
        f.instruction(&Instruction::I64Const((i32::MAX as i64 - 2) / 6));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::Unreachable);
        f.instruction(&Instruction::End);
        length.load(f);
        f.instruction(&Instruction::I64Const(6));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let build = StringConstruction::allocate(s, s.reserve_gc_local(f), capacity, f);
        let used = s.reserve_i32_local(f);
        let index = s.reserve_i64_local(f);
        let unit = s.reserve_i32_local(f);
        let next_unit = s.reserve_i32_local(f);
        let escape = s.reserve_i32_local(f);
        let lone = s.reserve_i32_local(f);
        let hex = s.reserve_i32_local(f);
        json_i32(used, 0, f);
        json_i64(index, 0, f);
        json_i32(unit, 34, f);
        emit_quote_unit(&build, used, unit, s, f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_json_peek(string, index, length, unit, f);
        json_increment(index, f);
        json_i32(escape, 0, f);
        for (code, escaped) in [
            (34, 34),
            (92, 92),
            (8, 98),
            (9, 116),
            (10, 110),
            (12, 102),
            (13, 114),
        ] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(code));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            json_i32(escape, escaped, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        escape.load(f);
        self.open_frame(ControlFrameKind::If, f);
        json_i32(hex, 92, f);
        emit_quote_unit(&build, used, hex, s, f);
        emit_quote_unit(&build, used, escape, s, f);
        f.instruction(&Instruction::Else);
        json_i32(lone, 0, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(0xd800));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(0x3ff));
        f.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_peek(string, index, length, next_unit, f);
        next_unit.load(f);
        f.instruction(&Instruction::I32Const(0xdc00));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(0x3ff));
        f.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, f);
        emit_quote_unit(&build, used, unit, s, f);
        emit_quote_unit(&build, used, next_unit, s, f);
        json_increment(index, f);
        self.emit_branch_to_target(next, f);
        f.instruction(&Instruction::Else);
        json_i32(lone, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        unit.load(f);
        f.instruction(&Instruction::I32Const(0xdc00));
        f.instruction(&Instruction::I32Sub);
        f.instruction(&Instruction::I32Const(0x3ff));
        f.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, f);
        json_i32(lone, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        unit.load(f);
        f.instruction(&Instruction::I32Const(32));
        f.instruction(&Instruction::I32LtU);
        lone.load(f);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        for code in [92, 117] {
            json_i32(hex, code, f);
            emit_quote_unit(&build, used, hex, s, f);
        }
        for shift in [12, 8, 4, 0] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(shift));
            f.instruction(&Instruction::I32ShrU);
            f.instruction(&Instruction::I32Const(15));
            f.instruction(&Instruction::I32And);
            hex.store(f);
            hex.load(f);
            f.instruction(&Instruction::I32Const(10));
            f.instruction(&Instruction::I32LtU);
            self.open_frame(ControlFrameKind::If, f);
            hex.load(f);
            f.instruction(&Instruction::I32Const(48));
            f.instruction(&Instruction::I32Add);
            hex.store(f);
            f.instruction(&Instruction::Else);
            hex.load(f);
            f.instruction(&Instruction::I32Const(87));
            f.instruction(&Instruction::I32Add);
            hex.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            emit_quote_unit(&build, used, hex, s, f);
        }
        f.instruction(&Instruction::Else);
        emit_quote_unit(&build, used, unit, s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        json_i32(unit, 34, f);
        emit_quote_unit(&build, used, unit, s, f);
        let full = s.reserve_gc_local(f).initialize(build.publish(s, f), f);
        json_i64(index, 0, f);
        used.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.store(f);
        let result = s
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(&full, index, length, f), f);
        full.clear(f);
        s.release_i32_local(hex, f);
        s.release_i32_local(lone, f);
        s.release_i32_local(escape, f);
        s.release_i32_local(next_unit, f);
        s.release_i32_local(unit, f);
        s.release_i64_local(index, f);
        s.release_i32_local(used, f);
        s.release_i32_local(capacity, f);
        s.release_i64_local(length, f);
        Ok(result)
    }
}
fn emit_quote_unit(
    build: &StringConstruction,
    index: I32Local,
    unit: I32Local,
    s: &RuntimeSchema,
    f: &mut Function,
) {
    build.write(index, unit, s, f);
    index.load(f);
    f.instruction(&Instruction::I32Const(1));
    f.instruction(&Instruction::I32Add);
    index.store(f);
}
