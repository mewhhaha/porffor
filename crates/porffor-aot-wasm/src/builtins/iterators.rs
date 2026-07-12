use super::super::*;

impl<'a> FunctionBuilder<'a> {
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
        self.emit_load_function_defining_realm_object_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_local_data(
            object_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data(object_local, "done", done, function)?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
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
        let regexp_prototype_payload_local = self.reserve_temp_local();
        let regexp_prototype_tag_local = self.reserve_temp_local();
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

        function.instruction(&Instruction::LocalGet(exec_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(regexp_prototype_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(regexp_prototype_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("exec")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            regexp_prototype_payload_local,
            regexp_prototype_tag_local,
            regexp_payload_local,
            regexp_tag_local,
            key_local,
            exec_payload_local,
            exec_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exec_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_arg_tag_local));
        self.emit_function_handle_call(
            exec_payload_local,
            exec_tag_local,
            Some((regexp_payload_local, Some(regexp_tag_local))),
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
        self.release_temp_local(regexp_prototype_tag_local);
        self.release_temp_local(regexp_prototype_payload_local);
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
