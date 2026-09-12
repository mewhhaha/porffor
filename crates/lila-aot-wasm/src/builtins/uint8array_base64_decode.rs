//! Base64 decoding shared by Uint8Array.fromBase64 and setFromBase64.

use super::super::*;
use super::uint8array_codecs::{
    Uint8ArrayBase64Alphabet, Uint8ArrayCodecAccess, Uint8ArrayCodecOption, Uint8ArrayCodecOptions,
};

enum Base64LastChunkHandling {
    Loose,
    Strict,
    StopBeforePartial,
}

impl Base64LastChunkHandling {
    const fn code(&self) -> i64 {
        match self {
            Self::Loose => 0,
            Self::Strict => 1,
            Self::StopBeforePartial => 2,
        }
    }
}

/// Temporary bytes belong to memory 0; an existing view uses buffer memory.
/// The same grammar commits complete chunks to either destination.
enum Base64DecodeDestination {
    Temporary {
        pointer_local: u32,
        capacity_local: u32,
    },
    Buffer {
        pointer_local: u32,
        capacity_local: u32,
    },
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_uint8_array_from_base64(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let source_length_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let alphabet_local = self.reserve_temp_local();
        let last_chunk_local = self.reserve_temp_local();
        let destination_local = self.reserve_temp_local();
        let read_local = self.reserve_temp_local();
        let written_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);
        self.emit_uint8_array_codec_string(
            source_payload_local,
            source_tag_local,
            source_pointer_local,
            source_length_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        let options = self.emit_uint8_array_codec_options(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_uint8_array_base64_alphabet(&options, alphabet_local, function)?;
        self.emit_uint8_array_base64_last_chunk(&options, last_chunk_local, function)?;

        // Decoded bytes never outnumber source bytes. The temporary allocation
        // does not create an observable ArrayBuffer before syntax validation.
        self.emit_heap_alloc_from_local(source_length_local, function)?;
        function.instruction(&Instruction::LocalSet(destination_local));
        self.emit_uint8_array_base64_decode(
            source_pointer_local,
            source_length_local,
            alphabet_local,
            last_chunk_local,
            Base64DecodeDestination::Temporary {
                pointer_local: destination_local,
                capacity_local: source_length_local,
            },
            read_local,
            written_local,
            function,
        )?;
        self.emit_uint8_array_codec_allocation(destination_local, written_local, function)?;

        self.release_temp_local(written_local);
        self.release_temp_local(read_local);
        self.release_temp_local(destination_local);
        self.release_temp_local(last_chunk_local);
        self.release_temp_local(alphabet_local);
        self.release_temp_local(options_tag_local);
        self.release_temp_local(options_payload_local);
        self.release_temp_local(source_length_local);
        self.release_temp_local(source_pointer_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        Ok(())
    }

    pub(super) fn emit_uint8_array_set_from_base64(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let source_length_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let alphabet_local = self.reserve_temp_local();
        let last_chunk_local = self.reserve_temp_local();
        let destination_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let read_local = self.reserve_temp_local();
        let written_local = self.reserve_temp_local();

        self.emit_uint8_array_codec_receiver(receiver_local, function)?;
        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);
        self.emit_uint8_array_codec_string(
            source_payload_local,
            source_tag_local,
            source_pointer_local,
            source_length_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        let options = self.emit_uint8_array_codec_options(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_uint8_array_base64_alphabet(&options, alphabet_local, function)?;
        self.emit_uint8_array_base64_last_chunk(&options, last_chunk_local, function)?;
        self.emit_uint8_array_codec_bytes(
            receiver_local,
            Uint8ArrayCodecAccess::Write,
            destination_local,
            capacity_local,
            function,
        )?;
        self.emit_uint8_array_base64_decode(
            source_pointer_local,
            source_length_local,
            alphabet_local,
            last_chunk_local,
            Base64DecodeDestination::Buffer {
                pointer_local: destination_local,
                capacity_local,
            },
            read_local,
            written_local,
            function,
        )?;
        self.emit_uint8_array_codec_result(read_local, written_local, function)?;

        self.release_temp_local(written_local);
        self.release_temp_local(read_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(destination_local);
        self.release_temp_local(last_chunk_local);
        self.release_temp_local(alphabet_local);
        self.release_temp_local(options_tag_local);
        self.release_temp_local(options_payload_local);
        self.release_temp_local(source_length_local);
        self.release_temp_local(source_pointer_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(receiver_local);
        Ok(())
    }

    fn emit_uint8_array_base64_last_chunk(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        last_chunk_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let literal_local = self.reserve_temp_local();
        self.emit_uint8_array_codec_option(
            options,
            Uint8ArrayCodecOption::LastChunkHandling,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(last_chunk_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            Base64LastChunkHandling::Loose.code(),
        ));
        function.instruction(&Instruction::LocalSet(last_chunk_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (name, mode) in [
            ("loose", Base64LastChunkHandling::Loose),
            ("strict", Base64LastChunkHandling::Strict),
            (
                "stop-before-partial",
                Base64LastChunkHandling::StopBeforePartial,
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(literal_local));
            self.emit_string_payload_equality_i32(payload_local, literal_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(mode.code()));
            function.instruction(&Instruction::LocalSet(last_chunk_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(last_chunk_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Uint8Array base64 lastChunkHandling must be loose, strict, or stop-before-partial",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(literal_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn emit_uint8_array_base64_syntax_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Invalid base64 string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_uint8_array_base64_skip_whitespace(
        &self,
        source_pointer_local: u32,
        source_length_local: u32,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(source_pointer_local, index_local, byte_local, function);
        for (index, byte) in [b'\t', b'\n', 0x0c, b'\r', b' '].into_iter().enumerate() {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            if index != 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_uint8_array_base64_chunk(
        &self,
        destination_local: u32,
        memory_index: u32,
        chunk_local: u32,
        chunk_length_local: u32,
        written_local: u32,
        function: &mut Function,
    ) {
        for (length, shifts) in [(2, &[4][..]), (3, &[10, 2][..]), (4, &[16, 8, 0][..])] {
            function.instruction(&Instruction::LocalGet(chunk_length_local));
            function.instruction(&Instruction::I64Const(length));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            for &shift in shifts {
                function.instruction(&Instruction::LocalGet(destination_local));
                function.instruction(&Instruction::LocalGet(written_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::LocalGet(chunk_local));
                function.instruction(&Instruction::I64Const(shift));
                function.instruction(&Instruction::I64ShrU);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Store8(Self::memarg8_in(memory_index, 0)));
                function.instruction(&Instruction::LocalGet(written_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(written_local));
            }
            function.instruction(&Instruction::End);
        }
    }

    fn emit_uint8_array_base64_decode(
        &mut self,
        source_pointer_local: u32,
        source_length_local: u32,
        alphabet_local: u32,
        last_chunk_local: u32,
        destination: Base64DecodeDestination,
        read_local: u32,
        written_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (destination_local, capacity_local, memory_index) = match destination {
            Base64DecodeDestination::Temporary {
                pointer_local,
                capacity_local,
            } => (pointer_local, capacity_local, 0),
            Base64DecodeDestination::Buffer {
                pointer_local,
                capacity_local,
            } => (pointer_local, capacity_local, self.buffer_memory_index()),
        };
        let index_local = self.reserve_temp_local();
        let chunk_local = self.reserve_temp_local();
        let chunk_length_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        for local in [
            index_local,
            chunk_local,
            chunk_length_local,
            read_local,
            written_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_uint8_array_base64_skip_whitespace(
            source_pointer_local,
            source_length_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(last_chunk_local));
        function.instruction(&Instruction::I64Const(
            Base64LastChunkHandling::StopBeforePartial.code(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(last_chunk_local));
        function.instruction(&Instruction::I64Const(
            Base64LastChunkHandling::Strict.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.emit_uint8_array_base64_chunk(
            destination_local,
            memory_index,
            chunk_local,
            chunk_length_local,
            written_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalSet(read_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        // skip_whitespace leaves the first non-whitespace byte in byte_local.
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.emit_uint8_array_base64_skip_whitespace(
            source_pointer_local,
            source_length_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(last_chunk_local));
        function.instruction(&Instruction::I64Const(
            Base64LastChunkHandling::StopBeforePartial.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(5));
        function.instruction(&Instruction::End);
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_uint8_array_base64_skip_whitespace(
            source_pointer_local,
            source_length_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(last_chunk_local));
        function.instruction(&Instruction::I64Const(
            Base64LastChunkHandling::Strict.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(chunk_local));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_uint8_array_base64_chunk(
            destination_local,
            memory_index,
            chunk_local,
            chunk_length_local,
            written_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_length_local));
        function.instruction(&Instruction::LocalSet(read_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(digit_local));
        for (first, last, base) in [(b'A', b'Z', 0), (b'a', b'z', 26), (b'0', b'9', 52)] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(first as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(last as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(first as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(base));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(digit_local));
            function.instruction(&Instruction::End);
        }
        for (alphabet, letters) in [
            (Uint8ArrayBase64Alphabet::Base64, [b'+', b'/']),
            (Uint8ArrayBase64Alphabet::Base64Url, [b'-', b'_']),
        ] {
            function.instruction(&Instruction::LocalGet(alphabet_local));
            function.instruction(&Instruction::I64Const(alphabet.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            for (index, letter) in letters.into_iter().enumerate() {
                function.instruction(&Instruction::LocalGet(byte_local));
                function.instruction(&Instruction::I64Const(letter as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(62 + index as i64));
                function.instruction(&Instruction::LocalSet(digit_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_syntax_error(function)?;
        function.instruction(&Instruction::End);

        // Validation precedes the lookahead capacity stop: AA# still throws
        // for a one-byte destination, while AAA leaves it unchanged.
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::LocalGet(written_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        for (index, remaining) in [1, 2].into_iter().enumerate() {
            function.instruction(&Instruction::LocalGet(remaining_local));
            function.instruction(&Instruction::I64Const(remaining));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(chunk_length_local));
            function.instruction(&Instruction::I64Const(remaining + 1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            if index != 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(chunk_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(chunk_local));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(chunk_length_local));
        function.instruction(&Instruction::LocalGet(chunk_length_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_uint8_array_base64_chunk(
            destination_local,
            memory_index,
            chunk_local,
            chunk_length_local,
            written_local,
            function,
        );
        for local in [chunk_local, chunk_length_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(read_local));
        function.instruction(&Instruction::LocalGet(written_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(remaining_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(chunk_length_local);
        self.release_temp_local(chunk_local);
        self.release_temp_local(index_local);
        Ok(())
    }
}
