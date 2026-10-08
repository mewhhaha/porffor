//! Hex decoding retains completed prefixes and writes before SyntaxError.
use super::super::*;
use super::uint8array_codecs::*;
use crate::gc_types::*;

impl FunctionBuilder<'_> {
    fn emit_uint8_hex_nibble(&mut self, unit: I32Local, nibble: I32Local, f: &mut Function) {
        unit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'0')));
        f.instruction(&Instruction::I32Sub);
        nibble.store(f);
        nibble.load(f);
        f.instruction(&Instruction::I32Const(9));
        f.instruction(&Instruction::I32GtU);
        self.open_frame(ControlFrameKind::If, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(0x20));
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Const(i32::from(b'a')));
        f.instruction(&Instruction::I32Sub);
        nibble.store(f);
        nibble.load(f);
        f.instruction(&Instruction::I32Const(5));
        f.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, f);
        nibble.load(f);
        f.instruction(&Instruction::I32Const(10));
        f.instruction(&Instruction::I32Add);
        nibble.store(f);
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I32Const(-1));
        nibble.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
    }
    fn emit_uint8_hex_decode(
        &mut self,
        source: &Uint8ArrayCodecString,
        maximum: I64Local,
        f: &mut Function,
    ) -> Result<Uint8ArrayDecodedBytes, EmitError> {
        let s = self.runtime_schema();
        let bound = s.reserve_i64_local(f);
        source.length.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64DivU);
        bound.store(f);
        // Odd UTF16 length fails even at zero destination capacity. Allocate
        // no payload for that path before constructing its SyntaxError.
        source.length.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        bound.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = self.emit_uint8_codec_decoded_list(bound, maximum, f);
        let unit = s.reserve_i32_local(f);
        let high = s.reserve_i32_local(f);
        let low = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let second = s.reserve_i64_local(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        source.length.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::HEXADECIMAL_STRING_LENGTH_MUST_BE_EVEN,
            &result,
            done,
            f,
        )?;
        let again = self.open_frame(ControlFrameKind::Loop, f);
        result.read.load(f);
        source.length.load(f);
        f.instruction(&Instruction::I64GeU);
        result.written.load(f);
        maximum.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::I32Or);
        self.emit_branch_if_to_target(done, f);
        self.emit_uint8_codec_string_unit(source, result.read, unit, f);
        self.emit_uint8_hex_nibble(unit, high, f);
        result.read.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        second.store(f);
        self.emit_uint8_codec_string_unit(source, second, unit, f);
        self.emit_uint8_hex_nibble(unit, low, f);
        high.load(f);
        low.load(f);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Const(0));
        f.instruction(&Instruction::I32LtS);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_HEXADECIMAL_DIGIT,
            &result,
            done,
            f,
        )?;
        high.load(f);
        f.instruction(&Instruction::I32Const(4));
        f.instruction(&Instruction::I32Shl);
        low.load(f);
        f.instruction(&Instruction::I32Or);
        byte.store(f);
        self.emit_uint8_codec_push_byte(&result, byte, f);
        self.emit_increment_local(result.read, 2, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i64_local(second, f);
        s.release_i32_local(byte, f);
        s.release_i32_local(low, f);
        s.release_i32_local(high, f);
        s.release_i32_local(unit, f);
        s.release_i64_local(bound, f);
        Ok(result)
    }
    pub(super) fn emit_uint8_array_from_hex(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let input = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let maximum = s.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        output.initialize(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let source = self.emit_uint8_codec_string(&input, &output, done, f)?;
        f.instruction(&Instruction::I64Const((1_i64 << 53) - 1));
        maximum.store(f);
        let result = self.emit_uint8_hex_decode(&source, maximum, f)?;
        self.emit_uint8_codec_static_result(&result, &pending, &output, done, f)?;
        result.clear(s, f);
        source.clear(s, f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        s.release_i64_local(maximum, f);
        output.clear(f);
        pending.clear(f);
        input.clear(f);
        Ok(())
    }
    pub(super) fn emit_uint8_array_set_from_hex(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let input = s.reserve_value_local(f);
        let this = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        self.compile_this_to_locals(&this, f)?;
        self.emit_builtin_arg_to_value(0, &input, f);
        output.initialize(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let receiver = self.emit_uint8_codec_receiver(
            Uint8ArrayCodecAccess::Write,
            &this,
            &pending,
            &output,
            done,
            f,
        )?;
        let source = self.emit_uint8_codec_string(&input, &output, done, f)?;
        self.emit_validate_typed_array_view(&receiver.object, length, &pending, f)?;
        self.emit_uint8_codec_abrupt_exit(&pending, &output, done, f);
        let result = self.emit_uint8_hex_decode(&source, length, f)?;
        self.emit_uint8_codec_copy_bytes(
            &receiver.object,
            &result.bytes,
            result.written,
            &pending,
            &output,
            done,
            f,
        )?;
        self.emit_uint8_codec_count_result(&result, &pending, &output, done, f)?;
        result.clear(s, f);
        source.clear(s, f);
        receiver.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        s.release_i64_local(length, f);
        output.clear(f);
        pending.clear(f);
        this.clear(f);
        input.clear(f);
        Ok(())
    }
    pub(super) fn emit_uint8_array_to_hex(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let this = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        let output_length = s.reserve_i64_local(f);
        let i = s.reserve_i64_local(f);
        let at = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        let size = s.reserve_i32_local(f);
        self.compile_this_to_locals(&this, f)?;
        output.initialize(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let receiver = self.emit_uint8_codec_receiver(
            Uint8ArrayCodecAccess::Read,
            &this,
            &pending,
            &output,
            done,
            f,
        )?;
        let bytes =
            self.emit_uint8_codec_snapshot(&receiver, length, &pending, &output, done, f)?;
        length.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Mul);
        output_length.store(f);
        output_length.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_range_error(
            RuntimeErrorMessage::HEXADECIMAL_OUTPUT_IS_TOO_LARGE,
            &output,
            f,
        )?;
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output_length.load(f);
        f.instruction(&Instruction::I32WrapI64);
        size.store(f);
        let construction = StringConstruction::allocate(s, s.reserve_gc_local(f), size, f);
        f.instruction(&Instruction::I64Const(0));
        i.store(f);
        let end = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        i.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(end, f);
        i.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        s.array_type::<ByteArray>()
            .read(&bytes, at, s, f)
            .store(byte, f);
        for (shift, delta) in [(4, 0), (0, 1)] {
            byte.load(f);
            f.instruction(&Instruction::I32Const(shift));
            f.instruction(&Instruction::I32ShrU);
            f.instruction(&Instruction::I32Const(15));
            f.instruction(&Instruction::I32And);
            unit.store(f);
            unit.load(f);
            f.instruction(&Instruction::I32Const(10));
            f.instruction(&Instruction::I32LtU);
            self.open_frame(ControlFrameKind::If, f);
            unit.load(f);
            f.instruction(&Instruction::I32Const(i32::from(b'0')));
            f.instruction(&Instruction::I32Add);
            unit.store(f);
            f.instruction(&Instruction::Else);
            unit.load(f);
            f.instruction(&Instruction::I32Const(i32::from(b'a') - 10));
            f.instruction(&Instruction::I32Add);
            unit.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            i.load(f);
            f.instruction(&Instruction::I64Const(2));
            f.instruction(&Instruction::I64Mul);
            f.instruction(&Instruction::I64Const(delta));
            f.instruction(&Instruction::I64Add);
            f.instruction(&Instruction::I32WrapI64);
            at.store(f);
            construction.write(at, unit, s, f);
        }
        self.emit_increment_local(i, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let text = s
            .reserve_gc_local(f)
            .initialize(construction.publish(s, f), f);
        output.value().set_reference(&text, s, f);
        output.set_kind(CompletionKind::Normal, f);
        text.clear(f);
        bytes.clear(f);
        receiver.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        for local in [size, unit, byte, at] {
            s.release_i32_local(local, f);
        }
        for local in [i, output_length, length] {
            s.release_i64_local(local, f);
        }
        output.clear(f);
        pending.clear(f);
        this.clear(f);
        Ok(())
    }
}
