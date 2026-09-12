//! Uint8Array hexadecimal conversion through validated byte storage.

use super::super::*;
use super::uint8array_codecs::Uint8ArrayCodecAccess;

enum HexDecodeDestination {
    Temporary {
        pointer_local: u32,
    },
    Buffer {
        pointer_local: u32,
        capacity_local: u32,
    },
}

impl FunctionBuilder<'_> {
    fn emit_uint8_array_hex_decode(
        &mut self,
        source_pointer_local: u32,
        source_byte_length_local: u32,
        destination: HexDecodeDestination,
        read_local: u32,
        written_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_length_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let high_local = self.reserve_temp_local();
        let low_local = self.reserve_temp_local();

        // Parity covers the entire UTF-16 string, including an unconsumed
        // suffix. Byte length would misclassify inputs such as "aaé".
        self.emit_utf16_code_unit_len_from_utf8_locals(
            source_pointer_local,
            source_byte_length_local,
            string_length_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(string_length_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Hexadecimal string length must be even",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let (destination_pointer_local, memory_index) = match destination {
            HexDecodeDestination::Temporary { pointer_local } => {
                function.instruction(&Instruction::LocalGet(string_length_local));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64DivU);
                function.instruction(&Instruction::LocalSet(capacity_local));
                self.emit_heap_alloc_from_local(capacity_local, function)?;
                function.instruction(&Instruction::LocalSet(pointer_local));
                (pointer_local, 0)
            }
            HexDecodeDestination::Buffer {
                pointer_local,
                capacity_local: destination_capacity_local,
            } => {
                function.instruction(&Instruction::LocalGet(destination_capacity_local));
                function.instruction(&Instruction::LocalSet(capacity_local));
                (pointer_local, self.buffer_memory_index())
            }
        };
        for local in [read_local, written_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read_local));
        function.instruction(&Instruction::LocalGet(string_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(written_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        // Every successfully consumed code unit is ASCII, so read remains
        // both the UTF-16 position and the byte offset until the first error.
        for (delta, nibble_local) in [(0, high_local), (1, low_local)] {
            self.emit_load_string_byte_at_delta(
                source_pointer_local,
                read_local,
                delta,
                byte_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(i64::from(b'0')));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalTee(nibble_local));
            function.instruction(&Instruction::I64Const(9));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(0x20));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::I64Const(i64::from(b'a')));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalTee(nibble_local));
            function.instruction(&Instruction::I64Const(5));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(nibble_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(nibble_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(high_local));
        function.instruction(&Instruction::LocalGet(low_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Invalid hexadecimal digit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(destination_pointer_local));
        function.instruction(&Instruction::LocalGet(written_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(high_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(low_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8_in(memory_index, 0)));
        for (local, increment) in [(read_local, 2), (written_local, 1)] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(increment));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            low_local,
            high_local,
            byte_local,
            capacity_local,
            string_length_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_uint8_array_from_hex(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let source_byte_length_local = self.reserve_temp_local();
        let temporary_pointer_local = self.reserve_temp_local();
        let read_local = self.reserve_temp_local();
        let written_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, payload_local, tag_local, function);
        self.emit_uint8_array_codec_string(
            payload_local,
            tag_local,
            source_pointer_local,
            source_byte_length_local,
            function,
        )?;
        self.emit_uint8_array_hex_decode(
            source_pointer_local,
            source_byte_length_local,
            HexDecodeDestination::Temporary {
                pointer_local: temporary_pointer_local,
            },
            read_local,
            written_local,
            function,
        )?;
        self.emit_uint8_array_codec_allocation(temporary_pointer_local, written_local, function)?;
        for local in [
            written_local,
            read_local,
            temporary_pointer_local,
            source_byte_length_local,
            source_pointer_local,
            tag_local,
            payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_uint8_array_set_from_hex(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let source_byte_length_local = self.reserve_temp_local();
        let destination_pointer_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let read_local = self.reserve_temp_local();
        let written_local = self.reserve_temp_local();
        self.emit_uint8_array_codec_receiver(
            receiver_local,
            Uint8ArrayCodecAccess::Write,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, payload_local, tag_local, function);
        self.emit_uint8_array_codec_string(
            payload_local,
            tag_local,
            source_pointer_local,
            source_byte_length_local,
            function,
        )?;
        self.emit_uint8_array_codec_bytes(
            receiver_local,
            destination_pointer_local,
            capacity_local,
            function,
        )?;
        self.emit_uint8_array_hex_decode(
            source_pointer_local,
            source_byte_length_local,
            HexDecodeDestination::Buffer {
                pointer_local: destination_pointer_local,
                capacity_local,
            },
            read_local,
            written_local,
            function,
        )?;
        self.emit_uint8_array_codec_result(read_local, written_local, function)?;
        for local in [
            written_local,
            read_local,
            capacity_local,
            destination_pointer_local,
            source_byte_length_local,
            source_pointer_local,
            tag_local,
            payload_local,
            receiver_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_uint8_array_to_hex(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_local = self.reserve_temp_local();
        let source_pointer_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let output_pointer_local = self.reserve_temp_local();
        let output_length_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let nibble_local = self.reserve_temp_local();
        self.emit_uint8_array_codec_receiver(
            receiver_local,
            Uint8ArrayCodecAccess::Read,
            function,
        )?;
        self.emit_uint8_array_codec_bytes(
            receiver_local,
            source_pointer_local,
            length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // The packed string representation has a 32-bit byte-length field.
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(i64::from(u32::MAX / 2)));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Hexadecimal output is too large",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(output_length_local));
        self.emit_heap_alloc_from_local(output_length_local, function)?;
        function.instruction(&Instruction::LocalSet(output_pointer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(source_pointer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(self.buffer_memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        for (shift, delta) in [(4, 0), (0, 1)] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::I64Const(15));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalTee(nibble_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::LocalGet(nibble_local));
            function.instruction(&Instruction::I64Const(i64::from(b'0')));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(nibble_local));
            function.instruction(&Instruction::I64Const(i64::from(b'a') - 10));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalSet(nibble_local));
            function.instruction(&Instruction::LocalGet(output_pointer_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(delta));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(nibble_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(output_pointer_local, output_length_local, function);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        for local in [
            nibble_local,
            byte_local,
            index_local,
            output_length_local,
            output_pointer_local,
            length_local,
            source_pointer_local,
            receiver_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
