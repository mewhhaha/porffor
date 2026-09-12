//! Base64 encoding over the Uint8Array's validated current byte range.

use super::super::*;
use super::uint8array_codecs::{
    Uint8ArrayBase64Alphabet, Uint8ArrayCodecAccess, Uint8ArrayCodecOption,
};

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_uint8_array_to_base64(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let option_payload_local = self.reserve_temp_local();
        let option_tag_local = self.reserve_temp_local();
        let alphabet_local = self.reserve_temp_local();
        let omit_padding_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let source_length_local = self.reserve_temp_local();
        let output_pointer_local = self.reserve_temp_local();
        let output_length_local = self.reserve_temp_local();
        let output_cursor_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let packed_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        self.emit_uint8_array_codec_receiver(
            receiver_local,
            Uint8ArrayCodecAccess::Read,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
        let options = self.emit_uint8_array_codec_options(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_uint8_array_base64_alphabet(&options, alphabet_local, function)?;
        self.emit_uint8_array_codec_option(
            &options,
            Uint8ArrayCodecOption::OmitPadding,
            option_payload_local,
            option_tag_local,
            function,
        )?;
        self.emit_to_boolean_payload_from_tagged_locals(
            option_tag_local,
            option_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(omit_padding_local));
        self.emit_uint8_array_codec_bytes(
            receiver_local,
            source_pointer_local,
            source_length_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalGet(omit_padding_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_length_local));
        function.instruction(&Instruction::LocalGet(output_length_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Base64 output is too large",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(output_length_local, function)?;
        function.instruction(&Instruction::LocalTee(output_pointer_local));
        function.instruction(&Instruction::LocalSet(output_cursor_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(packed_local));
        // Shared bytes are read once, in increasing order, after all option effects.
        for byte_index in 0..3 {
            function.instruction(&Instruction::LocalGet(remaining_local));
            function.instruction(&Instruction::I64Const(byte_index));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(packed_local));
            function.instruction(&Instruction::LocalGet(source_pointer_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(
                self.buffer_memarg8(byte_index as u64),
            ));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(16 - byte_index * 8));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(packed_local));
            function.instruction(&Instruction::End);
        }
        for digit_index in 0..4 {
            if digit_index >= 2 {
                function.instruction(&Instruction::LocalGet(remaining_local));
                function.instruction(&Instruction::I64Const(digit_index - 1));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            function.instruction(&Instruction::LocalGet(packed_local));
            function.instruction(&Instruction::I64Const(18 - digit_index * 6));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Const(63));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(digit_local));
            self.emit_append_uint8_array_base64_digit(
                digit_local,
                alphabet_local,
                output_cursor_local,
                function,
            );
            if digit_index >= 2 {
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(omit_padding_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(output_cursor_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Const(b'=' as i32));
                function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
                function.instruction(&Instruction::LocalGet(output_cursor_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(output_cursor_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_pack_string_payload(output_pointer_local, output_length_local, function);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(digit_local);
        self.release_temp_local(packed_local);
        self.release_temp_local(remaining_local);
        self.release_temp_local(index_local);
        self.release_temp_local(output_cursor_local);
        self.release_temp_local(output_length_local);
        self.release_temp_local(output_pointer_local);
        self.release_temp_local(source_length_local);
        self.release_temp_local(source_pointer_local);
        self.release_temp_local(omit_padding_local);
        self.release_temp_local(alphabet_local);
        self.release_temp_local(option_tag_local);
        self.release_temp_local(option_payload_local);
        self.release_temp_local(options_tag_local);
        self.release_temp_local(options_payload_local);
        self.release_temp_local(receiver_local);
        Ok(())
    }

    fn emit_append_uint8_array_base64_digit(
        &self,
        digit_local: u32,
        alphabet_local: u32,
        output_cursor_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(output_cursor_local));
        function.instruction(&Instruction::I32WrapI64);
        for (limit, offset) in [(26, 65), (52, 71), (62, -4)] {
            function.instruction(&Instruction::LocalGet(digit_local));
            function.instruction(&Instruction::I64Const(limit));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(digit_local));
            function.instruction(&Instruction::I64Const(offset));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::Else);
        }
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(62));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::LocalGet(alphabet_local));
        function.instruction(&Instruction::I64Const(
            Uint8ArrayBase64Alphabet::Base64Url.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(b'_' as i64));
        function.instruction(&Instruction::I64Const(b'/' as i64));
        function.instruction(&Instruction::LocalGet(alphabet_local));
        function.instruction(&Instruction::I64Const(
            Uint8ArrayBase64Alphabet::Base64Url.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::End);
        for _ in 0..3 {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(output_cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_cursor_local));
    }
}
