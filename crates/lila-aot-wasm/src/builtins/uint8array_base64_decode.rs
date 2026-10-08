//! Base64 decoding owns a completed byte prefix and its independent error.
use super::super::*;
use super::uint8array_codecs::*;
use crate::gc_types::*;

impl FunctionBuilder<'_> {
    fn emit_uint8_base64_skip_space(
        &mut self,
        source: &Uint8ArrayCodecString,
        index: I64Local,
        unit: I32Local,
        f: &mut Function,
    ) {
        let done = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        source.length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        self.emit_uint8_codec_string_unit(source, index, unit, f);
        f.instruction(&Instruction::I32Const(0));
        for space in [0x09, 0x0a, 0x0c, 0x0d, 0x20] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(space));
            f.instruction(&Instruction::I32Eq);
            f.instruction(&Instruction::I32Or);
        }
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(done, f);
        self.emit_increment_local(index, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
    }
    fn emit_uint8_base64_digit(
        &mut self,
        unit: I32Local,
        alphabet: &Base64AlphabetLocal,
        digit: I32Local,
        f: &mut Function,
    ) {
        f.instruction(&Instruction::I32Const(-1));
        digit.store(f);
        for (start, end, offset) in [(b'A', b'Z', 0), (b'a', b'z', 26), (b'0', b'9', 52)] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(i32::from(start)));
            f.instruction(&Instruction::I32Sub);
            f.instruction(&Instruction::I32Const(i32::from(end - start)));
            f.instruction(&Instruction::I32LeU);
            self.open_frame(ControlFrameKind::If, f);
            unit.load(f);
            f.instruction(&Instruction::I32Const(offset - i32::from(start)));
            f.instruction(&Instruction::I32Add);
            digit.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        for (char, domain, number) in [
            (b'+', Uint8ArrayBase64Alphabet::Base64, 62),
            (b'/', Uint8ArrayBase64Alphabet::Base64, 63),
            (b'-', Uint8ArrayBase64Alphabet::Base64Url, 62),
            (b'_', Uint8ArrayBase64Alphabet::Base64Url, 63),
        ] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(i32::from(char)));
            f.instruction(&Instruction::I32Eq);
            alphabet.load(f);
            f.instruction(&Instruction::I32Const(domain.code()));
            f.instruction(&Instruction::I32Eq);
            f.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::I32Const(number));
            digit.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
    }
    fn emit_uint8_base64_final_chunk(
        &mut self,
        result: &Uint8ArrayDecodedBytes,
        packed: I32Local,
        chunk: I32Local,
        strict: I32Local,
        byte: I32Local,
        done: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        chunk.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        packed.load(f);
        f.instruction(&Instruction::I32Const(15));
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Eqz);
        strict.load(f);
        f.instruction(&Instruction::I32And);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            result,
            done,
            f,
        )?;
        packed.load(f);
        f.instruction(&Instruction::I32Const(4));
        f.instruction(&Instruction::I32ShrU);
        f.instruction(&Instruction::I32Const(255));
        f.instruction(&Instruction::I32And);
        byte.store(f);
        self.emit_uint8_codec_push_byte(result, byte, f);
        f.instruction(&Instruction::Else);
        packed.load(f);
        f.instruction(&Instruction::I32Const(3));
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Eqz);
        strict.load(f);
        f.instruction(&Instruction::I32And);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            result,
            done,
            f,
        )?;
        for shift in [10, 2] {
            packed.load(f);
            f.instruction(&Instruction::I32Const(shift));
            f.instruction(&Instruction::I32ShrU);
            f.instruction(&Instruction::I32Const(255));
            f.instruction(&Instruction::I32And);
            byte.store(f);
            self.emit_uint8_codec_push_byte(result, byte, f);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_uint8_base64_decode(
        &mut self,
        source: &Uint8ArrayCodecString,
        alphabet: &Base64AlphabetLocal,
        mode: &Base64LastChunkLocal,
        maximum: I64Local,
        f: &mut Function,
    ) -> Result<Uint8ArrayDecodedBytes, EmitError> {
        let s = self.runtime_schema();
        let result = self.emit_uint8_codec_decoded_list(source.length, maximum, f);
        let index = s.reserve_i64_local(f);
        let remaining = s.reserve_i64_local(f);
        let unit = s.reserve_i32_local(f);
        let digit = s.reserve_i32_local(f);
        let packed = s.reserve_i32_local(f);
        let chunk = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let strict = s.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I32Const(0));
        packed.store(f);
        f.instruction(&Instruction::I32Const(0));
        chunk.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        maximum.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(done, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_uint8_base64_skip_space(source, index, unit, f);
        index.load(f);
        source.length.load(f);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        chunk.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        mode.load(f);
        f.instruction(&Instruction::I32Const(
            Uint8ArrayLastChunk::StopBeforePartial.code(),
        ));
        f.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(done, f);
        mode.load(f);
        f.instruction(&Instruction::I32Const(Uint8ArrayLastChunk::Strict.code()));
        f.instruction(&Instruction::I32Eq);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            &result,
            done,
            f,
        )?;
        f.instruction(&Instruction::I32Const(0));
        strict.store(f);
        self.emit_uint8_base64_final_chunk(&result, packed, chunk, strict, byte, done, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        source.length.load(f);
        result.read.store(f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);

        self.emit_uint8_codec_string_unit(source, index, unit, f);
        self.emit_increment_local(index, 1, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'=')));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32LtU);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            &result,
            done,
            f,
        )?;
        self.emit_uint8_base64_skip_space(source, index, unit, f);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        index.load(f);
        source.length.load(f);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        mode.load(f);
        f.instruction(&Instruction::I32Const(
            Uint8ArrayLastChunk::StopBeforePartial.code(),
        ));
        f.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(done, f);
        f.instruction(&Instruction::I32Const(1));
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            &result,
            done,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_uint8_codec_string_unit(source, index, unit, f);
        unit.load(f);
        f.instruction(&Instruction::I32Const(i32::from(b'=')));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_increment_local(index, 1, f);
        self.emit_uint8_base64_skip_space(source, index, unit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        index.load(f);
        source.length.load(f);
        f.instruction(&Instruction::I64LtU);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            &result,
            done,
            f,
        )?;
        mode.load(f);
        f.instruction(&Instruction::I32Const(Uint8ArrayLastChunk::Strict.code()));
        f.instruction(&Instruction::I32Eq);
        strict.store(f);
        self.emit_uint8_base64_final_chunk(&result, packed, chunk, strict, byte, done, f)?;
        source.length.load(f);
        result.read.store(f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);

        self.emit_uint8_base64_digit(unit, alphabet, digit, f);
        digit.load(f);
        f.instruction(&Instruction::I32Const(0));
        f.instruction(&Instruction::I32LtS);
        self.emit_uint8_codec_syntax_error_if(
            RuntimeErrorMessage::INVALID_BASE64_STRING,
            &result,
            done,
            f,
        )?;
        maximum.load(f);
        result.written.load(f);
        f.instruction(&Instruction::I64Sub);
        remaining.store(f);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32And);
        remaining.load(f);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Eq);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(3));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        self.emit_branch_if_to_target(done, f);
        packed.load(f);
        f.instruction(&Instruction::I32Const(6));
        f.instruction(&Instruction::I32Shl);
        digit.load(f);
        f.instruction(&Instruction::I32Or);
        packed.store(f);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        chunk.store(f);
        chunk.load(f);
        f.instruction(&Instruction::I32Const(4));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        for shift in [16, 8, 0] {
            packed.load(f);
            f.instruction(&Instruction::I32Const(shift));
            f.instruction(&Instruction::I32ShrU);
            f.instruction(&Instruction::I32Const(255));
            f.instruction(&Instruction::I32And);
            byte.store(f);
            self.emit_uint8_codec_push_byte(&result, byte, f);
        }
        f.instruction(&Instruction::I32Const(0));
        chunk.store(f);
        f.instruction(&Instruction::I32Const(0));
        packed.store(f);
        index.load(f);
        result.read.store(f);
        result.written.load(f);
        maximum.load(f);
        f.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        for local in [strict, byte, chunk, packed, digit, unit] {
            s.release_i32_local(local, f);
        }
        s.release_i64_local(remaining, f);
        s.release_i64_local(index, f);
        Ok(result)
    }
    pub(super) fn emit_uint8_array_from_base64(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let input = s.reserve_value_local(f);
        let option_arg = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let maximum = s.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_builtin_arg_to_value(1, &option_arg, f);
        output.initialize(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let source = self.emit_uint8_codec_string(&input, &output, done, f)?;
        let options = self.emit_uint8_codec_options(&option_arg, &output, done, f)?;
        let alphabet = self.emit_uint8_codec_alphabet(&options, &pending, &output, done, f)?;
        let mode = self.emit_uint8_codec_last_chunk(&options, &pending, &output, done, f)?;
        f.instruction(&Instruction::I64Const((1_i64 << 53) - 1));
        maximum.store(f);
        let result = self.emit_uint8_base64_decode(&source, &alphabet, &mode, maximum, f)?;
        self.emit_uint8_codec_static_result(&result, &pending, &output, done, f)?;
        result.clear(s, f);
        mode.clear(s, f);
        alphabet.clear(s, f);
        options.clear(f);
        source.clear(s, f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        s.release_i64_local(maximum, f);
        output.clear(f);
        pending.clear(f);
        option_arg.clear(f);
        input.clear(f);
        Ok(())
    }
    pub(super) fn emit_uint8_array_set_from_base64(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let input = s.reserve_value_local(f);
        let option_arg = s.reserve_value_local(f);
        let this = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        self.compile_this_to_locals(&this, f)?;
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_builtin_arg_to_value(1, &option_arg, f);
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
        let options = self.emit_uint8_codec_options(&option_arg, &output, done, f)?;
        let alphabet = self.emit_uint8_codec_alphabet(&options, &pending, &output, done, f)?;
        let mode = self.emit_uint8_codec_last_chunk(&options, &pending, &output, done, f)?;
        self.emit_validate_typed_array_view(&receiver.object, length, &pending, f)?;
        self.emit_uint8_codec_abrupt_exit(&pending, &output, done, f);
        let result = self.emit_uint8_base64_decode(&source, &alphabet, &mode, length, f)?;
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
        mode.clear(s, f);
        alphabet.clear(s, f);
        options.clear(f);
        source.clear(s, f);
        receiver.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        s.release_i64_local(length, f);
        output.clear(f);
        pending.clear(f);
        this.clear(f);
        option_arg.clear(f);
        input.clear(f);
        Ok(())
    }
}
