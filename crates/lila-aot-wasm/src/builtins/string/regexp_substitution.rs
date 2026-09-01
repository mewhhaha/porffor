use super::*;

enum RegExpSubstitutionKind {
    LiteralDollar,
    MatchedSubstring,
    Prefix,
    Suffix,
    NumberedCapture,
    NamedCapture,
}

impl RegExpSubstitutionKind {
    const ALL: [Self; 6] = [
        Self::LiteralDollar,
        Self::MatchedSubstring,
        Self::Prefix,
        Self::Suffix,
        Self::NumberedCapture,
        Self::NamedCapture,
    ];

    const fn runtime_code(&self) -> i64 {
        match self {
            Self::LiteralDollar => 1,
            Self::MatchedSubstring => 2,
            Self::Prefix => 3,
            Self::Suffix => 4,
            Self::NumberedCapture => 5,
            Self::NamedCapture => 6,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_regexp_get_substitution(
        &mut self,
        replacement_string_local: u32,
        input_string_local: u32,
        match_string_local: u32,
        match_result_local: u32,
        match_result_tag_local: u32,
        match_result_len_local: u32,
        named_captures_local: u32,
        named_captures_tag_local: u32,
        position_local: u32,
        match_len_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let replacement_offset_local = self.reserve_temp_local();
        let replacement_len_local = self.reserve_temp_local();
        let replacement_index_local = self.reserve_temp_local();
        let literal_start_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let substitution_kind_local = self.reserve_temp_local();
        let piece_len_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let input_offset_local = self.reserve_temp_local();
        let input_byte_len_local = self.reserve_temp_local();
        let input_len_local = self.reserve_temp_local();
        let match_end_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let capture_index_local = self.reserve_temp_local();
        let capture_count_local = self.reserve_temp_local();
        let capture_payload_local = self.reserve_temp_local();
        let capture_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let consumed_local = self.reserve_temp_local();
        let candidate_local = self.reserve_temp_local();
        let group_end_local = self.reserve_temp_local();
        let group_name_start_local = self.reserve_temp_local();
        let group_name_len_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            replacement_string_local,
            replacement_offset_local,
            replacement_len_local,
            function,
        );
        self.emit_unpack_string_payload(
            input_string_local,
            input_offset_local,
            input_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            input_offset_local,
            input_byte_len_local,
            input_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(replacement_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(literal_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::LocalGet(match_result_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(match_result_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(capture_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(substitution_kind_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(consumed_local));
        function.instruction(&Instruction::LocalGet(replacement_offset_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(i32::from(b'$')));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_offset_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(next_byte_local));
        for (byte, kind) in [
            (b'$', RegExpSubstitutionKind::LiteralDollar),
            (b'&', RegExpSubstitutionKind::MatchedSubstring),
            (b'`', RegExpSubstitutionKind::Prefix),
            (b'\'', RegExpSubstitutionKind::Suffix),
        ] {
            function.instruction(&Instruction::LocalGet(next_byte_local));
            function.instruction(&Instruction::I64Const(i64::from(byte)));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(kind.runtime_code()));
            function.instruction(&Instruction::LocalSet(substitution_kind_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(i64::from(b'<')));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(named_captures_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(group_end_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(group_end_local));
        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(replacement_offset_local));
        function.instruction(&Instruction::LocalGet(group_end_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(i32::from(b'>')));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            RegExpSubstitutionKind::NamedCapture.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(substitution_kind_local));
        function.instruction(&Instruction::LocalGet(group_end_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(consumed_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(group_end_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(group_end_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(i64::from(b'9')));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next_byte_local));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(capture_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(consumed_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_offset_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(candidate_local));
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::I64Const(i64::from(b'9')));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture_index_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(candidate_local));
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::LocalGet(capture_count_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::LocalSet(capture_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(consumed_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(capture_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(capture_index_local));
        function.instruction(&Instruction::LocalGet(capture_count_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            RegExpSubstitutionKind::NumberedCapture.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(substitution_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(substitution_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(replacement_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::LocalGet(literal_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            replacement_string_local,
            literal_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(output_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        for kind in RegExpSubstitutionKind::ALL {
            function.instruction(&Instruction::LocalGet(substitution_kind_local));
            function.instruction(&Instruction::I64Const(kind.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match &kind {
                RegExpSubstitutionKind::LiteralDollar => {
                    function.instruction(&Instruction::I64Const(self.strings.payload("$")));
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                }
                RegExpSubstitutionKind::MatchedSubstring => {
                    function.instruction(&Instruction::LocalGet(match_string_local));
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                }
                RegExpSubstitutionKind::Prefix => {
                    self.emit_utf16_code_unit_range_payload_from_locals(
                        input_string_local,
                        zero_local,
                        position_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                }
                RegExpSubstitutionKind::Suffix => {
                    function.instruction(&Instruction::LocalGet(position_local));
                    function.instruction(&Instruction::LocalGet(match_len_local));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(match_end_local));
                    function.instruction(&Instruction::LocalGet(match_end_local));
                    function.instruction(&Instruction::LocalGet(input_len_local));
                    function.instruction(&Instruction::I64GtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(input_len_local));
                    function.instruction(&Instruction::LocalSet(match_end_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(input_len_local));
                    function.instruction(&Instruction::LocalGet(match_end_local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(piece_len_local));
                    self.emit_utf16_code_unit_range_payload_from_locals(
                        input_string_local,
                        match_end_local,
                        piece_len_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                }
                RegExpSubstitutionKind::NumberedCapture => {
                    self.emit_index_to_flat_map_key_local(
                        capture_index_local,
                        number_payload_local,
                        key_local,
                        function,
                    )?;
                    self.emit_object_read(
                        match_result_local,
                        match_result_tag_local,
                        match_result_local,
                        match_result_tag_local,
                        key_local,
                        capture_payload_local,
                        capture_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::LocalGet(capture_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(self.strings.payload("")));
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(
                        capture_payload_local,
                        capture_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::End);
                }
                RegExpSubstitutionKind::NamedCapture => {
                    function.instruction(&Instruction::LocalGet(replacement_index_local));
                    function.instruction(&Instruction::I64Const(2));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(group_name_start_local));
                    function.instruction(&Instruction::LocalGet(group_end_local));
                    function.instruction(&Instruction::LocalGet(group_name_start_local));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(group_name_len_local));
                    self.emit_string_slice_payload_from_locals(
                        replacement_string_local,
                        group_name_start_local,
                        group_name_len_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_read(
                        named_captures_local,
                        named_captures_tag_local,
                        named_captures_local,
                        named_captures_tag_local,
                        key_local,
                        capture_payload_local,
                        capture_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::LocalGet(capture_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(self.strings.payload("")));
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(
                        capture_payload_local,
                        capture_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(piece_payload_local));
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::End);
                }
            }
            self.emit_concat_string_payloads_local(output_local, piece_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::LocalGet(consumed_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(replacement_index_local));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::LocalSet(literal_start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::LocalGet(literal_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            replacement_string_local,
            literal_start_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(output_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        for local in [
            group_name_len_local,
            group_name_start_local,
            group_end_local,
            candidate_local,
            consumed_local,
            number_payload_local,
            key_local,
            capture_tag_local,
            capture_payload_local,
            capture_count_local,
            capture_index_local,
            zero_local,
            match_end_local,
            input_len_local,
            input_byte_len_local,
            input_offset_local,
            piece_payload_local,
            piece_len_local,
            substitution_kind_local,
            next_byte_local,
            literal_start_local,
            replacement_index_local,
            replacement_len_local,
            replacement_offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
