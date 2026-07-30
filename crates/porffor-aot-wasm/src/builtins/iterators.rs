use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_string_iterator_create_from_local(
        &mut self,
        string_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_load_function_defining_realm_string_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_object_define_local_data(
            object_local,
            "$StringIterator.string",
            string_payload_local,
            string_tag_local,
            function,
        )?;
        self.emit_object_define_number_data_from_i64_const(
            object_local,
            "$StringIterator.index",
            0,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(string_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_array_iterator_create_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        kind: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_array_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_local_data(
            object_local,
            "$ArrayIterator.array",
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_object_define_number_data_from_i64_const(
            object_local,
            "$ArrayIterator.index",
            0,
            function,
        )?;
        self.emit_object_define_bool_data(object_local, "$ArrayIterator.done", false, function)?;
        self.emit_object_define_number_data_from_i64_const(
            object_local,
            "$ArrayIterator.kind",
            kind,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_typed_array_iterator_create_from_locals(
        &mut self,
        typed_array_payload_local: u32,
        kind: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_record_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_array_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(iterator_payload_local));
        self.emit_heap_alloc_const(HEAP_TYPED_ARRAY_ITERATOR_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(iterator_record_local));
        self.store_i64_local_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET,
            typed_array_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET,
            kind,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY_ITERATOR,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            iterator_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            iterator_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(iterator_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(prototype_local);
        self.release_temp_local(iterator_record_local);
        self.release_temp_local(iterator_payload_local);
        Ok(())
    }

    pub(crate) fn emit_typed_array_iterator_next_from_locals(
        &mut self,
        this_payload_local: u32,
        this_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterator_brand_local = self.reserve_temp_local();
        let iterator_record_local = self.reserve_temp_local();
        let typed_array_payload_local = self.reserve_temp_local();
        let typed_array_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let buffer_payload_local = self.reserve_temp_local();
        let byte_offset_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let entry_array_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(iterator_brand_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            iterator_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(iterator_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY_ITERATOR as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            iterator_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET,
            done_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_TYPED_ARRAY_PAYLOAD_OFFSET,
            typed_array_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(typed_array_tag_local));
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_KIND_OFFSET,
            kind_local,
            function,
        );
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
        self.emit_validate_typed_array_current_byte_length(
            typed_array_payload_local,
            typed_array_tag_local,
            buffer_payload_local,
            byte_offset_local,
            byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(length_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_DONE_OFFSET,
            1,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(ARRAY_ITERATOR_KIND_KEYS as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            typed_array_payload_local,
            typed_array_tag_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(ARRAY_ITERATOR_KIND_ENTRIES as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(entry_index_local));
        self.emit_alloc_array_payload_with_length(entry_index_local, entry_array_local, function)?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_index_local));
        self.emit_array_write(
            entry_array_local,
            entry_index_local,
            index_payload_local,
            index_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(entry_index_local));
        self.emit_array_write(
            entry_array_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entry_array_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.store_i64_local_at_offset(
            iterator_record_local,
            HEAP_TYPED_ARRAY_ITERATOR_NEXT_INDEX_OFFSET,
            index_local,
            function,
        );
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(entry_array_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(length_local);
        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(byte_offset_local);
        self.release_temp_local(buffer_payload_local);
        self.release_temp_local(done_local);
        self.release_temp_local(kind_local);
        self.release_temp_local(index_local);
        self.release_temp_local(typed_array_tag_local);
        self.release_temp_local(typed_array_payload_local);
        self.release_temp_local(iterator_record_local);
        self.release_temp_local(iterator_brand_local);
        Ok(())
    }

    pub(crate) fn emit_iterator_result_object_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        done: bool,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        self.emit_load_function_defining_realm_object_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_local_data_with_flags(
            object_local,
            "value",
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(i64::from(done)));
        function.instruction(&Instruction::LocalSet(done_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(done_tag_local));
        self.emit_object_define_local_data_with_flags(
            object_local,
            "done",
            done_payload_local,
            done_tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_string_iterator_next_from_locals(
        &mut self,
        this_payload_local: u32,
        this_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let slot_present_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_length_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let next_codepoint_local = self.reserve_temp_local();
        let next_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$StringIterator.string"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            string_payload_local,
            string_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(string_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$StringIterator.index"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            index_payload_local,
            index_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_length_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);

        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            string_offset_local,
            index_local,
            string_length_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        self.emit_is_high_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(string_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(
            string_offset_local,
            next_index_local,
            next_byte_local,
            function,
        );
        self.emit_decode_utf8_scalar_at_index(
            string_offset_local,
            next_index_local,
            string_length_local,
            next_byte_local,
            next_codepoint_local,
            next_advance_local,
            temp_local,
            function,
        );
        self.emit_is_low_surrogate_i32(next_codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_object_write(
            this_payload_local,
            this_tag_local,
            key_local,
            index_payload_local,
            index_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(advance_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            index_local,
            advance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_iterator_result_object_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            value_tag_local,
            value_payload_local,
            temp_local,
            next_advance_local,
            next_codepoint_local,
            next_byte_local,
            next_index_local,
            advance_local,
            codepoint_local,
            byte_local,
            string_length_local,
            string_offset_local,
            index_local,
            index_tag_local,
            index_payload_local,
            string_tag_local,
            string_payload_local,
            slot_present_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_regexp_string_iterator_create_from_locals(
        &mut self,
        regexp_payload_local: u32,
        regexp_tag_local: u32,
        string_payload_local: u32,
        global_local: u32,
        unicode_local: u32,
        last_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_load_function_defining_realm_array_iterator_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_local_data(
            object_local,
            "$RegExpStringIterator.regexp",
            regexp_payload_local,
            regexp_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        self.emit_object_define_local_data(
            object_local,
            "$RegExpStringIterator.string",
            string_payload_local,
            string_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            object_local,
            "$RegExpStringIterator.global",
            global_local,
            function,
        )?;
        self.emit_object_define_bool_data_from_local(
            object_local,
            "$RegExpStringIterator.unicode",
            unicode_local,
            function,
        )?;
        self.emit_object_define_bool_data(
            object_local,
            "$RegExpStringIterator.done",
            false,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write_strict(
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            index_payload_local,
            index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(key_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_string_iterator_next_from_locals(
        &mut self,
        this_payload_local: u32,
        this_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let slot_present_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let regexp_payload_local = self.reserve_temp_local();
        let regexp_tag_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let global_payload_local = self.reserve_temp_local();
        let global_tag_local = self.reserve_temp_local();
        let unicode_payload_local = self.reserve_temp_local();
        let unicode_tag_local = self.reserve_temp_local();
        let exec_payload_local = self.reserve_temp_local();
        let exec_tag_local = self.reserve_temp_local();
        let match_payload_local = self.reserve_temp_local();
        let match_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let match_string_payload_local = self.reserve_temp_local();
        let empty_string_payload_local = self.reserve_temp_local();
        let last_index_payload_local = self.reserve_temp_local();
        let last_index_tag_local = self.reserve_temp_local();
        let last_index_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let input_offset_local = self.reserve_temp_local();
        let input_len_local = self.reserve_temp_local();
        let input_utf16_length_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let match_array_payload_local = self.reserve_temp_local();
        let match_array_tag_local = self.reserve_temp_local();
        let string_arg_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$RegExpStringIterator.done"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            done_payload_local,
            done_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_tag_local));
        self.emit_iterator_result_object_from_locals(
            match_payload_local,
            match_tag_local,
            true,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$RegExpStringIterator.regexp"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            regexp_payload_local,
            regexp_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$RegExpStringIterator.string"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            string_payload_local,
            string_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$RegExpStringIterator.global"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            global_payload_local,
            global_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("$RegExpStringIterator.unicode"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            slot_present_local,
            unicode_payload_local,
            unicode_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator next called on incompatible receiver",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_payload_local,
            regexp_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            exec_payload_local,
            exec_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_is_callable_i32(exec_tag_local, exec_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(
            REGEXP_PROTOTYPE_EXEC_FUNCTION_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(exec_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(exec_tag_local));
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(exec_tag_local, exec_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_arg_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            exec_payload_local,
            exec_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            &[(string_payload_local, string_arg_tag_local)],
            match_payload_local,
            match_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "RegExp String Iterator exec returned non-object",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(match_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(last_index_local));
        self.compile_truthy_tagged_i32(global_tag_local, global_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_payload_local,
            regexp_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            last_index_payload_local,
            last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            last_index_tag_local,
            last_index_payload_local,
            last_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_unpack_string_payload(
            string_payload_local,
            input_offset_local,
            input_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::LocalGet(input_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            last_index_local,
            one_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_string_match_array_from_locals(
            string_payload_local,
            element_payload_local,
            index_payload_local,
            match_array_payload_local,
            match_array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(match_array_payload_local));
        function.instruction(&Instruction::LocalSet(match_payload_local));
        function.instruction(&Instruction::LocalGet(match_array_tag_local));
        function.instruction(&Instruction::LocalSet(match_tag_local));
        self.compile_truthy_tagged_i32(global_tag_local, global_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(last_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(last_index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(last_index_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            last_index_payload_local,
            last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_define_bool_data(
            this_payload_local,
            "$RegExpStringIterator.done",
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        self.emit_iterator_result_object_from_locals(
            element_payload_local,
            element_tag_local,
            true,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.compile_truthy_tagged_i32(global_tag_local, global_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            match_payload_local,
            match_tag_local,
            match_payload_local,
            match_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(match_string_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(empty_string_payload_local));
        self.emit_string_payload_equality_i32(
            match_string_payload_local,
            empty_string_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_payload_local,
            regexp_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            last_index_payload_local,
            last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            last_index_tag_local,
            last_index_payload_local,
            last_index_local,
            function,
        )?;
        self.emit_unpack_string_payload(
            string_payload_local,
            input_offset_local,
            input_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            input_offset_local,
            input_len_local,
            input_utf16_length_local,
            function,
        );
        self.emit_advance_string_index_from_locals(
            string_payload_local,
            input_utf16_length_local,
            last_index_local,
            unicode_payload_local,
            next_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(last_index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(last_index_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            last_index_payload_local,
            last_index_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_define_bool_data(
            this_payload_local,
            "$RegExpStringIterator.done",
            true,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_iterator_result_object_from_locals(
            match_payload_local,
            match_tag_local,
            false,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(string_arg_tag_local);
        self.release_temp_local(match_array_tag_local);
        self.release_temp_local(match_array_payload_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(one_local);
        self.release_temp_local(input_utf16_length_local);
        self.release_temp_local(input_len_local);
        self.release_temp_local(input_offset_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(last_index_local);
        self.release_temp_local(last_index_tag_local);
        self.release_temp_local(last_index_payload_local);
        self.release_temp_local(empty_string_payload_local);
        self.release_temp_local(match_string_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(match_tag_local);
        self.release_temp_local(match_payload_local);
        self.release_temp_local(exec_tag_local);
        self.release_temp_local(exec_payload_local);
        self.release_temp_local(unicode_tag_local);
        self.release_temp_local(unicode_payload_local);
        self.release_temp_local(global_tag_local);
        self.release_temp_local(global_payload_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(string_payload_local);
        self.release_temp_local(regexp_tag_local);
        self.release_temp_local(regexp_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(slot_present_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
