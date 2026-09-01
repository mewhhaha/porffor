use super::*;

enum StringLiteralReplacementScope {
    FirstOccurrence,
    AllOccurrences,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_string_replace_literal_first_occurrence_from_string_locals(
        &mut self,
        string_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        replacement_payload_local: u32,
        replacement_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_replace_literal_from_string_locals(
            StringLiteralReplacementScope::FirstOccurrence,
            string_local,
            search_payload_local,
            search_tag_local,
            replacement_payload_local,
            replacement_tag_local,
            function,
        )
    }

    pub(super) fn emit_string_replace_literal_all_occurrences_from_string_locals(
        &mut self,
        string_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        replacement_payload_local: u32,
        replacement_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_replace_literal_from_string_locals(
            StringLiteralReplacementScope::AllOccurrences,
            string_local,
            search_payload_local,
            search_tag_local,
            replacement_payload_local,
            replacement_tag_local,
            function,
        )
    }

    fn emit_string_replace_literal_from_string_locals(
        &mut self,
        scope: StringLiteralReplacementScope,
        string_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        replacement_payload_local: u32,
        replacement_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let search_string_local = self.reserve_temp_local();
        let replacement_string_local = self.reserve_temp_local();
        let source_offset_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let search_offset_local = self.reserve_temp_local();
        let search_len_local = self.reserve_temp_local();
        let replacement_offset_local = self.reserve_temp_local();
        let replacement_len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let last_end_local = self.reserve_temp_local();
        let compare_index_local = self.reserve_temp_local();
        let match_local = self.reserve_temp_local();
        let source_byte_local = self.reserve_temp_local();
        let search_byte_local = self.reserve_temp_local();
        let piece_len_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let functional_replacement_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let callback_string_local = self.reserve_temp_local();
        let position_local = self.reserve_temp_local();
        let position_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let substitution_local = self.reserve_temp_local();
        let replacement_index_local = self.reserve_temp_local();
        let literal_start_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let substitution_kind_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.emit_value_to_string_payload(search_payload_local, search_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(search_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(replacement_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(functional_replacement_local));
        function.instruction(&Instruction::LocalGet(functional_replacement_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(
            replacement_payload_local,
            replacement_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(replacement_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(
            string_local,
            source_offset_local,
            source_len_local,
            function,
        );
        self.emit_unpack_string_payload(
            search_string_local,
            search_offset_local,
            search_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(last_end_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::LocalGet(search_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(search_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::LocalGet(search_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(source_offset_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(source_byte_local));
        function.instruction(&Instruction::LocalGet(search_offset_local));
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(search_byte_local));
        function.instruction(&Instruction::LocalGet(source_byte_local));
        function.instruction(&Instruction::LocalGet(search_byte_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compare_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(match_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(last_end_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_end_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(self.result_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));

        function.instruction(&Instruction::LocalGet(functional_replacement_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_unpack_string_payload(
            replacement_string_local,
            replacement_offset_local,
            replacement_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(substitution_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(replacement_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(literal_start_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::LocalGet(replacement_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(substitution_kind_local));
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
        for (byte, kind) in [(b'$', 1), (b'&', 2), (b'`', 3), (b'\'', 4)] {
            function.instruction(&Instruction::LocalGet(next_byte_local));
            function.instruction(&Instruction::I64Const(i64::from(byte)));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(kind));
            function.instruction(&Instruction::LocalSet(substitution_kind_local));
            function.instruction(&Instruction::End);
        }
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
        self.emit_concat_string_payloads_local(substitution_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(substitution_local));
        for (kind, payload) in [
            (1, None),
            (2, Some(search_string_local)),
            (3, Some(string_local)),
            (4, Some(string_local)),
        ] {
            function.instruction(&Instruction::LocalGet(substitution_kind_local));
            function.instruction(&Instruction::I64Const(kind));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            if kind == 1 {
                function.instruction(&Instruction::I64Const(self.strings.payload("$")));
                function.instruction(&Instruction::LocalSet(piece_payload_local));
            } else if kind == 2 {
                function.instruction(&Instruction::LocalGet(payload.unwrap()));
                function.instruction(&Instruction::LocalSet(piece_payload_local));
            } else if kind == 3 {
                self.emit_string_slice_payload_from_locals(
                    payload.unwrap(),
                    zero_local,
                    scan_index_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(piece_payload_local));
            } else {
                function.instruction(&Instruction::LocalGet(source_len_local));
                function.instruction(&Instruction::LocalGet(scan_index_local));
                function.instruction(&Instruction::LocalGet(search_len_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(piece_len_local));
                function.instruction(&Instruction::LocalGet(scan_index_local));
                function.instruction(&Instruction::LocalGet(search_len_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(compare_index_local));
                self.emit_string_slice_payload_from_locals(
                    payload.unwrap(),
                    compare_index_local,
                    piece_len_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(piece_payload_local));
            }
            self.emit_concat_string_payloads_local(
                substitution_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(substitution_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(replacement_index_local));
        function.instruction(&Instruction::I64Const(2));
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
        self.emit_concat_string_payloads_local(substitution_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(substitution_local));
        function.instruction(&Instruction::Else);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            source_offset_local,
            scan_index_local,
            position_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(position_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(position_payload_local));
        self.emit_function_handle_call(
            replacement_payload_local,
            replacement_tag_local,
            Some((zero_local, Some(undefined_tag_local))),
            &[
                (search_string_local, string_tag_local),
                (position_payload_local, number_tag_local),
                (string_local, string_tag_local),
            ],
            callback_payload_local,
            callback_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(callback_payload_local, callback_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(callback_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(callback_string_local));
        function.instruction(&Instruction::LocalSet(substitution_local));
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(self.result_local, substitution_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(search_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(last_end_local));
        match &scope {
            StringLiteralReplacementScope::FirstOccurrence => {
                function.instruction(&Instruction::Br(2));
            }
            StringLiteralReplacementScope::AllOccurrences => {
                function.instruction(&Instruction::LocalGet(last_end_local));
                function.instruction(&Instruction::LocalSet(scan_index_local));
                function.instruction(&Instruction::LocalGet(search_len_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(scan_index_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(scan_index_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::LocalGet(last_end_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(piece_len_local));
        self.emit_string_slice_payload_from_locals(
            string_local,
            last_end_local,
            piece_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(self.result_local, piece_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        for local in [
            undefined_tag_local,
            zero_local,
            substitution_kind_local,
            next_byte_local,
            literal_start_local,
            replacement_index_local,
            substitution_local,
            string_tag_local,
            number_tag_local,
            position_payload_local,
            position_local,
            callback_string_local,
            callback_tag_local,
            callback_payload_local,
            functional_replacement_local,
            piece_payload_local,
            piece_len_local,
            search_byte_local,
            source_byte_local,
            match_local,
            compare_index_local,
            last_end_local,
            scan_index_local,
            replacement_len_local,
            replacement_offset_local,
            search_len_local,
            search_offset_local,
            source_len_local,
            source_offset_local,
            replacement_string_local,
            search_string_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
