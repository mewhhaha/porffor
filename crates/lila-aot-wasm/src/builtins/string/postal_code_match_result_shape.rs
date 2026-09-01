use super::*;

enum PostalCodeMatchResultShape {
    GlobalMatchArray,
    ExecMatchArray,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_string_match_postal_code_global_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_postal_code_from_string_locals(
            string_local,
            PostalCodeMatchResultShape::GlobalMatchArray,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_string_match_postal_code_exec_from_string_locals(
        &mut self,
        string_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_postal_code_from_string_locals(
            string_local,
            PostalCodeMatchResultShape::ExecMatchArray,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_string_match_postal_code_from_string_locals(
        &mut self,
        string_local: u32,
        result_shape: PostalCodeMatchResultShape,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let match_start_local = self.reserve_temp_local();
        let match_len_local = self.reserve_temp_local();
        let capture1_start_local = self.reserve_temp_local();
        let capture2_start_local = self.reserve_temp_local();
        let capture2_len_local = self.reserve_temp_local();
        let has_match_local = self.reserve_temp_local();
        let has_capture2_local = self.reserve_temp_local();
        let digit_match_local = self.reserve_temp_local();
        let sep_index_local = self.reserve_temp_local();
        let sep_byte_local = self.reserve_temp_local();
        let array_len_local = self.reserve_temp_local();
        let array_local = self.reserve_temp_local();
        let array_index_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let capture1_payload_local = self.reserve_temp_local();
        let capture2_payload_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_local, src_offset_local, src_len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_capture2_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture1_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture2_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(capture2_len_local));

        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_start_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(capture2_start_local));
        self.emit_ascii_digit_run_match_to_local(
            src_offset_local,
            match_start_local,
            5,
            digit_match_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(capture2_start_local));
        function.instruction(&Instruction::LocalSet(sep_index_local));
        self.emit_load_string_byte(src_offset_local, sep_index_local, sep_byte_local, function);
        function.instruction(&Instruction::LocalGet(sep_byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(sep_byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(sep_index_local));
        self.emit_ascii_digit_run_match_to_local(
            src_offset_local,
            sep_index_local,
            4,
            digit_match_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_match_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_capture2_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(match_start_local));
        function.instruction(&Instruction::LocalSet(capture1_start_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::LocalSet(capture2_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_match_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_start_local));
        self.emit_ascii_digit_run_match_to_local(
            src_offset_local,
            match_start_local,
            9,
            digit_match_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_match_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_capture2_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(match_start_local));
        function.instruction(&Instruction::LocalSet(capture1_start_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(capture2_start_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(capture2_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_match_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(match_start_local));
        self.emit_ascii_digit_run_match_to_local(
            src_offset_local,
            match_start_local,
            5,
            digit_match_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_capture2_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::LocalSet(match_len_local));
        function.instruction(&Instruction::LocalGet(match_start_local));
        function.instruction(&Instruction::LocalSet(capture1_start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_match_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(match &result_shape {
            PostalCodeMatchResultShape::GlobalMatchArray => 1,
            PostalCodeMatchResultShape::ExecMatchArray => 3,
        }));
        function.instruction(&Instruction::LocalSet(array_len_local));
        self.emit_alloc_array_payload_with_length(array_len_local, array_local, function)?;
        self.emit_string_slice_payload_from_locals(
            string_local,
            match_start_local,
            match_len_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_index_local));
        self.emit_array_write(
            array_local,
            array_index_local,
            match_payload_local,
            value_tag_local,
            function,
        )?;
        match &result_shape {
            PostalCodeMatchResultShape::GlobalMatchArray => {}
            PostalCodeMatchResultShape::ExecMatchArray => {
                function.instruction(&Instruction::I64Const(5));
                function.instruction(&Instruction::LocalSet(array_len_local));
                self.emit_string_slice_payload_from_locals(
                    string_local,
                    capture1_start_local,
                    array_len_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(capture1_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(array_index_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_array_write(
                    array_local,
                    array_index_local,
                    capture1_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::LocalSet(array_index_local));
                function.instruction(&Instruction::LocalGet(has_capture2_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_string_slice_payload_from_locals(
                    string_local,
                    capture2_start_local,
                    capture2_len_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(capture2_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_array_write(
                    array_local,
                    array_index_local,
                    capture2_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(capture2_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_array_write(
                    array_local,
                    array_index_local,
                    capture2_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);

                self.emit_utf16_code_unit_len_from_utf8_locals(
                    src_offset_local,
                    match_start_local,
                    array_len_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(array_len_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(index_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_array_define_builtin_named_data_property(
                    array_local,
                    HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                    HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                    HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
                    index_payload_local,
                    value_tag_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(self.strings.payload("index")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_array_define_named_data_property(
                    array_local,
                    key_local,
                    index_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_array_define_builtin_named_data_property(
                    array_local,
                    HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                    HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                    HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
                    string_local,
                    value_tag_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(self.strings.payload("input")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_array_define_named_data_property(
                    array_local,
                    key_local,
                    string_local,
                    value_tag_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(capture2_payload_local);
        self.release_temp_local(capture1_payload_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(array_index_local);
        self.release_temp_local(array_local);
        self.release_temp_local(array_len_local);
        self.release_temp_local(sep_byte_local);
        self.release_temp_local(sep_index_local);
        self.release_temp_local(digit_match_local);
        self.release_temp_local(has_capture2_local);
        self.release_temp_local(has_match_local);
        self.release_temp_local(capture2_len_local);
        self.release_temp_local(capture2_start_local);
        self.release_temp_local(capture1_start_local);
        self.release_temp_local(match_len_local);
        self.release_temp_local(match_start_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }
}
