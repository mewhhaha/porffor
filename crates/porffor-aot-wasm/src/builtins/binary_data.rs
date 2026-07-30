use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_array_buffer_memory_load(
        &self,
        _buffer_flags_local: u32,
        _result_type: ValType,
        private_load: Instruction<'static>,
        shared_load: Instruction<'static>,
        function: &mut Function,
    ) {
        function.instruction(if self.buffer_memory_index() == 1 {
            &shared_load
        } else {
            &private_load
        });
    }

    pub(crate) fn emit_is_typed_array_i32(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            function,
        );
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_load_typed_array_private_state(
        &self,
        typed_array_payload_local: u32,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        bytes_per_element_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            bytes_per_element_local,
            function,
        );
    }

    pub(crate) fn emit_initialize_array_buffer_private_state(
        &mut self,
        buffer_payload_local: u32,
        data_payload_local: u32,
        byte_length_local: u32,
        max_byte_length_local: u32,
        flags_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_local_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DATA_OFFSET,
            data_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.store_i64_local_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_MAX_BYTE_LENGTH_OFFSET,
            max_byte_length_local,
            function,
        );
        self.store_i64_const_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DETACH_KEY_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DETACH_KEY_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_FLAGS_OFFSET,
            flags_local,
            function,
        );
    }

    pub(crate) fn emit_require_array_buffer(
        &mut self,
        buffer_payload_local: u32,
        buffer_tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(buffer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_require_array_buffer_or_shared_array_buffer(
        &mut self,
        buffer_payload_local: u32,
        buffer_tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(buffer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_require_shared_array_buffer(
        &mut self,
        buffer_payload_local: u32,
        buffer_tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(buffer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_load_array_buffer_data(
        &self,
        buffer_payload_local: u32,
        destination_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DATA_OFFSET,
            destination_local,
            function,
        );
    }

    pub(crate) fn emit_load_array_buffer_byte_length(
        &self,
        buffer_payload_local: u32,
        destination_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_BYTE_LENGTH_OFFSET,
            destination_local,
            function,
        );
    }

    pub(crate) fn emit_load_array_buffer_max_byte_length(
        &self,
        buffer_payload_local: u32,
        destination_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_MAX_BYTE_LENGTH_OFFSET,
            destination_local,
            function,
        );
    }

    pub(crate) fn emit_load_array_buffer_flags(
        &self,
        buffer_payload_local: u32,
        destination_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_FLAGS_OFFSET,
            destination_local,
            function,
        );
    }

    pub(crate) fn emit_initialize_typed_array_from_array_buffer(
        &mut self,
        buffer_payload_local: u32,
        offset_payload_local: u32,
        offset_tag_local: u32,
        explicit_length_payload_local: u32,
        explicit_length_tag_local: u32,
        bytes_per_element_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        length_local: u32,
        length_tracking_local: u32,
        data_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_byte_length_local = self.reserve_temp_local();
        let buffer_flags_local = self.reserve_temp_local();

        self.emit_value_to_number_payload(offset_tag_local, offset_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(offset_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            offset_payload_local,
            byte_offset_local,
            "TypedArray byteOffset out of range",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_range_error(
            "TypedArray byteOffset must be aligned",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_tracking_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(explicit_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(
            explicit_length_tag_local,
            explicit_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(explicit_length_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(
            explicit_length_payload_local,
            length_local,
            "TypedArray length out of range",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_flags(buffer_payload_local, buffer_flags_local, function);
        function.instruction(&Instruction::LocalGet(buffer_flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "TypedArray backing buffer is detached",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_load_array_buffer_byte_length(
            buffer_payload_local,
            buffer_byte_length_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray byteOffset out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(explicit_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "TypedArray byteLength out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(buffer_byte_length_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        function.instruction(&Instruction::LocalGet(buffer_flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_RESIZABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_range_error(
            "TypedArray byteLength must be aligned",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(length_tracking_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_data(buffer_payload_local, data_payload_local, function);
        self.release_temp_local(buffer_flags_local);
        self.release_temp_local(buffer_byte_length_local);
        Ok(())
    }

    pub(crate) fn emit_detach_array_buffer(
        &mut self,
        buffer_payload_local: u32,
        buffer_tag_local: u32,
        detach_key_payload_local: u32,
        detach_key_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stored_key_payload_local = self.reserve_temp_local();
        let stored_key_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.emit_require_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            "detachArrayBuffer expects an ArrayBuffer",
            function,
        )?;
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DETACH_KEY_PAYLOAD_OFFSET,
            stored_key_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DETACH_KEY_TAG_OFFSET,
            stored_key_tag_local,
            function,
        );
        self.emit_tagged_payload_same_value_i32(
            stored_key_tag_local,
            stored_key_payload_local,
            detach_key_tag_local,
            detach_key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "detachArrayBuffer key does not match the ArrayBuffer detach key",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_load_array_buffer_flags(buffer_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_DETACHED as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.store_i64_local_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_FLAGS_OFFSET,
            flags_local,
            function,
        );
        self.store_i64_const_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_DATA_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            buffer_payload_local,
            HEAP_ARRAY_BUFFER_BYTE_LENGTH_OFFSET,
            0,
            function,
        );

        self.release_temp_local(flags_local);
        self.release_temp_local(stored_key_tag_local);
        self.release_temp_local(stored_key_payload_local);
        Ok(())
    }

    pub(crate) fn emit_throw_if_shared_array_buffer(
        &mut self,
        receiver_payload_local: u32,
        _receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_SHARED_ARRAY_BUFFER as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "ArrayBuffer receiver is SharedArrayBuffer",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_throw_if_array_buffer_immutable(
        &mut self,
        receiver_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flags_local = self.reserve_temp_local();
        self.emit_load_array_buffer_flags(receiver_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(ARRAY_BUFFER_FLAG_IMMUTABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "DataView backing buffer is immutable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(flags_local);
        Ok(())
    }

    pub(crate) fn emit_initialize_data_view_private_state(
        &self,
        view_payload_local: u32,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        length_tracking_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            view_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_DATA_VIEW,
            function,
        );
        self.store_i64_local_at_offset(
            view_payload_local,
            HEAP_DATA_VIEW_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            view_payload_local,
            HEAP_DATA_VIEW_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.store_i64_local_at_offset(
            view_payload_local,
            HEAP_DATA_VIEW_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.store_i64_local_at_offset(
            view_payload_local,
            HEAP_DATA_VIEW_LENGTH_TRACKING_OFFSET,
            length_tracking_local,
            function,
        );
    }

    pub(crate) fn emit_require_data_view(
        &mut self,
        view_payload_local: u32,
        view_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(brand_local));
        function.instruction(&Instruction::LocalGet(view_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            view_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_DATA_VIEW as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "DataView accessor requires DataView",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_validate_data_view_current_byte_length(
        &mut self,
        view_payload_local: u32,
        _view_tag_local: u32,
        buffer_payload_local: u32,
        data_ptr_local: u32,
        byte_offset_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tracking_payload_local = self.reserve_temp_local();
        let buffer_byte_length_local = self.reserve_temp_local();

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
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

        self.emit_load_array_buffer_byte_length(
            buffer_payload_local,
            buffer_byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            view_payload_local,
            HEAP_DATA_VIEW_LENGTH_TRACKING_OFFSET,
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
        self.release_temp_local(tracking_payload_local);
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

        self.emit_load_array_buffer_byte_length(
            buffer_payload_local,
            buffer_byte_length_local,
            function,
        );
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

    pub(crate) fn emit_typed_array_valid_integer_index_i32(
        &mut self,
        typed_array_payload_local: u32,
        typed_array_tag_local: u32,
        numeric_index_payload_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_payload_local = self.reserve_temp_local();
        let data_ptr_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            18_446_744_073_709_551_616.0,
        )));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(numeric_index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(index_local));

        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_payload_local,
            function,
        );
        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
        function.instruction(&Instruction::LocalGet(data_ptr_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTE_OFFSET,
            byte_offset_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            typed_array_payload_local,
            HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            bytes_per_element_local,
            function,
        );
        self.emit_typed_array_current_byte_length(
            typed_array_payload_local,
            typed_array_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(data_ptr_local);
        self.release_temp_local(buffer_payload_local);
        Ok(())
    }

    pub(crate) fn emit_validate_typed_array_current_byte_length(
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
        let data_ptr_local = self.reserve_temp_local();

        self.emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function);
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

        self.emit_load_array_buffer_byte_length(
            buffer_payload_local,
            buffer_byte_length_local,
            function,
        );
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
        self.release_temp_local(tracking_payload_local);
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
        fraction_local: u32,
        rounded_local: u32,
        remainder_local: u32,
        significand_local: u32,
        function: &mut Function,
    ) {
        // Binary16 must be rounded directly from f64. An f32 intermediate
        // double-rounds values immediately adjacent to binary16 midpoints.
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x8000));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(exp_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Const(0x000f_ffff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(fraction_local));

        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(fraction_local));
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
        function.instruction(&Instruction::I64Const(1009));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(16_777_216.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(1038));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(0x0010_0000_0000_0000));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand_local));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(42));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(rounded_local));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(0x0000_03ff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(remainder_local));

        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(0x0000_0200_0000_0000));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(0x0000_0200_0000_0000));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rounded_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rounded_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rounded_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(1008));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exp_local));
        function.instruction(&Instruction::LocalGet(rounded_local));
        function.instruction(&Instruction::I64Const(2048));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1024));
        function.instruction(&Instruction::LocalSet(rounded_local));
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exp_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(exp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(rounded_local));
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(half_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
