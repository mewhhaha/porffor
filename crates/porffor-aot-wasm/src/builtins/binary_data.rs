use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_throw_if_shared_array_buffer(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload(ARRAY_BUFFER_SHARED_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "ArrayBuffer receiver is SharedArrayBuffer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_throw_if_array_buffer_immutable(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload(ARRAY_BUFFER_IMMUTABLE_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "DataView backing buffer is immutable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_validate_data_view_current_byte_length(
        &mut self,
        view_payload_local: u32,
        view_tag_local: u32,
        buffer_payload_local: u32,
        data_ptr_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let tracking_payload_local = self.reserve_temp_local();
        let tracking_tag_local = self.reserve_temp_local();
        let buffer_byte_length_local = self.reserve_temp_local();

        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_DATA_PTR_SLOT,
            data_ptr_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "DataView backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            buffer_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(DATA_VIEW_LENGTH_TRACKING_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            view_payload_local,
            view_tag_local,
            view_payload_local,
            view_tag_local,
            key_local,
            tracking_payload_local,
            tracking_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(tracking_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tracking_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "DataView byteLength out of bounds",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "DataView byteLength out of bounds",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(buffer_byte_length_local);
        self.release_temp_local(tracking_tag_local);
        self.release_temp_local(tracking_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_typed_array_current_byte_length(
        &mut self,
        typed_array_payload_local: u32,
        _typed_array_tag_local: u32,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tracking_payload_local = self.reserve_temp_local();
        let buffer_byte_length_local = self.reserve_temp_local();

        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            buffer_byte_length_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,
            tracking_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tracking_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(buffer_byte_length_local);
        self.release_temp_local(tracking_payload_local);
        Ok(())
    }

    pub(crate) fn emit_validate_typed_array_current_byte_length(
        &mut self,
        typed_array_payload_local: u32,
        typed_array_tag_local: u32,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let tracking_payload_local = self.reserve_temp_local();
        let tracking_tag_local = self.reserve_temp_local();
        let buffer_byte_length_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();

        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_DATA_PTR_SLOT,
            data_ptr_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_read_number_slot_to_i64_local(
            buffer_payload_local,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            buffer_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_LENGTH_TRACKING_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            typed_array_payload_local,
            typed_array_tag_local,
            typed_array_payload_local,
            typed_array_tag_local,
            key_local,
            tracking_payload_local,
            tracking_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(tracking_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tracking_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray byteLength out of bounds",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "TypedArray byteLength out of bounds",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_byte_length_local);
        self.release_temp_local(tracking_tag_local);
        self.release_temp_local(tracking_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_array_buffer_slice_index_to_local(
        &mut self,
        arg_index: usize,
        length_local: u32,
        default_to_length: bool,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let int_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(arg_index, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(arg_index as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        if default_to_length {
            function.instruction(&Instruction::LocalGet(length_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(int_local));
        function.instruction(&Instruction::LocalGet(int_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalGet(int_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::LocalGet(dest_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(int_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(int_local));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(int_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_buffer_transfer_length_to_local(
        &mut self,
        default_length_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(default_length_local));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            MAX_ARRAY_BUFFER_BYTE_LENGTH as f64,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "ArrayBuffer transfer length is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn emit_half_bits_to_f64_payload(
        &mut self,
        half_local: u32,
        sign_local: u32,
        exp_local: u32,
        frac_local: u32,
        f32_bits_local: u32,
        norm_exp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(half_local));
        function.instruction(&Instruction::I64Const(0x8000));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(half_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x1f));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(exp_local));
        function.instruction(&Instruction::LocalGet(half_local));
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(frac_local));

        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(f32_bits_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-14));
        function.instruction(&Instruction::LocalSet(norm_exp_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(0x0400));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(frac_local));
        function.instruction(&Instruction::LocalGet(norm_exp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(norm_exp_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(frac_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(norm_exp_local));
        function.instruction(&Instruction::I64Const(127));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(f32_bits_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7f800000));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(f32_bits_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(112));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(f32_bits_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(f32_bits_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::F32ReinterpretI32);
        function.instruction(&Instruction::F64PromoteF32);
    }

    pub(crate) fn emit_f64_payload_to_half_bits_local(
        &mut self,
        value_payload_local: u32,
        half_local: u32,
        sign_local: u32,
        exp_local: u32,
        frac_local: u32,
        f32_bits_local: u32,
        shift_local: u32,
        temp_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x8000));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F32DemoteF64);
        function.instruction(&Instruction::I32ReinterpretF32);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(f32_bits_local));
        function.instruction(&Instruction::LocalGet(f32_bits_local));
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(exp_local));
        function.instruction(&Instruction::LocalGet(f32_bits_local));
        function.instruction(&Instruction::I64Const(0x7fffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(frac_local));

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.00006103515625)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            0.0000000298023223876953125,
        )));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            0.000000059604644775390625,
        )));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(65520.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(65504.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7bff));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(255));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0x0200));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(142));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(113));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(0x0fff));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(112));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0xffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(103));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(126));
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::I64Const(0x800000));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0xffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
