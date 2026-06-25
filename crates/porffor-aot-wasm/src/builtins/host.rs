use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_host_print_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argc_local = self.argc_param_local();
        let argv_local = self.argv_param_local();
        let output_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let arg_string_local = self.reserve_temp_local();
        let space_string_local = self.reserve_temp_local();
        let ptr_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(" ")));
        function.instruction(&Instruction::LocalSet(space_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_concat_string_payloads_local(output_local, space_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);

        self.emit_array_read(
            argv_local,
            index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_string_local));
        self.emit_concat_string_payloads_local(output_local, arg_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(output_local, ptr_local, len_local, function);
        function.instruction(&Instruction::LocalGet(ptr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Call(HOST_PRINT_IMPORT_FUNCTION_INDEX));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(len_local);
        self.release_temp_local(ptr_local);
        self.release_temp_local(space_string_local);
        self.release_temp_local(arg_string_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(output_local);
        Ok(())
    }

    pub(crate) fn compile_host_gc_builtin(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    pub(crate) fn compile_host_parse_int_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let radix_payload_local = self.reserve_temp_local();
        let radix_tag_local = self.reserve_temp_local();
        let radix_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let char_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let any_digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );

        self.emit_builtin_arg_to_locals(1, radix_payload_local, radix_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(radix_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(radix_tag_local, radix_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(radix_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64ExtendI32S);
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_digit_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));

        for bytes in [
            &[0xC2, 0xA0][..],       // U+00A0
            &[0xE1, 0x9A, 0x80][..], // U+1680
            &[0xE2, 0x80, 0x80][..], // U+2000
            &[0xE2, 0x80, 0x81][..], // U+2001
            &[0xE2, 0x80, 0x82][..], // U+2002
            &[0xE2, 0x80, 0x83][..], // U+2003
            &[0xE2, 0x80, 0x84][..], // U+2004
            &[0xE2, 0x80, 0x85][..], // U+2005
            &[0xE2, 0x80, 0x86][..], // U+2006
            &[0xE2, 0x80, 0x87][..], // U+2007
            &[0xE2, 0x80, 0x88][..], // U+2008
            &[0xE2, 0x80, 0x89][..], // U+2009
            &[0xE2, 0x80, 0x8A][..], // U+200A
            &[0xE2, 0x80, 0xA8][..], // U+2028
            &[0xE2, 0x80, 0xA9][..], // U+2029
            &[0xE2, 0x80, 0xAF][..], // U+202F
            &[0xE2, 0x81, 0x9F][..], // U+205F
            &[0xE3, 0x80, 0x80][..], // U+3000
            &[0xEF, 0xBB, 0xBF][..], // U+FEFF
        ] {
            Self::emit_parse_int_skip_utf8_whitespace(
                function,
                string_offset_local,
                string_len_local,
                index_local,
                char_local,
                bytes,
            );
        }

        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'x' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'X' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'x' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'X' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));

        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const((b'A' - 10) as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(any_digit_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(any_digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(any_digit_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(value_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(char_local);
        self.release_temp_local(index_local);
        self.release_temp_local(radix_local);
        self.release_temp_local(radix_tag_local);
        self.release_temp_local(radix_payload_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(string_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_parse_float_digit_loop(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        char_local: u32,
        digit_local: u32,
        value_local: u32,
        any_digit_local: u32,
        scale_local: Option<u32>,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        if let Some(scale_local) = scale_local {
            function.instruction(&Instruction::LocalGet(scale_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
            function.instruction(&Instruction::F64Mul);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(scale_local));
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(any_digit_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_parse_float_infinity_check(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        sign_local: u32,
        scratch_local: u32,
        function: &mut Function,
    ) {
        let matched_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(matched_local));
        for (offset, byte) in b"Infinity".iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(string_offset_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(scratch_local));
            function.instruction(&Instruction::LocalGet(matched_local));
            function.instruction(&Instruction::LocalGet(scratch_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(matched_local));
        }
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(matched_local);
    }

    pub(crate) fn emit_parse_float_exponent(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        char_local: u32,
        value_local: u32,
        scale_local: u32,
        exponent_sign_local: u32,
        exponent_local: u32,
        exponent_digit_local: u32,
        function: &mut Function,
    ) {
        let marker_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(marker_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(exponent_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent_digit_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(exponent_sign_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exponent_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(exponent_digit_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exponent_digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(exponent_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scale_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scale_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exponent_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(marker_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(marker_local);
    }

    pub(crate) fn compile_host_parse_float_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let char_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let any_digit_local = self.reserve_temp_local();
        let scale_local = self.reserve_temp_local();
        let exponent_sign_local = self.reserve_temp_local();
        let exponent_local = self.reserve_temp_local();
        let exponent_digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(string_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_digit_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scale_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));

        for bytes in [
            &[0xC2, 0xA0][..],       // U+00A0
            &[0xE1, 0x9A, 0x80][..], // U+1680
            &[0xE2, 0x80, 0x80][..], // U+2000
            &[0xE2, 0x80, 0x81][..], // U+2001
            &[0xE2, 0x80, 0x82][..], // U+2002
            &[0xE2, 0x80, 0x83][..], // U+2003
            &[0xE2, 0x80, 0x84][..], // U+2004
            &[0xE2, 0x80, 0x85][..], // U+2005
            &[0xE2, 0x80, 0x86][..], // U+2006
            &[0xE2, 0x80, 0x87][..], // U+2007
            &[0xE2, 0x80, 0x88][..], // U+2008
            &[0xE2, 0x80, 0x89][..], // U+2009
            &[0xE2, 0x80, 0x8A][..], // U+200A
            &[0xE2, 0x80, 0xA8][..], // U+2028
            &[0xE2, 0x80, 0xA9][..], // U+2029
            &[0xE2, 0x80, 0xAF][..], // U+202F
            &[0xE2, 0x81, 0x9F][..], // U+205F
            &[0xE3, 0x80, 0x80][..], // U+3000
            &[0xEF, 0xBB, 0xBF][..], // U+FEFF
        ] {
            Self::emit_parse_int_skip_utf8_whitespace(
                function,
                string_offset_local,
                string_len_local,
                index_local,
                char_local,
                bytes,
            );
        }

        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_parse_float_infinity_check(
            string_offset_local,
            string_len_local,
            index_local,
            sign_local,
            char_local,
            function,
        );

        self.emit_parse_float_digit_loop(
            string_offset_local,
            string_len_local,
            index_local,
            char_local,
            digit_local,
            value_local,
            any_digit_local,
            None,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_parse_float_digit_loop(
            string_offset_local,
            string_len_local,
            index_local,
            char_local,
            digit_local,
            value_local,
            any_digit_local,
            Some(scale_local),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(any_digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_parse_float_exponent(
            string_offset_local,
            string_len_local,
            index_local,
            char_local,
            value_local,
            scale_local,
            exponent_sign_local,
            exponent_local,
            exponent_digit_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(self.result_local));

        self.release_temp_local(exponent_digit_local);
        self.release_temp_local(exponent_local);
        self.release_temp_local(exponent_sign_local);
        self.release_temp_local(scale_local);
        self.release_temp_local(any_digit_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(value_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(char_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(string_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_parse_int_skip_utf8_whitespace(
        function: &mut Function,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        char_local: u32,
        bytes: &[u8],
    ) {
        debug_assert!(bytes.len() >= 2);

        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I64Const(bytes[0] as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const((bytes.len() - 1) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        for (offset, byte) in bytes.iter().copied().enumerate().skip(1) {
            function.instruction(&Instruction::LocalGet(string_offset_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I32Const(byte as i32));
            function.instruction(&Instruction::I32Eq);
            if offset > 1 {
                function.instruction(&Instruction::I32And);
            }
        }

        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_host_is_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);

        self.emit_is_constructor_i32(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn compile_host_create_realm_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let function_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function`",
                )
            })?;
        let object_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object`",
                )
            })?;
        let array_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let number_meta = self
            .functions
            .get(&StandardBuiltinId::NumberConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Number`",
                )
            })?;
        let string_meta = self
            .functions
            .get(&StandardBuiltinId::StringConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `String`",
                )
            })?;
        let boolean_meta = self
            .functions
            .get(&StandardBuiltinId::BooleanConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Boolean`",
                )
            })?;
        let array_buffer_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer`",
                )
            })?;
        let data_view_meta = self
            .functions
            .get(&StandardBuiltinId::DataViewConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `DataView`",
                )
            })?;
        let aggregate_error_meta = self
            .functions
            .get(&StandardBuiltinId::AggregateErrorConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `AggregateError`",
                )
            })?;
        let suppressed_error_meta = self
            .functions
            .get(&StandardBuiltinId::SuppressedErrorConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `SuppressedError`",
                )
            })?;
        let bigint_meta = self
            .functions
            .get(&StandardBuiltinId::BigIntConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt`",
                )
            })?;
        let proxy_meta = self
            .functions
            .get(&StandardBuiltinId::ProxyConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Proxy`",
                )
            })?;
        let proxy_revocable_meta = self
            .functions
            .get(&StandardBuiltinId::ProxyRevocable.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Proxy.revocable`",
                )
            })?;
        let regexp_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp`",
                )
            })?;
        let regexp_escape_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpEscape.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.escape`",
                )
            })?;
        let error_constructor_metas = [
            (
                ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::ErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Error`",
                        )
                    })?,
            ),
            (
                EVAL_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::EvalErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `EvalError`",
                        )
                    })?,
            ),
            (
                RANGE_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::RangeErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RangeError`",
                        )
                    })?,
            ),
            (
                REFERENCE_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::ReferenceErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `ReferenceError`",
                        )
                    })?,
            ),
            (
                SYNTAX_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::SyntaxErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `SyntaxError`",
                        )
                    })?,
            ),
            (
                TYPE_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::TypeErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `TypeError`",
                        )
                    })?,
            ),
            (
                URI_ERROR_NAME,
                self.functions
                    .get(&StandardBuiltinId::URIErrorConstructor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `URIError`",
                        )
                    })?,
            ),
        ];
        let realm_local = self.reserve_temp_local();
        let global_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let number_prototype_local = self.reserve_temp_local();
        let string_prototype_local = self.reserve_temp_local();
        let boolean_prototype_local = self.reserve_temp_local();
        let array_buffer_prototype_local = self.reserve_temp_local();
        let data_view_prototype_local = self.reserve_temp_local();
        let error_prototype_local = self.reserve_temp_local();
        let eval_error_prototype_local = self.reserve_temp_local();
        let range_error_prototype_local = self.reserve_temp_local();
        let reference_error_prototype_local = self.reserve_temp_local();
        let syntax_error_prototype_local = self.reserve_temp_local();
        let uri_error_prototype_local = self.reserve_temp_local();
        let aggregate_error_prototype_local = self.reserve_temp_local();
        let type_error_prototype_local = self.reserve_temp_local();
        let suppressed_error_prototype_local = self.reserve_temp_local();
        let regexp_prototype_local = self.reserve_temp_local();
        let function_constructor_local = self.reserve_temp_local();
        let object_constructor_local = self.reserve_temp_local();
        let array_constructor_local = self.reserve_temp_local();
        let number_constructor_local = self.reserve_temp_local();
        let string_constructor_local = self.reserve_temp_local();
        let boolean_constructor_local = self.reserve_temp_local();
        let array_buffer_constructor_local = self.reserve_temp_local();
        let data_view_constructor_local = self.reserve_temp_local();
        let aggregate_error_constructor_local = self.reserve_temp_local();
        let suppressed_error_constructor_local = self.reserve_temp_local();
        let bigint_constructor_local = self.reserve_temp_local();
        let proxy_constructor_local = self.reserve_temp_local();
        let regexp_constructor_local = self.reserve_temp_local();
        let mut error_constructor_locals = Vec::new();
        for _ in &error_constructor_metas {
            error_constructor_locals.push(self.reserve_temp_local());
        }
        let mut typed_array_prototype_locals = Vec::new();
        let mut typed_array_constructor_locals = Vec::new();
        for (builtin, _) in typed_array_constructor_bytes_per_element_entries() {
            typed_array_prototype_locals.push((builtin, self.reserve_temp_local()));
            typed_array_constructor_locals.push((builtin, self.reserve_temp_local()));
        }
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(number_prototype_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            number_prototype_local,
            BOXED_PRIMITIVE_KIND_NUMBER,
            self.scratch_local,
            tag_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(string_prototype_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            string_prototype_local,
            BOXED_PRIMITIVE_KIND_STRING,
            self.scratch_local,
            tag_local,
            function,
        );
        for builtin in [
            StandardBuiltinId::StringPrototypeToString,
            StandardBuiltinId::StringPrototypeValueOf,
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.store_i64_local_at_offset(
                method_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                method_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                method_payload_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data(
                string_prototype_local,
                builtin.string_prototype_method_name().unwrap(),
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(boolean_prototype_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            boolean_prototype_local,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            self.scratch_local,
            tag_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(array_buffer_prototype_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(data_view_prototype_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(eval_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(range_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(reference_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(syntax_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(uri_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(aggregate_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(type_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(suppressed_error_prototype_local));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(regexp_prototype_local));
        for (_, prototype_local) in &typed_array_prototype_locals {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(*prototype_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        for (builtin, prototype_local) in &typed_array_prototype_locals {
            let debug_slot = typed_array_realm_prototype_debug_slot(*builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing typed array realm prototype debug slot `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_local_data(
                array_buffer_prototype_local,
                debug_slot,
                *prototype_local,
                tag_local,
                function,
            )?;
        }

        self.emit_function_value_payload(&function_meta, function)?;
        function.instruction(&Instruction::LocalSet(function_constructor_local));
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            array_buffer_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            data_view_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            number_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            boolean_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
            error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
            eval_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
            range_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
            reference_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
            syntax_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
            uri_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_constructor_local,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
            suppressed_error_prototype_local,
            function,
        );
        self.store_typed_array_realm_prototype_locals(
            function_constructor_local,
            &typed_array_prototype_locals,
            function,
        )?;
        self.emit_set_function_prototype_data(
            function_constructor_local,
            function_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&object_meta, function)?;
        function.instruction(&Instruction::LocalSet(object_constructor_local));
        self.store_i64_local_at_offset(
            object_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_set_function_prototype_data(
            object_constructor_local,
            self.scratch_local,
            false,
            function,
        )?;

        self.emit_function_value_payload(&array_meta, function)?;
        function.instruction(&Instruction::LocalSet(array_constructor_local));
        self.store_i64_local_at_offset(
            array_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_set_function_prototype_data(
            array_constructor_local,
            self.scratch_local,
            false,
            function,
        )?;

        self.emit_function_value_payload(&number_meta, function)?;
        function.instruction(&Instruction::LocalSet(number_constructor_local));
        self.store_i64_local_at_offset(
            number_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            number_constructor_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            number_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            number_constructor_local,
            number_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&string_meta, function)?;
        function.instruction(&Instruction::LocalSet(string_constructor_local));
        self.store_i64_local_at_offset(
            string_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            string_constructor_local,
            string_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&boolean_meta, function)?;
        function.instruction(&Instruction::LocalSet(boolean_constructor_local));
        self.store_i64_local_at_offset(
            boolean_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            boolean_constructor_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            boolean_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            boolean_constructor_local,
            boolean_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&array_buffer_meta, function)?;
        function.instruction(&Instruction::LocalSet(array_buffer_constructor_local));
        self.store_i64_local_at_offset(
            array_buffer_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_buffer_constructor_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            array_buffer_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_buffer_constructor_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            data_view_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_buffer_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_typed_array_realm_prototype_locals(
            array_buffer_constructor_local,
            &typed_array_prototype_locals,
            function,
        )?;
        self.emit_set_function_prototype_data(
            array_buffer_constructor_local,
            array_buffer_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&data_view_meta, function)?;
        function.instruction(&Instruction::LocalSet(data_view_constructor_local));
        self.store_i64_local_at_offset(
            data_view_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            data_view_constructor_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            array_buffer_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            data_view_constructor_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            data_view_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            data_view_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_typed_array_realm_prototype_locals(
            data_view_constructor_local,
            &typed_array_prototype_locals,
            function,
        )?;
        self.emit_set_function_prototype_data(
            data_view_constructor_local,
            data_view_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&aggregate_error_meta, function)?;
        function.instruction(&Instruction::LocalSet(aggregate_error_constructor_local));
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            array_buffer_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            data_view_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
            error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
            suppressed_error_prototype_local,
            function,
        );
        self.store_typed_array_realm_prototype_locals(
            aggregate_error_constructor_local,
            &typed_array_prototype_locals,
            function,
        )?;
        self.emit_set_function_prototype_data(
            aggregate_error_constructor_local,
            aggregate_error_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&suppressed_error_meta, function)?;
        function.instruction(&Instruction::LocalSet(suppressed_error_constructor_local));
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
            error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
            suppressed_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            suppressed_error_constructor_local,
            suppressed_error_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&bigint_meta, function)?;
        function.instruction(&Instruction::LocalSet(bigint_constructor_local));
        self.store_i64_local_at_offset(
            bigint_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.load_i64_to_local_from_offset(
            self.scratch_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_set_function_prototype_data(
            bigint_constructor_local,
            self.scratch_local,
            false,
            function,
        )?;

        self.emit_function_value_payload(&proxy_meta, function)?;
        function.instruction(&Instruction::LocalSet(proxy_constructor_local));
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
            error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
            eval_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
            range_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
            reference_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
            syntax_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
            suppressed_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
            uri_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            number_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            proxy_constructor_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            boolean_prototype_local,
            function,
        );
        let revocable_key_local = self.reserve_temp_local();
        let revocable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("revocable")));
        function.instruction(&Instruction::LocalSet(revocable_key_local));
        self.emit_function_value_payload(&proxy_revocable_meta, function)?;
        function.instruction(&Instruction::LocalSet(revocable_payload_local));
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
            error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
            eval_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
            range_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
            reference_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
            syntax_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            aggregate_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
            suppressed_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
            uri_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            number_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            revocable_payload_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            boolean_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(
            proxy_constructor_local,
            revocable_key_local,
            revocable_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(revocable_payload_local);
        self.release_temp_local(revocable_key_local);

        self.emit_function_value_payload(&regexp_meta, function)?;
        function.instruction(&Instruction::LocalSet(regexp_constructor_local));
        self.store_i64_local_at_offset(
            regexp_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            regexp_constructor_local,
            regexp_prototype_local,
            true,
            function,
        )?;
        self.emit_object_define_function_data(
            regexp_constructor_local,
            "escape",
            &regexp_escape_meta,
            function,
        )?;

        for index in 0..error_constructor_metas.len() {
            let (_, meta) = &error_constructor_metas[index];
            let constructor_local = error_constructor_locals[index];
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                function_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
                error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
                eval_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
                range_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
                reference_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
                syntax_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
                aggregate_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
                suppressed_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
                uri_error_prototype_local,
                function,
            );
            if meta.name == ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == EVAL_ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    eval_error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == RANGE_ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    range_error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == REFERENCE_ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    reference_error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == SYNTAX_ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    syntax_error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == TYPE_ERROR_NAME {
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                    type_error_prototype_local,
                    function,
                );
                self.emit_set_function_prototype_data(
                    constructor_local,
                    type_error_prototype_local,
                    true,
                    function,
                )?;
            } else if meta.name == URI_ERROR_NAME {
                self.emit_set_function_prototype_data(
                    constructor_local,
                    uri_error_prototype_local,
                    true,
                    function,
                )?;
            } else {
                function.instruction(&Instruction::GlobalGet(ERROR_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.emit_set_function_prototype_data(
                    constructor_local,
                    self.scratch_local,
                    false,
                    function,
                )?;
            }
        }

        for index in 0..typed_array_constructor_locals.len() {
            let (builtin, constructor_local) = typed_array_constructor_locals[index];
            let prototype_local = typed_array_prototype_locals[index].1;
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(constructor_local));
            function.instruction(&Instruction::GlobalGet(
                TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
                array_buffer_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
                data_view_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
                aggregate_error_prototype_local,
                function,
            );
            self.store_typed_array_realm_prototype_locals(
                constructor_local,
                &typed_array_prototype_locals,
                function,
            )?;
            self.emit_set_function_prototype_data_with_flags(
                constructor_local,
                prototype_local,
                false,
                false,
                false,
                true,
                function,
            )?;
            self.emit_object_define_number_data_from_f64_const_with_flags(
                constructor_local,
                "BYTES_PER_ELEMENT",
                typed_array_bytes_per_element(builtin) as f64,
                false,
                false,
                false,
                function,
            )?;
            self.emit_object_define_number_data_from_f64_const_with_flags(
                prototype_local,
                "BYTES_PER_ELEMENT",
                typed_array_bytes_per_element(builtin) as f64,
                false,
                false,
                false,
                function,
            )?;
        }

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(global_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            global_local,
            FUNCTION_NAME,
            function_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            OBJECT_NAME,
            object_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            ARRAY_NAME,
            array_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            NUMBER_NAME,
            number_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            STRING_NAME,
            string_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            BOOLEAN_NAME,
            boolean_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            ARRAY_BUFFER_NAME,
            array_buffer_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            DATA_VIEW_NAME,
            data_view_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            AGGREGATE_ERROR_NAME,
            aggregate_error_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            SUPPRESSED_ERROR_NAME,
            suppressed_error_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            "BigInt",
            bigint_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            PROXY_NAME,
            proxy_constructor_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            REGEXP_NAME,
            regexp_constructor_local,
            tag_local,
            function,
        )?;
        for index in 0..error_constructor_metas.len() {
            let (name, _) = &error_constructor_metas[index];
            self.emit_object_define_local_data(
                global_local,
                name,
                error_constructor_locals[index],
                tag_local,
                function,
            )?;
        }
        for (builtin, constructor_local) in &typed_array_constructor_locals {
            self.emit_object_define_local_data(
                global_local,
                builtin.debug_name(),
                *constructor_local,
                tag_local,
                function,
            )?;
        }

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            realm_local,
            "global",
            global_local,
            tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(tag_local);
        for index in (0..typed_array_constructor_locals.len()).rev() {
            self.release_temp_local(typed_array_constructor_locals[index].1);
            self.release_temp_local(typed_array_prototype_locals[index].1);
        }
        for constructor_local in error_constructor_locals.into_iter().rev() {
            self.release_temp_local(constructor_local);
        }
        self.release_temp_local(regexp_constructor_local);
        self.release_temp_local(proxy_constructor_local);
        self.release_temp_local(bigint_constructor_local);
        self.release_temp_local(suppressed_error_constructor_local);
        self.release_temp_local(aggregate_error_constructor_local);
        self.release_temp_local(data_view_constructor_local);
        self.release_temp_local(array_buffer_constructor_local);
        self.release_temp_local(boolean_constructor_local);
        self.release_temp_local(string_constructor_local);
        self.release_temp_local(number_constructor_local);
        self.release_temp_local(array_constructor_local);
        self.release_temp_local(object_constructor_local);
        self.release_temp_local(function_constructor_local);
        self.release_temp_local(regexp_prototype_local);
        self.release_temp_local(suppressed_error_prototype_local);
        self.release_temp_local(type_error_prototype_local);
        self.release_temp_local(aggregate_error_prototype_local);
        self.release_temp_local(uri_error_prototype_local);
        self.release_temp_local(syntax_error_prototype_local);
        self.release_temp_local(reference_error_prototype_local);
        self.release_temp_local(range_error_prototype_local);
        self.release_temp_local(eval_error_prototype_local);
        self.release_temp_local(error_prototype_local);
        self.release_temp_local(data_view_prototype_local);
        self.release_temp_local(array_buffer_prototype_local);
        self.release_temp_local(boolean_prototype_local);
        self.release_temp_local(string_prototype_local);
        self.release_temp_local(number_prototype_local);
        self.release_temp_local(function_prototype_local);
        self.release_temp_local(global_local);
        self.release_temp_local(realm_local);
        Ok(())
    }

    pub(crate) fn compile_host_detach_array_buffer_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let slot_payload_local = self.reserve_temp_local();
        let slot_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, buffer_payload_local, buffer_tag_local, function);

        function.instruction(&Instruction::LocalGet(buffer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "detachArrayBuffer expects an ArrayBuffer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload(ARRAY_BUFFER_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            buffer_payload_local,
            buffer_tag_local,
            buffer_payload_local,
            buffer_tag_local,
            key_local,
            slot_payload_local,
            slot_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(slot_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "detachArrayBuffer expects an ArrayBuffer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_object_define_number_data_from_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_DATA_PTR_SLOT,
            zero_local,
            function,
        )?;
        self.emit_object_define_number_data_from_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            zero_local,
            function,
        )?;
        self.emit_object_define_number_data_from_i64_local(
            buffer_payload_local,
            "byteLength",
            zero_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(zero_local);
        self.release_temp_local(slot_tag_local);
        self.release_temp_local(slot_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        Ok(())
    }

    pub(crate) fn emit_math_min_max_combine_result(
        &mut self,
        builtin: StandardBuiltinId,
        arg_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        if builtin == StandardBuiltinId::MathMin {
            function.instruction(&Instruction::F64Min);
        } else {
            function.instruction(&Instruction::F64Max);
        }
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_host_assert_throws_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let expected_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let callback_env_local = self.reserve_temp_local();
        let callback_table_index_local = self.reserve_temp_local();
        let callback_flags_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let call_completion_local = self.reserve_temp_local();
        let call_aux_local = self.reserve_temp_local();
        let constructor_key_local = self.reserve_temp_local();
        let actual_constructor_payload_local = self.reserve_temp_local();
        let actual_constructor_tag_local = self.reserve_temp_local();
        let expected_error_prototype_local = self.reserve_temp_local();
        let expected_prototype_payload_local = self.reserve_temp_local();
        let expected_prototype_tag_local = self.reserve_temp_local();
        let actual_prototype_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, expected_payload_local, expected_tag_local, function);
        self.emit_builtin_arg_to_locals(1, callback_payload_local, callback_tag_local, function);

        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "assert.throws requires a function callback",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_pre_evaluated_arg_vector(&[], argc_local, argv_local, function)?;
        self.emit_load_function_object_fields(
            callback_payload_local,
            callback_env_local,
            callback_table_index_local,
            function,
        );
        self.emit_load_function_flags(callback_payload_local, callback_flags_local, function);
        function.instruction(&Instruction::LocalGet(callback_flags_local));
        function.instruction(&Instruction::I64Const(
            FUNCTION_FLAG_CLASS_CONSTRUCTOR as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(callback_env_local));
        self.emit_default_this(function);
        self.emit_undefined_new_target(function);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::LocalGet(argv_local));
        function.instruction(&Instruction::LocalGet(callback_table_index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::CallIndirect {
            type_index: JS_FUNCTION_TYPE_INDEX,
            table_index: 0,
        });
        self.store_call_results_to(
            call_payload_local,
            call_tag_local,
            call_completion_local,
            call_aux_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "assert.throws callback is a class constructor",
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalSet(call_completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(call_aux_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(call_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_is_heap_object_like_tag_i32(call_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            ERROR_NAME,
            "assert.throws expected an error object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            call_payload_local,
            call_tag_local,
            call_payload_local,
            call_tag_local,
            constructor_key_local,
            actual_constructor_payload_local,
            actual_constructor_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expected_error_prototype_local));
        for (constructor_global, prototype_global) in [
            (ERROR_CONSTRUCTOR_GLOBAL_INDEX, ERROR_PROTOTYPE_GLOBAL_INDEX),
            (
                EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(expected_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(expected_payload_local));
            function.instruction(&Instruction::GlobalGet(constructor_global));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::GlobalGet(prototype_global));
            function.instruction(&Instruction::LocalSet(expected_error_prototype_local));
            function.instruction(&Instruction::End);
        }
        self.load_i64_to_local_from_offset(
            call_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            actual_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        self.emit_object_read(
            expected_payload_local,
            expected_tag_local,
            expected_payload_local,
            expected_tag_local,
            constructor_key_local,
            expected_prototype_payload_local,
            expected_prototype_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(actual_constructor_tag_local));
        function.instruction(&Instruction::LocalGet(expected_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(actual_constructor_payload_local));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(expected_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(actual_prototype_local));
        function.instruction(&Instruction::LocalGet(expected_error_prototype_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(expected_prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(actual_prototype_local));
        function.instruction(&Instruction::LocalGet(expected_prototype_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(expected_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(expected_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            ERROR_NAME,
            "assert.throws received the wrong error constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            ERROR_NAME,
            "assert.throws expected a throw",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(actual_prototype_local);
        self.release_temp_local(expected_prototype_tag_local);
        self.release_temp_local(expected_prototype_payload_local);
        self.release_temp_local(expected_error_prototype_local);
        self.release_temp_local(actual_constructor_tag_local);
        self.release_temp_local(actual_constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        self.release_temp_local(call_aux_local);
        self.release_temp_local(call_completion_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(callback_flags_local);
        self.release_temp_local(callback_table_index_local);
        self.release_temp_local(callback_env_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(expected_tag_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }
}
