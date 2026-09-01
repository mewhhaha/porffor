use super::*;

enum GlobalAsciiClassQuantifier {
    DigitOnce,
    DigitTwice,
    NonDigitTwice,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_string_match_global_ascii_digit_once_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            string_local,
            GlobalAsciiClassQuantifier::DigitOnce,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_string_match_global_ascii_digit_twice_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            string_local,
            GlobalAsciiClassQuantifier::DigitTwice,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_string_match_global_ascii_non_digit_twice_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_global_ascii_class_quantifier_from_string_locals(
            string_local,
            GlobalAsciiClassQuantifier::NonDigitTwice,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_string_match_global_ascii_class_quantifier_from_string_locals(
        &mut self,
        string_local: u32,
        class_quantifier: GlobalAsciiClassQuantifier,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let result_array_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let probe_index_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Const(match &class_quantifier {
            GlobalAsciiClassQuantifier::DigitOnce => 1,
            GlobalAsciiClassQuantifier::DigitTwice | GlobalAsciiClassQuantifier::NonDigitTwice => 2,
        }));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_load_string_byte(
            src_offset_local,
            probe_index_local,
            first_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            probe_index_local,
            src_len_local,
            first_byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        match &class_quantifier {
            GlobalAsciiClassQuantifier::DigitOnce | GlobalAsciiClassQuantifier::DigitTwice => {}
            GlobalAsciiClassQuantifier::NonDigitTwice => {
                function.instruction(&Instruction::I32Eqz);
            }
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(probe_index_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            scan_index_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_array_write(
            result_array_local,
            write_index_local,
            match_payload_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(probe_index_local));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(
            src_offset_local,
            scan_index_local,
            first_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            scan_index_local,
            src_len_local,
            first_byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(string_tag_local);
        self.release_temp_local(match_len_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(match_local);
        self.release_temp_local(compare_index_local);
        self.release_temp_local(probe_index_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(result_array_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }
}
