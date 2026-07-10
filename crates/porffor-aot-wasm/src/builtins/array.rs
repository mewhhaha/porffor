use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_init_array_constructor_slot(
        &self,
        array_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET,
            0,
            function,
        );
        for (descriptor_offset, tag_offset, payload_offset) in [
            (
                HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,
            ),
            (
                HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,
                HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,
            ),
        ] {
            self.store_i64_const_at_offset(array_local, descriptor_offset, 0, function);
            self.store_i64_const_at_offset(
                array_local,
                tag_offset,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(array_local, payload_offset, 0, function);
        }
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET, 0, function);
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET, 0, function);
    }

    pub(crate) fn emit_array_is_concat_spreadable_read(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            getter_payload_local,
            getter_tag_local,
            Some((array_local, Some(receiver_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_array_is_concat_spreadable_write(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET,
            u64::MAX,
            function,
        );
        function.instruction(&Instruction::Else);
        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_IS_CONCAT_SPREADABLE_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_array_constructor_read(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        fallback_to_array_constructor: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(ARRAY_DESCRIPTOR_OWN_PROPERTY as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if fallback_to_array_constructor {
            function.instruction(&Instruction::GlobalGet(ARRAY_CONSTRUCTOR_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            prototype_payload_local,
            prototype_tag_local,
            array_local,
            receiver_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_CONSTRUCTOR_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            getter_payload_local,
            getter_tag_local,
            Some((array_local, Some(receiver_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_mark_skip_species_for_cross_realm_array_constructor(
        &mut self,
        constructor_payload_local: u32,
        constructor_table_index_local: u32,
        skip_species_local: u32,
        array_constructor_table_index: i64,
        function: &mut Function,
    ) {
        let constructor_realm_local = self.reserve_temp_local();
        let current_function_realm_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(constructor_table_index_local));
        function.instruction(&Instruction::I64Const(array_constructor_table_index));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            constructor_realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            current_function_realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_realm_local));
        function.instruction(&Instruction::LocalGet(current_function_realm_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(current_function_realm_local);
        self.release_temp_local(constructor_realm_local);
    }

    pub(crate) fn compile_array_literal_payload(
        &mut self,
        elements: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let capacity = (elements.len() as u64).max(MIN_HEAP_CAPACITY);
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        self.emit_heap_alloc_const(capacity * HEAP_ARRAY_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(
            array_local,
            HEAP_LEN_OFFSET,
            elements.len() as u64,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_CAP_OFFSET, capacity, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_init_array_constructor_slot(array_local, function);

        let entry_local = self.reserve_temp_local();
        let present_index_local = self.reserve_temp_local();
        for (index, element) in elements.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            if matches!(element.expr, ExprIr::ArrayHole) {
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_TAG_OFFSET,
                    HEAP_ARRAY_HOLE_TAG as u64,
                    function,
                );
                self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                    0,
                    function,
                );
                continue;
            }
            let value_payload = self.reserve_temp_local();
            let value_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(element, value_payload, value_tag, function)?;
            self.emit_propagate_throw_from_locals_if_needed(value_payload, value_tag, function)?;
            self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, value_tag, function);
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_PAYLOAD_OFFSET,
                value_payload,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                ARRAY_DESCRIPTOR_NORMAL_DATA,
                function,
            );
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(present_index_local));
            self.emit_array_append_present_index(
                array_local,
                present_index_local,
                value_payload,
                value_tag,
                function,
            )?;
            self.release_temp_local(value_tag);
            self.release_temp_local(value_payload);
        }
        self.release_temp_local(present_index_local);
        self.release_temp_local(entry_local);

        function.instruction(&Instruction::LocalGet(array_local));
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_array_sparse_present_read(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
    }

    pub(crate) fn emit_array_sparse_present_get(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_throw_extra_depth(
            getter_payload_local,
            getter_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[],
            payload_local,
            tag_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        Ok(())
    }

    pub(crate) fn emit_array_read(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_named_prop_read(
            array_local,
            key_local,
            payload_local,
            tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_read(
            array_local,
            index_local,
            payload_local,
            tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_index_get(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_named_prop_read(
            array_local,
            key_local,
            payload_local,
            tag_local,
            found_output_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_get(
            array_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            found_output_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_throw_extra_depth(
            getter_payload_local,
            getter_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &[],
            payload_local,
            tag_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_HOLE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_index_get_with_prototype(
        &mut self,
        array_local: u32,
        index_local: u32,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let found_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();

        self.emit_array_index_get(
            array_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            payload_local,
            tag_local,
            Some(found_local),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.emit_object_read_ordinary(
            prototype_local,
            prototype_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(found_local);
        Ok(())
    }

    pub(crate) fn emit_array_own_index_present_i64(
        &mut self,
        array_local: u32,
        index_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let scratch_payload_local = self.reserve_temp_local();
        let scratch_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_sparse_present_read(
            array_local,
            index_local,
            scratch_payload_local,
            scratch_tag_local,
            Some(found_local),
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(scratch_tag_local);
        self.release_temp_local(scratch_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_assignment_write(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let present_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_array_own_index_present_i64(array_local, index_local, present_local, function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_inherited_index_set_state(
            array_local,
            index_local,
            payload_local,
            tag_local,
            state_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if self.is_current_function_strict() {
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Cannot assign to read only property",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(array_local, index_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(state_local);
        self.release_temp_local(present_local);
        Ok(())
    }

    pub(crate) fn emit_array_inherited_index_set_state(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        state_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype_buffer_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();
        let prototype_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let prototype_proxy_kind_local = self.reserve_temp_local();
        let reflect_set_payload_local = self.reserve_temp_local();
        let reflect_set_tag_local = self.reserve_temp_local();
        let reflect_set_result_payload_local = self.reserve_temp_local();
        let reflect_set_result_tag_local = self.reserve_temp_local();
        let reflect_set_truthy_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_payload_local,
            function,
        )?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            prototype_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&reflect_set_meta, function)?;
        function.instruction(&Instruction::LocalSet(reflect_set_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(reflect_set_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        self.emit_function_handle_call(
            reflect_set_payload_local,
            reflect_set_tag_local,
            None,
            &[
                (prototype_local, prototype_tag_local),
                (key_payload_local, key_tag_local),
                (payload_local, tag_local),
                (array_local, receiver_tag_local),
            ],
            reflect_set_result_payload_local,
            reflect_set_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(
            reflect_set_result_tag_local,
            reflect_set_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(reflect_set_truthy_local));
        function.instruction(&Instruction::LocalGet(reflect_set_truthy_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_PTR_OFFSET,
            prototype_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(prototype_buffer_local));
        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));

        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(entry_key_local, key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(prototype_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.emit_function_handle_call(
            setter_payload_local,
            setter_tag_local,
            Some((array_local, Some(prototype_tag_local))),
            &[(payload_local, tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(reflect_set_truthy_local);
        self.release_temp_local(reflect_set_result_tag_local);
        self.release_temp_local(reflect_set_result_payload_local);
        self.release_temp_local(reflect_set_tag_local);
        self.release_temp_local(reflect_set_payload_local);
        self.release_temp_local(prototype_proxy_kind_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(prototype_index_local);
        self.release_temp_local(prototype_len_local);
        self.release_temp_local(prototype_buffer_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_string_index_read(
        &mut self,
        string_payload_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let unit_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let code_unit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.emit_unpack_string_payload(
            string_payload_local,
            offset_local,
            byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            offset_local,
            byte_len_local,
            unit_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(unit_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            offset_local,
            byte_index_local,
            byte_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::LocalSet(code_unit_local));
        function.instruction(&Instruction::End);
        self.emit_string_payload_from_utf16_code_unit_local(
            code_unit_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(code_unit_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(unit_advance_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(unit_index_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(unit_len_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(offset_local);
        Ok(())
    }

    pub(crate) fn emit_string_payload_from_utf16_code_unit_local(
        &mut self,
        code_unit_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(code_unit_local));
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(code_unit_local));
        function.instruction(&Instruction::I64Const(0x800));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_from_local(len_local, function)?;
        function.instruction(&Instruction::LocalSet(offset_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalSet(pos_local));
        self.emit_store_utf8_codepoint(pos_local, code_unit_local, temp_local, function);
        self.emit_pack_string_payload(offset_local, len_local, function);
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(temp_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(offset_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_array_length(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, payload_local, function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
    }

    pub(crate) fn emit_array_or_object_length_read(
        &mut self,
        target_local: u32,
        target_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(target_local, payload_local, tag_local, function);
        function.instruction(&Instruction::Else);

        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            target_local,
            target_tag_local,
            target_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(key_local);

        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_length_writable_i64(
        &mut self,
        array_local: u32,
        writable_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_key_local = self.reserve_temp_local();
        let length_key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(writable_local));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            stored_key_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(length_key_local));
        self.emit_string_payload_equality_i32(stored_key_local, length_key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(length_key_local);
        self.release_temp_local(stored_key_local);
        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_array_store_length_writable_descriptor(
        &mut self,
        array_local: u32,
        writable_payload_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as i64,
        ));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_DATA_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_GETTER_PAYLOAD_OFFSET,
            self.strings.payload("length") as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            array_local,
            HEAP_ARRAY_PROP_SETTER_PAYLOAD_OFFSET,
            0,
            function,
        );

        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_to_length_i64_from_value_locals(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            MAX_SAFE_INTEGER as f64,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_to_repeat_count_i64_from_value_locals(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_return_current_completion_if_throw(function);

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
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "repeat count must be non-negative and finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_grow_buffer(
        &mut self,
        array_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        required_index_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let required_len_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(required_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(required_len_local));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(required_len_local));
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(required_len_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_array_append_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_cap_local = self.reserve_temp_local();
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_list_ptr_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            list_cap_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(list_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_list_ptr_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));
        function.instruction(&Instruction::LocalGet(new_list_ptr_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));
        for offset in [
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_list_ptr_local));
        function.instruction(&Instruction::LocalSet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(list_cap_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            list_cap_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            index_local,
            function,
        );
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            new_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_list_ptr_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        self.release_temp_local(list_cap_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        Ok(())
    }

    pub(crate) fn emit_array_write(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_DENSE_ARRAY_INDEX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(
            SPARSE_ARRAY_DENSE_GROW_FACTOR as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            cap_local,
            cap_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_append_present_index(
            array_local,
            index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_create_data_property_silent(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let can_define_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(can_define_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(can_define_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(can_define_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(array_local, index_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(can_define_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_set_length_from_number_payload(
        &mut self,
        array_local: u32,
        length_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_len_local = self.reserve_temp_local();
        let old_len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let fill_index_local = self.reserve_temp_local();
        let fill_entry_local = self.reserve_temp_local();
        let fill_descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(new_len_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, old_len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);

        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(fill_index_local));
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            old_len_local,
            cap_local,
            fill_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(old_len_local));
        function.instruction(&Instruction::LocalSet(fill_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::LocalGet(new_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(fill_index_local));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fill_entry_local));
        self.load_i64_to_local_from_offset(
            fill_entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            fill_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(fill_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(fill_descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fill_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_len_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            fill_entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_HOLE_TAG as u64,
            function,
        );
        self.store_i64_const_at_offset(fill_entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            fill_entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, new_len_local, function);

        self.release_temp_local(fill_descriptor_kind_local);
        self.release_temp_local(fill_entry_local);
        self.release_temp_local(fill_index_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(old_len_local);
        self.release_temp_local(new_len_local);
        Ok(())
    }

    pub(crate) fn emit_known_array_index_from_property_key(
        &mut self,
        key_local: u32,
        index_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        self.emit_unpack_string_payload(key_local, string_offset_local, string_len_local, function);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
    }

    pub(crate) fn emit_array_descriptor_flags_to_local(
        &mut self,
        descriptor_base: u64,
        writable_payload_local: Option<u32>,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(descriptor_base as i64));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        if let Some(writable_payload_local) = writable_payload_local {
            function.instruction(&Instruction::LocalGet(writable_payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_store_array_descriptor_at_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_store_array_sparse_present_descriptor_at_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
    }

    pub(crate) fn emit_store_array_descriptor_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind_local: u32,
        function: &mut Function,
    ) {
        let cap_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_array_descriptor_at_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_store_array_sparse_present_descriptor_at_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(cap_local);
    }

    pub(crate) fn emit_store_array_descriptor_const_at_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        descriptor_kind: u64,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind,
            function,
        );
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_define_data_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        value_present_local: u32,
        writable_present_local: u32,
        enumerable_present_local: u32,
        configurable_present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_payload_local = self.reserve_temp_local();
        let stored_tag_local = self.reserve_temp_local();
        let flag_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(stored_payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(stored_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(existing_descriptor_kind_local));

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            stored_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            stored_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_write(
            array_local,
            index_local,
            stored_payload_local,
            stored_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as i64,
        ));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));

        for (present_local, payload_local, flag) in [
            (
                writable_present_local,
                writable_payload_local,
                OBJECT_DESCRIPTOR_WRITABLE,
            ),
            (
                enumerable_present_local,
                enumerable_payload_local,
                OBJECT_DESCRIPTOR_ENUMERABLE,
            ),
            (
                configurable_present_local,
                configurable_payload_local,
                OBJECT_DESCRIPTOR_CONFIGURABLE,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(flag_payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            function.instruction(&Instruction::End);
        }

        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        self.release_temp_local(flag_payload_local);
        self.release_temp_local(stored_tag_local);
        self.release_temp_local(stored_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_accessor_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let descriptor_kind_local = self.reserve_temp_local();
        self.emit_array_write(
            array_local,
            index_local,
            getter_payload_local,
            getter_tag_local,
            function,
        )?;
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_payload_local,
            configurable_payload_local,
            descriptor_kind_local,
            function,
        );
        self.emit_store_array_descriptor_for_index(
            array_local,
            index_local,
            descriptor_kind_local,
            function,
        );
        self.release_temp_local(descriptor_kind_local);
        Ok(())
    }

    pub(crate) fn emit_array_named_props_grow_buffer(
        &mut self,
        array_local: u32,
        buffer_local: u32,
        len_local: u32,
        cap_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(new_cap_local));

        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_entry_local));

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(old_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_entry_local, offset, self.scratch_local, function);
        }

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(buffer_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::LocalSet(cap_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(size_local);
        self.release_temp_local(new_cap_local);
        Ok(())
    }

    pub(crate) fn emit_descriptor_flag_payload_from_new_descriptor(
        &mut self,
        requested_payload_local: u32,
        present_local: Option<u32>,
        flag_payload_local: u32,
        function: &mut Function,
    ) {
        if let Some(present_local) = present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
        }
    }

    pub(crate) fn emit_descriptor_flag_payload_from_existing_descriptor(
        &mut self,
        existing_descriptor_kind_local: u32,
        requested_payload_local: u32,
        present_local: Option<u32>,
        flag: u64,
        flag_payload_local: u32,
        function: &mut Function,
    ) {
        if let Some(present_local) = present_local {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(requested_payload_local));
            function.instruction(&Instruction::LocalSet(flag_payload_local));
        }
    }

    pub(crate) fn emit_array_define_named_data_descriptor(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        value_present_local: Option<u32>,
        writable_present_local: Option<u32>,
        enumerable_present_local: Option<u32>,
        configurable_present_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_data_tag_local = self.reserve_temp_local();
        let stored_data_payload_local = self.reserve_temp_local();
        let writable_flag_payload_local = self.reserve_temp_local();
        let enumerable_flag_payload_local = self.reserve_temp_local();
        let configurable_flag_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        if let Some(value_present_local) = value_present_local {
            function.instruction(&Instruction::LocalGet(value_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_TAG_OFFSET,
                stored_data_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
                stored_data_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        }
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            writable_payload_local,
            writable_present_local,
            OBJECT_DESCRIPTOR_WRITABLE,
            writable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            enumerable_payload_local,
            enumerable_present_local,
            OBJECT_DESCRIPTOR_ENUMERABLE,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            configurable_payload_local,
            configurable_present_local,
            OBJECT_DESCRIPTOR_CONFIGURABLE,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            Some(writable_flag_payload_local),
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            stored_data_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            stored_data_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if let Some(value_present_local) = value_present_local {
            function.instruction(&Instruction::LocalGet(value_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::LocalSet(stored_data_tag_local));
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::LocalSet(stored_data_payload_local));
        }
        self.emit_descriptor_flag_payload_from_new_descriptor(
            writable_payload_local,
            writable_present_local,
            writable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            enumerable_payload_local,
            enumerable_present_local,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            configurable_payload_local,
            configurable_present_local,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA,
            Some(writable_flag_payload_local),
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_props_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            stored_data_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            stored_data_payload_local,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_GETTER_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_SETTER_PAYLOAD_OFFSET, 0, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(configurable_flag_payload_local);
        self.release_temp_local(enumerable_flag_payload_local);
        self.release_temp_local(writable_flag_payload_local);
        self.release_temp_local(stored_data_payload_local);
        self.release_temp_local(stored_data_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_named_accessor_descriptor(
        &mut self,
        array_local: u32,
        key_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        getter_present_local: Option<u32>,
        setter_present_local: Option<u32>,
        enumerable_present_local: Option<u32>,
        configurable_present_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let existing_descriptor_kind_local = self.reserve_temp_local();
        let stored_getter_tag_local = self.reserve_temp_local();
        let stored_getter_payload_local = self.reserve_temp_local();
        let stored_setter_tag_local = self.reserve_temp_local();
        let stored_setter_payload_local = self.reserve_temp_local();
        let enumerable_flag_payload_local = self.reserve_temp_local();
        let configurable_flag_payload_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_CAP_OFFSET,
            cap_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_string_payload_equality_i32(self.scratch_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            existing_descriptor_kind_local,
            function,
        );
        if let Some(getter_present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(getter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_TAG_OFFSET,
                stored_getter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
                stored_getter_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
        }
        if let Some(setter_present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(setter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(existing_descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_TAG_OFFSET,
                stored_setter_tag_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
                stored_setter_payload_local,
                function,
            );
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        }
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            enumerable_payload_local,
            enumerable_present_local,
            OBJECT_DESCRIPTOR_ENUMERABLE,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_existing_descriptor(
            existing_descriptor_kind_local,
            configurable_payload_local,
            configurable_present_local,
            OBJECT_DESCRIPTOR_CONFIGURABLE,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            stored_getter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            stored_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        if let Some(getter_present_local) = getter_present_local {
            function.instruction(&Instruction::LocalGet(getter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(getter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_getter_tag_local));
            function.instruction(&Instruction::LocalGet(getter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_getter_payload_local));
        }
        if let Some(setter_present_local) = setter_present_local {
            function.instruction(&Instruction::LocalGet(setter_present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::LocalGet(setter_tag_local));
            function.instruction(&Instruction::LocalSet(stored_setter_tag_local));
            function.instruction(&Instruction::LocalGet(setter_payload_local));
            function.instruction(&Instruction::LocalSet(stored_setter_payload_local));
        }
        self.emit_descriptor_flag_payload_from_new_descriptor(
            enumerable_payload_local,
            enumerable_present_local,
            enumerable_flag_payload_local,
            function,
        );
        self.emit_descriptor_flag_payload_from_new_descriptor(
            configurable_payload_local,
            configurable_present_local,
            configurable_flag_payload_local,
            function,
        );
        self.emit_array_descriptor_flags_to_local(
            ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR,
            None,
            enumerable_flag_payload_local,
            configurable_flag_payload_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_props_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_OBJECT_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            stored_getter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            stored_getter_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            stored_setter_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            stored_setter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(configurable_flag_payload_local);
        self.release_temp_local(enumerable_flag_payload_local);
        self.release_temp_local(stored_setter_payload_local);
        self.release_temp_local(stored_setter_tag_local);
        self.release_temp_local(stored_getter_payload_local);
        self.release_temp_local(stored_getter_tag_local);
        self.release_temp_local(existing_descriptor_kind_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_named_data_property(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_array_define_named_data_descriptor(
            array_local,
            key_local,
            payload_local,
            tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            None,
            None,
            None,
            None,
            function,
        )?;
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_define_builtin_named_data_property(
        &mut self,
        array_local: u32,
        descriptor_offset: u64,
        tag_offset: u64,
        payload_offset: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            array_local,
            descriptor_offset,
            (ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_DATA) as u64,
            function,
        );
        self.store_i64_local_at_offset(array_local, tag_offset, tag_local, function);
        self.store_i64_local_at_offset(array_local, payload_offset, payload_local, function);
    }

    pub(crate) fn emit_array_read_builtin_named_data_property(
        &mut self,
        array_local: u32,
        descriptor_offset: u64,
        tag_offset: u64,
        payload_offset: u64,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            array_local,
            descriptor_offset,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(array_local, payload_offset, payload_local, function);
        self.load_i64_to_local_from_offset(array_local, tag_offset, tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(descriptor_kind_local);
    }

    pub(crate) fn emit_array_named_prop_read(
        &mut self,
        array_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        found_output_local: Option<u32>,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let stored_key_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            stored_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(stored_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        if let Some(values_meta) = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
            .cloned()
        {
            self.emit_function_value_payload(&values_meta, function)
                .expect("Array.prototype.values builtin should be emitted");
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("push")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let prototype_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype_found_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.emit_object_own_data_field_read(
            prototype_local,
            prototype_tag_local,
            key_local,
            prototype_found_local,
            payload_local,
            tag_local,
            function,
        );
        if let Some(found_output_local) = found_output_local {
            function.instruction(&Instruction::LocalGet(prototype_found_local));
            function.instruction(&Instruction::LocalSet(found_output_local));
        }
        self.release_temp_local(prototype_found_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_local);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        self.release_temp_local(found_local);
        self.release_temp_local(stored_key_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_named_prop_descriptor_read(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(entry_key_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        for (flag, payload_local) in [
            (OBJECT_DESCRIPTOR_WRITABLE, writable_payload_local),
            (OBJECT_DESCRIPTOR_ENUMERABLE, enumerable_payload_local),
            (OBJECT_DESCRIPTOR_CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(payload_local));
        }
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        function.instruction(&Instruction::End);
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_named_props_count(
        &mut self,
        array_local: u32,
        count_local: u32,
        enumerable_only: bool,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        if enumerable_only {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
        }
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        if enumerable_only {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_named_props_write_keys(
        &mut self,
        array_local: u32,
        result_payload_local: u32,
        write_index_local: u32,
        enumerable_only: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        if enumerable_only {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        if enumerable_only {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_array_delete_property_key(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_delete(array_local, index_local, result_local, function);
        function.instruction(&Instruction::Else);
        self.emit_array_named_prop_delete(array_local, key_local, result_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_array_named_prop_delete(
        &mut self,
        array_local: u32,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let shift_index_local = self.reserve_temp_local();
        let current_entry_local = self.reserve_temp_local();
        let next_entry_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_string_payload_equality_i32(key_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_entry_local));
        function.instruction(&Instruction::LocalGet(current_entry_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_entry_local));

        for offset in [
            HEAP_OBJECT_KEY_OFFSET,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
        ] {
            self.load_i64_from_offset(next_entry_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                current_entry_local,
                offset,
                self.scratch_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_entry_local);
        self.release_temp_local(current_entry_local);
        self.release_temp_local(shift_index_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_delete(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            HEAP_ARRAY_HOLE_TAG as u64,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET, 0, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_has_index_i32(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    /// Reads the own-property descriptor kind for `index_local` on an
    /// array-shaped heap object (array or arguments object), consulting the
    /// dense buffer when the index is within `cap` and falling back to the
    /// sparse present-index list otherwise. Writes `0` into `result_local`
    /// when the index has no own property (out of `len` bounds, or simply
    /// absent from both the dense buffer and the present-index list).
    ///
    /// This mirrors the bounds-checked lookup used by
    /// [`Self::emit_array_has_index_i32`] and
    /// [`Self::emit_array_advance_to_next_present_index`]; callers that need
    /// to inspect flags (e.g. `OBJECT_DESCRIPTOR_ENUMERABLE`) for an index
    /// already known to be present should use this instead of indexing the
    /// dense buffer directly, which is unsafe for indices `>= cap`.
    pub(crate) fn emit_array_descriptor_kind_for_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_advance_to_next_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let best_index_local = self.reserve_temp_local();
        let list_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(0));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_entry_local));
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(list_entry_local);
        self.release_temp_local(best_index_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_retreat_to_previous_present_index(
        &mut self,
        array_local: u32,
        index_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let list_ptr_local = self.reserve_temp_local();
        let list_len_local = self.reserve_temp_local();
        let list_index_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();
        let best_index_local = self.reserve_temp_local();
        let list_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            list_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            list_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::LocalGet(list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(list_ptr_local));
        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_entry_local));
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            candidate_index_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            list_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalSet(best_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(list_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(list_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(best_index_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(list_entry_local);
        self.release_temp_local(best_index_local);
        self.release_temp_local(candidate_index_local);
        self.release_temp_local(list_index_local);
        self.release_temp_local(list_len_local);
        self.release_temp_local(list_ptr_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_string_index_0_to_4_or_minus_one(
        &mut self,
        key_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));

        self.emit_unpack_string_payload(key_local, string_offset_local, string_len_local, function);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
    }

    pub(crate) fn emit_index_to_flat_map_key_local(
        &mut self,
        index_local: u32,
        number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("4")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_flat_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.flat receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.flat receiver tag",
            )
        })?;
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let depth_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let stack_values_local = self.reserve_temp_local();
        let stack_depths_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let stack_len_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let current_depth_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let next_depth_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let flatten_payload_local = self.reserve_temp_local();
        let flatten_tag_local = self.reserve_temp_local();
        let insert_index_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert object to number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(depth_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            this_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(zero_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(zero_local, stack_values_local, function)?;
        self.emit_alloc_array_payload_with_length(zero_local, stack_depths_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(flatten_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flat receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_flat_append_depth_one_value(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            depth_local,
            key_local,
            has_property_local,
            object_length_payload_local,
            object_length_tag_local,
            index_number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_write(
            stack_values_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            stack_len_local,
            depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        self.emit_array_read(
            stack_values_local,
            stack_len_local,
            current_payload_local,
            current_tag_local,
            function,
        );
        self.emit_array_read(
            stack_depths_local,
            stack_len_local,
            current_depth_local,
            arg_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(current_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            current_payload_local,
            current_tag_local,
            current_payload_local,
            current_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(next_depth_local));
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            current_payload_local,
            current_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            current_payload_local,
            current_tag_local,
            current_payload_local,
            current_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_stack_insert(
            stack_values_local,
            stack_depths_local,
            insert_index_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            next_depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(src_index_local));
        self.emit_array_read(
            current_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            stack_values_local,
            stack_len_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            stack_len_local,
            next_depth_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            current_payload_local,
            current_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(insert_index_local);
        self.release_temp_local(flatten_tag_local);
        self.release_temp_local(flatten_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(next_depth_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(current_depth_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(stack_len_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(stack_depths_local);
        self.release_temp_local(stack_values_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(depth_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_flat_append_depth_one_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        out_index_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        depth_local: u32,
        key_local: u32,
        has_property_local: u32,
        object_length_payload_local: u32,
        object_length_tag_local: u32,
        index_number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flatten_payload_local = self.reserve_temp_local();
        let flatten_tag_local = self.reserve_temp_local();
        let child_len_local = self.reserve_temp_local();
        let child_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(flatten_tag_local));

        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_LEN_OFFSET,
            child_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(child_len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(child_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child_index_local));
        function.instruction(&Instruction::LocalGet(child_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            element_payload_local,
            child_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            child_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            element_payload_local,
            element_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            element_payload_local,
            child_index_local,
            child_payload_local,
            child_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            child_payload_local,
            child_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(child_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(child_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_flat_target_write(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(child_index_local);
        self.release_temp_local(child_len_local);
        self.release_temp_local(flatten_tag_local);
        self.release_temp_local(flatten_payload_local);
        Ok(())
    }

    pub(crate) fn emit_flat_target_write(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        out_index_local: u32,
        payload_local: u32,
        tag_local: u32,
        index_number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            payload_local,
            tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_stack_insert(
        &mut self,
        stack_values_local: u32,
        stack_depths_local: u32,
        insert_index_local: u32,
        stack_len_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        depth_payload_local: u32,
        depth_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let shift_index_local = self.reserve_temp_local();
        let shifted_payload_local = self.reserve_temp_local();
        let shifted_tag_local = self.reserve_temp_local();
        let shifted_depth_payload_local = self.reserve_temp_local();
        let shifted_depth_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(
            stack_values_local,
            self.scratch_local,
            shifted_payload_local,
            shifted_tag_local,
            function,
        );
        self.emit_array_read(
            stack_depths_local,
            self.scratch_local,
            shifted_depth_payload_local,
            shifted_depth_tag_local,
            function,
        );
        self.emit_array_write(
            stack_values_local,
            shift_index_local,
            shifted_payload_local,
            shifted_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            shift_index_local,
            shifted_depth_payload_local,
            shifted_depth_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(shift_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_array_write(
            stack_values_local,
            insert_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_array_write(
            stack_depths_local,
            insert_index_local,
            depth_payload_local,
            depth_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(stack_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(stack_len_local));

        self.release_temp_local(shifted_depth_tag_local);
        self.release_temp_local(shifted_depth_payload_local);
        self.release_temp_local(shifted_tag_local);
        self.release_temp_local(shifted_payload_local);
        self.release_temp_local(shift_index_local);
        Ok(())
    }

    pub(crate) fn emit_concat_create_target_property(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        index_number_payload_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let array_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(array_index_local));
        self.emit_array_write(
            target_payload_local,
            array_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.release_temp_local(array_index_local);
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            "Array.prototype.concat cannot define non-configurable target property",
            "Array.prototype.concat cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_concat_length_of_array_like(
        &mut self,
        item_payload_local: u32,
        item_tag_local: u32,
        length_local: u32,
        object_length_payload_local: u32,
        object_length_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(
            item_payload_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            item_payload_local,
            HEAP_LEN_OFFSET,
            length_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(length_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_concat_typed_array_has_index_i32(
        &mut self,
        item_payload_local: u32,
        item_tag_local: u32,
        index_local: u32,
        result_local: u32,
        typed_array_like_local: u32,
        function: &mut Function,
    ) {
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let slot_payload_local = self.reserve_temp_local();
        let slot_tag_local = self.reserve_temp_local();
        let byte_length_local = self.reserve_temp_local();
        let bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));

        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            item_payload_local,
            item_tag_local,
            key_local,
            present_local,
            slot_payload_local,
            slot_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(slot_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));

        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            item_payload_local,
            item_tag_local,
            key_local,
            present_local,
            slot_payload_local,
            slot_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(byte_length_local));

        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            item_payload_local,
            item_tag_local,
            key_local,
            present_local,
            slot_payload_local,
            slot_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(slot_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(bytes_per_element_local));

        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(bytes_per_element_local);
        self.release_temp_local(byte_length_local);
        self.release_temp_local(slot_tag_local);
        self.release_temp_local(slot_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(key_local);
    }

    pub(crate) fn compile_array_prototype_concat_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.concat receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.concat receiver tag",
            )
        })?;
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let item_index_local = self.reserve_temp_local();
        let arg_index_local = self.reserve_temp_local();
        let item_payload_local = self.reserve_temp_local();
        let item_tag_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let spreadable_payload_local = self.reserve_temp_local();
        let spreadable_tag_local = self.reserve_temp_local();
        let spreadable_flag_local = self.reserve_temp_local();
        let typed_array_like_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            this_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.concat constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(zero_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(zero_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(item_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(item_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(item_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(arg_index_local));
        let saved_out_index_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::LocalSet(saved_out_index_local));
        self.emit_array_read(
            self.argv_param_local(),
            arg_index_local,
            item_payload_local,
            item_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(saved_out_index_local));
        function.instruction(&Instruction::LocalSet(out_index_local));
        self.release_temp_local(saved_out_index_local);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(spreadable_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(spreadable_tag_local));

        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.isConcatSpreadable"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_is_concat_spreadable_read(
            item_payload_local,
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_is_concat_spreadable_read(
            item_payload_local,
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            spreadable_payload_local,
            spreadable_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(spreadable_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_truthy_tagged_i32(spreadable_tag_local, spreadable_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            item_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(spreadable_flag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(spreadable_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_length_of_array_like(
            item_payload_local,
            item_tag_local,
            src_len_local,
            object_length_payload_local,
            object_length_tag_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_like_local));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            item_payload_local,
            item_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            item_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_concat_typed_array_has_index_i32(
            item_payload_local,
            item_tag_local,
            src_index_local,
            has_property_local,
            typed_array_like_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            item_payload_local,
            item_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            item_payload_local,
            src_index_local,
            item_payload_local,
            item_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(item_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            item_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_array_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            item_payload_local,
            item_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_concat_create_target_property(
            target_payload_local,
            target_tag_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(spreadable_flag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_create_target_property(
            target_payload_local,
            target_tag_local,
            out_index_local,
            item_payload_local,
            item_tag_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(item_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(item_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            out_index_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_write(
            target_payload_local,
            target_tag_local,
            key_local,
            index_number_payload_local,
            number_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(typed_array_like_local);
        self.release_temp_local(spreadable_flag_local);
        self.release_temp_local(spreadable_tag_local);
        self.release_temp_local(spreadable_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(item_tag_local);
        self.release_temp_local(item_payload_local);
        self.release_temp_local(arg_index_local);
        self.release_temp_local(item_index_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_flat_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.flatMap receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.flatMap receiver tag",
            )
        })?;
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_flatten_payload_local = self.reserve_temp_local();
        let mapped_flatten_tag_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_data_ptr_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let typed_address_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, mapper_payload_local, mapper_tag_local, function);
        function.instruction(&Instruction::LocalGet(mapper_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            this_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flatMap constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flatMap constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.flatMap receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            object_length_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            constructor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(zero_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            this_payload_local,
            src_index_local,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_has_property_i32_ordinary(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            mapper_payload_local,
            mapper_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(mapped_flatten_tag_local));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            mapped_flatten_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mapped_flatten_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            mapped_flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            mapped_flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(mapped_flatten_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_LEN_OFFSET,
            mapped_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(mapped_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(mapped_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            mapped_payload_local,
            mapped_index_local,
            child_payload_local,
            child_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            mapped_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            mapped_payload_local,
            mapped_tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_len_local);
        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_address_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_data_ptr_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_flatten_tag_local);
        self.release_temp_local(mapped_flatten_payload_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_map_length_read_side_effect(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        key_local: u32,
        length_payload_local: u32,
        length_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(length_tag_local, length_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(length_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_string_hex_length_to_i64_local(
        &mut self,
        string_payload_local: u32,
        result_local: u32,
        success_local: u32,
        function: &mut Function,
    ) {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'x' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'X' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_hex_value_or_minus_one(byte_local, digit_local, function);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(success_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
    }

    pub(crate) fn compile_array_prototype_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.map receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.map receiver tag",
            )
        })?;
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_flatten_payload_local = self.reserve_temp_local();
        let mapped_flatten_tag_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let length_hex_success_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_source_payload_local = self.reserve_temp_local();
        let species_source_tag_local = self.reserve_temp_local();
        let species_source_is_array_local = self.reserve_temp_local();
        let species_proxy_kind_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_data_ptr_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let typed_address_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.map called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.map called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(species_source_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(species_source_tag_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.map mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, mapper_payload_local, mapper_tag_local, function);
        function.instruction(&Instruction::LocalGet(mapper_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.map mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            species_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            species_source_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            species_source_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(species_source_is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            species_source_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.map constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.map constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_hex_success_local));
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_hex_length_to_i64_local(
            object_length_payload_local,
            current_len_local,
            length_hex_success_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_hex_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.map receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64Const(4_294_967_295_i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pre_evaluated_arg_vector(
            &[(index_number_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            this_payload_local,
            src_index_local,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_has_property_i32_ordinary(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            mapper_payload_local,
            mapper_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_flatten_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(mapped_flatten_tag_local));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            mapped_flatten_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mapped_flatten_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            mapped_flatten_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            mapped_flatten_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            mapped_payload_local,
            HEAP_LEN_OFFSET,
            mapped_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(mapped_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(mapped_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(mapped_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            mapped_payload_local,
            mapped_index_local,
            child_payload_local,
            child_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            mapped_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_has_property_i32(
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_object_read(
            mapped_payload_local,
            mapped_tag_local,
            mapped_payload_local,
            mapped_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            child_payload_local,
            child_tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            src_index_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            mapped_payload_local,
            mapped_tag_local,
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_len_local);
        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_address_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_data_ptr_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(species_proxy_kind_local);
        self.release_temp_local(species_source_is_array_local);
        self.release_temp_local(species_source_tag_local);
        self.release_temp_local(species_source_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(length_hex_success_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_flatten_tag_local);
        self.release_temp_local(mapped_flatten_payload_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_every_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.every receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.every receiver tag",
            )
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_flatten_payload_local = self.reserve_temp_local();
        let mapped_flatten_tag_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let length_hex_success_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_source_payload_local = self.reserve_temp_local();
        let species_source_tag_local = self.reserve_temp_local();
        let species_source_is_array_local = self.reserve_temp_local();
        let species_proxy_kind_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_data_ptr_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let typed_address_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.every called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.every called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(species_source_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(species_source_tag_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.every callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.every callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            species_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            species_source_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            species_source_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(species_source_is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            species_source_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.every constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.every constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_hex_success_local));
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_hex_length_to_i64_local(
            object_length_payload_local,
            current_len_local,
            length_hex_success_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_hex_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.every receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        self.store_i64_local_at_offset(result_payload_local, HEAP_LEN_OFFSET, zero_local, function);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[(index_number_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            this_payload_local,
            src_index_local,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_has_property_i32_ordinary(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        self.compile_truthy_tagged_i32(mapped_tag_local, mapped_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_len_local);
        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_address_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_data_ptr_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(species_proxy_kind_local);
        self.release_temp_local(species_source_is_array_local);
        self.release_temp_local(species_source_tag_local);
        self.release_temp_local(species_source_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(length_hex_success_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_flatten_tag_local);
        self.release_temp_local(mapped_flatten_payload_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_some_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.some receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.some receiver tag",
            )
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_flatten_payload_local = self.reserve_temp_local();
        let mapped_flatten_tag_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let length_hex_success_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_source_payload_local = self.reserve_temp_local();
        let species_source_tag_local = self.reserve_temp_local();
        let species_source_is_array_local = self.reserve_temp_local();
        let species_proxy_kind_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_data_ptr_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let typed_address_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.some called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.some called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(species_source_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(species_source_tag_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.some callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.some callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            species_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            species_source_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            species_source_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(species_source_is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            species_source_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.some constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.some constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_hex_success_local));
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_hex_length_to_i64_local(
            object_length_payload_local,
            current_len_local,
            length_hex_success_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_hex_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.some receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        self.store_i64_local_at_offset(result_payload_local, HEAP_LEN_OFFSET, zero_local, function);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[(index_number_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            this_payload_local,
            src_index_local,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_has_property_i32_ordinary(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        self.compile_truthy_tagged_i32(mapped_tag_local, mapped_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_len_local);
        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_address_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_data_ptr_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(species_proxy_kind_local);
        self.release_temp_local(species_source_is_array_local);
        self.release_temp_local(species_source_tag_local);
        self.release_temp_local(species_source_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(length_hex_success_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_flatten_tag_local);
        self.release_temp_local(mapped_flatten_payload_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_filter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_constructor_table_index = self
            .functions
            .get(&StandardBuiltinId::ArrayConstructor.function_id())
            .map(|meta| meta.table_index as i64)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array`",
                )
            })?;
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.filter receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.filter receiver tag",
            )
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let out_index_local = self.reserve_temp_local();
        let current_len_local = self.reserve_temp_local();
        let src_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_flatten_payload_local = self.reserve_temp_local();
        let mapped_flatten_tag_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let object_length_payload_local = self.reserve_temp_local();
        let object_length_tag_local = self.reserve_temp_local();
        let length_hex_success_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let constructor_table_index_local = self.reserve_temp_local();
        let skip_species_local = self.reserve_temp_local();
        let species_source_payload_local = self.reserve_temp_local();
        let species_source_tag_local = self.reserve_temp_local();
        let species_source_is_array_local = self.reserve_temp_local();
        let species_proxy_kind_local = self.reserve_temp_local();
        let species_payload_local = self.reserve_temp_local();
        let species_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_data_ptr_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let typed_address_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();
        let prototype_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.filter called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.filter called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(species_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(skip_species_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(species_source_payload_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(species_source_tag_local));

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.filter callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_array_map_length_read_side_effect(
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.filter callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            this_payload_local,
            this_tag_local,
            this_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(species_source_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_source_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            species_proxy_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(species_proxy_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            species_source_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            species_source_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            species_source_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(species_source_is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_constructor_read(
            species_source_payload_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.filter constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            constructor_table_index_local,
            function,
        );
        self.emit_mark_skip_species_for_cross_realm_array_constructor(
            constructor_payload_local,
            constructor_table_index_local,
            skip_species_local,
            array_constructor_table_index,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(skip_species_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            species_payload_local,
            species_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.filter constructor is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_LEN_OFFSET,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            this_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_hex_success_local));
        function.instruction(&Instruction::LocalGet(object_length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_hex_length_to_i64_local(
            object_length_payload_local,
            current_len_local,
            length_hex_success_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_hex_success_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(
            object_length_tag_local,
            object_length_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_length_payload_local));
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(object_length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(current_len_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.filter receiver is not array",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64Const(4_294_967_295_i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Invalid array length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(zero_local, result_payload_local, function)?;
        self.store_i64_local_at_offset(result_payload_local, HEAP_LEN_OFFSET, zero_local, function);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::LocalGet(species_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[(index_number_payload_local, number_tag_local)],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_construct_with_argv(
            species_payload_local,
            species_tag_local,
            species_payload_local,
            species_tag_local,
            argc_local,
            argv_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            target_payload_local,
            target_tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_LEN_OFFSET,
            prototype_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            this_payload_local,
            src_index_local,
            current_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(current_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            this_payload_local,
            this_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            src_index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_has_property_i32_ordinary(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            this_payload_local,
            src_index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("1")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("2")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("3")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_object_has_property_i32(
            this_payload_local,
            this_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            this_payload_local,
            src_index_local,
            this_payload_local,
            this_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            this_payload_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            this_payload_local,
            this_tag_local,
            src_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local,
            this_tag_local,
            this_payload_local,
            this_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local,
            mapped_tag_local,
            function,
        )?;

        self.compile_truthy_tagged_i32(mapped_tag_local, mapped_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            target_payload_local,
            out_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_create_data_property_or_throw(
            target_payload_local,
            target_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            "Array.prototype.filter cannot define non-configurable target property",
            "Array.prototype.filter cannot add property to non-extensible target",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(out_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(src_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(prototype_len_local);
        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_address_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_data_ptr_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(species_tag_local);
        self.release_temp_local(species_payload_local);
        self.release_temp_local(species_proxy_kind_local);
        self.release_temp_local(species_source_is_array_local);
        self.release_temp_local(species_source_tag_local);
        self.release_temp_local(species_source_payload_local);
        self.release_temp_local(skip_species_local);
        self.release_temp_local(constructor_table_index_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(length_hex_success_local);
        self.release_temp_local(object_length_tag_local);
        self.release_temp_local(object_length_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_flatten_tag_local);
        self.release_temp_local(mapped_flatten_payload_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(src_index_local);
        self.release_temp_local(current_len_local);
        self.release_temp_local(out_index_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_concat_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeConcat.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.concat`",
                )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_function_value_payload(&meta, function)?;
        function.instruction(&Instruction::LocalSet(callee_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callee_tag_local));
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_function_handle_call_with_argv(
            callee_payload_local,
            callee_tag_local,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_flat_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFlat.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flat`",
                )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let mut arg_locals = Vec::new();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        for arg in args {
            let arg_payload_local = self.reserve_temp_local();
            let arg_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
            arg_locals.push((arg_payload_local, arg_tag_local));
        }
        self.emit_direct_js_call(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &arg_locals,
            payload_local,
            tag_local,
            function,
        )?;

        for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(arg_tag_local);
            self.release_temp_local(arg_payload_local);
        }
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_flat_map_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFlatMap.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flatMap`",
                )
        })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let mut arg_locals = Vec::new();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        for arg in args {
            let arg_payload_local = self.reserve_temp_local();
            let arg_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
            arg_locals.push((arg_payload_local, arg_tag_local));
        }
        self.emit_direct_js_call(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &arg_locals,
            payload_local,
            tag_local,
            function,
        )?;

        for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(arg_tag_local);
            self.release_temp_local(arg_payload_local);
        }
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_map_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeMap.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.map`",
                )
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let mut arg_locals = Vec::new();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        for arg in args {
            let arg_payload_local = self.reserve_temp_local();
            let arg_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(
                arg_payload_local,
                arg_tag_local,
                function,
            )?;
            arg_locals.push((arg_payload_local, arg_tag_local));
        }
        self.emit_direct_js_call(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &arg_locals,
            payload_local,
            tag_local,
            function,
        )?;

        for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(arg_tag_local);
            self.release_temp_local(arg_payload_local);
        }
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_filter_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFilter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.filter`",
                )
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_direct_js_call_with_argv(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_every_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeEvery.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.every`",
                )
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let mut arg_locals = Vec::new();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        for arg in args {
            let arg_payload_local = self.reserve_temp_local();
            let arg_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(
                arg_payload_local,
                arg_tag_local,
                function,
            )?;
            arg_locals.push((arg_payload_local, arg_tag_local));
        }
        self.emit_direct_js_call(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &arg_locals,
            payload_local,
            tag_local,
            function,
        )?;

        for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(arg_tag_local);
            self.release_temp_local(arg_payload_local);
        }
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_some_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeSome.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.some`",
                )
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let mut arg_locals = Vec::new();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        for arg in args {
            let arg_payload_local = self.reserve_temp_local();
            let arg_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(
                arg_payload_local,
                arg_tag_local,
                function,
            )?;
            arg_locals.push((arg_payload_local, arg_tag_local));
        }
        self.emit_direct_js_call(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            &arg_locals,
            payload_local,
            tag_local,
            function,
        )?;

        for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
            self.release_temp_local(arg_tag_local);
            self.release_temp_local(arg_payload_local);
        }
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_find_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::ArrayPrototypeFind,
            "Array.prototype.find",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_array_find_index_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::ArrayPrototypeFindIndex,
            "Array.prototype.findIndex",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_array_find_last_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::ArrayPrototypeFindLast,
            "Array.prototype.findLast",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_array_find_last_index_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_direct_builtin_method_call(
            StandardBuiltinId::ArrayPrototypeFindLastIndex,
            "Array.prototype.findLastIndex",
            receiver,
            args,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_array_direct_builtin_method_call(
        &mut self,
        builtin: StandardBuiltinId,
        label: &str,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{label}`"
                ))
            })?;
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
        self.emit_direct_js_call_with_argv(
            &meta,
            Some((receiver_payload_local, Some(receiver_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_push_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let length_writable_local = self.reserve_temp_local();
        let index_set_state_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Array.prototype.push receiver is not array",
            payload_local,
            tag_local,
            1,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            index_local,
            function,
        );
        self.emit_array_length_writable_i64(
            receiver_payload_local,
            length_writable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(length_writable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Array.prototype.push length is not writable",
            payload_local,
            tag_local,
            1,
            function,
        )?;
        function.instruction(&Instruction::End);
        for arg in args {
            if let ExprIr::String(value) = &arg.expr {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            } else {
                self.compile_expr_to_locals(
                    arg,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
            }
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("4294967295")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(index_number_payload_local));
            self.emit_number_to_string_payload(index_number_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::End);
            self.emit_array_define_named_data_property(
                receiver_payload_local,
                key_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_array_inherited_index_set_state(
                receiver_payload_local,
                index_local,
                self.result_local,
                self.result_tag_local,
                index_set_state_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(index_set_state_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "Array.prototype.push index write failed",
                payload_local,
                tag_local,
                1,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(index_set_state_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_write(
                receiver_payload_local,
                index_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            RANGE_ERROR_NAME,
            "Invalid array length",
            payload_local,
            tag_local,
            1,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_length_writable_i64(
            receiver_payload_local,
            length_writable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(length_writable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Array.prototype.push length is not writable",
            payload_local,
            tag_local,
            1,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(key_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_set_state_local);
        self.release_temp_local(length_writable_local);
        self.release_temp_local(index_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_join_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let separator_payload_local = self.reserve_temp_local();
        let separator_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();
        let tmp_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.join receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if let Some(separator) = args.first() {
            self.compile_expr_to_locals(
                separator,
                separator_payload_local,
                separator_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(separator_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(",")));
            function.instruction(&Instruction::LocalSet(separator_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_value_to_string_payload(
                separator_payload_local,
                separator_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(separator_payload_local));
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::I64Const(self.strings.payload(",")));
            function.instruction(&Instruction::LocalSet(separator_payload_local));
        }

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_concat_string_payloads_local(joined_local, separator_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(element_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        self.release_temp_local(tmp_local);
        self.release_temp_local(joined_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(separator_tag_local);
        self.release_temp_local(separator_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_reverse_method_call(
        &mut self,
        receiver: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let left_local = self.reserve_temp_local();
        let right_local = self.reserve_temp_local();
        let left_payload_local = self.reserve_temp_local();
        let left_tag_local = self.reserve_temp_local();
        let right_payload_local = self.reserve_temp_local();
        let right_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.reverse receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(left_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(right_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(left_local));
        function.instruction(&Instruction::LocalGet(right_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            receiver_payload_local,
            left_local,
            left_payload_local,
            left_tag_local,
            function,
        );
        self.emit_array_read(
            receiver_payload_local,
            right_local,
            right_payload_local,
            right_tag_local,
            function,
        );
        self.emit_array_write(
            receiver_payload_local,
            left_local,
            right_payload_local,
            right_tag_local,
            function,
        )?;
        self.emit_array_write(
            receiver_payload_local,
            right_local,
            left_payload_local,
            left_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(left_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(left_local));
        function.instruction(&Instruction::LocalGet(right_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(right_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(right_tag_local);
        self.release_temp_local(right_payload_local);
        self.release_temp_local(left_tag_local);
        self.release_temp_local(left_payload_local);
        self.release_temp_local(right_local);
        self.release_temp_local(left_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_splice_insert_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if args.len() < 2 {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Array.prototype.splice requires start and deleteCount",
            ));
        }
        if args.len() == 2
            && matches!(&args[1].expr, ExprIr::Number(bits) if f64::from_bits(*bits) == 1.0)
        {
            return self.emit_array_splice_delete_one_method_call(
                receiver,
                &args[0],
                payload_local,
                tag_local,
                function,
            );
        }
        let start = match &args[0].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Array.prototype.splice start must be static number",
                ))
            }
        };
        let delete_count = match &args[1].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Array.prototype.splice deleteCount must be static zero",
                ))
            }
        };
        if delete_count != 0.0 {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Array.prototype.splice only supports zero deleteCount",
            ));
        }

        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let read_index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let deleted_len_local = self.reserve_temp_local();
        let deleted_payload_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            if start.is_finite() && start > 0.0 {
                start.trunc() as i64
            } else {
                0
            },
        ));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        let insert_count = args.len().saturating_sub(2) as i64;
        if insert_count > 0 {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::LocalSet(read_index_local));
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(read_index_local));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(read_index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(read_index_local));
            self.emit_array_read(
                receiver_payload_local,
                read_index_local,
                element_payload_local,
                element_tag_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(read_index_local));
            function.instruction(&Instruction::I64Const(insert_count));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(write_index_local));
            self.emit_array_write(
                receiver_payload_local,
                write_index_local,
                element_payload_local,
                element_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        for (offset, arg) in args.iter().skip(2).enumerate() {
            self.compile_expr_to_locals(arg, element_payload_local, element_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(write_index_local));
            self.emit_array_write(
                receiver_payload_local,
                write_index_local,
                element_payload_local,
                element_tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(deleted_len_local));
        self.emit_alloc_array_payload_with_length(
            deleted_len_local,
            deleted_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(deleted_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(deleted_payload_local);
        self.release_temp_local(deleted_len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(read_index_local);
        self.release_temp_local(start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_splice_delete_one_method_call(
        &mut self,
        receiver: &TypedExpr,
        start: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let deleted_len_local = self.reserve_temp_local();
        let deleted_payload_local = self.reserve_temp_local();
        let start_payload_local = self.reserve_temp_local();
        let start_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.compile_expr_to_locals(start, start_payload_local, start_tag_local, function)?;
        self.emit_value_to_number_payload(start_tag_local, start_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(start_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(start_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(deleted_len_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(deleted_len_local));
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(
            deleted_len_local,
            deleted_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(deleted_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            receiver_payload_local,
            start_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(next_index_local));
        self.emit_array_write(
            deleted_payload_local,
            next_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        self.emit_array_read(
            receiver_payload_local,
            next_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            receiver_payload_local,
            start_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(deleted_len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(deleted_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(start_tag_local);
        self.release_temp_local(start_payload_local);
        self.release_temp_local(deleted_payload_local);
        self.release_temp_local(deleted_len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_splice_from_array_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if args.len() != 3 {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Array.prototype.splice array spread requires one source array",
            ));
        }
        let start = match &args[0].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Array.prototype.splice start must be static number",
                ))
            }
        };
        let delete_count = match &args[1].expr {
            ExprIr::Number(bits) => f64::from_bits(*bits),
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Array.prototype.splice deleteCount must be static zero",
                ))
            }
        };
        if delete_count != 0.0 {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Array.prototype.splice only supports zero deleteCount",
            ));
        }

        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let insert_payload_local = self.reserve_temp_local();
        let insert_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let insert_len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let read_index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let insert_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let deleted_len_local = self.reserve_temp_local();
        let deleted_payload_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.compile_expr_to_locals(&args[2], insert_payload_local, insert_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(insert_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.splice receiver is not array",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            insert_payload_local,
            HEAP_LEN_OFFSET,
            insert_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            if start.is_finite() && start > 0.0 {
                start.trunc() as i64
            } else {
                0
            },
        ));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(read_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(read_index_local));
        self.emit_array_read(
            receiver_payload_local,
            read_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(read_index_local));
        function.instruction(&Instruction::LocalGet(insert_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.emit_array_write(
            receiver_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::LocalGet(insert_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            insert_payload_local,
            insert_index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.emit_array_write(
            receiver_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(insert_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(insert_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(deleted_len_local));
        self.emit_alloc_array_payload_with_length(
            deleted_len_local,
            deleted_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(deleted_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(deleted_payload_local);
        self.release_temp_local(deleted_len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(insert_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(read_index_local);
        self.release_temp_local(start_local);
        self.release_temp_local(insert_len_local);
        self.release_temp_local(len_local);
        self.release_temp_local(insert_tag_local);
        self.release_temp_local(insert_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_includes_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.includes receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.includes receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_includes_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.indexOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.indexOf receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_last_index_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.lastIndexOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.lastIndexOf receiver tag",
            )
        })?;
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, search_payload_local, search_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(from_payload_local));
        // Internal sentinel: omitted fromIndex differs from explicit undefined.
        function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
        function.instruction(&Instruction::LocalSet(from_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, from_payload_local, from_tag_local, function);
        function.instruction(&Instruction::End);

        self.emit_array_last_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_at_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.at receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.at receiver tag",
            )
        })?;
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, index_payload_local, index_tag_local, function);
        self.emit_array_at_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_payload_local,
            index_tag_local,
            self.result_local,
            self.result_tag_local,
            false,
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_at_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        if let Some(index) = args.first() {
            self.compile_expr_to_locals(index, index_payload_local, index_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(index_tag_local));
        }

        self.emit_array_at_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_payload_local,
            index_tag_local,
            payload_local,
            tag_local,
            typed_expr_has_typed_array_shape(receiver),
            function,
        )?;

        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_at_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        index_payload_local: u32,
        index_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        validate_typed_array: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let typed_slot_present_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let relative_index_local = self.reserve_temp_local();
        let negative_bound_local = self.reserve_temp_local();
        let k_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_slot_present_local,
            length_payload_local,
            length_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_slot_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_slot_present_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        );
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        if validate_typed_array {
            self.emit_validate_typed_array_current_byte_length(
                receiver_payload_local,
                receiver_tag_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                function,
            )?;
        } else {
            self.emit_typed_array_current_byte_length(
                receiver_payload_local,
                receiver_tag_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            length_tag_local,
            length_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.at called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalSet(k_local));
        self.emit_value_to_number_payload(index_tag_local, index_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(relative_index_local));

        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(negative_bound_local));
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::LocalGet(negative_bound_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(k_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(relative_index_local));
        function.instruction(&Instruction::LocalSet(k_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(k_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            k_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(k_local);
        self.release_temp_local(negative_bound_local);
        self.release_temp_local(relative_index_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_slot_present_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_array_includes_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        if let Some(search) = args.first() {
            self.compile_expr_to_locals(search, search_payload_local, search_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(search_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(search_tag_local));
        }
        if let Some(from_index) = args.get(1) {
            self.compile_expr_to_locals(from_index, from_payload_local, from_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(from_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(from_tag_local));
        }

        self.emit_array_includes_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_index_of_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        if let Some(search) = args.first() {
            self.compile_expr_to_locals(search, search_payload_local, search_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(search_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(search_tag_local));
        }
        if let Some(from_index) = args.get(1) {
            self.compile_expr_to_locals(from_index, from_payload_local, from_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(from_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(from_tag_local));
        }

        self.emit_array_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_last_index_of_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let from_payload_local = self.reserve_temp_local();
        let from_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        if let Some(search) = args.first() {
            self.compile_expr_to_locals(search, search_payload_local, search_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(search_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(search_tag_local));
        }
        if let Some(from_index) = args.get(1) {
            self.compile_expr_to_locals(from_index, from_payload_local, from_tag_local, function)?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(from_payload_local));
            // Internal sentinel: omitted fromIndex differs from explicit undefined.
            function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
            function.instruction(&Instruction::LocalSet(from_tag_local));
        }

        self.emit_array_last_index_of_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            search_payload_local,
            search_tag_local,
            from_payload_local,
            from_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(from_tag_local);
        self.release_temp_local(from_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_includes_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_slot_present_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.includes called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_slice_index_clamped_to_string_len(
            from_payload_local,
            len_local,
            index_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            element_payload_local,
            element_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_same_value_zero_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_slot_present_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_index_of_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_slot_present_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.indexOf called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_slice_index_clamped_to_string_len(
            from_payload_local,
            len_local,
            index_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            element_payload_local,
            element_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_equality_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_slot_present_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_array_last_index_of_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        search_payload_local: u32,
        search_tag_local: u32,
        from_payload_local: u32,
        from_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_slot_present_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.lastIndexOf called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(from_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Dynamic.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(from_tag_local, from_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(from_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(from_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_retreat_to_previous_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            element_payload_local,
            element_tag_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_tagged_payload_equality_i32(
            element_tag_local,
            element_payload_local,
            search_tag_local,
            search_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_slot_present_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(done_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_find_like_builtin(
        &mut self,
        function: &mut Function,
        return_index: bool,
        reverse: bool,
    ) -> Result<(), EmitError> {
        let method_name = match (return_index, reverse) {
            (false, false) => "Array.prototype.find",
            (true, false) => "Array.prototype.findIndex",
            (false, true) => "Array.prototype.findLast",
            (true, true) => "Array.prototype.findLastIndex",
        };
        let nullish_message = match (return_index, reverse) {
            (false, false) => "Array.prototype.find called on null or undefined",
            (true, false) => "Array.prototype.findIndex called on null or undefined",
            (false, true) => "Array.prototype.findLast called on null or undefined",
            (true, true) => "Array.prototype.findLastIndex called on null or undefined",
        };
        let predicate_not_callable_message = match (return_index, reverse) {
            (false, false) => "Array.prototype.find predicate is not callable",
            (true, false) => "Array.prototype.findIndex predicate is not callable",
            (false, true) => "Array.prototype.findLast predicate is not callable",
            (true, true) => "Array.prototype.findLastIndex predicate is not callable",
        };
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing {method_name} receiver"
            ))
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing {method_name} receiver tag"
            ))
        })?;
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let callback_result_payload_local = self.reserve_temp_local();
        let callback_result_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        if return_index {
            function.instruction(&Instruction::I64Const((-1.0f64).to_bits() as i64));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            nullish_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            nullish_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            predicate_not_callable_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            predicate_not_callable_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        if reverse {
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            element_payload_local,
            element_tag_local,
            2,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_number_payload_local, number_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            callback_result_payload_local,
            callback_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            callback_result_payload_local,
            callback_result_tag_local,
            2,
            function,
        )?;

        self.compile_truthy_tagged_i32(
            callback_result_tag_local,
            callback_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        if return_index {
            function.instruction(&Instruction::LocalGet(index_number_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
        } else {
            function.instruction(&Instruction::LocalGet(element_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(element_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
        }
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if reverse {
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(index_local));
        } else {
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(callback_result_tag_local);
        self.release_temp_local(callback_result_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_iteration_box_receiver_if_primitive(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_array_iteration_nullish_receiver_throw_or_zero_length(
        &mut self,
        receiver_tag_local: u32,
        len_local: u32,
        payload_local: u32,
        tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_object_prototype_to_string_result_from_locals(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tag_payload_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let to_string_tag_key_local = self.reserve_temp_local();
        let custom_tag_payload_local = self.reserve_temp_local();
        let custom_tag_tag_local = self.reserve_temp_local();
        let prefix_local = self.reserve_temp_local();
        let suffix_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));

        for (kind, tag) in [
            (ValueKind::Undefined, "[object Undefined]"),
            (ValueKind::Null, "[object Null]"),
            (ValueKind::Boolean, "[object Boolean]"),
            (ValueKind::Number, "[object Number]"),
            (ValueKind::String, "[object String]"),
            (ValueKind::Symbol, "[object Symbol]"),
            (ValueKind::Object, "[object Object]"),
            (ValueKind::Array, "[object Array]"),
            (ValueKind::Function, "[object Function]"),
            (ValueKind::Arguments, "[object Arguments]"),
            (ValueKind::BigInt, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            brand_local,
            function,
        );
        for (boxed_kind, tag) in [
            (BOXED_PRIMITIVE_KIND_BOOLEAN, "[object Boolean]"),
            (BOXED_PRIMITIVE_KIND_NUMBER, "[object Number]"),
            (BOXED_PRIMITIVE_KIND_STRING, "[object String]"),
            (BOXED_PRIMITIVE_KIND_SYMBOL, "[object Symbol]"),
            (BOXED_PRIMITIVE_KIND_BIGINT, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(boxed_kind as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Error]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(to_string_tag_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            to_string_tag_key_local,
            custom_tag_payload_local,
            custom_tag_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(custom_tag_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("[object ")));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_concat_string_payloads_local(prefix_local, custom_tag_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(prefix_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(prefix_local, suffix_local, function)?;
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(tag_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));

        self.release_temp_local(suffix_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(custom_tag_tag_local);
        self.release_temp_local(custom_tag_payload_local);
        self.release_temp_local(to_string_tag_key_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(tag_payload_local);
        Ok(())
    }

    pub(crate) fn compile_typed_array_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing TypedArray.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing TypedArray.prototype.toString receiver tag",
            )
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();
        let typed_slot_present_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_local));

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_slot_present_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(typed_slot_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_prototype_to_string_result_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_prototype_to_string_result_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_validate_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_concat_string_payloads_local(joined_local, key_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(element_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_slot_present_local);
        self.release_temp_local(joined_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn compile_array_prototype_to_locale_string_builtin(
        &mut self,
        validate_typed_array_receiver: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.toLocaleString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.toLocaleString receiver tag",
            )
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let original_element_payload_local = self.reserve_temp_local();
        let original_element_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let element_string_local = self.reserve_temp_local();
        let joined_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(joined_local));

        if validate_typed_array_receiver {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(
                self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                receiver_payload_local,
                receiver_tag_local,
                receiver_payload_local,
                receiver_tag_local,
                key_local,
                typed_buffer_payload_local,
                typed_buffer_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(typed_buffer_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "TypedArray.prototype.toLocaleString requires TypedArray",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            self.emit_throw_current_function_realm_type_error(
                "TypedArray.prototype.toLocaleString requires TypedArray",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.toLocaleString called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        if validate_typed_array_receiver {
            self.emit_validate_typed_array_current_byte_length(
                receiver_payload_local,
                receiver_tag_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                function,
            )?;
        } else {
            self.emit_typed_array_current_byte_length(
                receiver_payload_local,
                receiver_tag_local,
                typed_buffer_payload_local,
                typed_byte_offset_local,
                typed_byte_length_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_concat_string_payloads_local(joined_local, key_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_index_to_flat_map_key_local(
            index_local,
            index_number_payload_local,
            key_local,
            function,
        )?;
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.compile_nullish_tagged_i32(element_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::LocalSet(original_element_payload_local));
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::LocalSet(original_element_tag_local));
        self.emit_array_iteration_box_receiver_if_primitive(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("toLocaleString"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            original_element_payload_local,
            original_element_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.toLocaleString element method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_pre_evaluated_arg_vector(&[], argc_local, argv_local, function)?;
        self.emit_function_handle_call_with_argv(
            method_payload_local,
            method_tag_local,
            Some((
                original_element_payload_local,
                Some(original_element_tag_local),
            )),
            argc_local,
            argv_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_string_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(joined_local, element_string_local, function)?;
        function.instruction(&Instruction::LocalSet(joined_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(joined_local);
        self.release_temp_local(element_string_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(original_element_tag_local);
        self.release_temp_local(original_element_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_object_has_array_index_key_in_range_i32(
        &mut self,
        object_payload_local: u32,
        start_index_local: u32,
        end_len_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let candidate_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_local,
            function,
        );
        self.emit_string_index_0_to_4_or_minus_one(key_local, candidate_index_local, function);

        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(start_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(candidate_index_local));
        function.instruction(&Instruction::LocalGet(end_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_index_local);
        self.release_temp_local(key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
    }

    pub(crate) fn emit_array_hole_prototype_clean_i32(
        &mut self,
        array_payload_local: u32,
        array_tag_local: u32,
        index_local: u32,
        len_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let prototype_payload_local = self.reserve_temp_local();
        let parent_prototype_local = self.reserve_temp_local();
        let prototype_has_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(array_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            prototype_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            parent_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_prototype_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(parent_prototype_local));
        self.load_i64_to_local_from_offset(
            parent_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));

        self.emit_object_has_array_index_key_in_range_i32(
            prototype_payload_local,
            index_local,
            len_local,
            prototype_has_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_has_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_array_index_key_in_range_i32(
            parent_prototype_local,
            index_local,
            len_local,
            prototype_has_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_has_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_has_index_local);
        self.release_temp_local(parent_prototype_local);
        self.release_temp_local(prototype_payload_local);
    }

    pub(crate) fn compile_array_prototype_for_each_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.forEach receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing Array.prototype.forEach receiver tag",
            )
        })?;
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let typed_receiver_local = self.reserve_temp_local();
        let typed_buffer_payload_local = self.reserve_temp_local();
        let typed_buffer_tag_local = self.reserve_temp_local();
        let typed_byte_offset_local = self.reserve_temp_local();
        let typed_byte_length_local = self.reserve_temp_local();
        let typed_bytes_per_element_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));

        self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(this_arg_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        function.instruction(&Instruction::End);

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_BYTE_LENGTH_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_ordinary(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            typed_buffer_payload_local,
            typed_buffer_tag_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            typed_byte_offset_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            typed_byte_length_local,
            function,
        )?;
        self.emit_object_read_number_slot_to_i64_local(
            receiver_payload_local,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            typed_bytes_per_element_local,
            function,
        )?;
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            self.result_local,
            self.result_tag_local,
            "Array.prototype.forEach called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.forEach callback is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_current_byte_length(
            receiver_payload_local,
            receiver_tag_local,
            typed_buffer_payload_local,
            typed_byte_offset_local,
            typed_byte_length_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(typed_byte_length_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(typed_receiver_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, index_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(typed_bytes_per_element_local);
        self.release_temp_local(typed_byte_length_local);
        self.release_temp_local(typed_byte_offset_local);
        self.release_temp_local(typed_buffer_tag_local);
        self.release_temp_local(typed_buffer_payload_local);
        self.release_temp_local(typed_receiver_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_array_for_each_method_call(
        &mut self,
        receiver: &TypedExpr,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let has_property_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let callback_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let array_hole_prototype_clean_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        if let Some(callback) = args.first() {
            self.compile_expr_to_locals(
                callback,
                callback_payload_local,
                callback_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                callback_payload_local,
                callback_tag_local,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(callback_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(callback_tag_local));
        }

        if let Some(this_arg) = args.get(1) {
            self.compile_expr_to_locals(
                this_arg,
                this_arg_payload_local,
                this_arg_tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                this_arg_payload_local,
                this_arg_tag_local,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(this_arg_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(this_arg_tag_local));
        }

        self.emit_array_iteration_box_receiver_if_primitive(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            element_tag_local,
            element_payload_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_iteration_nullish_receiver_throw_or_zero_length(
            receiver_tag_local,
            len_local,
            payload_local,
            tag_local,
            "Array.prototype.forEach called on null or undefined",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callback_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Array.prototype.forEach callback is not callable",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_hole_prototype_clean_i32(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            len_local,
            array_hole_prototype_clean_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(array_hole_prototype_clean_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            receiver_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_index_to_flat_map_key_local(
            index_local,
            index_payload_local,
            key_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_property_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_has_index_i32(
            receiver_payload_local,
            index_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_property_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            receiver_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_read(
            receiver_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(index_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, index_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_handle_call_with_argv(
            callback_payload_local,
            callback_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(array_hole_prototype_clean_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callback_tag_local);
        self.release_temp_local(callback_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(has_property_local);
        self.release_temp_local(key_local);
        self.release_temp_local(index_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_array_payload_with_length(
        &mut self,
        len_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(array_alloc_function_index) = self.array_alloc_function_index {
            let buffer_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::Call(array_alloc_function_index));
            function.instruction(&Instruction::LocalSet(buffer_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            self.release_temp_local(buffer_local);
            return Ok(());
        }
        let array_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let size_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(array_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(MIN_HEAP_CAPACITY as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size_local));
        self.emit_heap_alloc_from_local(size_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.store_i64_local_at_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.emit_init_array_constructor_slot(array_local, function);
        function.instruction(&Instruction::LocalGet(array_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        self.release_temp_local(size_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn emit_array_like_snapshot_payload(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        wrong_type_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let dst_payload_local = self.reserve_temp_local();
        let dst_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_alloc_array_payload_with_length(len_local, payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            input_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(len_local, dst_payload_local, function)?;
        self.load_i64_to_local_from_offset(
            dst_payload_local,
            HEAP_PTR_OFFSET,
            dst_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            input_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            input_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(len_local, dst_payload_local, function)?;
        self.load_i64_to_local_from_offset(
            dst_payload_local,
            HEAP_PTR_OFFSET,
            dst_buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_arguments_read(
            input_payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(dst_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(dst_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            wrong_type_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            self.result_local,
            self.result_tag_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_buffer_local);
        self.release_temp_local(dst_payload_local);
        self.release_temp_local(len_local);
        Ok(())
    }
}
