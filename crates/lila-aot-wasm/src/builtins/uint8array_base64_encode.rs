//! Base64 encoding snapshots each unordered Uint8 access once, after options.
use super::super::*;
use super::uint8array_codecs::*;
use crate::gc_types::*;

impl FunctionBuilder<'_> {
    fn emit_uint8_base64_encoded_digit(
        &mut self,
        digit: I32Local,
        alphabet: &Base64AlphabetLocal,
        unit: I32Local,
        f: &mut Function,
    ) {
        digit.load(f);
        f.instruction(&Instruction::I32Const(26));
        f.instruction(&Instruction::I32LtU);
        self.open_frame(ControlFrameKind::If, f);
        digit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'A')));
        f.instruction(&Instruction::I32Add);
        unit.store(f);
        f.instruction(&Instruction::Else);
        digit.load(f);
        f.instruction(&Instruction::I32Const(52));
        f.instruction(&Instruction::I32LtU);
        self.open_frame(ControlFrameKind::If, f);
        digit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'a') - 26));
        f.instruction(&Instruction::I32Add);
        unit.store(f);
        f.instruction(&Instruction::Else);
        digit.load(f);
        f.instruction(&Instruction::I32Const(62));
        f.instruction(&Instruction::I32LtU);
        self.open_frame(ControlFrameKind::If, f);
        digit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'0') - 52));
        f.instruction(&Instruction::I32Add);
        unit.store(f);
        f.instruction(&Instruction::Else);
        digit.load(f);
        f.instruction(&Instruction::I32Const(62));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I32Const(i32::from(b'-')));
        f.instruction(&Instruction::I32Const(i32::from(b'+')));
        alphabet.load(f);
        f.instruction(&Instruction::I32Const(
            Uint8ArrayBase64Alphabet::Base64Url.code(),
        ));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::Select);
        unit.store(f);
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I32Const(i32::from(b'_')));
        f.instruction(&Instruction::I32Const(i32::from(b'/')));
        alphabet.load(f);
        f.instruction(&Instruction::I32Const(
            Uint8ArrayBase64Alphabet::Base64Url.code(),
        ));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::Select);
        unit.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
    }
    fn emit_uint8_base64_append_unit(
        &self,
        construction: &StringConstruction,
        index: I64Local,
        at: I32Local,
        unit: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        construction.write(at, unit, s, f);
        self.emit_increment_local(index, 1, f);
    }
    pub(super) fn emit_uint8_array_to_base64(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let this = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        let output_length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let cursor = s.reserve_i64_local(f);
        let remaining = s.reserve_i64_local(f);
        let source_index = s.reserve_i64_local(f);
        let omit = s.reserve_i32_local(f);
        let size = s.reserve_i32_local(f);
        let at = s.reserve_i32_local(f);
        let packed = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let digit = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        self.compile_this_to_locals(&this, f)?;
        self.emit_builtin_arg_to_value(0, &argument, f);
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
        let options = self.emit_uint8_codec_options(&argument, &output, done, f)?;
        let alphabet = self.emit_uint8_codec_alphabet(&options, &pending, &output, done, f)?;
        self.emit_uint8_codec_option(&options, Uint8ArrayCodecOption::OmitPadding, &pending, f)?;
        self.emit_uint8_codec_abrupt_exit(&pending, &output, done, f);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        omit.store(f);
        let bytes =
            self.emit_uint8_codec_snapshot(&receiver, length, &pending, &output, done, f)?;
        length.load(f);
        f.instruction(&Instruction::I64Const(3));
        f.instruction(&Instruction::I64RemU);
        remaining.store(f);
        length.load(f);
        f.instruction(&Instruction::I64Const(3));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64Mul);
        output_length.store(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        output_length.load(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(4));
        omit.load(f);
        f.instruction(&Instruction::Select);
        f.instruction(&Instruction::I64Add);
        output_length.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output_length.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_range_error(
            RuntimeErrorMessage::BASE64_OUTPUT_IS_TOO_LARGE,
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
        index.store(f);
        f.instruction(&Instruction::I64Const(0));
        cursor.store(f);
        let end = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(end, f);
        length.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Sub);
        remaining.store(f);
        f.instruction(&Instruction::I32Const(0));
        packed.store(f);
        for ordinal in 0..3 {
            remaining.load(f);
            f.instruction(&Instruction::I64Const(ordinal));
            f.instruction(&Instruction::I64GtU);
            self.open_frame(ControlFrameKind::If, f);
            index.load(f);
            f.instruction(&Instruction::I64Const(ordinal));
            f.instruction(&Instruction::I64Add);
            source_index.store(f);
            source_index.load(f);
            f.instruction(&Instruction::I32WrapI64);
            at.store(f);
            s.array_type::<ByteArray>()
                .read(&bytes, at, s, f)
                .store(byte, f);
            packed.load(f);
            byte.load(f);
            f.instruction(&Instruction::I32Const((16 - ordinal * 8) as i32));
            f.instruction(&Instruction::I32Shl);
            f.instruction(&Instruction::I32Or);
            packed.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        for ordinal in 0..4 {
            if ordinal >= 2 {
                remaining.load(f);
                f.instruction(&Instruction::I64Const(ordinal - 1));
                f.instruction(&Instruction::I64GtU);
                self.open_frame(ControlFrameKind::If, f);
            }
            packed.load(f);
            f.instruction(&Instruction::I32Const((18 - ordinal * 6) as i32));
            f.instruction(&Instruction::I32ShrU);
            f.instruction(&Instruction::I32Const(63));
            f.instruction(&Instruction::I32And);
            digit.store(f);
            self.emit_uint8_base64_encoded_digit(digit, &alphabet, unit, f);
            self.emit_uint8_base64_append_unit(&construction, cursor, at, unit, f);
            if ordinal >= 2 {
                f.instruction(&Instruction::Else);
                omit.load(f);
                f.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, f);
                f.instruction(&Instruction::I32Const(i32::from(b'=')));
                unit.store(f);
                self.emit_uint8_base64_append_unit(&construction, cursor, at, unit, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
        }
        self.emit_increment_local(index, 3, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        cursor.load(f);
        output_length.load(f);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let text = s
            .reserve_gc_local(f)
            .initialize(construction.publish(s, f), f);
        output.value().set_reference(&text, s, f);
        output.set_kind(CompletionKind::Normal, f);
        text.clear(f);
        bytes.clear(f);
        alphabet.clear(s, f);
        options.clear(f);
        receiver.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        for local in [unit, digit, byte, packed, at, size, omit] {
            s.release_i32_local(local, f);
        }
        for local in [
            source_index,
            remaining,
            cursor,
            index,
            output_length,
            length,
        ] {
            s.release_i64_local(local, f);
        }
        output.clear(f);
        pending.clear(f);
        argument.clear(f);
        this.clear(f);
        Ok(())
    }
}
