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

    pub(crate) fn compile_host_gc_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if heap_collector_is_executable() {
            return Err(EmitError::unsupported(
                "heap collector is marked executable but host gc emitter is not wired",
            ));
        }
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "gc requires a real collector in wasm-aot",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
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
        let function_prototype_method_metas = [
            (
                "call",
                self.functions
                    .get(&StandardBuiltinId::FunctionPrototypeCall.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.call`",
                        )
                    })?,
            ),
            (
                "apply",
                self.functions
                    .get(&StandardBuiltinId::FunctionPrototypeApply.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.apply`",
                        )
                    })?,
            ),
            (
                "bind",
                self.functions
                    .get(&StandardBuiltinId::FunctionPrototypeBind.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.bind`",
                        )
                    })?,
            ),
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::FunctionPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.toString`",
                        )
                    })?,
            ),
        ];
        let object_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object`",
                )
            })?;
        let object_prototype_method_metas = [
            (
                "hasOwnProperty",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypeHasOwnProperty.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.hasOwnProperty`",
                        )
                    })?,
            ),
            (
                "propertyIsEnumerable",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypePropertyIsEnumerable.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.propertyIsEnumerable`",
                        )
                    })?,
            ),
            (
                "isPrototypeOf",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypeIsPrototypeOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.isPrototypeOf`",
                        )
                    })?,
            ),
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.toString`",
                        )
                    })?,
            ),
            (
                "toLocaleString",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypeToLocaleString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.toLocaleString`",
                        )
                    })?,
            ),
            (
                "valueOf",
                self.functions
                    .get(&StandardBuiltinId::ObjectPrototypeValueOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.valueOf`",
                        )
                    })?,
            ),
        ];
        let object_static_method_metas = [
            (
                "create",
                self.functions
                    .get(&StandardBuiltinId::ObjectCreate.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.create`",
                        )
                    })?,
            ),
            (
                "getPrototypeOf",
                self.functions
                    .get(&StandardBuiltinId::ObjectGetPrototypeOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getPrototypeOf`",
                        )
                    })?,
            ),
            (
                "setPrototypeOf",
                self.functions
                    .get(&StandardBuiltinId::ObjectSetPrototypeOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.setPrototypeOf`",
                        )
                    })?,
            ),
            (
                "defineProperty",
                self.functions
                    .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                        )
                    })?,
            ),
            (
                "defineProperties",
                self.functions
                    .get(&StandardBuiltinId::ObjectDefineProperties.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperties`",
                        )
                    })?,
            ),
            (
                "getOwnPropertyDescriptor",
                self.functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                        )
                    })?,
            ),
            (
                "getOwnPropertyNames",
                self.functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertyNames.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyNames`",
                        )
                    })?,
            ),
            (
                "getOwnPropertySymbols",
                self.functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertySymbols.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertySymbols`",
                        )
                    })?,
            ),
            (
                "hasOwn",
                self.functions
                    .get(&StandardBuiltinId::ObjectHasOwn.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.hasOwn`",
                        )
                    })?,
            ),
            (
                "is",
                self.functions
                    .get(&StandardBuiltinId::ObjectIs.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.is`",
                        )
                    })?,
            ),
            (
                "isSealed",
                self.functions
                    .get(&StandardBuiltinId::ObjectIsSealed.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isSealed`",
                        )
                    })?,
            ),
            (
                "isFrozen",
                self.functions
                    .get(&StandardBuiltinId::ObjectIsFrozen.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isFrozen`",
                        )
                    })?,
            ),
            (
                "freeze",
                self.functions
                    .get(&StandardBuiltinId::ObjectFreeze.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.freeze`",
                        )
                    })?,
            ),
            (
                "isExtensible",
                self.functions
                    .get(&StandardBuiltinId::ObjectIsExtensible.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isExtensible`",
                        )
                    })?,
            ),
            (
                "preventExtensions",
                self.functions
                    .get(&StandardBuiltinId::ObjectPreventExtensions.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.preventExtensions`",
                        )
                    })?,
            ),
            (
                "values",
                self.functions
                    .get(&StandardBuiltinId::ObjectValues.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.values`",
                        )
                    })?,
            ),
            (
                "keys",
                self.functions
                    .get(&StandardBuiltinId::ObjectKeys.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.keys`",
                        )
                    })?,
            ),
        ];
        let reflect_static_method_metas = [
            (
                "construct",
                self.functions
                    .get(&StandardBuiltinId::ReflectConstruct.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.construct`",
                        )
                    })?,
            ),
            (
                "apply",
                self.functions
                    .get(&StandardBuiltinId::ReflectApply.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.apply`",
                        )
                    })?,
            ),
            (
                "get",
                self.functions
                    .get(&StandardBuiltinId::ReflectGet.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.get`",
                        )
                    })?,
            ),
            (
                "getPrototypeOf",
                self.functions
                    .get(&StandardBuiltinId::ReflectGetPrototypeOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getPrototypeOf`",
                        )
                    })?,
            ),
            (
                "getOwnPropertyDescriptor",
                self.functions
                    .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                        )
                    })?,
            ),
            (
                "set",
                self.functions
                    .get(&StandardBuiltinId::ReflectSet.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                        )
                    })?,
            ),
            (
                "has",
                self.functions
                    .get(&StandardBuiltinId::ReflectHas.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.has`",
                        )
                    })?,
            ),
            (
                "defineProperty",
                self.functions
                    .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                        )
                    })?,
            ),
            (
                "deleteProperty",
                self.functions
                    .get(&StandardBuiltinId::ReflectDeleteProperty.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.deleteProperty`",
                        )
                    })?,
            ),
            (
                "isExtensible",
                self.functions
                    .get(&StandardBuiltinId::ReflectIsExtensible.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.isExtensible`",
                        )
                    })?,
            ),
            (
                "preventExtensions",
                self.functions
                    .get(&StandardBuiltinId::ReflectPreventExtensions.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.preventExtensions`",
                        )
                    })?,
            ),
            (
                "setPrototypeOf",
                self.functions
                    .get(&StandardBuiltinId::ReflectSetPrototypeOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.setPrototypeOf`",
                        )
                    })?,
            ),
            (
                "ownKeys",
                self.functions
                    .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
                        )
                    })?,
            ),
        ];
        let global_function_metas = [
            ("eval", StandardBuiltinId::EvalFunction),
            ("isFinite", StandardBuiltinId::GlobalIsFinite),
            ("isNaN", StandardBuiltinId::GlobalIsNaN),
            ("escape", StandardBuiltinId::Escape),
            ("unescape", StandardBuiltinId::Unescape),
        ]
        .into_iter()
        .map(|(name, builtin)| {
            self.functions
                .get(&builtin.function_id())
                .cloned()
                .map(|meta| (name, meta))
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })
        })
        .collect::<Result<Vec<_>, EmitError>>()?;
        let global_host_function_metas = [
            (
                "parseInt",
                self.functions
                    .get(&HostBuiltinId::ParseInt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `parseInt`",
                        )
                    })?,
            ),
            (
                "parseFloat",
                self.functions
                    .get(&HostBuiltinId::ParseFloat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `parseFloat`",
                        )
                    })?,
            ),
        ];
        let math_static_method_metas = [
            ("abs", StandardBuiltinId::MathAbs),
            ("acos", StandardBuiltinId::MathAcos),
            ("acosh", StandardBuiltinId::MathAcosh),
            ("asin", StandardBuiltinId::MathAsin),
            ("asinh", StandardBuiltinId::MathAsinh),
            ("atan", StandardBuiltinId::MathAtan),
            ("atan2", StandardBuiltinId::MathAtan2),
            ("atanh", StandardBuiltinId::MathAtanh),
            ("cbrt", StandardBuiltinId::MathCbrt),
            ("ceil", StandardBuiltinId::MathCeil),
            ("clz32", StandardBuiltinId::MathClz32),
            ("cos", StandardBuiltinId::MathCos),
            ("cosh", StandardBuiltinId::MathCosh),
            ("exp", StandardBuiltinId::MathExp),
            ("expm1", StandardBuiltinId::MathExpm1),
            ("f16round", StandardBuiltinId::MathF16Round),
            ("floor", StandardBuiltinId::MathFloor),
            ("fround", StandardBuiltinId::MathFround),
            ("hypot", StandardBuiltinId::MathHypot),
            ("imul", StandardBuiltinId::MathImul),
            ("log", StandardBuiltinId::MathLog),
            ("log10", StandardBuiltinId::MathLog10),
            ("log1p", StandardBuiltinId::MathLog1p),
            ("log2", StandardBuiltinId::MathLog2),
            ("max", StandardBuiltinId::MathMax),
            ("min", StandardBuiltinId::MathMin),
            ("pow", StandardBuiltinId::MathPow),
            ("random", StandardBuiltinId::MathRandom),
            ("round", StandardBuiltinId::MathRound),
            ("sign", StandardBuiltinId::MathSign),
            ("sin", StandardBuiltinId::MathSin),
            ("sinh", StandardBuiltinId::MathSinh),
            ("sqrt", StandardBuiltinId::MathSqrt),
            ("sumPrecise", StandardBuiltinId::MathSumPrecise),
            ("tan", StandardBuiltinId::MathTan),
            ("tanh", StandardBuiltinId::MathTanh),
            ("trunc", StandardBuiltinId::MathTrunc),
        ]
        .into_iter()
        .map(|(name, builtin)| {
            self.functions
                .get(&builtin.function_id())
                .cloned()
                .map(|meta| (name, meta))
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Math.{name}`"
                    ))
                })
        })
        .collect::<Result<Vec<_>, EmitError>>()?;
        let json_static_method_metas = [
            (
                "parse",
                self.functions
                    .get(&StandardBuiltinId::JsonParse.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `JSON.parse`",
                        )
                    })?,
            ),
            (
                "stringify",
                self.functions
                    .get(&StandardBuiltinId::JsonStringify.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `JSON.stringify`",
                        )
                    })?,
            ),
            (
                "rawJSON",
                self.functions
                    .get(&StandardBuiltinId::JsonRawJson.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `JSON.rawJSON`",
                        )
                    })?,
            ),
            (
                "isRawJSON",
                self.functions
                    .get(&StandardBuiltinId::JsonIsRawJson.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `JSON.isRawJSON`",
                        )
                    })?,
            ),
        ];
        let array_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator`",
                )
            })?;
        let iterator_from_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.from`",
                )
            })?;
        let iterator_constructor_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeConstructorGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get Iterator.prototype.constructor`",
                )
            })?;
        let iterator_constructor_setter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeConstructorSetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `set Iterator.prototype.constructor`",
                )
            })?;
        let iterator_symbol_dispose_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeSymbolDispose.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype[Symbol.dispose]`",
                )
            })?;
        let iterator_to_string_tag_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeToStringTagGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get Iterator.prototype[Symbol.toStringTag]`",
                )
            })?;
        let iterator_to_string_tag_setter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeToStringTagSetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `set Iterator.prototype[Symbol.toStringTag]`",
                )
            })?;
        let iterator_prototype_method_metas = [
            (
                "toArray",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeToArray.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.toArray`",
                        )
                    })?,
            ),
            (
                "forEach",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeForEach.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.forEach`",
                        )
                    })?,
            ),
            (
                "every",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeEvery.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.every`",
                        )
                    })?,
            ),
            (
                "some",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeSome.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.some`",
                        )
                    })?,
            ),
            (
                "find",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeFind.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.find`",
                        )
                    })?,
            ),
            (
                "reduce",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeReduce.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.reduce`",
                        )
                    })?,
            ),
            (
                "map",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeMap.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.map`",
                        )
                    })?,
            ),
            (
                "filter",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeFilter.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.filter`",
                        )
                    })?,
            ),
            (
                "flatMap",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeFlatMap.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.flatMap`",
                        )
                    })?,
            ),
            (
                "take",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeTake.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.take`",
                        )
                    })?,
            ),
            (
                "drop",
                self.functions
                    .get(&StandardBuiltinId::IteratorPrototypeDrop.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.drop`",
                        )
                    })?,
            ),
        ];
        let array_species_meta = self
            .functions
            .get(&StandardBuiltinId::ArraySpeciesGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array[Symbol.species]`",
                )
            })?;
        let array_static_method_metas = [
            (
                "from",
                self.functions
                    .get(&StandardBuiltinId::ArrayFrom.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.from`",
                        )
                    })?,
            ),
            (
                "of",
                self.functions
                    .get(&StandardBuiltinId::ArrayOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.of`",
                        )
                    })?,
            ),
            (
                "isArray",
                self.functions
                    .get(&StandardBuiltinId::ArrayIsArray.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.isArray`",
                        )
                    })?,
            ),
        ];
        let array_prototype_method_metas = [
            (
                "toLocaleString",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeToLocaleString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.toLocaleString`",
                        )
                    })?,
            ),
            (
                "at",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeAt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.at`",
                        )
                    })?,
            ),
            (
                "includes",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeIncludes.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.includes`",
                        )
                    })?,
            ),
            (
                "indexOf",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeIndexOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.indexOf`",
                        )
                    })?,
            ),
            (
                "lastIndexOf",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.lastIndexOf`",
                        )
                    })?,
            ),
            (
                "find",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFind.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.find`",
                        )
                    })?,
            ),
            (
                "findIndex",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindIndex.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findIndex`",
                        )
                    })?,
            ),
            (
                "findLast",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindLast.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findLast`",
                        )
                    })?,
            ),
            (
                "findLastIndex",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findLastIndex`",
                        )
                    })?,
            ),
            (
                "forEach",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeForEach.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.forEach`",
                        )
                    })?,
            ),
            (
                "every",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeEvery.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.every`",
                        )
                    })?,
            ),
            (
                "some",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeSome.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.some`",
                        )
                    })?,
            ),
            (
                "filter",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFilter.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.filter`",
                        )
                    })?,
            ),
            (
                "map",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeMap.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.map`",
                        )
                    })?,
            ),
            (
                "pop",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypePop.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.pop`",
                        )
                    })?,
            ),
            (
                "push",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypePush.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.push`",
                        )
                    })?,
            ),
            (
                "concat",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeConcat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.concat`",
                        )
                    })?,
            ),
            (
                "flat",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFlat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flat`",
                        )
                    })?,
            ),
            (
                "flatMap",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeFlatMap.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flatMap`",
                        )
                    })?,
            ),
            (
                "keys",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeKeys.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.keys`",
                        )
                    })?,
            ),
            (
                "entries",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeEntries.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.entries`",
                        )
                    })?,
            ),
            (
                "values",
                self.functions
                    .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.values`",
                        )
                    })?,
            ),
        ];
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
        let number_static_method_metas = [
            (
                "isInteger",
                self.functions
                    .get(&StandardBuiltinId::NumberIsInteger.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isInteger`",
                        )
                    })?,
            ),
            (
                "isSafeInteger",
                self.functions
                    .get(&StandardBuiltinId::NumberIsSafeInteger.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isSafeInteger`",
                        )
                    })?,
            ),
            (
                "isFinite",
                self.functions
                    .get(&StandardBuiltinId::NumberIsFinite.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isFinite`",
                        )
                    })?,
            ),
            (
                "isNaN",
                self.functions
                    .get(&StandardBuiltinId::NumberIsNaN.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isNaN`",
                        )
                    })?,
            ),
            (
                "parseInt",
                self.functions
                    .get(&HostBuiltinId::ParseInt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.parseInt`",
                        )
                    })?,
            ),
            (
                "parseFloat",
                self.functions
                    .get(&HostBuiltinId::ParseFloat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.parseFloat`",
                        )
                    })?,
            ),
        ];
        let number_prototype_method_metas = [
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::NumberPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.prototype.toString`",
                        )
                    })?,
            ),
            (
                "toLocaleString",
                self.functions
                    .get(&StandardBuiltinId::NumberPrototypeToLocaleString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.prototype.toLocaleString`",
                        )
                    })?,
            ),
            (
                "valueOf",
                self.functions
                    .get(&StandardBuiltinId::NumberPrototypeValueOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.prototype.valueOf`",
                        )
                    })?,
            ),
        ];
        let string_prototype_method_metas = [
            (
                "at",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeAt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.at`",
                        )
                    })?,
            ),
            (
                "charAt",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeCharAt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.charAt`",
                        )
                    })?,
            ),
            (
                "endsWith",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeEndsWith.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.endsWith`",
                        )
                    })?,
            ),
            (
                "includes",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeIncludes.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.includes`",
                        )
                    })?,
            ),
            (
                "indexOf",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeIndexOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.indexOf`",
                        )
                    })?,
            ),
            (
                "isWellFormed",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeIsWellFormed.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.isWellFormed`",
                        )
                    })?,
            ),
            (
                "match",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeMatch.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.match`",
                        )
                    })?,
            ),
            (
                "matchAll",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeMatchAll.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.matchAll`",
                        )
                    })?,
            ),
            (
                "padEnd",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypePadEnd.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.padEnd`",
                        )
                    })?,
            ),
            (
                "padStart",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypePadStart.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.padStart`",
                        )
                    })?,
            ),
            (
                "repeat",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeRepeat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.repeat`",
                        )
                    })?,
            ),
            (
                "replace",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeReplace.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.replace`",
                        )
                    })?,
            ),
            (
                "replaceAll",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeReplaceAll.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.replaceAll`",
                        )
                    })?,
            ),
            (
                "search",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeSearch.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.search`",
                        )
                    })?,
            ),
            (
                "slice",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeSlice.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.slice`",
                        )
                    })?,
            ),
            (
                "split",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeSplit.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.split`",
                        )
                    })?,
            ),
            (
                "startsWith",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeStartsWith.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.startsWith`",
                        )
                    })?,
            ),
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.toString`",
                        )
                    })?,
            ),
            (
                "toUpperCase",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeToUpperCase.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.toUpperCase`",
                        )
                    })?,
            ),
            (
                "toWellFormed",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeToWellFormed.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.toWellFormed`",
                        )
                    })?,
            ),
            (
                "trim",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeTrim.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.trim`",
                        )
                    })?,
            ),
            (
                "trimEnd",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeTrimEnd.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.trimEnd`",
                        )
                    })?,
            ),
            (
                "trimStart",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeTrimStart.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.trimStart`",
                        )
                    })?,
            ),
            (
                "valueOf",
                self.functions
                    .get(&StandardBuiltinId::StringPrototypeValueOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype.valueOf`",
                        )
                    })?,
            ),
        ];
        let boolean_prototype_method_metas = [
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::BooleanPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Boolean.prototype.toString`",
                        )
                    })?,
            ),
            (
                "valueOf",
                self.functions
                    .get(&StandardBuiltinId::BooleanPrototypeValueOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Boolean.prototype.valueOf`",
                        )
                    })?,
            ),
        ];
        let array_buffer_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer`",
                )
            })?;
        let array_buffer_is_view_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferIsView.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer.isView`",
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
        let bigint_static_method_metas = [
            (
                "asIntN",
                self.functions
                    .get(&StandardBuiltinId::BigIntAsIntN.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt.asIntN`",
                        )
                    })?,
            ),
            (
                "asUintN",
                self.functions
                    .get(&StandardBuiltinId::BigIntAsUintN.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt.asUintN`",
                        )
                    })?,
            ),
        ];
        let bigint_prototype_method_metas = [
            (
                "toString",
                self.functions
                    .get(&StandardBuiltinId::BigIntPrototypeToString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt.prototype.toString`",
                        )
                    })?,
            ),
            (
                "toLocaleString",
                self.functions
                    .get(&StandardBuiltinId::BigIntPrototypeToLocaleString.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt.prototype.toLocaleString`",
                        )
                    })?,
            ),
            (
                "valueOf",
                self.functions
                    .get(&StandardBuiltinId::BigIntPrototypeValueOf.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `BigInt.prototype.valueOf`",
                        )
                    })?,
            ),
        ];
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
        let date_meta = self
            .functions
            .get(&StandardBuiltinId::DateConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Date`",
                )
            })?;
        let date_static_method_metas = [
            (
                "now",
                self.functions
                    .get(&StandardBuiltinId::DateNow.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Date.now`",
                        )
                    })?,
            ),
            (
                "UTC",
                self.functions
                    .get(&StandardBuiltinId::DateUtc.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Date.UTC`",
                        )
                    })?,
            ),
        ];
        let date_prototype_method_metas = [
            ("getTime", StandardBuiltinId::DatePrototypeGetTime),
            ("setTime", StandardBuiltinId::DatePrototypeSetTime),
            ("valueOf", StandardBuiltinId::DatePrototypeValueOf),
            ("getFullYear", StandardBuiltinId::DatePrototypeGetFullYear),
            (
                "getUTCFullYear",
                StandardBuiltinId::DatePrototypeGetUtcFullYear,
            ),
            ("getMonth", StandardBuiltinId::DatePrototypeGetMonth),
            ("getUTCMonth", StandardBuiltinId::DatePrototypeGetUtcMonth),
            ("getDate", StandardBuiltinId::DatePrototypeGetDate),
            ("getUTCDate", StandardBuiltinId::DatePrototypeGetUtcDate),
            ("getDay", StandardBuiltinId::DatePrototypeGetDay),
            ("getUTCDay", StandardBuiltinId::DatePrototypeGetUtcDay),
            ("getHours", StandardBuiltinId::DatePrototypeGetHours),
            ("getUTCHours", StandardBuiltinId::DatePrototypeGetUtcHours),
            ("getMinutes", StandardBuiltinId::DatePrototypeGetMinutes),
            (
                "getUTCMinutes",
                StandardBuiltinId::DatePrototypeGetUtcMinutes,
            ),
            ("getSeconds", StandardBuiltinId::DatePrototypeGetSeconds),
            (
                "getUTCSeconds",
                StandardBuiltinId::DatePrototypeGetUtcSeconds,
            ),
            (
                "getMilliseconds",
                StandardBuiltinId::DatePrototypeGetMilliseconds,
            ),
            (
                "getUTCMilliseconds",
                StandardBuiltinId::DatePrototypeGetUtcMilliseconds,
            ),
            (
                "getTimezoneOffset",
                StandardBuiltinId::DatePrototypeGetTimezoneOffset,
            ),
            ("getYear", StandardBuiltinId::DatePrototypeGetYear),
            ("setYear", StandardBuiltinId::DatePrototypeSetYear),
            ("setFullYear", StandardBuiltinId::DatePrototypeSetFullYear),
            (
                "setUTCFullYear",
                StandardBuiltinId::DatePrototypeSetUtcFullYear,
            ),
            ("setMonth", StandardBuiltinId::DatePrototypeSetMonth),
            ("setUTCMonth", StandardBuiltinId::DatePrototypeSetUtcMonth),
            ("setDate", StandardBuiltinId::DatePrototypeSetDate),
            ("setUTCDate", StandardBuiltinId::DatePrototypeSetUtcDate),
            ("setHours", StandardBuiltinId::DatePrototypeSetHours),
            ("setUTCHours", StandardBuiltinId::DatePrototypeSetUtcHours),
            ("setMinutes", StandardBuiltinId::DatePrototypeSetMinutes),
            (
                "setUTCMinutes",
                StandardBuiltinId::DatePrototypeSetUtcMinutes,
            ),
            ("setSeconds", StandardBuiltinId::DatePrototypeSetSeconds),
            (
                "setUTCSeconds",
                StandardBuiltinId::DatePrototypeSetUtcSeconds,
            ),
            (
                "setMilliseconds",
                StandardBuiltinId::DatePrototypeSetMilliseconds,
            ),
            (
                "setUTCMilliseconds",
                StandardBuiltinId::DatePrototypeSetUtcMilliseconds,
            ),
            ("toUTCString", StandardBuiltinId::DatePrototypeToUtcString),
        ]
        .map(|(name, builtin)| {
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
            Ok::<_, EmitError>((name, meta))
        });
        let date_prototype_method_metas = date_prototype_method_metas
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let regexp_escape_meta = self
            .functions
            .get(&StandardBuiltinId::RegExpEscape.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.escape`",
                )
            })?;
        let regexp_prototype_symbol_method_metas = [
            (
                "Symbol.match",
                StandardBuiltinId::RegExpPrototypeSymbolMatch,
            ),
            (
                "Symbol.matchAll",
                StandardBuiltinId::RegExpPrototypeSymbolMatchAll,
            ),
            (
                "Symbol.search",
                StandardBuiltinId::RegExpPrototypeSymbolSearch,
            ),
        ]
        .map(|(name, builtin)| {
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
            Ok::<_, EmitError>((name, meta))
        });
        let regexp_prototype_symbol_method_metas = regexp_prototype_symbol_method_metas
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
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
        let error_prototype_to_string_meta = self
            .functions
            .get(&StandardBuiltinId::ErrorPrototypeToString.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.prototype.toString`",
                )
            })?;
        let error_is_error_meta = self
            .functions
            .get(&StandardBuiltinId::ErrorIsError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.isError`",
                )
            })?;
        let array_iterator_next_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorNext.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype.next`",
                )
            })?;
        let array_iterator_identity_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype[Symbol.iterator]`",
                )
            })?;
        let realm_local = self.reserve_temp_local();
        let realm_record_local = self.reserve_temp_local();
        let global_local = self.reserve_temp_local();
        let reflect_object_local = self.reserve_temp_local();
        let math_object_local = self.reserve_temp_local();
        let json_object_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let iterator_prototype_local = self.reserve_temp_local();
        let array_iterator_prototype_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let number_prototype_local = self.reserve_temp_local();
        let string_prototype_local = self.reserve_temp_local();
        let boolean_prototype_local = self.reserve_temp_local();
        let bigint_prototype_local = self.reserve_temp_local();
        let array_buffer_prototype_local = self.reserve_temp_local();
        let data_view_prototype_local = self.reserve_temp_local();
        let typed_array_prototype_local = self.reserve_temp_local();
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
        let date_prototype_local = self.reserve_temp_local();
        let function_constructor_local = self.reserve_temp_local();
        let object_constructor_local = self.reserve_temp_local();
        let iterator_constructor_local = self.reserve_temp_local();
        let array_constructor_local = self.reserve_temp_local();
        let number_constructor_local = self.reserve_temp_local();
        let string_constructor_local = self.reserve_temp_local();
        let boolean_constructor_local = self.reserve_temp_local();
        let array_buffer_constructor_local = self.reserve_temp_local();
        let data_view_constructor_local = self.reserve_temp_local();
        let typed_array_constructor_local = self.reserve_temp_local();
        let aggregate_error_constructor_local = self.reserve_temp_local();
        let suppressed_error_constructor_local = self.reserve_temp_local();
        let bigint_constructor_local = self.reserve_temp_local();
        let proxy_constructor_local = self.reserve_temp_local();
        let regexp_constructor_local = self.reserve_temp_local();
        let date_constructor_local = self.reserve_temp_local();
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
        let value_payload_local = self.reserve_temp_local();

        // Each realm build mints its own canonical parseInt/parseFloat objects;
        // clear the get-or-create slots so this realm does not alias the
        // previous realm's (or the main realm's) function identities.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(PARSE_INT_FUNCTION_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(PARSE_FLOAT_FUNCTION_GLOBAL_INDEX));

        self.emit_alloc_realm_record(0, 1, realm_record_local, function)?;

        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        self.emit_store_realm_object_prototype(
            realm_record_local,
            object_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(iterator_prototype_local));
        self.emit_alloc_plain_object_with_prototype(
            Some(iterator_prototype_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(array_iterator_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(array_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
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
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
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
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
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
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(bigint_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(array_buffer_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(data_view_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(typed_array_prototype_local));
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
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
        self.emit_store_realm_type_error_prototype(
            realm_record_local,
            type_error_prototype_local,
            function,
        );
        self.emit_store_realm_array_iterator_prototype(
            realm_record_local,
            array_iterator_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(error_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(suppressed_error_prototype_local));
        for (name, meta) in &string_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        for (name, meta) in &array_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                array_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        for (name, meta) in &object_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                object_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        let error_to_string_payload_local = self.reserve_temp_local();
        self.emit_function_value_payload(&error_prototype_to_string_meta, function)?;
        function.instruction(&Instruction::LocalSet(error_to_string_payload_local));
        self.emit_store_function_defining_realm(
            error_to_string_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            error_to_string_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            error_to_string_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            error_to_string_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            error_prototype_local,
            "toString",
            error_to_string_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(error_to_string_payload_local);
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(regexp_prototype_local));
        for (name, meta) in &regexp_prototype_symbol_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                regexp_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(date_prototype_local));
        for (name, meta) in &date_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                date_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            if *name == "toUTCString" {
                self.emit_object_define_local_data(
                    date_prototype_local,
                    "toGMTString",
                    method_payload_local,
                    tag_local,
                    function,
                )?;
            }
            self.release_temp_local(method_payload_local);
        }
        for (name, meta) in &function_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                function_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        for (_, prototype_local) in &typed_array_prototype_locals {
            self.emit_alloc_plain_object_with_prototype(
                Some(typed_array_prototype_local),
                None,
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
        self.emit_store_function_defining_realm(
            function_constructor_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            object_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            object_constructor_local,
            object_prototype_local,
            false,
            function,
        )?;
        for (name, meta) in &object_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                object_constructor_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(reflect_object_local));
        for (name, meta) in &reflect_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
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
                reflect_object_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(math_object_local));
        self.emit_object_define_string_data(
            math_object_local,
            "Symbol.toStringTag",
            MATH_NAME,
            function,
        )?;
        for (name, value) in [
            ("E", std::f64::consts::E),
            ("LN10", std::f64::consts::LN_10),
            ("LN2", std::f64::consts::LN_2),
            ("LOG10E", std::f64::consts::LOG10_E),
            ("LOG2E", std::f64::consts::LOG2_E),
            ("PI", std::f64::consts::PI),
            ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
            ("SQRT2", std::f64::consts::SQRT_2),
        ] {
            self.emit_object_define_number_data_from_f64_const_with_flags(
                math_object_local,
                name,
                value,
                false,
                false,
                false,
                function,
            )?;
        }
        for (name, meta) in &math_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
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
                math_object_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(json_object_local));
        self.emit_object_define_string_data(
            json_object_local,
            "Symbol.toStringTag",
            JSON_NAME,
            function,
        )?;
        for (name, meta) in &json_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
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
                json_object_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_function_value_payload(&iterator_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_constructor_local));
        self.emit_store_function_defining_realm(
            iterator_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_constructor_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_constructor_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            iterator_constructor_local,
            iterator_prototype_local,
            true,
            function,
        )?;
        let iterator_from_payload_local = self.reserve_temp_local();
        self.emit_function_value_payload(&iterator_from_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_from_payload_local));
        self.emit_store_function_defining_realm(
            iterator_from_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_from_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_from_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_from_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            iterator_constructor_local,
            "from",
            iterator_from_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(iterator_from_payload_local);
        let iterator_identity_payload_local = self.reserve_temp_local();
        self.emit_function_value_payload(&array_iterator_identity_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_identity_payload_local));
        self.emit_store_function_defining_realm(
            iterator_identity_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_identity_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_identity_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_identity_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            iterator_prototype_local,
            "Symbol.iterator",
            iterator_identity_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(iterator_identity_payload_local);
        let iterator_method_payload_local = self.reserve_temp_local();
        for (name, meta) in &iterator_prototype_method_metas {
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(iterator_method_payload_local));
            self.emit_store_function_defining_realm(
                iterator_method_payload_local,
                realm_record_local,
                function,
            );
            self.store_i64_local_at_offset(
                iterator_method_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                iterator_method_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                iterator_method_payload_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                iterator_method_payload_local,
                HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
                range_error_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data(
                iterator_prototype_local,
                name,
                iterator_method_payload_local,
                tag_local,
                function,
            )?;
        }
        self.release_temp_local(iterator_method_payload_local);
        let iterator_accessor_key_local = self.reserve_temp_local();
        let iterator_getter_payload_local = self.reserve_temp_local();
        let iterator_getter_tag_local = self.reserve_temp_local();
        let iterator_setter_payload_local = self.reserve_temp_local();
        let iterator_setter_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(iterator_accessor_key_local));
        self.emit_function_value_payload(&iterator_constructor_getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_getter_payload_local));
        self.emit_store_function_defining_realm(
            iterator_getter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_constructor_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_getter_tag_local));
        self.emit_function_value_payload(&iterator_constructor_setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_setter_payload_local));
        self.emit_store_function_defining_realm(
            iterator_setter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_setter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_setter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_setter_tag_local));
        self.emit_object_define_accessor(
            iterator_prototype_local,
            iterator_accessor_key_local,
            Some((iterator_getter_payload_local, iterator_getter_tag_local)),
            Some((iterator_setter_payload_local, iterator_setter_tag_local)),
            function,
        )?;

        self.emit_function_value_payload(&iterator_symbol_dispose_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_getter_payload_local));
        self.emit_store_function_defining_realm(
            iterator_getter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_getter_tag_local));
        self.emit_object_define_local_data(
            iterator_prototype_local,
            "Symbol.dispose",
            iterator_getter_payload_local,
            iterator_getter_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(iterator_accessor_key_local));
        self.emit_function_value_payload(&iterator_to_string_tag_getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_getter_payload_local));
        self.emit_store_function_defining_realm(
            iterator_getter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_getter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_getter_tag_local));
        self.emit_function_value_payload(&iterator_to_string_tag_setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(iterator_setter_payload_local));
        self.emit_store_function_defining_realm(
            iterator_setter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_setter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            iterator_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_setter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_setter_tag_local));
        self.emit_object_define_accessor(
            iterator_prototype_local,
            iterator_accessor_key_local,
            Some((iterator_getter_payload_local, iterator_getter_tag_local)),
            Some((iterator_setter_payload_local, iterator_setter_tag_local)),
            function,
        )?;

        self.release_temp_local(iterator_setter_tag_local);
        self.release_temp_local(iterator_setter_payload_local);
        self.release_temp_local(iterator_getter_tag_local);
        self.release_temp_local(iterator_getter_payload_local);
        self.release_temp_local(iterator_accessor_key_local);

        self.emit_function_value_payload(&array_meta, function)?;
        function.instruction(&Instruction::LocalSet(array_constructor_local));
        self.emit_store_function_defining_realm(
            array_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            array_constructor_local,
            array_prototype_local,
            true,
            function,
        )?;
        for (name, meta) in &array_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                array_constructor_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        let species_key_local = self.reserve_temp_local();
        let species_getter_payload_local = self.reserve_temp_local();
        let species_getter_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(species_key_local));
        self.emit_function_value_payload(&array_species_meta, function)?;
        function.instruction(&Instruction::LocalSet(species_getter_payload_local));
        self.emit_store_function_defining_realm(
            species_getter_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            species_getter_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            species_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            species_getter_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_getter_tag_local));
        self.emit_object_define_accessor(
            array_constructor_local,
            species_key_local,
            Some((species_getter_payload_local, species_getter_tag_local)),
            None,
            function,
        )?;
        self.release_temp_local(species_getter_tag_local);
        self.release_temp_local(species_getter_payload_local);
        self.release_temp_local(species_key_local);
        for (name, meta) in [
            ("next", &array_iterator_next_meta),
            ("Symbol.iterator", &array_iterator_identity_meta),
        ] {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                array_iterator_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        self.emit_object_define_string_data(
            array_iterator_prototype_local,
            "Symbol.toStringTag",
            "Array Iterator",
            function,
        )?;

        self.emit_function_value_payload(&number_meta, function)?;
        function.instruction(&Instruction::LocalSet(number_constructor_local));
        self.emit_store_function_defining_realm(
            number_constructor_local,
            realm_record_local,
            function,
        );
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
        for (name, value) in [
            ("NaN", f64::NAN),
            ("POSITIVE_INFINITY", f64::INFINITY),
            ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
            ("MAX_VALUE", f64::MAX),
            ("MIN_VALUE", f64::from_bits(1)),
            ("EPSILON", f64::EPSILON),
            ("MAX_SAFE_INTEGER", 9007199254740991.0),
            ("MIN_SAFE_INTEGER", -9007199254740991.0),
        ] {
            self.emit_object_define_number_data_from_f64_const_with_flags(
                number_constructor_local,
                name,
                value,
                false,
                false,
                false,
                function,
            )?;
        }
        for (name, meta) in &number_static_method_metas {
            if let Some(global_index) = canonical_host_function_global_index_by_name(name) {
                self.emit_define_canonical_realm_host_function(
                    number_constructor_local,
                    name,
                    meta,
                    global_index,
                    realm_record_local,
                    type_error_prototype_local,
                    function,
                )?;
                continue;
            }
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
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
                number_constructor_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        for (name, meta) in &number_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                number_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_function_value_payload(&string_meta, function)?;
        function.instruction(&Instruction::LocalSet(string_constructor_local));
        self.emit_store_function_defining_realm(
            string_constructor_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            boolean_constructor_local,
            realm_record_local,
            function,
        );
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
        for (name, meta) in &boolean_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                boolean_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_function_value_payload(&array_buffer_meta, function)?;
        function.instruction(&Instruction::LocalSet(array_buffer_constructor_local));
        self.emit_store_function_defining_realm(
            array_buffer_constructor_local,
            realm_record_local,
            function,
        );
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
        let array_buffer_is_view_payload_local = self.reserve_temp_local();
        self.emit_function_value_payload(&array_buffer_is_view_meta, function)?;
        function.instruction(&Instruction::LocalSet(array_buffer_is_view_payload_local));
        self.emit_store_function_defining_realm(
            array_buffer_is_view_payload_local,
            realm_record_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            array_buffer_constructor_local,
            "isView",
            array_buffer_is_view_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(array_buffer_is_view_payload_local);

        self.emit_function_value_payload(&data_view_meta, function)?;
        function.instruction(&Instruction::LocalSet(data_view_constructor_local));
        self.emit_store_function_defining_realm(
            data_view_constructor_local,
            realm_record_local,
            function,
        );
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

        self.emit_function_value_payload(&function_meta, function)?;
        function.instruction(&Instruction::LocalSet(typed_array_constructor_local));
        self.emit_store_function_defining_realm(
            typed_array_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            typed_array_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            typed_array_constructor_local,
            typed_array_prototype_local,
            true,
            function,
        )?;

        self.emit_function_value_payload(&aggregate_error_meta, function)?;
        function.instruction(&Instruction::LocalSet(aggregate_error_constructor_local));
        self.emit_store_function_defining_realm(
            aggregate_error_constructor_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            suppressed_error_constructor_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            bigint_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            bigint_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data_with_flags(
            bigint_constructor_local,
            bigint_prototype_local,
            false,
            false,
            false,
            true,
            function,
        )?;
        for (name, meta) in &bigint_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
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
                bigint_constructor_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }
        for (name, meta) in &bigint_prototype_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                bigint_prototype_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        self.emit_function_value_payload(&proxy_meta, function)?;
        function.instruction(&Instruction::LocalSet(proxy_constructor_local));
        self.emit_store_function_defining_realm(
            proxy_constructor_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            revocable_payload_local,
            realm_record_local,
            function,
        );
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
        self.emit_store_function_defining_realm(
            regexp_constructor_local,
            realm_record_local,
            function,
        );
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
        let regexp_escape_key_local = self.reserve_temp_local();
        let regexp_escape_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("escape")));
        function.instruction(&Instruction::LocalSet(regexp_escape_key_local));
        self.emit_function_value_payload(&regexp_escape_meta, function)?;
        function.instruction(&Instruction::LocalSet(regexp_escape_payload_local));
        self.emit_store_function_defining_realm(
            regexp_escape_payload_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            regexp_escape_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            regexp_escape_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            regexp_escape_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(
            regexp_constructor_local,
            regexp_escape_key_local,
            regexp_escape_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(regexp_escape_payload_local);
        self.release_temp_local(regexp_escape_key_local);

        self.emit_function_value_payload(&date_meta, function)?;
        function.instruction(&Instruction::LocalSet(date_constructor_local));
        self.emit_store_function_defining_realm(
            date_constructor_local,
            realm_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            date_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            date_constructor_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        self.emit_set_function_prototype_data(
            date_constructor_local,
            date_prototype_local,
            true,
            function,
        )?;
        for (name, meta) in &date_static_method_metas {
            let method_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.emit_store_function_defining_realm(
                method_payload_local,
                realm_record_local,
                function,
            );
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
                date_constructor_local,
                name,
                method_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(method_payload_local);
        }

        for index in 0..error_constructor_metas.len() {
            let (_, meta) = &error_constructor_metas[index];
            let constructor_local = error_constructor_locals[index];
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_store_function_defining_realm(
                constructor_local,
                realm_record_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                function_prototype_local,
                function,
            );
            if meta.name != ERROR_NAME {
                self.store_i64_local_at_offset(
                    constructor_local,
                    HEAP_PROTOTYPE_OFFSET,
                    error_constructor_locals[0],
                    function,
                );
                self.store_i64_const_at_offset(
                    constructor_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
            }
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
                self.emit_set_function_prototype_data(
                    constructor_local,
                    error_prototype_local,
                    false,
                    function,
                )?;
            }
        }
        self.store_i64_local_at_offset(
            aggregate_error_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            error_constructor_locals[0],
            function,
        );
        self.store_i64_const_at_offset(
            aggregate_error_constructor_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            suppressed_error_constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            error_constructor_locals[0],
            function,
        );
        self.store_i64_const_at_offset(
            suppressed_error_constructor_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );

        let error_is_error_payload_local = self.reserve_temp_local();
        self.emit_function_value_payload(&error_is_error_meta, function)?;
        function.instruction(&Instruction::LocalSet(error_is_error_payload_local));
        self.emit_store_function_defining_realm(
            error_is_error_payload_local,
            realm_record_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            error_constructor_locals[0],
            "isError",
            error_is_error_payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(error_is_error_payload_local);

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
            self.emit_store_function_defining_realm(
                constructor_local,
                realm_record_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                typed_array_constructor_local,
                function,
            );
            self.store_i64_const_at_offset(
                constructor_local,
                HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                ValueKind::Function.tag() as u64,
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

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(global_local));
        self.store_i64_local_at_offset(
            realm_record_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            global_local,
            function,
        );
        self.store_i64_local_at_offset(
            realm_record_local,
            HEAP_REALM_GLOBAL_THIS_OFFSET,
            global_local,
            function,
        );
        self.emit_object_define_number_data_from_f64_const_with_flags(
            global_local,
            "Infinity",
            f64::INFINITY,
            false,
            false,
            false,
            function,
        )?;
        self.emit_object_define_number_data_from_f64_const_with_flags(
            global_local,
            "NaN",
            f64::NAN,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data_with_flags(
            global_local,
            "undefined",
            value_payload_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data_with_flags(
            global_local,
            GLOBAL_THIS_NAME,
            global_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;
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
            "Iterator",
            iterator_constructor_local,
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
        self.emit_object_define_local_data(
            global_local,
            DATE_NAME,
            date_constructor_local,
            tag_local,
            function,
        )?;
        for index in 0..error_constructor_metas.len() {
            let (name, _) = &error_constructor_metas[index];
            if *name == AGGREGATE_ERROR_NAME || *name == SUPPRESSED_ERROR_NAME {
                continue;
            }
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
        for (name, meta) in &global_function_metas {
            let function_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(function_payload_local));
            self.emit_store_function_defining_realm(
                function_payload_local,
                realm_record_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                function_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_payload_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            self.emit_object_define_local_data(
                global_local,
                name,
                function_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(function_payload_local);
        }
        for (name, meta) in &global_host_function_metas {
            if let Some(global_index) = canonical_host_function_global_index_by_name(name) {
                self.emit_define_canonical_realm_host_function(
                    global_local,
                    name,
                    meta,
                    global_index,
                    realm_record_local,
                    type_error_prototype_local,
                    function,
                )?;
                continue;
            }
            let function_payload_local = self.reserve_temp_local();
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(function_payload_local));
            self.emit_store_function_defining_realm(
                function_payload_local,
                realm_record_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                function_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                function_payload_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            self.emit_object_define_local_data(
                global_local,
                name,
                function_payload_local,
                tag_local,
                function,
            )?;
            self.release_temp_local(function_payload_local);
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            global_local,
            "Reflect",
            reflect_object_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            MATH_NAME,
            math_object_local,
            tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            global_local,
            JSON_NAME,
            json_object_local,
            tag_local,
            function,
        )?;

        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
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

        self.release_temp_local(value_payload_local);
        self.release_temp_local(tag_local);
        for index in (0..typed_array_constructor_locals.len()).rev() {
            self.release_temp_local(typed_array_constructor_locals[index].1);
            self.release_temp_local(typed_array_prototype_locals[index].1);
        }
        for constructor_local in error_constructor_locals.into_iter().rev() {
            self.release_temp_local(constructor_local);
        }
        self.release_temp_local(date_constructor_local);
        self.release_temp_local(regexp_constructor_local);
        self.release_temp_local(proxy_constructor_local);
        self.release_temp_local(bigint_constructor_local);
        self.release_temp_local(suppressed_error_constructor_local);
        self.release_temp_local(aggregate_error_constructor_local);
        self.release_temp_local(typed_array_constructor_local);
        self.release_temp_local(data_view_constructor_local);
        self.release_temp_local(array_buffer_constructor_local);
        self.release_temp_local(boolean_constructor_local);
        self.release_temp_local(string_constructor_local);
        self.release_temp_local(number_constructor_local);
        self.release_temp_local(array_constructor_local);
        self.release_temp_local(iterator_constructor_local);
        self.release_temp_local(object_constructor_local);
        self.release_temp_local(function_constructor_local);
        self.release_temp_local(date_prototype_local);
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
        self.release_temp_local(typed_array_prototype_local);
        self.release_temp_local(data_view_prototype_local);
        self.release_temp_local(array_buffer_prototype_local);
        self.release_temp_local(bigint_prototype_local);
        self.release_temp_local(boolean_prototype_local);
        self.release_temp_local(string_prototype_local);
        self.release_temp_local(number_prototype_local);
        self.release_temp_local(function_prototype_local);
        self.release_temp_local(array_prototype_local);
        self.release_temp_local(array_iterator_prototype_local);
        self.release_temp_local(iterator_prototype_local);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(json_object_local);
        self.release_temp_local(math_object_local);
        self.release_temp_local(reflect_object_local);
        self.release_temp_local(global_local);
        self.release_temp_local(realm_record_local);
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
